# Changelog

All notable changes to the `md-splice` Python bindings are tracked in this file.
The format loosely follows `Keep a Changelog <https://keepachangelog.com/>`_
and the project adheres to semantic versioning in lockstep with the Rust core.

## [Unreleased]

### Packaging
- Added complete project metadata (license, homepage, repository URLs) to the
  Python `pyproject.toml` and Rust crates to satisfy `cargo package` checks.
- Include the license, README, and changelog in source distributions
  so PyPI uploads mirror the Rust release.

## [0.5.0] - 2025-10-21

### Added
- Comprehensive bindings for the Rust `MarkdownDocument`, including selectors,
  transactional `apply`/`preview`, diff helpers, and frontmatter access.
- Dataclass-based operation model that mirrors the CLI schema and round-trips
  through ``loads_operations``/``dumps_operations`` in YAML or JSON formats.
- File I/O ergonomics: ``from_file``, ``write_in_place`` with optional backups,
  and ``write_to`` for atomic writes to new locations.
- Exception hierarchy that maps Rust ``SpliceError`` variants to Python classes
  for precise error handling.

### Changed
- ``write_in_place(backup=True)`` now creates ``path~`` backups to match the
  specification's durability requirements.
- Published the Python package at ``0.5.0`` to mirror the Rust core version.

### Documentation
- Detailed docstrings across the public API, including selector flag semantics
  and transactional guarantees.
- README quickstart covering installation, operations schema interop, preview
  flows, and supported regular expression flags.
