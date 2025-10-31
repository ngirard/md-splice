"""Python exception hierarchy for the md-splice bindings.

Each exception mirrors a Rust ``SpliceError`` variant so Python callers can
write precise ``except`` clauses. The base :class:`MdSpliceError` is surfaced
from the native extension but documented here for discoverability.
"""

from __future__ import annotations

from ._native import MdSpliceError

if not MdSpliceError.__doc__:
    MdSpliceError.__doc__ = (
        "Base class for all exceptions raised by the md-splice Python bindings."
    )


class NodeNotFoundError(MdSpliceError):
    """Raised when a selector fails to match any nodes in the document."""


class InvalidChildInsertionError(MdSpliceError):
    """Raised when attempting to insert a child into an incompatible node."""


class AmbiguousContentSourceError(MdSpliceError):
    """Raised when multiple mutually-exclusive content sources are provided."""


class NoContentError(MdSpliceError):
    """Raised when an operation that requires content receives none."""


class InvalidListItemContentError(MdSpliceError):
    """Raised when list-item operations receive non-list Markdown content."""


class AmbiguousStdinSourceError(MdSpliceError):
    """Raised when both the source document and splice content read from stdin."""


class InvalidSectionDeleteError(MdSpliceError):
    """Raised when deleting a section from a non-heading target."""


class SectionRequiresHeadingError(MdSpliceError):
    """Raised when section semantics are requested on a non-heading selector."""


class ConflictingScopeError(MdSpliceError):
    """Raised when `after` and `within` scopes are combined in a selector."""


class RangeRequiresBlockError(MdSpliceError):
    """Raised when range selectors are applied to non-block selections."""


class FrontmatterMissingError(MdSpliceError):
    """Raised when attempting to mutate or read frontmatter that does not exist."""


class FrontmatterKeyNotFoundError(MdSpliceError):
    """Raised when a requested frontmatter key path is absent."""


class FrontmatterParseError(MdSpliceError):
    """Raised when frontmatter parsing fails."""


class FrontmatterSerializeError(MdSpliceError):
    """Raised when converting Python values to frontmatter fails."""


class MarkdownParseError(MdSpliceError):
    """Raised when the Markdown source cannot be parsed."""


class OperationParseError(MdSpliceError):
    """Raised when an operations file cannot be parsed."""


class OperationFailedError(MdSpliceError):
    """Raised when an operation fails during execution."""


class IoError(MdSpliceError):
    """Raised for underlying I/O errors."""


class InvalidRegexError(MdSpliceError):
    """Raised when a provided regular expression pattern is invalid."""


class SelectorAliasNotDefinedError(MdSpliceError):
    """Raised when a selector alias reference has not been defined."""


class SelectorAliasAlreadyDefinedError(MdSpliceError):
    """Raised when attempting to redefine an existing selector alias."""


class AmbiguousSelectorSourceError(MdSpliceError):
    """Raised when both selector and selector_ref inputs are provided."""


class AmbiguousNestedSelectorSourceError(MdSpliceError):
    """Raised when nested selectors specify both inline and referenced handles."""


__all__ = [
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
    "SelectorAliasNotDefinedError",
    "SelectorAliasAlreadyDefinedError",
    "AmbiguousSelectorSourceError",
    "AmbiguousNestedSelectorSourceError",
]
