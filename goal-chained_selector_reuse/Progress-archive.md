## update 01
- Reviewed `Strategy.md` to confirm milestone ordering and required file changes.
- Established initial audit of code surfaces touched by selector alias support.

### Audit checklist
- [x] `md-splice-lib/src/transaction.rs` — reviewed existing selector/operation schema.
- [x] `md-splice-lib/src/lib.rs` — inspected selector construction helpers and operation executors.
- [x] `md-splice-lib/src/error.rs` — noted current error variants for future extension.
- [x] `md-splice-py/md_splice/types.py` — reviewed Python-side dataclasses for selectors and operations.
- [x] `md-splice-py/md_splice/errors.py` — reviewed available exception classes.
- [x] `md-splice-py/src/lib.rs` — reviewed conversion helpers between Python and Rust layers.

### Status summary
- Milestone 1 (data model updates) ready to implement; no prior work detected.
- Downstream milestones (errors, core logic, bindings, tests, docs) remain untouched.

### Next focus
- Implement Milestone 1 changes for Rust and Python data models, ensuring serde/dataclass compatibility.
- Prepare to introduce new error variants (Milestone 2) immediately after data model changes compile.

## update 02
- Completed Milestone 1 by extending Rust `transaction` structs and Python dataclasses to support selector aliases, reference handles, and mutual exclusivity validation.
- Delivered Milestone 2 by introducing new alias-related error variants across Rust and Python layers with appropriate exception mapping.
- Implemented Milestone 3 core logic: selector resolution helper, alias registry, and updated operation execution to honor alias references while preventing duplicate definitions.
- Advanced Milestone 4 by updating Python binding conversions, YAML serializers, and CLI constructors to propagate the new schema end-to-end.
- Added regression coverage (Milestone 5 scope) by adapting existing unit, integration, and snapshot tests along with the crate doctest to the new selector interface.
- Ran `cargo test` to verify the full Rust test suite passes with alias reuse in place.

### Evidence
- `cargo test`
  - ✅ Full suite passed after updates.

### Follow-ups
- Write dedicated alias-specific test cases (Rust + Python) to exercise `selector_ref`/`until_ref` behaviors and error variants.
- Extend documentation (Milestone 6) in both READMEs with examples demonstrating selector reuse workflows.
