# Chained Selector Reuse for Transactional Markdown Edits

## 1. Feedback and Problem Statement
- **User request**: “Having a native way to chain multiple edits against a previously matched selector (e.g., reuse the same heading context) would still shave off YAML boilerplate in longer transactions.”
- **Context**: `md-splice` already supports multi-operation transactions via the `apply` subcommand, letting users run a batch of AST-aware edits atomically with selectors expressed in JSON/YAML.【F:README.md†L78-L175】 Yet every operation must currently repeat the full selector tree, even when several edits target the same heading or scoped region.

## 2. Current Capabilities and Constraints
### 2.1 Transaction Schema
- Each `insert`, `replace`, or `delete` operation holds its own `selector` and optional range delimiters (`until`) as fully inlined objects. Nested scoping (`after`, `within`) is also encoded inline, requiring repetition whenever later edits share the same context.【F:md-splice-lib/src/transaction.rs†L28-L116】
- There is no aliasing or handle mechanism inside the transaction data model; deserialization yields standalone `Selector` structs for every operation.【F:md-splice-lib/src/transaction.rs†L28-L116】

### 2.2 CLI Construction Path
- CLI subcommands (`insert`, `replace`, `delete`) collect selector flags (`--select-*`, `--after-*`, `--within-*`) and build a fresh `Selector` struct for each invocation before dispatching the transaction operation.【F:md-splice/src/app.rs†L220-L393】
- The `apply` subcommand simply deserializes the operations array; it does not maintain state about prior selectors or offer syntax for references.【F:md-splice/src/app.rs†L425-L465】

### 2.3 Execution Pipeline
- During execution, every operation resolves its selector into a locator query and runs it independently. The executor (`apply_operations_with_ambiguity`) treats operations sequentially but without any caching of selector intent beyond each struct instance.【F:md-splice-lib/src/lib.rs†L210-L351】
- Because selectors are owned by each operation, the library cannot currently infer that multiple operations should target the same AST node unless they repeat identical selector data.

## 3. Pain Points Observed
1. **Boilerplate in transactions**: Longer YAML batches repeat identical heading contexts and nested scopes, increasing file size and risk of typos. This is especially painful for LLM-generated operations where prompt length and structural consistency matter.
2. **Lack of semantic chaining**: Even though operations execute sequentially, there is no idiomatic way to say “reuse the last selector” or “reuse the selector named X,” so users must re-specify the entire structure.
3. **Limited agent ergonomics**: Automated tooling must emit verbose selectors repeatedly, consuming tokens and making diffs noisier when only the operation type changes.

## 4. Design Goals
- **Selector reuse**: Allow operations to reference a previously resolved selector (including nested `after`/`within` trees) without restating YAML.
- **Deterministic behavior**: Selector references should resolve unambiguously, fail fast when undefined, and be immutable once bound.
- **Backward compatibility**: Existing operations files and CLI flags must continue to work without modification.
- **Agent friendliness**: The feature should offer clear affordances (e.g., aliases) that an LLM can reliably produce and validate.

## 5. Solution Space Analysis
### 5.1 Selector Alias Handles (Recommended)
**Idea**: Extend the operation schema to let any inline selector declare an `alias`, while other operations can specify `selector_ref` (and optionally `within_ref`, `after_ref`, `until_ref`) to reuse the resolved selector tree.

Example YAML:
```yaml
- op: replace
  selector:
    alias: changelog_h2
    select_type: h2
    select_contains: "Changelog"
  content: |
    ## Changelog
    …
- op: insert
  selector_ref: changelog_h2
  position: append_child
  content: "- Added selector reuse"
```

**Pros**
- Minimal schema changes with high expressiveness; works for headings and arbitrary selectors.
- Keeps operations flat—no need for additional block structure in YAML.
- Aliases can be nested (e.g., alias a `within` selector) to reuse scoped contexts for multiple descendants.

**Cons / Considerations**
- Requires introducing a `SelectorHandle` abstraction and alias map in the executor.
- Error reporting must be clear when a reference is missing or duplicated.

### 5.2 Transaction Blocks with Scoped Context
**Idea**: Introduce a higher-level `with-selector` block that opens a context for subsequent operations.
```yaml
- scope:
    select_type: h2
    select_contains: "Changelog"
  operations:
    - op: replace
      …
    - op: insert
      …
```
**Pros**
- Makes chaining explicit and groups related edits.
- Could allow entering/exiting nested contexts with indentation.
**Cons**
- Requires significant restructuring of the transaction schema, serializer, and executor.
- Harder to integrate with existing CLI flag surface without inventing new commands.

### 5.3 Implicit “Previous Selector” Reference
**Idea**: Allow `selector: ditto` or `selector: { reuse_previous: true }` semantics so an operation reuses the immediately prior selector if not provided.
**Pros**
- Very concise for straight-line scripts.
**Cons**
- Brittle once operations are reordered or when multiple selectors need reuse; lacks named handles for non-linear references.
- Harder for agents to manage when generating YAML non-sequentially.

