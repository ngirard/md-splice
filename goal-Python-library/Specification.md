# Python library specification for `md-splice`’s Rust core

This document specifies a Python library that wraps the `md-splice-lib` Rust crate to provide precise, AST-aware manipulation of Markdown from Python. The library is designed for programmatic edits by agents and tools that need reliable, semantic operations rather than brittle text search and replace.

The Python package exposes the same model, semantics, and guarantees as the Rust core: selectors operate on the Markdown AST; operations run in transactions with all-or-nothing atomicity; frontmatter is preserved and round-tripped; and heading “section” logic is first-class.

---

## Goals and non-goals

**Goals**

* Provide a Pythonic, ergonomic API over the Rust core without reducing capabilities.
* Preserve all behavioral guarantees: atomic transactions, section semantics, selector scoping (`after`, `within`), and range selection (`until`).
* Offer safe file I/O with atomic writes and optional dry-run and diff support.
* Interoperate cleanly with Python YAML values for frontmatter updates.

**Non-goals**

* Implement an alternative parser or printer in Python (Rust remains the sole engine).
* Provide partial edits that violate transaction atomicity.
* Expose unstable internal Rust AST types directly to Python callers.

---

## packaging, build, and platform support

* **Crate → wheel binding**: `pyo3` + `maturin`.
* **Package name**: `md_splice` (Python import: `import md_splice`).
* **Minimum Python**: 3.8+.
* **Platforms**: Linux x86_64/aarch64, macOS x86_64/arm64, Windows x86_64 (musl wheels optional).
* **Versioning**: Mirror Rust crate minor/patch (e.g., Rust `0.5.0` → Python `0.5.0`). Add a python build metadata tag if needed (e.g., `0.5.0.post1`) without changing Rust semantics.
* **Distribution**: sdist and manylinux/macos/windows wheels via `maturin build`. Wheels vendor the Rust core; no runtime Rust toolchain required.

---

## top-level modules and objects

```text
md_splice/
  __init__.py
  _native.*        # pyo3 extension module
  types.py         # thin Python dataclasses/enums mirroring Rust
  errors.py        # Python exception hierarchy
  io.py            # helpers: atomic write, unified diff
```

---

## data model

### enums

* `InsertPosition` (enum): `BEFORE`, `AFTER`, `PREPEND_CHILD`, `APPEND_CHILD`.
* `FrontmatterFormat` (enum): `YAML`, `TOML`.

### selectors

`Selector` (frozen dataclass)

* `select_type: Optional[str]`
  Matches block type, e.g., `p`, `heading`, `h2`, `list`, `li`, `table`, `blockquote`, `code`, `html`, `definition`, `footnotedefinition`, `thematicbreak`, or GitHub alert aliases like `note`, `alert-warning`, etc.
* `select_contains: Optional[str]`
  Substring match on rendered text.
* `select_regex: Optional[Pattern[str]]`
  Compiled Python regex, passed through to Rust (ECMAScript-like via Rust `regex` crate). Invalid patterns raise `InvalidRegexError` on construction.
* `select_ordinal: int = 1`
  One-indexed ordinal among matches after all filters.
* `after: Optional[Selector]`
  Landmark scoping. Mutually exclusive with `within`.
* `within: Optional[Selector]`
  Section or container scoping. Mutually exclusive with `after`.

**Notes**

* Using a list-item type (`li`, `item`, `listitem`) toggles nested item selection rather than block selection.
* Combining `after` and `within` raises `ConflictingScopeError`.

### operations

Each operation is a frozen dataclass. The `content` and `value` fields accept native Python types that map to strings (for Markdown body) or YAML values (for frontmatter).

* `InsertOperation`

  * `selector: Selector`
  * `content: Optional[str]` (Markdown)
  * `position: InsertPosition = InsertPosition.AFTER`
* `ReplaceOperation`

  * `selector: Selector`
  * `content: Optional[str]` (Markdown)
  * `until: Optional[Selector]`
* `DeleteOperation`

  * `selector: Selector`
  * `section: bool = False`
  * `until: Optional[Selector]`
* `SetFrontmatterOperation`

  * `key: str` (dot and array notation, e.g., `reviewers[0].email`)
  * `value: Any` (Python object → YAML)
  * `format: Optional[FrontmatterFormat] = None` (used only when creating a new block)
* `DeleteFrontmatterOperation`

  * `key: str`
* `ReplaceFrontmatterOperation`

  * `content: Any` (entire frontmatter as YAML)
  * `format: Optional[FrontmatterFormat] = None`

`Operation` is a `typing.Union` of the six dataclasses above and is also supported as a JSON/YAML loadable schema (see serialization).

---

## public api

### document lifecycle

