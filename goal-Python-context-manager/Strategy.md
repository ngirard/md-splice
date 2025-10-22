# Strategy for implementing the Python context managers for `md-splice`

This strategy describes how an LLM (or any contributor) should implement the Python context managers specified in `Specification.md`. It emphasizes an incremental, test-driven development process, explicit checkpoints, and documentation updates. It assumes familiarity with the repository layout shown in your snapshot and the behaviors normatively defined in `Specification.md`. Where the strategy gives prescriptive instructions, those are intended to be followed exactly unless `Specification.md` is revised.

---

## Goals and scope

The end state is a pair of context managers that satisfy `Specification.md`:

1. `MdEdit`: yields a `MarkdownDocument`, applies user edits within the context, then commits on clean exit with safety features.
2. `MdBatchEdit`: yields a batching proxy that collects operations and applies them once on exit before committing.

The implementation must:

* Obey the commit, ambiguity, stale-write, backup, and diff semantics defined in `Specification.md`.
* Provide clean, well-structured tests that prove conformance.
* Update user-facing documentation so the new APIs are discoverable and correctly explained.

---

## high-level plan

1. establish a minimal design surface (module, classes, import paths).
2. close any native bindings gap needed to implement the spec cleanly (prefer deterministic ambiguity detection via an outcome value).
3. implement the context managers in small, testable pieces.
4. instrument comprehensive tests first (TDD), then make them pass incrementally.
5. update documentation and examples; wire into ci and release artifacts.

Each numbered step below ends with concrete “definition of done” checks.

---

## step 1: align on interfaces and file placement

### decisions

* Place Python code in `md-splice-py/md_splice/ctx.py` (new file). The canonical import path used in docs will be `from md_splice.ctx import MdEdit, MdBatchEdit`.
* Keep public names stable and minimal: `MdEdit`, `MdBatchEdit`.
* Do **not** export these from `md_splice.__init__` in the initial PR to reduce risk; once tests pass, add them to `__all__` for top-level convenience if desired in a follow-up.

### skeletons (non-functional)

```python
# md-splice-py/md_splice/ctx.py

from __future__ import annotations
from pathlib import Path
import os
import warnings
from typing import Optional, List

from . import (
    MarkdownDocument,
    diff_unified,
)
from .types import Operation

class MdEdit:
    def __init__(self, path, *,
                 backup: bool = True,
                 fail_on_ambiguity: bool = True,
                 check_stale: bool = True,
                 preview_diff: bool = False,
                 commit: bool = True):
        ...

    def __enter__(self) -> MarkdownDocument:
        ...

    def __exit__(self, exc_type, exc, tb) -> bool:
        ...

class MdBatchEdit(MdEdit):
    def __enter__(self) -> "MdBatchEdit":
        ...
    def apply(self, op: Operation) -> None:
        ...
```

### definition of done

* Stubs exist, importable without side effects.
* Unit tests can import the classes (even if they only test “not implemented yet”).

---

## step 2: choose the ambiguity detection mechanism

`Specification.md` allows three approaches, with a strong preference for a deterministic outcome flag over warning promotion. The Rust core already defines `ApplyOutcome { ambiguity_detected: bool, ... }`. The Python bindings currently do not explicitly export that outcome.

### preferred approach

* **Expose an `apply_with_outcome` in the native module** that returns `(outcome, None)` or raises the same exceptions as `apply`. This is a thin wrapper over Rust’s `MarkdownDocument::apply_with_ambiguity` returning a Python object with `ambiguity_detected: bool` and `frontmatter_mutated: bool`.

### fallback approach

* If exposing outcome is not immediately feasible, implement a **scoped warning promotion** inside the context managers:

  * Inside `__enter__`, install a temporary `warnings.filterwarnings("error", category=UserWarning, module="^md_splice")` when `fail_on_ambiguity=True`.
  * Ensure restoration in `__exit__` via `try/finally`.

Both paths are compliant with `Specification.md`. Prefer the native outcome path for precision and testability.

### definition of done

* Either: a new `_native` function is available and imported in Python, or the scoped warning promotion is implemented and verified by tests to be scoped.

