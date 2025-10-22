# md-splice Python context manager specification

This document is the single source of truth for designing, implementing, testing, and documenting the Python context manager(s) that simplify safe, transactional Markdown edits using `md-splice`. It normatively defines names, behaviors, error handling, edge cases, and testing obligations.

The intended audience is contributors to the Rust core and Python bindings, as well as users who need exact semantics.

## Overview

The context manager provides a disciplined “edit session” around a `MarkdownDocument`. It opens a file, lets the caller apply one or more AST-aware operations, and—on clean exit—commits the result with atomic replacement. On any unhandled exception, nothing is written. The manager centralizes safety policies (ambiguity handling, stale-write protection, backups, and optional diff previews) so callers do not have to remember them per call.

Two complementary forms are specified:

1. **`MdEdit`**: yields a `MarkdownDocument` so callers can call `doc.apply(...)` any number of times inside the `with` block. It commits on graceful exit.
2. **`MdBatchEdit`**: yields a batching proxy that **collects** operations and calls `doc.apply(collected_ops)` exactly once on exit, improving atomicity and selector stability across a set of changes.

Both provide identical guardrails and commit rules unless stated otherwise.

## Goals

1. Make the safe path the easy path for humans and LLM agents.
2. Prevent accidental data loss via atomic writes and file change detection.
3. Make selector ambiguity a deliberate choice rather than a foot-gun.
4. Provide a minimal, unsurprising API that composes with the existing bindings.
5. Keep implementation details portable across platforms.

## Non-goals

1. Replace the CLI workflows (`apply`, `--diff`, `--dry-run`). The context manager complements them.
2. Provide concurrent edit resolution beyond last-writer-wins prevention. This spec only mandates refusing writes when the file changed on disk since entry.
3. Implicitly escalate all warnings in user code. Only ambiguity handling within the context manager is controlled here.

## Terminology

* **Commit**: an atomic in-place write of the modified document to the original path (via `MarkdownDocument.write_in_place`).
* **Clean exit**: exiting a `with` block without any unhandled exception.
* **Ambiguity**: a selector matches more than one node (per Rust core’s detection).
* **Stale write**: a write attempted when the underlying file has changed since context entry.

## User-facing API summary

### Constructors

```python
MdEdit(path, *,
       backup=True,
       fail_on_ambiguity=True,
       check_stale=True,
       preview_diff=False,
       commit=True)

MdBatchEdit(path, *,
            backup=True,
            fail_on_ambiguity=True,
            check_stale=True,
            preview_diff=False,
            commit=True)
```

### Usage

```python
from md_splice import Selector, InsertOperation, InsertPosition
from md_splice.ctx import MdEdit, MdBatchEdit  # module name is advisory; see packaging

# 1) Freeform editing with multiple apply() calls
with MdEdit("README.md") as doc:
    doc.apply([InsertOperation(
        selector=Selector(select_type="h2", select_contains="Changelog"),
        content="## Release notes\n- First Python binding\n",
        position=InsertPosition.AFTER
    )])
    # more doc.apply(...) calls are allowed
# commits here on clean exit

# 2) Batch mode: queue operations; apply once on exit
with MdBatchEdit("README.md") as edit:
    edit.apply(InsertOperation(
        selector=Selector(select_type="h2", select_contains="Changelog"),
        content="## Release notes\n- First Python binding\n",
        position=InsertPosition.AFTER
    ))
    # ...more edit.apply(op) calls...
# applies once and commits here on clean exit
```

## Semantics

### Enter behavior

* `path` must be a filesystem path. Passing `"-"` (stdin) **MUST** raise `ValueError`.
* The manager **MUST** read and cache a *stale-write token* consisting of `(mtime_ns, size)` of `path` at entry. An implementation **MAY** add a fast content hash; if present it augments, not replaces, the token.
* The manager **MUST** load `MarkdownDocument` via `MarkdownDocument.from_file(str(path))`.

### Exit behavior and commit rules

