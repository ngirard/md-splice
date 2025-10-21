"""Python bindings for the md-splice Markdown editing toolkit."""

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
    "__version__",
]