---

## step 3: implement stale-write, diff preview, and commit semantics

Follow `Specification.md` verbatim:

* Reject `path == "-"` with `ValueError`.
* Record a stale token `(st_mtime_ns, st_size)` at `__enter__` time via `os.stat`.
* On exit:

  * If exception occurred: do nothing (no write) and propagate.
  * If `commit=False`: do nothing and return normally.
  * In `MdBatchEdit`, if any ops were queued: invoke `doc.apply(ops)` once (or `doc.apply_with_outcome(ops)` when available), then handle ambiguity policy.
  * If `check_stale=True`: `stat` again; mismatch ⇒ raise `RuntimeError` “refusing to write… changed on disk”.
  * If `preview_diff=True`: compute `before = open(path).read()`, `after = doc.render()` (expose via native if needed; `MarkdownDocument` already renders implicitly as part of `write_in_place`, so add a small helper if the public API does not expose `render()`), then `print(diff_unified(before, after, fromfile="original", tofile="modified"))`.
  * Call `doc.write_in_place(backup=backup)` and let errors propagate.

Also implement “no nested edit sessions on the same path” by keeping a process-local `WeakSet[Path]` or dict of in-flight paths. Entering a second session for the same canonical path raises `RuntimeError` with a clear message. Remove on exit in `finally`.

### definition of done

* A complete implementation exists behind the stubs.
* Local manual trials (not committed) show correct behavior on a trivial file.

---

## step 4: test-driven development plan

Create a new test module `md-splice-py/tests/test_ctx.py`. Write tests **before** finishing each corresponding feature. Use pytest. Where possible, use the smallest Markdown fixtures in temporary dirs.

Map tests one-to-one to `Specification.md`’s “testing requirements” plus strategy-specific cases:

1. **`test_commit_on_clean_exit`**

   * Arrange: file with simple content.
   * Act: with `MdEdit(path)` and a simple `ReplaceOperation`.
   * Assert: content replaced; backup exists by default.

2. **`test_no_commit_on_exception`**

   * Raise inside the `with` block after a change is staged; verify file unchanged.

3. **`test_stale_write_refused`**

   * Modify the file externally between enter and exit.
   * Expect `RuntimeError` and no write; original on-disk content remains.

4. **`test_commit_flag_false_skips_write`**

   * Open with `commit=False`, make a change, exit cleanly; assert file unchanged.

5. **`test_diff_preview_prints`**

   * Enable `preview_diff=True`; capture `capsys.readouterr()`; verify unified diff header and that commit still occurs.

6. **`test_ambiguity_escalates_by_outcome_or_warning`**

   * Craft an ambiguous selector (e.g., two paragraphs containing “Note”).
   * With `fail_on_ambiguity=True`, expect an exception and no write.
   * With `False`, expect commit; optionally assert a warning when using warning-promotion path.

7. **`test_batch_applies_once`**

   * For `MdBatchEdit`, queue two operations affecting the same section; assert final content reflects both and only a single apply occurred (observable via behavior—e.g., operations assume previous modifications).

8. **`test_nested_same_path_rejected`**

   * Enter a context, then inside it attempt to enter a second context on the same `path`; expect `RuntimeError`.

9. **`test_stdin_path_rejected`**

   * `MdEdit("-")` raises `ValueError`.

10. **`test_warning_filter_scope`**

    * Simulate an ambiguous operation outside of any context to ensure global warnings behavior remains unchanged; perform the same inside the context with `fail_on_ambiguity=True` to ensure promotion happens only there.

11. **`test_windows_atomic_write_error_propagation`** (conditional)

    * Marked with `pytest.mark.skipif(sys.platform != "win32", ...)` and use a file-handle lock trick to force an `IoError`; assert propagation without retries.

Write all test names and docstrings in sentence case to match repository conventions.

### definition of done

* Tests compile and initially fail against stubs.
* As features are implemented, tests flip to green.
* The test list fully covers the spec’s required cases.

---

## step 5: documentation updates

Update user-facing materials in the same PR:

