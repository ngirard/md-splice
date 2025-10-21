"""Tests for the read-only `MarkdownDocument.get` API."""

from __future__ import annotations

from textwrap import dedent

import pytest
import re

from md_splice import MarkdownDocument, Selector
from md_splice.errors import (
    InvalidRegexError,
    RangeRequiresBlockError,
    SectionRequiresHeadingError,
)


def test_get_returns_single_block_by_contains():
    doc = MarkdownDocument.from_string(
        dedent(
            """
            Intro paragraph.

            Another block.
            """
        ).lstrip()
    )

    result = doc.get(Selector(select_contains="Intro"))

    assert result == "Intro paragraph.\n"


def test_get_supports_regex_filter():
    doc = MarkdownDocument.from_string(
        dedent(
            """
            First paragraph.

            Second paragraph with token.
            """
        ).lstrip()
    )

    result = doc.get(Selector(select_regex=r"token\.$"))

    assert result == "Second paragraph with token.\n"


def test_get_section_returns_heading_and_descendants():
    doc = MarkdownDocument.from_string(
        dedent(
            """
            ## Alpha

            Prelude line.

            ### Detail

            Deep dive.

            ## Beta

            Tail text.
            """
        ).lstrip()
    )

    result = doc.get(
        Selector(select_type="h2", select_contains="Alpha"),
        section=True,
    )

    assert result == (
        "## Alpha\n\n"
        "Prelude line.\n\n"
        "### Detail\n\n"
        "Deep dive.\n"
    )


def test_get_section_requires_heading():
    doc = MarkdownDocument.from_string("Paragraph only.\n")

    with pytest.raises(SectionRequiresHeadingError):
        doc.get(Selector(select_type="p"), section=True)


def test_get_with_until_extends_range_to_next_selector():
    doc = MarkdownDocument.from_string(
        dedent(
            """
            ## Alpha

            First paragraph.

            Second paragraph.

            ## Beta

            Tail text.
            """
        ).lstrip()
    )

    result = doc.get(
        Selector(select_type="p", select_contains="First"),
        until=Selector(select_type="h2", select_contains="Beta"),
    )

    assert result == "First paragraph.\n\nSecond paragraph.\n"


def test_get_until_disallows_list_items():
    doc = MarkdownDocument.from_string(
        dedent(
            """
            - Alpha item
            - Beta item
            """
        ).lstrip()
    )

    with pytest.raises(RangeRequiresBlockError):
        doc.get(
            Selector(select_type="li", select_contains="Alpha"),
            until=Selector(select_type="li", select_contains="Beta"),
        )


def test_get_select_all_returns_all_matches():
    doc = MarkdownDocument.from_string(
        dedent(
            """
            - Alpha
            - Beta
            - Gamma
            """
        ).lstrip()
    )

    result = doc.get(Selector(select_type="li"), select_all=True)

    assert result == [" - Alpha\n", " - Beta\n", " - Gamma\n"]


def test_get_select_all_disallows_until():
    doc = MarkdownDocument.from_string("Paragraph.\n")

    with pytest.raises(ValueError):
        doc.get(
            Selector(select_type="p"),
            select_all=True,
            until=Selector(select_type="p", select_contains="Paragraph"),
        )


def test_get_select_all_returns_empty_list_when_no_matches():
    doc = MarkdownDocument.from_string("Paragraph.\n")

    result = doc.get(Selector(select_type="h2"), select_all=True)

    assert result == []


def test_get_respects_regex_ignore_case_flag():
    doc = MarkdownDocument.from_string(
        dedent(
            """
            Paragraph One.

            Another block with beta token.
            """
        ).lstrip()
    )

    pattern = re.compile(r"beta", re.IGNORECASE)
    result = doc.get(Selector(select_type="p", select_regex=pattern))

    assert "beta token" in result


def test_get_respects_regex_multiline_flag():
    doc = MarkdownDocument.from_string(
        dedent(
            """
            ```
            Alpha
            Beta
            ```
            """
        ).lstrip()
    )

    pattern = re.compile(r"^Beta$", re.MULTILINE)
    result = doc.get(Selector(select_type="code", select_regex=pattern))

    assert "Beta" in result


def test_get_respects_regex_dotall_flag():
    doc = MarkdownDocument.from_string(
        dedent(
            """
            ```
            Alpha
            Gamma
            ```
            """
        ).lstrip()
    )

    pattern = re.compile(r"Alpha.*Gamma", re.DOTALL)
    result = doc.get(Selector(select_type="code", select_regex=pattern))

    assert "Alpha" in result and "Gamma" in result


def test_get_rejects_unsupported_regex_flags():
    doc = MarkdownDocument.from_string("Paragraph.\n")
    pattern = re.compile(r"paragraph", re.VERBOSE)

    with pytest.raises(InvalidRegexError):
        doc.get(Selector(select_type="p", select_regex=pattern))