```python
from md_splice import MarkdownDocument, Selector, InsertOperation, InsertPosition

doc = MarkdownDocument.from_string(markdown: str) -> MarkdownDocument
doc = MarkdownDocument.from_file(path: Union[str, Path]) -> MarkdownDocument

rendered: str = doc.render()
doc.write_in_place(backup: bool = False) -> None
doc.write_to(path: Union[str, Path]) -> None
```

**Semantics**

* `from_*` parses frontmatter (YAML `---` or TOML `+++`) and Markdown body into the internal AST.
* `render` re-emits the full document (frontmatter + body) preserving original frontmatter delimiter style and using the Rust printer defaults for body.
* `write_in_place` performs an atomic replace (write to tempfile + rename). If `backup=True`, create `path~` before replacing.
* `write_to` writes to a different path; the original file remains unchanged.

### transactions and atomicity

```python
doc.apply(ops: list[Operation], *, warn_on_ambiguity: bool = True) -> None
```

* Applies all operations against an in-memory working copy; on any failure, raises and leaves `doc` unchanged.
* When a selector matches multiple nodes, the first match is used. If `warn_on_ambiguity=True` and more than one match exists, a `UserWarning` is emitted. Set to `False` to silence warnings (behavior matches Rust: warning via logger).

### read-only selection

```python
doc.get(selector: Selector,
        *,
        select_all: bool = False,
        section: bool = False,
        until: Optional[Selector] = None) -> str | list[str]
```

* Returns rendered Markdown snippet(s) for the selection.
* `section=True` when the selector targets a heading: include the heading and its section.
* `until` extends the selection to (but not including) the `until` match; only valid for block-level targets. Applying to a list item raises `RangeRequiresBlockError`.
* `select_all=True` returns a list of snippets for all matches (order preserved).

### diffing and dry runs

```python
from md_splice import diff_unified

preview: str = doc.preview(ops: list[Operation])            # render string after ops, doc unchanged
diff: str = diff_unified(before: str, after: str,
                         fromfile: str = "original",
                         tofile: str = "modified")          # standard unified diff text
```

* `preview` runs a transaction on a cloned document and returns the rendered Markdown for inspection. The original `doc` is not mutated.
* `diff_unified` uses Rust’s `similar` crate equivalence; line-based unified diff.

### frontmatter helpers

```python
fm = doc.frontmatter() -> Any | None          # Python object (via YAML) or None
fmt = doc.frontmatter_format() -> FrontmatterFormat | None
```

* Mirrors Rust: returns parsed structure as Python objects convertible by PyYAML semantics.
* `frontmatter_format` reveals the storage format if present.

### json/yaml (de)serialization for operations

```python
from md_splice import loads_operations, dumps_operations

ops = loads_operations(text: str, *, format: Literal["json", "yaml"] | None = None) -> list[Operation]
text = dumps_operations(ops: list[Operation], *, format: Literal["json", "yaml"] = "yaml") -> str
```

* `loads_operations` autodetects format when `format=None` (YAML if parseable; otherwise JSON).
* The schema matches the CLI/README structure verbatim.

---

## error handling

All exceptions inherit from `MdSpliceError`:

* `NodeNotFoundError` — selector matched nothing.
* `InvalidChildInsertionError` — attempted `PREPEND_CHILD` or `APPEND_CHILD` into a non-container (e.g., `Paragraph`).
* `AmbiguousContentSourceError` — internal guard; not generally reachable from Python API (only one content channel).
* `NoContentError` — required content/value missing.
* `InvalidListItemContentError` — provided Markdown could not be parsed as list item(s) for a list-item targeted operation.
* `AmbiguousStdinSourceError` — not applicable in Python (no stdin duality); reserved for parity.
* `InvalidSectionDeleteError` — `section=True` but selector does not target a heading.
* `SectionRequiresHeadingError` — same family (kept for parity).
* `ConflictingScopeError` — both `after` and `within` are set on the same selector.
* `RangeRequiresBlockError` — `until` is only supported for block-level selections.
* `FrontmatterMissingError` — no frontmatter for a read/delete expectation.
* `FrontmatterKeyNotFoundError(key: str)` — deletion path not present.
* `FrontmatterParseError(msg: str)` — parsing failure for existing block.
* `FrontmatterSerializeError(msg: str)` — failed to render block back.
* `MarkdownParseError(msg: str)` — body parse failure.
* `OperationParseError(msg: str)` — JSON/YAML operation parse failure.
* `OperationFailedError(msg: str)` — wrapped failure of any single operation.
* `IoError(msg: str)` — I/O failure.
* `InvalidRegexError(msg: str)` — regex compilation failure for `select_regex`.

Exceptions preserve Rust error messages where valuable and add Python stack context.

---

## behavioral guarantees

