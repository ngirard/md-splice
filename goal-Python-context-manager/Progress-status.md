update 01
- Implemented the `md_splice.ctx` module with `MdEdit` and `MdBatchEdit`, covering ambiguity promotion, stale-write detection, diff previews, and nested-context rejection as required by the specification.
- Added `tests/test_context_managers.py` with the full suite of mandated scenarios (clean commit, exception rollback, stale-write guard, ambiguity toggles, diff preview, batch apply-once, nested path rejection, stdin path validation, commit flag handling, and warning scope restoration). All tests pass.
- Documented the new API in `md-splice-py/README.md`, referencing `Specification.md` and explaining defaults, diff previews, nested restrictions, and Windows caveats.
- Local status: ✅ Core context manager goal fully implemented and verified.
