"""Python bindings for the md-splice Markdown editing toolkit.

The package re-exports the native ``MarkdownDocument`` type, helper operations,
and the rich exception hierarchy defined for the Rust core bindings.
"""

from . import errors
from .errors import (
    AmbiguousContentSourceError,
    AmbiguousStdinSourceError,
    ConflictingScopeError,
    FrontmatterKeyNotFoundError,
    FrontmatterMissingError,
    FrontmatterParseError,
    FrontmatterSerializeError,
    InvalidChildInsertionError,
    InvalidListItemContentError,
    InvalidRegexError,
    InvalidSectionDeleteError,
    IoError,
    MarkdownParseError,
    MdSpliceError,
    NoContentError,
    NodeNotFoundError,
    OperationFailedError,
    OperationParseError,
    RangeRequiresBlockError,
    SectionRequiresHeadingError,
)
from ._native import (
    MarkdownDocument,
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

__all__.extend(
    [
        "MdSpliceError",
        "NodeNotFoundError",
        "InvalidChildInsertionError",
        "AmbiguousContentSourceError",
        "NoContentError",
        "InvalidListItemContentError",
        "AmbiguousStdinSourceError",
        "InvalidSectionDeleteError",
        "SectionRequiresHeadingError",
        "ConflictingScopeError",
        "RangeRequiresBlockError",
        "FrontmatterMissingError",
        "FrontmatterKeyNotFoundError",
        "FrontmatterParseError",
        "FrontmatterSerializeError",
        "MarkdownParseError",
        "OperationParseError",
        "OperationFailedError",
        "IoError",
        "InvalidRegexError",
        "errors",
    ]
)