* On **any unhandled exception** inside the `with` block, **no commit** occurs. The exception propagates unchanged.
* If `commit=False`, the manager **MUST NOT** write, even on clean exit.
* If `check_stale=True`, before writing the manager **MUST** re-stat the file and compare `(mtime_ns, size)` to the entry token. If either differs, it **MUST** raise `RuntimeError` (no write).
* When writing, the manager **MUST** call `MarkdownDocument.write_in_place(backup=backup)` and rely on its atomic replace guarantees (see platform notes below).

### Ambiguity policy

* When `fail_on_ambiguity=True` (the default), any ambiguity detected during `apply(...)` **MUST** cause the context to fail the session, i.e., raise an error and skip the commit.
* Acceptable implementations:

  * Convert the Rust core’s ambiguity signal into an exception, **or**
  * Temporarily promote `UserWarning` (emitted by bindings on ambiguity) to an exception **within the context only**, **or**
  * Use an API that returns an `ApplyOutcome` and raise if `ambiguity_detected=True`.
* The raised exception type **SHOULD** be `OperationFailedError` if surfaced by the core; otherwise `RuntimeError` is permissible, but error text **MUST** include “ambiguity” for diagnosability.
* With `fail_on_ambiguity=False`, ambiguity **MUST NOT** block commit; the context **MUST** surface warnings normally.

### Diff preview

* When `preview_diff=True`, the manager **MUST** compute a unified diff between the on-disk content at exit and the in-memory rendered content, and print it to stdout **before** attempting the commit.
* The diff **MUST** be produced via `md_splice.diff_unified(before, after, fromfile="original", tofile="modified")`.
* Diff preview is informational and **MUST NOT** change commit rules.

### Backups

* When `backup=True` (default), `write_in_place(backup=True)` **MUST** be used. The extension **MUST** create a sibling backup per the existing contract (e.g., `path~`) before the atomic swap.

### Batch mode semantics

* `MdBatchEdit.apply(op)` **MUST** append operations to an internal list without mutating the document immediately.
* On clean exit, if at least one operation was added, the manager **MUST** call `doc.apply(collected_ops)` exactly once, then proceed with ambiguity checks, diff preview, stale check, and commit as above.
* If no operations were added, the manager **MUST NOT** write and **MUST NOT** raise.

### Nested contexts

* Nested contexts on the **same** `path` **MUST** be rejected. When the inner context attempts to enter, it **MUST** raise `RuntimeError` explaining that nested edits on the same path are unsupported.
* Nested contexts on **different** paths are allowed.
* Re-entering the **same** context object (reusing it) is **MUST NOT** be supported (standard Python context manager practice).

### Reentrancy and thread safety

* The classes are not thread-safe. Concurrent usage across threads on the same path is undefined; the stale-write check is best effort, not a lock.
* Implementations **SHOULD** avoid holding open unnecessary file handles outside of `MarkdownDocument` itself, minimizing Windows handle contention.

### Interaction with stdin and content files

* When the source document is `"-"`, the context manager **MUST** raise `ValueError` (not supported).
* Operations inside the context remain free to read splice content from stdin (`content_file="-"`). However, if both the document and any operation attempt to read from `"-"`, the underlying core will raise `AmbiguousStdinSourceError`; the context **MUST** let it propagate.

### Encoding and newlines

* The manager **MUST** honor the encoding semantics of `MarkdownDocument.from_file` and `write_in_place`. It **MUST NOT** alter encoding or newline normalization itself.

### Platform notes for atomic replace

* On Windows, atomic replacement can fail if any other process holds an open handle on the target. The context **MUST** propagate the underlying `IoError` unchanged and **MUST NOT** retry silently.
* On POSIX, standard atomic rename semantics apply.

## API details

### `MdEdit`

**Module**: `md_splice.ctx` (recommended; the concrete module/package path is an implementation detail but must be documented).

**Signature**:

```python
class MdEdit:
    def __init__(self, path, *,
                 backup: bool = True,
                 fail_on_ambiguity: bool = True,
                 check_stale: bool = True,
                 preview_diff: bool = False,
                 commit: bool = True): ...
    def __enter__(self) -> MarkdownDocument: ...
    def __exit__(self, exc_type, exc, tb) -> bool: ...
```

**Attributes (read-only during the session)**:

