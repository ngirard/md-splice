update 01: Reviewed specification and existing codebase. Preparing to implement `get` command following Strategy.md phases (starting with CLI scaffolding and baseline tests).
update 02: Added CLI scaffolding, `process_get`, rendering helpers, and initial integration test for single-node retrieval. Basic `get` command now returns Markdown to stdout without mutating files.
update 03: Implemented `--section` handling with section rendering and error reporting; expanded tests to cover heading sections and invalid usage.
update 04: Introduced multi-node support via `--select-all`/`--separator`, created `locate_all`, enhanced list-item text handling for task selectors, and covered cases with new tests.
update 05: Updated CLI help snapshot, refreshed README with `get` command documentation, and ran full test suite (all passing).
