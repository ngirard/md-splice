"""Property-style tests covering invariants from the specification."""

import pytest
from hypothesis import given, strategies as st

from md_splice import DeleteOperation, MarkdownDocument, ReplaceOperation, Selector
from md_splice.errors import OperationFailedError


text_strategy = st.text(
    alphabet=st.characters(blacklist_categories=("Cs",)),
    min_size=0,
    max_size=128,
)


@given(text_strategy)
def test_apply_failure_keeps_document_unchanged(replacement_text: str) -> None:
    """Applying ops that fail must leave the document unchanged."""
    doc = MarkdownDocument.from_string("Paragraph.\n")
    original = doc.render()

    with pytest.raises(OperationFailedError):
        doc.apply(
            [
                ReplaceOperation(
                    selector=Selector(select_type="p"),
                    content=replacement_text,
                ),
                DeleteOperation(
                    selector=Selector(select_type="h2", select_contains="missing"),
                ),
            ],
            warn_on_ambiguity=False,
        )

    assert doc.render() == original


@given(text_strategy)
def test_preview_never_mutates_original(replacement_text: str) -> None:
    """Preview should not mutate the original document regardless of content."""
    doc = MarkdownDocument.from_string("Paragraph.\n")
    original = doc.render()

    preview = doc.preview(
        [
            ReplaceOperation(
                selector=Selector(select_type="p"),
                content=replacement_text,
            )
        ],
        warn_on_ambiguity=False,
    )

    assert preview != original or replacement_text == "Paragraph.\n"
    assert doc.render() == original