* **Atomic transactions**: `apply` mutates in memory and commits only if every operation succeeds. On error, the document object is unchanged.
* **Selector semantics**: identical to Rust, including:

  * Ordinal is one-indexed and applied after filters.
  * `within` a heading scopes to that heading’s section; `within` a list scopes to list items; other `within` targets are invalid.
  * `after` landmark selects nodes after the landmark; for lists, “after” an item restricts to later items in the same list first, then the rest of the document.
* **Section operations**: heading sections extend to the next heading of the same or higher level, or end of document.
* **Range operations**: `until` endpoints are exclusive. Missing endpoint means “to end of document.”
* **Frontmatter**:

  * YAML and TOML are detected via `---` / `+++` delimiters.
  * Frontmatter values are transported as YAML in memory; TOML is losslessly converted to YAML and back to TOML on render if that was the original (or chosen) format.
  * Empty maps/arrays collapse to removal of the block.
  * Path rules: dot segments for mappings; `[N]` for array indices; indexes must exist when writing into arrays; nested mappings are created on demand.

---

## examples

### append a checklist item within a section

```python
from md_splice import (
    MarkdownDocument, Selector,
    InsertOperation, InsertPosition
)

doc = MarkdownDocument.from_string("""
## High Priority

- [ ] Ship metrics
""".lstrip())

op = InsertOperation(
    selector=Selector(
        select_type="list",
        within=Selector(select_type="h2", select_contains="High Priority")
    ),
    content="- [ ] Review newly filed issues",
    position=InsertPosition.APPEND_CHILD,
)

doc.apply([op])
print(doc.render())
```

### replace a heading section up to another heading

```python
from md_splice import MarkdownDocument, Selector, ReplaceOperation

doc = MarkdownDocument.from_string("""
## Deprecated API
Old details

## Examples
...
""".lstrip())

doc.apply([
    ReplaceOperation(
        selector=Selector(select_type="h2", select_contains="Deprecated API"),
        until=Selector(select_type="h2", select_contains="Examples"),
        content="## Deprecated API\nThis section has been removed.\n",
    )
])
```

### delete a list item after a landmark within a section

```python
from md_splice import MarkdownDocument, Selector, DeleteOperation

doc = MarkdownDocument.from_string("""
## Future Features
- [ ] Task Alpha
- [ ] Task Beta
- [ ] Task Gamma
""".lstrip())

doc.apply([
    DeleteOperation(
        selector=Selector(
            select_type="li",
            select_contains="Task Beta",
            after=Selector(select_type="li", select_contains="Task Alpha"),
            within=Selector(select_type="h2", select_contains="Future Features"),
        )
    )
])
```

### frontmatter updates in one transaction

```python
from md_splice import (MarkdownDocument,
    SetFrontmatterOperation, DeleteFrontmatterOperation, ReplaceFrontmatterOperation,
    FrontmatterFormat)

doc = MarkdownDocument.from_string("""---
title: Spec
status: draft
reviewers:
  - email: a@example.com
---
# Title
""")

ops = [
    SetFrontmatterOperation(key="status", value="approved"),
    SetFrontmatterOperation(key="last_updated", value="2025-10-21"),
    DeleteFrontmatterOperation(key="reviewers[0]"),
    ReplaceFrontmatterOperation(content={"title": "Spec", "version": 2},
                                format=FrontmatterFormat.TOML),
]
doc.apply(ops)
```

### read-only selection and diff

```python
from md_splice import MarkdownDocument, Selector, diff_unified

doc = MarkdownDocument.from_file("README.md")
before = doc.render()

snippet = doc.get(Selector(select_type="h2", select_contains="Installation"),
                  section=True)

# ...prepare ops...
after = doc.preview(ops)
print(diff_unified(before, after, fromfile="original", tofile="modified"))
```

---

## yaml value mapping

The binding uses `serde_yaml` internally. Python objects map as:

* `None` ↔ YAML `null`
* `bool` ↔ YAML boolean
* `int/float` ↔ YAML number
* `str` ↔ YAML string
* `list` ↔ YAML sequence
* `dict[str, Any]` ↔ YAML mapping

If you need tagged scalars or advanced YAML features, pass preloaded PyYAML nodes as plain data (tags are not preserved by design; parity with Rust).

---

## performance and threading

* Parsing and printing are native Rust; large documents are handled efficiently.
* The `MarkdownDocument` object is **not** thread-safe for concurrent mutation. Clone via `doc.clone()` (provided) for parallel previews.
* File writes use atomic replace on POSIX and `ReplaceFile` semantics on Windows with a temporary path.

---

## edge cases and exact error conditions

* Supplying both `after` and `within` on the same selector raises `ConflictingScopeError`.
* Using `section=True` without a heading target raises `InvalidSectionDeleteError`.
* Providing `until` for a list item raises `RangeRequiresBlockError`.
* Replacing a list item with content that does not parse as a list raises `InvalidListItemContentError`.
* Attempting to set `reviewers[3]` when the array has fewer items raises a bounds error wrapped as `OperationFailedError` with details.
* Regex patterns compile on selector creation; invalid patterns raise `InvalidRegexError`.

