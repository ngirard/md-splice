"""Public data types for the md-splice Python bindings."""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum
import re
from typing import Any, Pattern, Union

from .errors import ConflictingScopeError, InvalidRegexError


class FrontmatterFormat(str, Enum):
    """Frontmatter serialization format detected in a Markdown document."""

    YAML = "yaml"
    TOML = "toml"


class InsertPosition(str, Enum):
    """Insertion position relative to a selector target."""

    BEFORE = "before"
    AFTER = "after"
    PREPEND_CHILD = "prepend_child"
    APPEND_CHILD = "append_child"


@dataclass(frozen=True, slots=True)
class Selector:
    """Criteria for locating Markdown nodes via the Rust core.

    Regex selectors accept either pattern strings or compiled ``re.Pattern``
    instances. The bridge honors ``re.IGNORECASE``, ``re.MULTILINE``, and
    ``re.DOTALL`` flags exactly as Python applies them before handing the
    pattern to Rust, and tolerates ``re.UNICODE`` (Python's default). Any other
    flag (e.g. ``re.VERBOSE`` or ``re.ASCII``) raises
    :class:`md_splice.errors.InvalidRegexError`, matching the limitations
    described in ``goal-Python-library/Specification.md``.
    """

    select_type: str | None = None
    select_contains: str | None = None
    select_regex: Pattern[str] | str | None = field(default=None, repr=False)
    select_ordinal: int = 1
    after: Selector | None = None
    within: Selector | None = None

    def __post_init__(self) -> None:  # noqa: D401 - dataclass validation hook
        if self.after is not None and self.within is not None:
            raise ConflictingScopeError(
                "Selector cannot specify both 'after' and 'within' scopes."
            )

        if self.select_ordinal < 1:
            raise ValueError("select_ordinal must be a positive integer")

        pattern = self.select_regex
        if isinstance(pattern, str):
            try:
                compiled = re.compile(pattern)
            except re.error as exc:  # pragma: no cover - exercised in tests
                raise InvalidRegexError(str(exc)) from exc
            object.__setattr__(self, "select_regex", compiled)
        elif pattern is None or isinstance(pattern, re.Pattern):
            # Already compiled or absent; no action needed.
            pass
        else:  # pragma: no cover - defensive branch
            raise TypeError(
                "select_regex must be a str, compiled Pattern, or None"
            )


@dataclass(frozen=True, slots=True)
class InsertOperation:
    """Insert Markdown content relative to a selector.

    ``position`` controls where the new content lands with respect to the
    matched node (before, after, or as a child), matching the CLI schema
    defined in ``goal-Python-library/Specification.md``.
    """

    selector: Selector
    content: str | None = None
    position: InsertPosition = InsertPosition.AFTER


@dataclass(frozen=True, slots=True)
class ReplaceOperation:
    """Replace Markdown matched by a selector, optionally up to ``until``.

    When ``until`` is provided the replacement covers the range from the
    selector through (but excluding) the ``until`` target, mirroring the Rust
    transaction semantics.
    """

    selector: Selector
    content: str | None = None
    until: Selector | None = None


@dataclass(frozen=True, slots=True)
class DeleteOperation:
    """Delete Markdown matched by a selector.

    Setting ``section=True`` removes the entire heading section for a heading
    match. Providing ``until`` deletes a range ending before the ``until``
    selector. Both behaviors mirror the CLI and Rust core.
    """

    selector: Selector
    section: bool = False
    until: Selector | None = None


@dataclass(frozen=True, slots=True)
class SetFrontmatterOperation:
    """Assign a value at the given frontmatter key path.

    Nested keys follow dot and array notation (for example ``authors[0].name``)
    and accept native Python values that are converted to YAML/TOML by the Rust
    layer.
    """

    key: str
    value: Any
    format: FrontmatterFormat | None = None


@dataclass(frozen=True, slots=True)
class DeleteFrontmatterOperation:
    """Remove a key from document frontmatter."""

    key: str


@dataclass(frozen=True, slots=True)
class ReplaceFrontmatterOperation:
    """Replace the entire frontmatter payload.

    ``content`` should be a Python mapping or scalar that can be serialized to
    YAML or TOML. ``format`` only applies when the document previously lacked
    frontmatter, aligning with the specification's rules for new blocks.
    """

    content: Any
    format: FrontmatterFormat | None = None


Operation = Union[
    InsertOperation,
    ReplaceOperation,
    DeleteOperation,
    SetFrontmatterOperation,
    DeleteFrontmatterOperation,
    ReplaceFrontmatterOperation,
]


__all__ = [
    "FrontmatterFormat",
    "InsertPosition",
    "Selector",
    "InsertOperation",
    "ReplaceOperation",
    "DeleteOperation",
    "SetFrontmatterOperation",
    "DeleteFrontmatterOperation",
    "ReplaceFrontmatterOperation",
    "Operation",
]