* `path: pathlib.Path`
* `backup: bool`
* `fail_on_ambiguity: bool`
* `check_stale: bool`
* `preview_diff: bool`
* `commit: bool`

**Methods**:

* `abort()` (optional convenience): sets `commit=False` for the current instance. If provided, it **MUST** be idempotent.

**Behavioral notes**:

* `__enter__` **MUST** precompute stale token and construct `MarkdownDocument`.
* `__exit__` **MUST** implement the commit rules defined above.

### `MdBatchEdit`

**Signature**:

```python
class MdBatchEdit(MdEdit):
    def __enter__(self) -> "MdBatchEdit": ...
    def apply(self, op: Operation) -> None: ...
```

**Differences from `MdEdit`**:

* `__enter__` returns `self` (not the document).
* `apply(op)` **MUST** accept any union member from `md_splice.types.Operation` and only queue it.
* `__exit__` **MUST** call `doc.apply(collected_ops)` once when `collected_ops` is non-empty, then proceed with the base class’ exit logic.
* Any exception raised while queuing operations or during the single `apply` **MUST** abort the commit as per the base rules.

## Error handling and exception mapping

The context manager **MUST** not wrap or transform domain errors from the core unless required by the ambiguity policy or nested-context detection. Specifically:

* Propagate as-is:

  * `NodeNotFoundError`, `InvalidChildInsertionError`, `InvalidListItemContentError`,
  * `InvalidSectionDeleteError`, `SectionRequiresHeadingError`, `ConflictingScopeError`,
  * `RangeRequiresBlockError`, `FrontmatterMissingError`, `FrontmatterKeyNotFoundError`,
  * `FrontmatterParseError`, `FrontmatterSerializeError`, `MarkdownParseError`,
  * `OperationParseError`, `OperationFailedError`, `IoError`, `InvalidRegexError`.
* Raise `RuntimeError` in these cases (with clear messages):

  * stale-write mismatch (`check_stale=True`),
  * nested context on the same path,
  * stdin document path is `"-"`.
* For ambiguity with `fail_on_ambiguity=True`:

  * If the binding exposes `ApplyOutcome`, raise `OperationFailedError("ambiguous selector...")`.
  * If the binding signals via `UserWarning`, temporarily promote that warning to an exception (preferred) and let it propagate as `UserWarning` converted to `RuntimeError` with message including “ambiguity”.

## Logging and warnings

* The context **MUST NOT** emit its own warnings in the success path.
* When promoting ambiguity warnings, the promotion **MUST** be scoped to the active context only and reverted on exit, regardless of success or failure.
* Implementations **MAY** log debug messages behind a dedicated logger (`md_splice.ctx`) but **MUST NOT** print in the success path unless `preview_diff=True`.

## Examples

### Preview and safe commit

```python
from md_splice.ctx import MdEdit
from md_splice import Selector, ReplaceOperation

with MdEdit("guide.md", preview_diff=True) as doc:
    doc.apply([ReplaceOperation(
        selector=Selector(select_type="h2", select_contains="Installation"),
        content="## Installation\nUpdated steps.\n",
    )])
# prints a unified diff and commits if no exception occurred
```

### Abort programmatically

```python
with MdEdit("doc.md") as doc:
    try:
        # Attempt edits...
        ...
    except Exception:
        # decide to skip commit but swallow the error
        doc_ctx = doc  # doc has no abort; use the context instead if exposed:
        # md_splice.ctx.abort_current() could be provided; otherwise, re-raise and rely on no commit
        raise  # will abort commit
```

### Batch operations with strict ambiguity

```python
from md_splice.ctx import MdBatchEdit
from md_splice import loads_operations

ops = loads_operations("""
- op: insert
  selector:
    select_type: h2
    select_contains: Future Features
  position: append_child
  content: "- [ ] Implement unit tests"
""")

with MdBatchEdit("ROADMAP.md", fail_on_ambiguity=True) as edit:
    for op in ops:
        edit.apply(op)
# applies once; fails (no write) if any selector was ambiguous
```

## Reference algorithms

### `MdEdit.__enter__`

