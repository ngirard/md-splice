"""Context managers for transactional Markdown edits via :mod:`md_splice`.

These helpers wrap :class:`md_splice.MarkdownDocument` to provide the commit
and safety semantics mandated by ``goal-Python-context-manager/Specification.md``.
They enforce stale-write detection, optional backup files, ambiguity handling,
and unified diff previews so callers do not have to reimplement the guardrails
manually for each edit session.
"""

from __future__ import annotations

from contextlib import AbstractContextManager
from dataclasses import dataclass
from pathlib import Path
import os
import typing as _t
import warnings

from . import MarkdownDocument, diff_unified
from .errors import OperationFailedError
from .types import Operation

__all__ = ["MdEdit", "MdBatchEdit"]


# Known substring emitted by the native layer when ambiguity occurs.
_AMBIGUITY_WARNING_SNIPPET = "matched multiple nodes"

# Cache the concrete operation classes for runtime isinstance checks.
_OPERATION_TYPES: tuple[type[object], ...] = _t.get_args(Operation)


@dataclass(slots=True)
class _StaleToken:
    """Stat snapshot captured when entering a context."""

    mtime_ns: int
    size: int


class MdEdit:
    """Safely edit a Markdown file with automatic commit semantics.

    The context manager loads ``path`` into a :class:`MarkdownDocument`, yields
    it to the caller, and on clean exit commits the in-memory document back to
    disk using :meth:`MarkdownDocument.write_in_place` with optional backups.

    Key behaviors follow ``goal-Python-context-manager/Specification.md``:

    * ``fail_on_ambiguity`` promotes ambiguity warnings to
      :class:`OperationFailedError` exceptions that include "ambiguity" in the
      message.
    * ``check_stale`` prevents writes when the file changed on disk since
      entry.
    * ``preview_diff`` prints a unified diff via :func:`diff_unified` before
      committing.

    Parameters mirror the specification and default to the safe choices used by
    the CLI.
    """

    _active_paths: set[Path] = set()

    def __init__(
        self,
        path: os.PathLike[str] | str,
        *,
        backup: bool = True,
        fail_on_ambiguity: bool = True,
        check_stale: bool = True,
        preview_diff: bool = False,
        commit: bool = True,
    ) -> None:
        if path == "-":
            raise ValueError("stdin path '-' is not supported for MdEdit")

        self._raw_path = Path(path).expanduser()
        self._resolved_path = self._raw_path.resolve()
        self._backup = backup
        self._fail_on_ambiguity = fail_on_ambiguity
        self._check_stale = check_stale
        self._preview_diff = preview_diff
        self._commit = commit

        self._doc: MarkdownDocument | None = None
        self._stale_token: _StaleToken | None = None
        self._warnings_cm: AbstractContextManager[None] | None = None
        self._previous_showwarning: _t.Callable[..., _t.Any] | None = None

    # Public API ---------------------------------------------------------
    def __enter__(self) -> MarkdownDocument:
        if self._resolved_path in self._active_paths:
            raise RuntimeError(
                f"Cannot enter MdEdit for '{self._raw_path}': context already active"
            )

        self._active_paths.add(self._resolved_path)
        try:
            stat_info = os.stat(self._resolved_path)
            self._stale_token = _StaleToken(stat_info.st_mtime_ns, stat_info.st_size)

            if self._fail_on_ambiguity:
                self._install_ambiguity_filter()

            self._doc = MarkdownDocument.from_file(str(self._raw_path))
            return self._doc
        except Exception:
            self._cleanup_entry()
            raise

    def __exit__(self, exc_type, exc, tb) -> bool:
        try:
            if exc_type is not None:
                return False

            if not self._commit:
                return False

            assert self._doc is not None, "Document should be available on exit"

            self._pre_commit()

            if self._check_stale:
                self._ensure_not_stale()

            if self._preview_diff:
                self._print_diff()

            self._doc.write_in_place(backup=self._backup)
            return False
        finally:
            self._restore_warnings()
            self._active_paths.discard(self._resolved_path)

    # Hooks --------------------------------------------------------------
    def _pre_commit(self) -> None:
        """Hook for subclasses to perform work prior to commit."""

    # Internal helpers ---------------------------------------------------
    def _install_ambiguity_filter(self) -> None:
        self._warnings_cm = warnings.catch_warnings()
        self._warnings_cm.__enter__()
        warnings.filterwarnings(
            "always",
            category=UserWarning,
            module=r"^md_splice(\\.|$)",
        )
        self._previous_showwarning = warnings.showwarning

        previous = self._previous_showwarning

        def _raise_on_warning(
            message, category, filename, lineno, file=None, line=None
        ) -> None:
            if category is UserWarning and _AMBIGUITY_WARNING_SNIPPET in str(message):
                raise OperationFailedError(f"Ambiguity detected: {message}")
            if previous is not None:
                previous(message, category, filename, lineno, file=file, line=line)
                return

            original = getattr(warnings, "_showwarning_orig", None)
            if original is not None:  # pragma: no cover - safety net
                original(message, category, filename, lineno, file=file, line=line)
            else:  # pragma: no cover - ultimate fallback
                raise category(message)

        warnings.showwarning = _raise_on_warning  # type: ignore[assignment]

    def _restore_warnings(self) -> None:
        if self._previous_showwarning is not None:
            warnings.showwarning = self._previous_showwarning  # type: ignore[assignment]
            self._previous_showwarning = None

        if self._warnings_cm is not None:
            self._warnings_cm.__exit__(None, None, None)
            self._warnings_cm = None

    def _ensure_not_stale(self) -> None:
        assert self._stale_token is not None, "Stale token missing"
        stat_info = os.stat(self._resolved_path)
        current = _StaleToken(stat_info.st_mtime_ns, stat_info.st_size)
        if (current.mtime_ns, current.size) != (
            self._stale_token.mtime_ns,
            self._stale_token.size,
        ):
            raise RuntimeError(
                f"Refusing to write '{self._raw_path}': file changed on disk"
            )

    def _print_diff(self) -> None:
        assert self._doc is not None
        before = self._resolved_path.read_text(encoding="utf-8")
        after = self._doc.render()
        diff = diff_unified(before, after, fromfile="original", tofile="modified")
        print(diff, end="")

    def _cleanup_entry(self) -> None:
        self._restore_warnings()
        self._active_paths.discard(self._resolved_path)


class MdBatchEdit(MdEdit):
    """Batch Markdown edits and commit atomically.

    ``MdBatchEdit`` collects operations via :meth:`apply` inside the ``with``
    block and applies them once during :meth:`__exit__`. This guarantees that
    selector resolution happens exactly once, improving atomicity when multiple
    operations must be coordinated.
    """

    def __init__(self, path: os.PathLike[str] | str, **kwargs) -> None:
        super().__init__(path, **kwargs)
        self._batched_ops: list[Operation] = []

    def __enter__(self) -> "MdBatchEdit":
        super().__enter__()
        self._batched_ops.clear()
        return self

    def apply(self, op: Operation) -> None:
        if not isinstance(op, _OPERATION_TYPES):
            expected = ", ".join(cls.__name__ for cls in _OPERATION_TYPES)
            raise TypeError(
                f"Unsupported operation type {type(op).__name__}; expected one of: {expected}"
            )
        self._batched_ops.append(op)

    def _pre_commit(self) -> None:
        if self._batched_ops:
            assert self._doc is not None
            self._doc.apply(self._batched_ops)
