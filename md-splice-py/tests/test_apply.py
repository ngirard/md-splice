"""Tests covering transactional operations exposed via `MarkdownDocument.apply`."""

from __future__ import annotations

import warnings
from textwrap import dedent

import pytest

from md_splice import (
    DeleteFrontmatterOperation,
    DeleteOperation,
    FrontmatterFormat,
    InsertOperation,
    InsertPosition,
    MarkdownDocument,
    ReplaceOperation,
    Selector,
    SetFrontmatterOperation,
    diff_unified,
)
from md_splice.errors import OperationFailedError


def test_apply_insert_and_render_updates_document() -> None:
    doc = MarkdownDocument.from_string(
        dedent(
            """
            # Tasks

            - [ ] Write documentation
            """
        ).lstrip()
    )

    doc.apply(
        [
            InsertOperation(
                selector=Selector(select_type="li", select_contains="Write documentation"),
                content="- [ ] Add integration tests",
                position=InsertPosition.BEFORE,
            )
        ]
    )

    rendered = doc.render()
    assert "Add integration tests" in rendered
    assert rendered.count("- [ ]") == 2


def test_apply_insert_preserves_list_marker_spacing() -> None:
    doc = MarkdownDocument.from_string(
        dedent(
            """
            # Lorem

            ## Changelog
            Ipsum

            ## Dolor
            Sit amet
            """
        ).lstrip()
    )

    doc.apply(
        [
            InsertOperation(
                selector=Selector(select_type="h2", select_contains="Changelog"),
                content=dedent(
                    """
                    ## Release notes
                    - Initial Python bindings
                    """
                ).strip(),
                position=InsertPosition.AFTER,
            )
        ]
    )

    rendered = doc.render()
    assert "## Release notes\n\n- Initial Python bindings" in rendered
    assert "\n - Initial Python bindings" not in rendered


def test_apply_replace_until_range_substitutes_multiple_blocks() -> None:
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

    doc.apply(
        [
            ReplaceOperation(
                selector=Selector(select_type="p", select_contains="First"),
                content="Replacement paragraph.",
                until=Selector(select_type="h2", select_contains="Beta"),
            )
        ]
    )

    rendered = doc.render()
    assert "Replacement paragraph." in rendered
    assert "Second paragraph" not in rendered


def test_apply_delete_section_removes_heading_and_body() -> None:
    doc = MarkdownDocument.from_string(
        dedent(
            """
            ## Alpha

            Context

            ### Nested

            Details

            ## Beta
            """
        ).lstrip()
    )

    doc.apply(
        [
            DeleteOperation(
                selector=Selector(select_type="h2", select_contains="Alpha"),
                section=True,
            )
        ]
    )

    rendered = doc.render()
    assert "Alpha" not in rendered
    assert "Nested" not in rendered
    assert "Beta" in rendered


def test_apply_warns_on_ambiguity() -> None:
    doc = MarkdownDocument.from_string("Paragraph one.\n\nParagraph two.\n")

    with pytest.warns(UserWarning):
        doc.apply(
            [
                ReplaceOperation(
                    selector=Selector(select_type="p"),
                    content="Updated paragraph.",
                )
            ]
        )

    no_warning_doc = MarkdownDocument.from_string("Paragraph one.\n\nParagraph two.\n")
    with warnings.catch_warnings(record=True) as captured:
        warnings.simplefilter("always")
        no_warning_doc.apply(
            [
                ReplaceOperation(
                    selector=Selector(select_type="p"),
                    content="Updated paragraph.",
                )
            ],
            warn_on_ambiguity=False,
        )

    assert captured == []


def test_apply_frontmatter_operations_modify_payload() -> None:
    doc = MarkdownDocument.from_string(
        dedent(
            """
            ---
            title: Sample
            reviewers: []
            ---

            Body
            """
        ).lstrip()
    )

    doc.apply(
        [
            SetFrontmatterOperation(
                key="reviewers",
                value=[{"name": "Ada"}],
                format=FrontmatterFormat.YAML,
            ),
            DeleteFrontmatterOperation(key="title"),
        ]
    )

    frontmatter = doc.frontmatter()
    assert frontmatter == {"reviewers": [{"name": "Ada"}]}
    assert doc.frontmatter_format() == FrontmatterFormat.YAML


def test_apply_is_atomic_on_failure() -> None:
    doc = MarkdownDocument.from_string("Paragraph.\n")
    original = doc.render()

    with pytest.raises(OperationFailedError):
        doc.apply(
            [
                ReplaceOperation(
                    selector=Selector(select_type="p"),
                    content="Updated.",
                ),
                DeleteOperation(
                    selector=Selector(select_type="h2"),
                ),
            ]
        )

    assert doc.render() == original


def test_preview_returns_transformed_string_without_mutating() -> None:
    doc = MarkdownDocument.from_string("Paragraph.\n")

    preview = doc.preview(
        [
            ReplaceOperation(
                selector=Selector(select_type="p"),
                content="Updated.",
            )
        ]
    )

    assert preview.strip() == "Updated."
    assert doc.render().strip() == "Paragraph."


def test_diff_unified_includes_custom_headers() -> None:
    before = "Line one\nLine two\n"
    after = "Line one\nLine three\n"

    diff = diff_unified(before, after, fromfile="before.md", tofile="after.md")

    assert diff.startswith("--- before.md\n+++ after.md")
    assert "-Line two" in diff
    assert "+Line three" in diff
