# Strategy.md

This document outlines a practical strategy for implementing the Python library described in `Specification.md`, optimized for an LLM or agentic workflow. The plan embraces test-driven development from the first commit, preserves parity with the Rust core, and continuously updates user-facing documentation so that humans and tools can rely on stable, accurate guidance.

## Purpose and scope

We will deliver a production-quality Python package that wraps the `md-splice-lib` Rust crate using `pyo3` and `maturin`. The package will expose the API, behaviors, and guarantees defined in `Specification.md`, including atomic transactions, selector semantics with `after`/`within`/`until`, section logic for headings, list-item operations, and frontmatter editing with YAML/TOML round-tripping.

## Guiding principles

1. Test first, code second: every capability in `Specification.md` starts as a failing test.
2. Parity over novelty: mirror Rust semantics; do not invent divergent behavior.
3. Safety by default: atomic mutations, precise selectors, and explicit error types.
4. Documentation is part of the deliverable: docs evolve with code in the same PRs.
5. Minimal friction for users: prebuilt wheels, clear import surface, predictable versioning.

## Architecture and module layout

Bind the Rust core directly; do not reimplement logic in Python.

* Native extension: `md_splice/_native` (pyo3).

  * `#[pyclass] MarkdownDocument` holding `ParsedDocument` and `Document`.
  * Converters for selectors, operations, YAML values, and errors.
* Python façade:

  * `md_splice/__init__.py` re-exports public API.
  * `md_splice/types.py` small dataclasses and enums (`Selector`, ops, `InsertPosition`, `FrontmatterFormat`).
  * `md_splice/errors.py` Python exception hierarchy mapped 1-to-1 to Rust errors.
  * `md_splice/io.py` helpers for atomic write and unified diff (delegating to Rust).
* Build system: `pyproject.toml` with `maturin`, Rust in `Cargo.toml`.
* Continuous integration: matrix for Linux/macOS/Windows; build wheels and run tests.

## Implementation plan (TDD milestones)

Each milestone begins by writing tests that encode the acceptance criteria defined in `Specification.md`. Only then implement bindings to make tests pass.

### Milestone 1: project scaffolding

* Write tests:

  * Package imports and version string.
  * `MarkdownDocument.from_string` parses trivial Markdown and renders unchanged.
  * `frontmatter()` returns `None` without a block.
* Implement:

  * `pyproject.toml`, `maturin` config, and minimal `_native` with `MarkdownDocument::from_string`, `render`.
  * Error base class `MdSpliceError`.
* Documentation updates:

  * Add “installation and quickstart” to the Python section in `README.md`, referencing `Specification.md` for the full API.

### Milestone 2: frontmatter parsing and serialization

* Tests (fixtures: YAML and TOML from the Rust tests):

  * Parsing YAML and TOML frontmatter; format detection.
  * Empty frontmatter round-trips and collapses correctly.
  * `frontmatter_format()` reveals the correct enum.
* Implement:

  * Expose `frontmatter()` and `frontmatter_format()` via pyo3 and Python enums.
* Docs:

  * Add examples for reading frontmatter in the “usage” docs and cross-link to the frontmatter section in `Specification.md`.

### Milestone 3: selector modeling and error mapping

* Tests:

  * Construct `Selector` with `select_type`, `select_contains`, `select_regex`, and `select_ordinal`.
  * Invalid regex raises `InvalidRegexError`.
  * Conflicting `after` and `within` raises `ConflictingScopeError`.
* Implement:

  * Python `Selector` dataclass → Rust `locator::Selector` conversion.
  * Exception mapping table from Rust `SpliceError` to Python classes.
* Docs:

  * Document selector composition and the ambiguity behavior, linking to `Specification.md` for the canonical semantics.

### Milestone 4: read-only selection (`get`)

