# Progress Status

update 01
- Scaffolded the `md-splice` Python package with `pyproject.toml`, Rust extension crate, and minimal Python API surface.
- Added initial pytest covering import/version, Markdown parse/render round-trip, frontmatter `None`, and base error exposure.
- Verified compilation via `maturin develop` and passing tests with `python -m pytest tests` inside the new virtual environment.
- Next: flesh out frontmatter bridging, richer error mapping, and additional API surface per Milestone 1/2 of `Strategy.md`.
