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