1. Validate `path != "-"`; else raise `ValueError`.
2. Record `stat(path)` → `(mtime_ns, size)`.
3. `doc = MarkdownDocument.from_file(str(path))`.
4. If `fail_on_ambiguity=True`, install a *scoped* warning filter that turns `UserWarning` from the md-splice binding into an exception. Record prior filter state; restore on exit.
5. Return `doc`.

### `MdBatchEdit.__enter__`

1. Call `super().__enter__()`.
2. Initialize empty `ops` list.
3. Return `self`.

### `MdBatchEdit.apply(op)`

1. Validate `op` is an instance of the supported union.
2. Append to `ops`.

### `MdEdit.__exit__` (shared finalization)

1. If `exc_type is not None`: restore warning state; return `False` (propagate).
2. If `commit=False`: restore warning state; return `False`.
3. If this is `MdBatchEdit` and `ops` is non-empty: call `doc.apply(ops)`. Let exceptions propagate.
4. If `check_stale=True`: re-`stat(path)`; if `(mtime_ns, size)` differ, raise `RuntimeError` (“refusing to write: file changed”).
5. If `preview_diff=True`: read current file bytes as text; obtain rendered string from `doc` (implementation may use an exposed `render()` or equivalent); print `diff_unified(...)`.
6. Call `doc.write_in_place(backup=backup)`. Let errors propagate.
7. Restore warning state; return `False`.

## Testing requirements

The following tests are **REQUIRED** (names are indicative):

1. **`test_no_commit_on_exception`**: raise inside block; assert file unchanged.
2. **`test_commit_on_clean_exit`**: edit succeeds; assert file contains change and a backup exists when `backup=True`.
3. **`test_stale_write_refused`**: mutate file externally between enter and exit; expect `RuntimeError` and no write.
4. **`test_ambiguity_escalates`**: arrange an ambiguous selector; with `fail_on_ambiguity=True` expect an exception, with `False` expect commit and a warning (if bindings emit one).
5. **`test_diff_preview_prints`**: enable `preview_diff`; capture stdout; verify unified diff header; commit still occurs.
6. **`test_batch_applies_once`**: spy or counter to ensure exactly one `apply()` call is made in `MdBatchEdit`.
7. **`test_nested_same_path_rejected`**: open inner context on same path; expect `RuntimeError`.
8. **`test_stdin_path_rejected`**: `MdEdit("-")` raises `ValueError`.
9. **`test_commit_flag_false_skips_write`**: `commit=False`; verify no write even on clean exit.
10. **`test_warning_filter_scope`**: ensure ambiguity warning promotion only applies inside context.

## Compatibility and versioning

* The API specified herein targets `md-splice` Python package version `>= 0.5.0`.
* Backwards-compatible additions (e.g., new keyword parameters with defaults) are allowed in minor releases.
* Removing fields, changing defaults, or altering commit semantics requires a major version bump.

## Documentation obligations

* Add a new `Context managers` section to `md-splice-py/README.md` with the usage shown above.
* Reference ambiguity and stale-write policies with short, explicit examples.
* Clearly document platform caveats (Windows handle locking).

## Future extensions (non-normative)

* Optional `content_hash` stale token for stronger change detection.
* Global preference toggles via environment variables (e.g., `MD_SPLICE_FAIL_ON_AMBIGUITY=0`).
* `abort()` on the context object to opt out of commit without raising.
* An `MdEdit.session` object exposing `before_text`, `after_text`, and `diff_text` for programmatic inspection.

## Appendix: rationale for defaults

* **`fail_on_ambiguity=True`**: safer for agents; forces selectors to be explicit.
* **`check_stale=True`**: prevents silent clobber in multi-tool pipelines.
* **`backup=True`**: matches the package’s safety posture and CLI behavior.
* **`preview_diff=False`**: keeps logs clean by default; easy to enable when needed.
* **`commit=True`**: conforms to “commit on clean exit” mental model; the explicit flag allows dry-runs in code without changing call sites.

---

This specification is authoritative. Implementations must adhere to it; discrepancies should be resolved by updating the implementation or, if needed, revising this document with an explicit rationale.