* Tests:

  * Single block selection by type, contains, and regex.
  * Heading section with `section=True`.
  * Range read with `until` (block-level only); list-item `until` raises `RangeRequiresBlockError`.
  * `select_all=True` returns an ordered list of snippets.
* Implement:

  * `MarkdownDocument.get(...)` in native layer using `locator::locate`/`locate_all`, render via `markdown-ppp` printer.
* Docs:

  * Add a “reading without mutating” section with examples.

### Milestone 5: operations and atomic `apply`

* Tests:

  * Insert before/after/append-child/prepend-child on blocks and list items.
  * Replace single node and replace range with `until`.
  * Delete node, delete range, and delete heading section with `section=True`.
  * Invalid child insertion yields `InvalidChildInsertionError`.
  * List-item replace with non-list content yields `InvalidListItemContentError`.
  * Atomicity: second failing op leaves the document unchanged.
  * Ambiguity warning is emitted when multiple matches exist (first match used).
* Implement:

  * Map Python operation dataclasses → Rust `transaction::Operation` variants.
  * Bridge YAML values for frontmatter ops using serde bridging.
  * Call Rust `apply_operations` and refresh frontmatter on mutation.
* Docs:

  * Expand examples for transactions, including an “all-or-nothing” demonstration.

### Milestone 6: frontmatter operations

* Tests:

  * `set_frontmatter` with path creation for mappings; array index bounds are enforced.
  * `delete_frontmatter` removes empty containers and collapses an empty block.
  * `replace_frontmatter` to YAML and TOML with format hint behavior.
  * Missing key deletion raises `FrontmatterKeyNotFoundError`.
* Implement:

  * Bind `SetFrontmatterOperation`, `DeleteFrontmatterOperation`, `ReplaceFrontmatterOperation`.
* Docs:

  * Cookbook recipes for common metadata tasks.

### Milestone 7: serialization of operations and diffs

* Tests:

  * `loads_operations` accepts YAML and JSON schemas identical to the CLI and round-trips with `dumps_operations`.
  * `preview` returns rendered text without mutating the document.
  * `diff_unified` yields stable headers `original`/`modified`.
* Implement:

  * Serde load/dump in Rust or Python depending on complexity; ensure parity with CLI schema.
  * Diff powered by Rust `similar`.
* Docs:

  * “From ops files to Python” section with examples, and “preview and diff” workflow.

### Milestone 8: file I/O and wheels

* Tests:

  * `write_in_place` is atomic (simulate crash by not calling, but verify temp strategy) and optional `backup`.
  * `write_to` writes elsewhere without modifying the source.
  * Wheels import on all CI targets; smoke tests.
* Implement:

  * Atomic write using Rust `tempfile` and rename semantics.
  * CI matrix for building wheels and running tests.
* Docs:

  * Add platform support matrix and guidance on offline installation.

## Test strategy in detail

We will adhere to strict TDD:

1. Write failing tests that assert the behavior defined in `Specification.md`.
2. Implement the minimal binding to pass those tests.
3. Refactor only with green tests.

Test types:

* **Unit tests (pytest)**: constructor validations, error mapping, data conversions, I/O helpers.
* **Golden tests**: snapshot the rendered Markdown for representative operations. Keep these synchronized with Rust `insta` snapshots to ensure printer parity; store compact fixtures to avoid brittle diffs.
* **Parity tests with CLI**: for a subset of scenarios, run the CLI (optional in CI) against fixtures and assert that Python `render()` matches CLI output byte-for-byte.
* **Property tests (hypothesis)**: idempotence of `render(parse(markdown))`, and atomicity invariant (on any failing op sequence, `before == after`).
* **Performance checks**: large document smoke tests to detect regressions.

Test layout:

```
tests/
  test_imports.py
  test_frontmatter.py
  test_selectors.py
  test_get.py
  test_apply_blocks.py
  test_apply_list_items.py
  test_frontmatter_ops.py
  test_ops_io.py
  test_diff_and_preview.py
  test_io_atomic.py
  snapshots/  # golden outputs
  fixtures/   # md sources with and without frontmatter
```

