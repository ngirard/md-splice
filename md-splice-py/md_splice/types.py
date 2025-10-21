"""Public data types for the md-splice Python bindings."""

from __future__ import annotations

from enum import Enum


class FrontmatterFormat(str, Enum):
    """Frontmatter serialization format detected in a Markdown document."""

    YAML = "yaml"
    TOML = "toml"


__all__ = ["FrontmatterFormat"]
