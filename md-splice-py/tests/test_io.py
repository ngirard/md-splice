from __future__ import annotations

from pathlib import Path
from textwrap import dedent

import pytest

from md_splice import (
    InsertOperation,
    InsertPosition,
    MarkdownDocument,
    ReplaceOperation,
    Selector,
)
from md_splice.errors import IoError


def test_from_file_reads_contents(tmp_path: Path) -> None:
    source_path = tmp_path / "doc.md"
    source_path.write_text("# Heading\n\nParagraph.\n", encoding="utf-8")

    doc = MarkdownDocument.from_file(source_path)

    assert doc.render().rstrip("\n") == "# Heading\n\nParagraph."


def test_write_in_place_persists_changes_atomically(tmp_path: Path) -> None:
    source_path = tmp_path / "tasks.md"
    source_path.write_text(
        dedent(
            """
            # Tasks

            - [ ] Write documentation
            """
        ).lstrip(),
        encoding="utf-8",
    )

    doc = MarkdownDocument.from_file(source_path)
    doc.apply(
        [
            InsertOperation(
                selector=Selector(select_type="li", select_contains="Write documentation"),
                content="- [ ] Ship release",
                position=InsertPosition.AFTER,
            )
        ]
    )

    doc.write_in_place()

    updated = source_path.read_text(encoding="utf-8")
    assert "Ship release" in updated
    assert updated.count("- [ ]") == 2


def test_write_in_place_can_create_backup(tmp_path: Path) -> None:
    source_path = tmp_path / "notes.md"
    original = dedent(
        """
        # Notes

        - Original item
        """
    ).lstrip()
    source_path.write_text(original, encoding="utf-8")

    doc = MarkdownDocument.from_file(source_path)
    doc.apply(
        [
            InsertOperation(
                selector=Selector(select_type="li", select_contains="Original item"),
                content="- Added later",
                position=InsertPosition.AFTER,
            )
        ]
    )

    doc.write_in_place(backup=True)

    backup_path = source_path.with_name(f"{source_path.name}~")
    assert backup_path.exists()
    assert backup_path.read_text(encoding="utf-8") == original

    updated = source_path.read_text(encoding="utf-8")
    assert "Added later" in updated


def test_write_in_place_without_path_raises() -> None:
    doc = MarkdownDocument.from_string("Paragraph.\n")

    with pytest.raises(IoError):
        doc.write_in_place()


def test_write_to_creates_new_file(tmp_path: Path) -> None:
    doc = MarkdownDocument.from_string("Paragraph.\n")
    output_path = tmp_path / "out.md"

    doc.apply(
        [
            ReplaceOperation(
                selector=Selector(select_type="p"),
                content="Rewritten paragraph.",
            )
        ]
    )

    doc.write_to(output_path)

    assert output_path.read_text(encoding="utf-8") == doc.render()


def test_write_to_supports_relative_paths(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.chdir(tmp_path)
    doc = MarkdownDocument.from_string("Paragraph.\n")

    relative_path = Path("relative.md")
    doc.write_to(relative_path)

    assert relative_path.exists()
    assert relative_path.read_text(encoding="utf-8") == doc.render()