## Documentation and developer experience

Documentation tasks accompany every milestone, and PRs must update docs alongside code.

* **Specification reference**: `Strategy.md` relies on `Specification.md` as the normative API contract. Each user-facing doc section links back to the relevant section of `Specification.md`.
* **User docs**:

  * Update `README.md` with installation, quickstart, and short examples.
  * Create `docs/` pages (or extend the README) for:

    * Selectors and scoping.
    * Transactions and atomicity.
    * Frontmatter operations.
    * Diffing and preview.
    * Interop with CLI and ops files.
* **Docstrings**: comprehensive docstrings for all public classes and methods, mirroring `Specification.md`.
* **Changelog**: maintain `CHANGELOG.md` with sections for “Added/Changed/Fixed,” referencing PRs and linking to `Specification.md` when semantics are involved.
* **Examples**: runnable examples under `examples/` kept in CI to prevent drift.
* **Versioning notice**: document that Python versions mirror Rust crate versions; note any `.postN` build tags for packaging-only changes.

## Continuous integration and release management

* **CI jobs**:

  * Lint: `ruff`, `black` (check), and `mypy` (if we type-annotate façade).
  * Build: `maturin build` for sdist and wheels on Linux/macOS/Windows (x86_64 and arm64 where supported).
  * Test: run pytest across wheels; optionally run a subset against the CLI binary.
* **Release**:

  * Tag `v0.5.x` aligned with the Rust crate.
  * Publish wheels to PyPI and update `README.md` badges.
  * Definition of done includes passing CI, updated docs, and an entry in `CHANGELOG.md`.

## Error mapping and diagnostics

* Implement a single mapping layer in Rust (`impl From<SpliceError> for PyErr`) to produce precise Python subclasses.
* When selectors match multiple nodes, surface a Python `warnings.warn` unless the caller sets `warn_on_ambiguity=False`, mirroring the Rust log behavior.
* Ensure error messages retain the actionable details from Rust (e.g., which key path failed, which type was invalid).

## Risk management and mitigations

* **Parser/printer drift**: keep snapshot tests tied to Rust `insta` fixtures and pin `markdown-ppp` to the same version as the Rust crates.
* **Regex dialect mismatch**: document supported flags and fail early on unsupported Python `re` flags.
* **YAML/TOML fidelity**: clearly document that in-memory values are YAML; TOML round-trip uses conversion and may reorder keys; provide examples.
* **Windows atomic writes**: cover with integration tests and use established `tempfile` + replace semantics.

## Contribution workflow optimized for agents

* Each feature branch begins with tests derived from `Specification.md` examples.
* Commit messages follow Conventional Commits (`feat:`, `fix:`, `docs:`) with a trailing reference to the spec section.
* Open a PR with:

  * Checklist: tests passing, docs updated, examples updated, changelog entry added.
  * Links to the exact `Specification.md` paragraphs implemented.
* Require review approval before merge; disallow direct pushes to `main`.

## Milestone acceptance criteria

A milestone is complete when all of the following are true:

1. All new tests pass locally and in CI; coverage on the new module is ≥ 90% lines.
2. User-facing documentation is updated and links to `Specification.md`.
3. Public API remains consistent with `Specification.md` and semantically matches the Rust core on the tested fixtures.
4. Wheels are built successfully for all target platforms.
5. Changelog is updated.

## Roadmap beyond the initial delivery

* Expose an optional structured “AST view” read-only API for inspection (without committing to stable internal types).
* Provide a thin synchronous CLI shim for Python users who prefer shell pipelines.
* Add async helpers for file batches (still single-threaded inside the Rust core, but convenient orchestration).

---

By following this strategy, an LLM can iteratively realize the Python bindings defined in `Specification.md`, while guaranteeing correctness through test-driven development and keeping the user documentation accurate at every step.