---

## serialization schema for operations

The Python loader accepts the same JSON/YAML schema documented in the repository README for the CLI. Python construction and round-trip (`dumps_operations`) produce the same keys (`op`, `selector`, `position`, `until`, etc.), so operation files are interchangeable between the CLI and Python.

---

## logging and warnings

* Ambiguous selector matches set an internal ambiguity flag. By default, `apply` emits a `UserWarning` when ambiguity is detected and proceeds with the first match (parity with Rust’s `log::warn!`). Applications may silence this globally or per-call.

---

## api reference (concise signatures)

```python
class MarkdownDocument:
    @classmethod
    def from_string(cls, markdown: str) -> "MarkdownDocument": ...
    @classmethod
    def from_file(cls, path: os.PathLike | str, /) -> "MarkdownDocument": ...
    def render(self) -> str: ...
    def write_in_place(self, backup: bool = False) -> None: ...
    def write_to(self, path: os.PathLike | str, /) -> None: ...
    def apply(self, ops: list[Operation], *, warn_on_ambiguity: bool = True) -> None: ...
    def preview(self, ops: list[Operation], *, warn_on_ambiguity: bool = True) -> str: ...
    def get(self, selector: Selector, *,
            select_all: bool = False,
            section: bool = False,
            until: Selector | None = None) -> str | list[str]: ...
    def frontmatter(self) -> Any | None: ...
    def frontmatter_format(self) -> FrontmatterFormat | None: ...
    def clone(self) -> "MarkdownDocument": ...

@dataclass(frozen=True)
class Selector: ...
@dataclass(frozen=True)
class InsertOperation: ...
@dataclass(frozen=True)
class ReplaceOperation: ...
@dataclass(frozen=True)
class DeleteOperation: ...
@dataclass(frozen=True)
class SetFrontmatterOperation: ...
@dataclass(frozen=True)
class DeleteFrontmatterOperation: ...
@dataclass(frozen=True)
class ReplaceFrontmatterOperation: ...

def loads_operations(text: str, *, format: Literal["json", "yaml"] | None = None) -> list[Operation]: ...
def dumps_operations(ops: list[Operation], *, format: Literal["json", "yaml"] = "yaml") -> str: ...
def diff_unified(before: str, after: str, *,
                 fromfile: str = "original", tofile: str = "modified") -> str: ...

class MdSpliceError(Exception): ...
# … plus the concrete subclasses listed earlier
```

---

## testing strategy

* Golden tests ported from Rust’s `insta` snapshots for parity on printing and section logic.
* Cross-language invariants:

  * Load → apply ops → render in Python equals running the same ops via the CLI.
  * Frontmatter round-trip (YAML and TOML) equality tests.
  * Selector ambiguity detection mirrored (warnings vs. logs).
* Property tests for “apply fails leaves document unchanged.”

---

## implementation notes (binding)

* **pyo3 class layout**
  Map `MarkdownDocument` to a `#[pyclass]` holding `ParsedDocument` and `Document` as in Rust. Expose methods that call the existing internal functions: `apply_operations`, `refresh_frontmatter_block`, `render_markdown`, etc.

* **error mapping**
  Implement `From<SpliceError> for PyErr` mapping to the Python subclasses; preserve messages.

* **regex**
  Accept Python `re.Pattern`; extract pattern string and flags. Translate flags supported by Rust’s `regex` (`IGNORECASE`, `MULTILINE`, `DOTALL`) and document unsupported flags (`UNICODE` is default; `VERBOSE` not supported).

* **yaml bridging**
  Accept Python objects; serialize to YAML via `serde_yaml` on the Rust side using `pyo3`’s `PyAny` → `serde` bridge (`serde-pyobject` or custom visitor). Return values as native Python via the same bridge.

* **atomic write**
  Implement `write_in_place` using Rust `tempfile` + atomic rename; Windows fallback uses replace semantics.

---

## migration and versioning policy

* The Python API is source-compatible across patch versions. Minor versions track the Rust crate’s minor versions.
* Any behavioral changes to selector semantics or printing will be documented in the changelog and mirrored between the CLI, Rust library, and Python package.

---

## quickstart

```bash
pip install md-splice==0.5.*
```

```python
from md_splice import *

doc = MarkdownDocument.from_file("TODO.md")
ops = loads_operations("""
- op: insert
  selector:
    select_type: li
    select_contains: "Write documentation"
  position: before
  content: "- [ ] Implement unit tests"
""")
print(doc.preview(ops))      # inspect
doc.apply(ops)               # commit in memory
doc.write_in_place()         # atomically persist
```
