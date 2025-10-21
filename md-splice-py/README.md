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
