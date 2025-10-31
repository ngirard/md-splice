from __future__ import annotations

from textwrap import dedent
import warnings

import pytest

from md_splice import (
    InsertOperation,
    InsertPosition,
    MarkdownDocument,
    ReplaceOperation,
    Selector,
)
from md_splice.ctx import MdBatchEdit, MdEdit
from md_splice.errors import OperationFailedError


def _write_sample(path) -> None:
    path.write_text(
        dedent(
            """
            # Heading

            Paragraph one.

            Paragraph two.
            """
        ).lstrip(),
        encoding="utf-8",
    )


def _make_replace_operation() -> ReplaceOperation:
    return ReplaceOperation(
        selector=Selector(select_type="p", select_contains="Paragraph one."),
        content="Updated paragraph.\n",
    )


def _make_ambiguous_operation() -> ReplaceOperation:
    return ReplaceOperation(
        selector=Selector(select_type="p"),
        content="Updated paragraph.\n",
    )


def test_no_commit_on_exception(tmp_path) -> None:
    target = tmp_path / "doc.md"
    _write_sample(target)
    original = target.read_text(encoding="utf-8")

    with pytest.raises(RuntimeError, match="boom"):
        with MdEdit(target) as doc:
            doc.apply([_make_replace_operation()])
            raise RuntimeError("boom")

    assert target.read_text(encoding="utf-8") == original
    assert not target.with_name("doc.md~").exists()


def test_commit_on_clean_exit_creates_backup(tmp_path) -> None:
    target = tmp_path / "doc.md"
    _write_sample(target)
    backup_path = tmp_path / "doc.md~"

    with MdEdit(target) as doc:
        doc.apply(
            [
                InsertOperation(
                    selector=Selector(select_type="h1"),
                    position=InsertPosition.AFTER,
                    content="Inserted line.\n",
                )
            ]
        )

    assert "Inserted line." in target.read_text(encoding="utf-8")
    assert backup_path.exists()
    assert "Inserted line." not in backup_path.read_text(encoding="utf-8")


def test_stale_write_refused(tmp_path) -> None:
    target = tmp_path / "doc.md"
    _write_sample(target)

    with pytest.raises(RuntimeError, match="file changed"):
        with MdEdit(target) as doc:
            doc.apply([_make_replace_operation()])
            target.write_text("External change.\n", encoding="utf-8")

    assert target.read_text(encoding="utf-8") == "External change.\n"


def test_ambiguity_escalates_by_default(tmp_path) -> None:
    target = tmp_path / "doc.md"
    _write_sample(target)

    with pytest.raises(OperationFailedError, match="Ambiguity detected"):
        with MdEdit(target) as doc:
            doc.apply([_make_ambiguous_operation()])

    # Allow ambiguity when explicitly disabled and surface warning.
    _write_sample(target)
    with warnings.catch_warnings(record=True) as captured:
        warnings.simplefilter("always")
        with MdEdit(target, fail_on_ambiguity=False) as doc:
            doc.apply([_make_ambiguous_operation()])
    assert captured
    assert any("matched multiple nodes" in str(warning.message) for warning in captured)


def test_diff_preview_prints_unified_diff(tmp_path, capsys) -> None:
    target = tmp_path / "doc.md"
    _write_sample(target)

    with MdEdit(target, preview_diff=True) as doc:
        doc.apply([_make_replace_operation()])

    output = capsys.readouterr().out
    assert "--- original" in output
    assert "+++ modified" in output


def test_batch_applies_once(monkeypatch, tmp_path) -> None:
    target = tmp_path / "doc.md"
    _write_sample(target)

    calls: list[int] = []
    original_apply = MarkdownDocument.apply

    def counting_apply(self, operations, *args, **kwargs):
        calls.append(1)
        return original_apply(self, operations, *args, **kwargs)

    monkeypatch.setattr(MarkdownDocument, "apply", counting_apply)

    with MdBatchEdit(target) as edit:
        edit.apply(_make_replace_operation())
        edit.apply(
            InsertOperation(
                selector=Selector(select_type="h1"),
                position=InsertPosition.AFTER,
                content="Appended line.\n",
            )
        )

    assert len(calls) == 1
    assert "Updated paragraph." in target.read_text(encoding="utf-8")
    assert "Appended line." in target.read_text(encoding="utf-8")


def test_nested_same_path_rejected(tmp_path) -> None:
    target = tmp_path / "doc.md"
    _write_sample(target)

    with MdEdit(target):
        with pytest.raises(RuntimeError, match="already active"):
            with MdEdit(target):
                pass


def test_stdin_path_rejected() -> None:
    with pytest.raises(ValueError):
        MdEdit("-")


def test_commit_flag_false_skips_write(tmp_path) -> None:
    target = tmp_path / "doc.md"
    _write_sample(target)
    original = target.read_text(encoding="utf-8")

    with MdEdit(target, commit=False) as doc:
        doc.apply([_make_replace_operation()])

    assert target.read_text(encoding="utf-8") == original
    assert not target.with_name("doc.md~").exists()


def test_warning_filter_scope(tmp_path) -> None:
    target = tmp_path / "doc.md"
    _write_sample(target)

    outside_doc = MarkdownDocument.from_file(str(target))
    with warnings.catch_warnings(record=True) as captured:
        warnings.simplefilter("always")
        outside_doc.apply([_make_ambiguous_operation()])
    assert captured

    _write_sample(target)
    with pytest.raises(OperationFailedError):
        with MdEdit(target) as doc:
            doc.apply([_make_ambiguous_operation()])

    fresh_doc = MarkdownDocument.from_file(str(target))
    with warnings.catch_warnings(record=True) as captured_again:
        warnings.simplefilter("always")
        fresh_doc.apply([_make_ambiguous_operation()])
    assert captured_again
