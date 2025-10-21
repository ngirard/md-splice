# Progress Status

update 01
- Scaffolded the `md-splice` Python package with `pyproject.toml`, Rust extension crate, and minimal Python API surface.
- Added initial pytest covering import/version, Markdown parse/render round-trip, frontmatter `None`, and base error exposure.
- Verified compilation via `maturin develop` and passing tests with `python -m pytest tests` inside the new virtual environment.
- Next: flesh out frontmatter bridging, richer error mapping, and additional API surface per Milestone 1/2 of `Strategy.md`.

update 02
- Implemented YAML value bridging in the Rust extension so `MarkdownDocument.frontmatter()` returns native Python structures.
- Added Python `FrontmatterFormat` enum and exposed `MarkdownDocument.frontmatter_format()` with Rust-to-Python mapping.
- Created frontmatter-focused pytest suite (YAML/TOML/empty) and verified via `maturin develop` + `pytest` in a dedicated venv.
- Next: expand selector dataclasses/enums and error mapping to mirror Rust semantics (Milestone 3).
update 03
- Added Python `Selector` dataclass with validation, `InsertPosition` enum, and regex compilation to align with Milestone 3 data model needs.
- Expanded Python error hierarchy and implemented Rust-to-Python exception mapping so core errors surface as typed subclasses.
- Introduced selector-focused pytest coverage and error mapping regression test, all passing under `maturin develop` + `.venv/bin/python -m pytest`.
- Next: wire selectors into native `get`/operation bindings and continue fleshing out operation dataclasses.

update 04
- Implemented `MarkdownDocument.get` in the Rust extension with selector conversion, range/section rendering, and list-item handling for parity with the CLI get command.
- Added Rust helpers for regex bridging, heading section computation, and markdown rendering plus a Python test suite (`test_get.py`) covering type/regex filters, sections, ranges, and select-all semantics.
- Verified editable build via `.venv/bin/maturin develop --manifest-path md-splice-py/Cargo.toml --release` and passing tests with `.venv/bin/python -m pytest md-splice-py/tests`.
- Next: extend bindings to cover transactional operations (`apply`) and diff/preview helpers per Milestone 5.

update 05
- Added transactional operation support by mirroring Rust `Operation` enums into Python dataclasses and bridging them through the native layer, including YAML conversion and selector translation.
- Exposed `MarkdownDocument.apply`, `preview`, and `clone`, plus `diff_unified`, with ambiguity warnings surfaced via Python's warnings system.
- Extended Rust core with `ApplyOutcome` metadata and Python bindings to emit warnings, alongside a new test suite (`test_apply.py`) covering insert/replace/delete/frontmatter ops, atomicity, preview, and diff helpers.
- Verified editable build with `.venv/bin/maturin develop --manifest-path md-splice-py/Cargo.toml --release` and `pytest` across the full test matrix.

update 06
- Added file I/O ergonomics with `MarkdownDocument.from_file`, `write_in_place` (atomic replace), and `write_to`, tracking source paths and surfacing Python `IoError` on failure.
- Implemented shared ops serialization via `loads_operations`/`dumps_operations` bridging YAML/JSON schemas with the Python dataclasses while rejecting unsupported file-based fields.
- Expanded tests with `test_io.py`, `test_operations_io.py`, and import smoke checks plus refreshed README quickstart documenting the new workflow.
- Confirmed coverage by rebuilding the extension with `maturin develop` and running `pytest` against the augmented suite.

update 07
- Extended `MarkdownDocument.write_in_place` with a keyword-only `backup` flag that snapshots the current file before atomic replacement to satisfy the safety guarantees in the spec.
- Added Rust-side backup helper that copies to a `.bak` sibling and enforced existence checks to surface `IoError` when the backing file is missing.
- Augmented `test_io.py` with coverage proving backups preserve the original content alongside atomic writes, and re-ran the full `maturin develop --release` + `pytest` flow successfully.
- Next: tighten regex flag parity so Python `re` selectors honor case-insensitive/multiline/dotall semantics.

update 08
- Translated Python `re` flags into Rust regex builder settings so selectors respect `IGNORECASE`, `MULTILINE`, and `DOTALL`, while rejecting unsupported flags with a typed `InvalidRegexError`.
- Added regression tests in `test_get.py` covering case-insensitive, multiline, and dotall regex matches plus the unsupported flag failure path.
- Rebuilt the extension via `.venv/bin/maturin develop --manifest-path md-splice-py/Cargo.toml --release` and reran `pytest md-splice-py/tests` to confirm the expanded suite passes.
- Next: document the supported regex flags in the Python README and surface them in docstrings per the specification's regex notes.

update 09
- Documented the supported/unsupported regex flags in the Python README to guide users working with `Selector.select_regex`.
- Expanded the `Selector` docstring to detail accepted `re` flags and the corresponding `InvalidRegexError` behavior.
- Next: continue aligning documentation and docstrings with the remaining Specification sections (e.g., diff helpers, transaction semantics).

update 10
- Added comprehensive docstrings to the `MarkdownDocument` bindings and helper functions so Python introspection mirrors the Specification's guarantees for transactions, I/O, preview, and selector semantics.
- Expanded the Python README with sections covering transactional safety, preview/diff helpers, and operations schema interoperability for parity with the spec.
- Enriched operation dataclass docstrings to describe range, section, and frontmatter behaviors consistent with the CLI schema.
- Next: Audit remaining public APIs (e.g., module-level `__all__`, error classes) for docstring completeness and add changelog entries summarizing the Python bindings.

update 11
- Re-exported the full Python error hierarchy from `md_splice.__all__` and documented the base `MdSpliceError` so introspection exposes the spec-mandated exception surface.
- Added a `CHANGELOG.md` that summarizes the Python binding capabilities delivered so far and codifies the Keep a Changelog structure for future releases.
- Extended the import smoke tests to assert error docstrings and top-level availability, rerunning `maturin develop --manifest-path md-splice-py/Cargo.toml --release` followed by `.venv/bin/python -m pytest md-splice-py/tests` successfully.
