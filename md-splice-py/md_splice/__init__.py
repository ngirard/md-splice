"""Python bindings for the md-splice Markdown editing toolkit.

This module mirrors the public surface documented in
``goal-Python-library/Specification.md`` by re-exporting the native
``MarkdownDocument`` type, operation helpers, and the rich exception hierarchy.
"""

from . import errors as errors
from ._native import (
    MarkdownDocument,
    MdSpliceError,
    __version__,
    diff_unified,
    dumps_operations,
    loads_operations,
)
from .types import (
    DeleteFrontmatterOperation,
    DeleteOperation,
    FrontmatterFormat,
    InsertOperation,
    InsertPosition,
    Operation,
    ReplaceFrontmatterOperation,
    ReplaceOperation,
    Selector,
    SetFrontmatterOperation,
)

__all__ = [
    "MarkdownDocument",
    "MdSpliceError",
    "diff_unified",
    "loads_operations",
    "dumps_operations",
    "InsertPosition",
    "Selector",
    "FrontmatterFormat",
    "InsertOperation",
    "ReplaceOperation",
    "DeleteOperation",
    "SetFrontmatterOperation",
    "DeleteFrontmatterOperation",
    "ReplaceFrontmatterOperation",
    "Operation",
    "errors",
    "__version__",
]

for _name in errors.__all__:
    globals()[_name] = getattr(errors, _name)
    if _name not in __all__:
        __all__.append(_name)

del _name