## 6. Recommended Strategy: Alias-Based Selector Handles
Alias handles strike a balance between ergonomics and implementation complexity. They preserve the flat list of operations, allow both humans and agents to name contexts, and align with the way selectors are already constructed per operation. Implementing alias support enables the exact workflow requested—multiple edits against a reused heading—while remaining backward compatible.

### Key Behaviors
- A selector may include `alias: <string>`. The resolved selector (after nested `after`/`within` resolution) is stored under that alias for later operations in the same transaction.
- Operations may specify `selector_ref: <string>` instead of an inline selector. Optional range delimiters (`until`, nested selectors) could also accept `alias`/`*_ref` fields for completeness.
- Aliases are single-assignment; redefining the same alias results in a validation error before any edits run.
- References to unknown aliases surface a descriptive error, aborting the transaction atomically.

## 7. Implementation Blueprint for an LLM Agent
1. **Introduce Selector Handles in the Transaction Model**
   - Create a `SelectorHandle` enum in `md-splice-lib/src/transaction.rs` with variants for inline selectors (with optional `alias`) and references (`selector_ref`). Update `InsertOperation`, `ReplaceOperation`, and `DeleteOperation` to store handles instead of bare `Selector` structs.【F:md-splice-lib/src/transaction.rs†L28-L116】
   - Update serde annotations to remain backward compatible (e.g., `#[serde(untagged)]`) and add unit tests covering serialization/deserialization cases.

2. **Update CLI Construction Logic**
   - Extend the CLI argument structs to accept `--selector-alias` and `--selector-ref`, enforcing mutual exclusivity with existing selector flags in `md-splice/src/cli.rs` and wiring them through the builder helpers in `md-splice/src/app.rs` so that operations constructed via the CLI can define or reference aliases.【F:md-splice/src/cli.rs†L113-L195】【F:md-splice/src/app.rs†L220-L335】
   - For now, gate `--selector-ref` usage behind contexts where multiple operations are bundled (e.g., future CLI multi-op mode) or document that it is primarily for YAML transactions until CLI batching exists.

3. **Resolve Handles During Execution**
   - Modify `apply_operations_with_ambiguity` in `md-splice-lib/src/lib.rs` to maintain a map of alias ➜ concrete locator selectors. Before executing an operation, resolve its handles into existing `Selector` structs expected by `build_locator_selector` and friends.【F:md-splice-lib/src/lib.rs†L210-L351】
   - Ensure nested selectors (`after`, `within`, `until`) also leverage handle resolution so that heading contexts can be aliased and reused in range boundaries.

4. **Validation & Error Handling**
   - Introduce new `SpliceError` variants for “alias already defined” and “alias not found” so users receive actionable diagnostics when references fail.
   - Reject circular references (e.g., alias A referencing alias B defined later that points back to A) by resolving handles eagerly and disallowing forward references unless the alias has already been registered.

5. **Documentation and Examples**
   - Update README transaction examples to showcase alias usage for chaining edits against a heading, demonstrating the reduction in YAML boilerplate.【F:README.md†L94-L175】
   - Provide CLI help text and release notes summarizing the new flags and behaviors.

## 8. Testing and Validation Strategy
- **Deserialization tests**: Cover inline selectors without aliases, inline selectors with aliases, reference-only operations, missing alias errors, and duplicate alias definitions in `md-splice-lib/src/transaction.rs` tests.
- **Execution tests**: Add integration tests in `md-splice-lib/src/lib.rs` that run transactions reusing selectors to modify a heading section multiple times, ensuring the output matches expectations and that alias scoping persists across sequential operations.【F:md-splice-lib/src/lib.rs†L210-L351】
- **CLI tests / snapshots**: Where applicable, extend CLI integration tests or snapshots to cover help text for new flags and ensure invalid flag combinations are rejected.

## 9. Risks and Mitigations
| Risk | Impact | Mitigation |
| --- | --- | --- |
| Alias reference order | Users may attempt to reference an alias before it is defined, causing runtime failures. | Resolve handles in a preparatory pass and emit `SpliceError::SelectorAliasUndefined` before mutating the document. |
| Backward compatibility regressions | Changes to serde structures could break existing YAML. | Use `#[serde(untagged)]` handles and preserve default implementations, complemented by regression tests on legacy fixtures. |
| Increased cognitive load | Users unfamiliar with aliases might be confused by additional fields. | Keep aliases optional, document them with clear examples, and ensure default behavior remains unchanged. |
| CLI state expectations | Without a multi-operation CLI mode, `--selector-ref` may not be immediately useful. | Document CLI flag as forward-compatible; prioritize YAML transaction support while exploring CLI batching in follow-up work. |

## 10. Open Questions for Further Exploration
- Should aliases be limited to the primary selector, or may nested selectors (`after`, `within`, `until`) also be aliased/referenced independently?
- Do we need namespace scoping (e.g., automatically prefixing nested aliases) to avoid collisions in large transactions generated by agents?
- Would it be beneficial to expose alias resolution outcomes in dry-run/diff output to help users debug references?

---
By introducing selector aliases and references, `md-splice` can honor the user’s request to chain multiple edits against a previously matched heading or context. The outlined strategy minimizes schema churn, keeps backwards compatibility intact, and provides an actionable roadmap for an LLM agent to implement the enhancement end-to-end.