1. **`md-splice-py/README.md`**

   * Add a new “Context managers” section that:

     * References **`Specification.md`** as the normative definition.
     * Shows short examples for `MdEdit` and `MdBatchEdit`.
     * Explains defaults (`fail_on_ambiguity`, `check_stale`, `backup`, `preview_diff`, `commit`) and why they are safe.
     * Notes nested context restrictions and Windows caveat.
     * Demonstrates `preview_diff=True` with sample output.

2. **changelog**

   * If there is a `CHANGELOG.md`, add an entry under “Unreleased” summarizing the new API and any native binding addition (e.g., outcome-returning apply).

3. **docstrings**

   * Write comprehensive docstrings on `MdEdit` and `MdBatchEdit` mirroring the README snippets and linking to `Specification.md`.

### definition of done

* README renders correctly; examples import the documented paths.
* Docstrings exist and pass a quick `pydoc`/`help()` manual check.

---

## step 6: continuous integration and packaging

* Ensure the new tests run as part of the existing Python test job. If the workflow copies tests into a wheel smoke environment, confirm `md_splice/ctx.py` is included in the wheel (already covered by `python-source = "."` in `pyproject.toml`).
* If a native symbol was added (e.g., `apply_with_outcome`), ensure `maturin` builds cleanly for all matrices (no API breakage).
* No additional linters are required, but keep import ordering and typing clean.

### definition of done

* CI green across platforms.
* Wheels contain the new module.

---

## step 7: release checklist

* All tests green locally and on CI.
* README and docstrings updated.
* `Specification.md` referenced from README and from the new module docstring.
* If native API surface changed, bump the Python package minor version (e.g., `0.5.1`) and note compatibility in the changelog.

---

## risk management and mitigations

* **ambiguity detection instability**: Prefer exposing the core’s `ApplyOutcome` to avoid brittle warning promotion. If not immediately possible, encapsulate warning filters in a dedicated helper that is trivially removed once the outcome path is available.
* **windows atomic replace quirks**: Do not implement silent retries; propagate errors as required by `Specification.md`. Mention this explicitly in the README.
* **stale-write false negatives**: Use both `mtime_ns` and `size`. Optionally add a short-circuit content hash behind an environment flag in a future patch; for now, keep the token simple and spec-compliant.
* **nested contexts**: Normalize the path with `Path(path).resolve()` to avoid duplicates caused by different relative paths to the same file.

---

## timeline and milestones (suggested)

* Milestone 1: create stubs, write the full test suite (all red).
* Milestone 2: implement `MdEdit` enter/exit, stale token, commit logic; make basic tests pass.
* Milestone 3: implement ambiguity policy (outcome or warning promotion), make related tests pass.
* Milestone 4: implement `MdBatchEdit`, make batching tests pass.
* Milestone 5: finalize docs, ensure ci is green; prepare release notes.

---

## notes for contributors and llm agents

* `Specification.md` is the source of truth; if this strategy conflicts, update this file only after reconciling with `Specification.md`.
* Default to clarity and small functions over cleverness.
* When uncertain about binding availability (e.g., outcome-returning apply), first try to use it; if unavailable, implement the warning-promotion fallback and open a follow-up task to expose the outcome natively.

---

## appendix: mapping from specification to code tasks

* **enter behavior** → `MdEdit.__enter__`: validate path, record stale token, load `MarkdownDocument`, install warning policy.
* **exit behavior** → `MdEdit.__exit__`: handle exceptions, commit flag, batch application, stale check, diff, `write_in_place`, restore warnings.
* **ambiguity policy** → outcome-based check or scoped warning promotion; error messages must contain the word “ambiguity” when failing due to ambiguity.
* **batch mode** → `MdBatchEdit.apply` queue; single `doc.apply(...)` on exit.
* **nested contexts** → process-local guard map keyed by resolved path; `RuntimeError` on reentry for the same path.
* **docs** → new README section; link to `Specification.md` and code docstrings.
* **tests** → one per requirement; use small temp files; make platform-conditional tests explicit.

By following this strategy, the implementation will satisfy `Specification.md`, maintain a high bar for safety, and remain approachable for both humans and LLM agents.
