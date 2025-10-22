# md-splice Python bindings

This package provides the Python interface for the `md-splice` Markdown editing toolkit. See
`goal-Python-library/Specification.md` for the normative API contract.

## Quick usage

```python
from md_splice import (
    InsertOperation,
    InsertPosition,
    MarkdownDocument,
    Selector,
    dumps_operations,
    loads_operations,
)

doc = MarkdownDocument.from_file("README.md")
operations = loads_operations("""
- op: insert
  selector:
    select_type: h2
    select_contains: Changelog
  position: after
  content: |
    ## Release notes
    - Initial Python bindings
""")

doc.apply(operations)
doc.write_in_place()  # atomic write back to README.md

serialized = dumps_operations(operations, format="json")
print(serialized)
```

The `MarkdownDocument` class also exposes `write_to(path)` for directing output to a
new file, and the operations helpers round-trip between the Python dataclasses and the
YAML/JSON schema shared with the CLI tooling.

## Transactions and atomic writes

Operations passed to `MarkdownDocument.apply()` run inside a single transaction. If
any operation fails the document reverts to its pre-transaction state, matching the
"all-or-nothing" semantics defined in
[`goal-Python-library/Specification.md`](../goal-Python-library/Specification.md).
Ambiguity warnings mirror the CLI: when a selector matches multiple nodes a
`UserWarning` is emitted unless `warn_on_ambiguity=False`.

Persisting edits uses atomic file replacement. `write_in_place(backup=True)` first
creates a `path~` sibling of the current file before atomically swapping in the new
content, satisfying the safety guarantees described in the specification. Use
`write_to(path)` to atomically write to a new location.

## Preview and diff helpers

`MarkdownDocument.preview()` simulates a transaction on a clone and returns the
rendered Markdown without mutating the original document. To inspect textual changes
between two versions, call `md_splice.diff_unified(before, after, fromfile=...,` from
Python and display the resulting unified diff. The helper matches the CLI diff output
and accepts optional labels for the "from" and "to" files.

## Operations schema interop

The Python dataclasses in `md_splice.types` map directly to the CLI operations schema.
Use `loads_operations(text, format="yaml" | "json")` to parse YAML or JSON operation
files into the typed dataclasses, and `dumps_operations(ops, format="yaml")` to emit a
schema-compatible string. Omitting `format` while loading attempts YAML first and then
JSON, mirroring the CLI's flexibility.

## Regex selector flags

`Selector.select_regex` accepts either pattern strings or compiled `re.Pattern`
objects. The bindings translate Python flags to Rust's regex engine and honor
`re.IGNORECASE`, `re.MULTILINE`, and `re.DOTALL` exactly as Python would when
evaluating the pattern (with `re.UNICODE` tolerated as Python's default). Any
other flag—such as `re.VERBOSE`, `re.ASCII`, `re.LOCALE`, or `re.DEBUG`—will
raise `md_splice.errors.InvalidRegexError`, matching the limitations outlined
in `goal-Python-library/Specification.md`.

## Building distributable artifacts

The project ships both source distributions and wheels generated via
[`maturin`](https://github.com/PyO3/maturin). To build release artifacts aligned
with the Rust `0.5.0` tag:

```bash
python -m pip install maturin
python -m maturin build --release          # wheels for the current interpreter
python -m maturin sdist                     # source distribution with metadata
```

The resulting files appear under `target/` (for wheels) and `dist/` (for the
sdist). Both artifacts embed the synchronized version number and include the
project license, changelog, and README so that PyPI consumers receive the same
context as the Rust crate.

## Continuous integration for wheels

The repository's [**Python Wheels** workflow](../.github/workflows/python-wheels.yml)
builds and tests distributable artifacts on every push and pull request. The
matrix covers manylinux (x86_64 and aarch64), musllinux (x86_64), macOS (x86_64
and arm64), and Windows (x86_64) across Python 3.8 through 3.12. Each job
installs the freshly built wheel into a clean virtual environment and runs the
full pytest suite copied into a temporary directory so imports exercise the
published package. The resulting wheels and sdist are uploaded as workflow
artifacts, making it straightforward to promote the verified artifacts to PyPI
once a release tag is cut.

## Publishing releases

Cutting a Git tag matching `v*` re-runs the wheel matrix and, when the
`PYPI_API_TOKEN` repository secret is present, automatically uploads all wheels
and the source distribution to PyPI via `pypa/gh-action-pypi-publish`. The token
should be a scoped PyPI API token with upload rights for the `md-splice` project
and can be rotated without touching the workflow. If the secret is absent the
publish step is skipped, allowing dry-run release rehearsals before credentials
are provisioned.
