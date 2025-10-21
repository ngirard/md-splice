"""Public data types for the md-splice Python bindings."""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum
import re
from typing import Pattern

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
    """Criteria for locating Markdown nodes via the Rust core."""

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


__all__ = ["FrontmatterFormat", "InsertPosition", "Selector"]
