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
creates a `.bak` sibling of the current file before atomically swapping in the new
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
