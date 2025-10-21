"""Tests for Python error mapping."""

import pytest

from md_splice import MarkdownDocument
from md_splice.errors import FrontmatterParseError


def test_frontmatter_parse_error_is_mapped():
    with pytest.raises(FrontmatterParseError):
        MarkdownDocument.from_string("---\ninvalid: [\n---\nbody\n")
