# Selector Reuse Flag: Feasibility and Implementation Strategy

## 1. User Feedback and Context
- **Feedback**: “A native flag to reuse the same selector for multiple sequential edits would make multi-section updates even faster.”
- **Implication**: Power users performing chained edits currently need to restate identical selectors for each operation, either across repeated CLI invocations or inside a single transactional `apply` file. This slows large-scale, section-oriented updates.
- **Product scope**: `md-splice` is a CLI and library for AST-aware Markdown manipulation, supporting insert/replace/delete/get commands and transactional batches via `apply`. Selectors can target nodes by type, textual content, ordinal, and scoped relationships like `after` and `within`.【F:README.md†L1-L172】【F:md-splice/src/cli.rs†L27-L195】

## 2. Current Workflow and Limitations
### 2.1 CLI commands (`insert`, `replace`, `delete`, `get`)
- Each command expects the full set of selector flags (`--select-*`, `--after-*`, `--within-*`, `--until-*`). The selector is re-parsed for every invocation and embedded in the constructed transaction operation.【F:md-splice/src/app.rs†L220-L392】
- There is no persistent CLI state: if a user wants to run `replace` followed by `insert` against the same heading, they must repeat the selector flags manually.

### 2.2 Transactional `apply`
- Operations in YAML/JSON form also embed full selector objects. Even when multiple operations share the same selector tree, it must be duplicated verbatim, increasing authoring effort and chances of drift.【F:README.md†L94-L173】【F:md-splice-lib/src/transaction.rs†L28-L177】
- `md-splice-lib` currently deserializes each operation into a `Selector` struct; there is no concept of aliases or references that would let one operation reuse a selector defined earlier in the transaction.【F:md-splice-lib/src/transaction.rs†L28-L116】

### 2.3 Resulting Pain Points
1. **Human inefficiency**: repeating long selector expressions (especially nested `after`/`within` trees) is error-prone.
2. **LLM orchestration friction**: automated agents scripting multi-step edits must re-supply selectors, increasing prompt size and chance of mismatch.
3. **No native caching**: there is no facility for the CLI to remember the previous selector or to name a selector for later reuse.

## 3. Goals and Non-Goals for a "Reuse Selector" Flag
### Goals
- Allow users to define a selector once and reuse it for subsequent operations in the same workflow (CLI sequence or operations file).
- Preserve backward compatibility: existing commands and operation files must continue to work unchanged.
- Keep schema expressive enough to support nested selectors (`after`, `within`, `until`).
- Provide a capability that an autonomous agent can reliably drive without external state hacks.

### Non-Goals
- Persisting selectors across separate CLI processes or shell sessions.
- Changing selector semantics or execution order.
- Introducing fully general variables/macros inside operation files (the aim is scoped aliasing, not a new scripting language).

## 4. Proposed Capability Design
### 4.1 Schema Extensions
Introduce selector handles that can be either inline definitions or references:
```yaml
- op: replace
  selector:
    alias: main_changelog
    select_type: h2
    select_contains: "Changelog"
  content: "## Changelog\n\nUpdated entries."
- op: insert
  selector_ref: main_changelog
  position: append_child
  content: "- Added selector reuse support"
```
Key pieces:
1. **`alias` field** (optional) on any inline selector to register a name in a transaction-scoped map.
2. **`selector_ref` field** on operations to reuse a previously aliased selector. Exactly one of `selector` or `selector_ref` must be provided.
3. Optional **`until_ref`** (and potentially `after_ref`/`within_ref`) to reuse range delimiters or nested scopes when needed.
4. CLI equivalents: `--selector-alias <NAME>` to name the selector on the current command, and `--selector-ref <NAME>` to reuse a previously named selector within the same invocation (e.g., multi-edit mode or future batching support).

### 4.2 Execution Semantics
- Maintain an ordered map of alias ➜ resolved `Selector` inside the transaction executor. When an operation defines an alias, resolve the selector first, then store it for later reference.
- When encountering `selector_ref`, look up the alias, clone the stored selector, and use it as if it were inlined. Missing aliases should raise a user-facing error before any mutations are committed.
- Aliases should be immutable once defined to prevent surprising rebindings. The first definition wins; further attempts with the same alias should be rejected.
- Nested selectors (`after`, `within`, `until`) can themselves carry aliases or references, enabling reuse for landmarks and range end-points.

### 4.3 User Experience Examples
1. **CLI scripting**: `md-splice --file README.md replace --select-type h2 --select-contains "Changelog" --selector-alias changelog ...` followed by `md-splice --file README.md insert --selector-ref changelog ...` in the same future “multi-op” wrapper without respecifying selectors.
2. **Operations file**: As above, multiple operations referencing `selector_ref` shrink YAML duplication and help human or LLM authors stay consistent.

## 5. Implementation Strategy for an LLM Agent
### Step 1: Data Model Updates (Library Layer)
1. Extend `Selector` representation to carry optional `alias` metadata without altering existing deserialization for inline selectors. One approach is to introduce a new `SelectorHandle` enum with variants `Inline { alias: Option<String>, fields... }` and `Reference { selector_ref: String }`, using `#[serde(untagged)]` for backward compatibility.【F:md-splice-lib/src/transaction.rs†L28-L116】
2. Update transaction operation structs (`InsertOperation`, `ReplaceOperation`, `DeleteOperation`) to accept `SelectorHandle` for the primary selector and `Option<SelectorHandle>` for `until` (plus nested handles for `after`/`within`). Preserve existing defaults so legacy YAML still deserializes.
3. Adjust internal APIs (e.g., `MarkdownDocument::apply`) so that before execution, each handle is resolved into a concrete `Selector`. Maintain a per-transaction alias map to look up references, detecting cycles or missing aliases early.

### Step 2: CLI Argument Surface
1. Add optional `selector_alias` and `selector_ref` fields to `ModificationArgs`, `DeleteArgs`, and `GetArgs`, making sure mutual exclusivity is enforced via Clap (e.g., `conflicts_with` annotations).【F:md-splice/src/cli.rs†L113-L257】
2. Update builder functions in `app.rs` (`build_insert_operation`, `build_replace_operation`, `build_delete_operation`, and selector helpers) to construct `SelectorHandle` values. When `selector_ref` is set, skip inline construction and mark the operation to use the reference instead.【F:md-splice/src/app.rs†L220-L640】
3. Ensure validation errors surface cleanly when a user requests a reference without prior alias definition in the same transaction or multi-op context.

### Step 3: Execution Flow Changes
1. Enhance the loop in `MarkdownDocument::apply` to maintain an alias map while walking operations. Upon encountering an inline selector with an alias, resolve and store it before executing the operation.【F:md-splice-lib/src/lib.rs†L102-L249】
2. Before invoking existing `replace_blocks`, `insert_blocks`, or `delete_blocks`, ensure the selector handles are fully resolved. This might involve a helper like `resolve_selector_handle(handle, aliases)` returning a cloned `Selector` or raising `SpliceError::SelectorAliasMissing`.
3. Propagate alias support into nested selectors: when deserializing `after`, `within`, or `until`, resolve references recursively so downstream code receives the same `Selector` struct it expects today.

### Step 4: Testing Strategy
1. **Unit tests** in `md-splice-lib/src/transaction.rs` to cover serialization/deserialization of alias/reference combinations, mutual exclusivity, and error cases (duplicate alias, missing alias, cyclic references).
2. **Integration tests** in `md-splice-lib/src/lib.rs` verifying that operations using `selector_ref` behave identically to those with duplicated selectors, including combinations with `until` ranges and nested scopes.【F:md-splice-lib/src/lib.rs†L907-L1194】
3. **CLI tests** (if applicable) or doc tests showing new flags in action, ensuring Clap rejects invalid flag combinations.
4. **Regression tests** to guarantee legacy operation files without alias/ref fields still pass.

### Step 5: Documentation & Tooling Updates
1. Update `README.md` and transaction specification docs to describe alias/ref usage, providing examples aligned with user feedback scenarios.【F:README.md†L94-L173】
2. Mention new CLI flags in help text and man pages (if any).
3. For agent workflows, add snippets in internal playbooks (e.g., under `devel/prompts`) to demonstrate how LLMs should leverage aliases when generating multi-step edits.

## 6. Risks, Open Questions, and Mitigations
| Risk | Description | Mitigation |
| --- | --- | --- |
| Alias resolution order | Referencing an alias before it is defined could cause runtime failures. | Validate and error early during deserialization or preflight resolution before applying any mutation. |
| Schema complexity | `#[serde(untagged)]` enums can complicate error messages. | Provide custom error messages guiding users toward correct `selector` vs `selector_ref` usage. |
| CLI state reuse | The CLI currently executes a single operation per process, so `--selector-ref` is only meaningful once multi-op CLI workflows exist. | Limit scope initially to `apply` transactions and document CLI flag as future-facing or require a `--multi`/`apply` context for alias persistence. |
| Backward compatibility | Existing serde expectations assume `selector` is always present. | Keep defaults and optional fields so older files deserialize without changes. |

## 7. Recommended Next Steps
1. Prototype alias-aware selectors inside the library, gated behind feature tests.
2. Extend transaction schema and update serialization tests to lock in backward compatibility.
3. Once stable, expose CLI flags and refresh documentation, enabling both human and agent operators to adopt the new capability.
4. Gather feedback from early adopters (especially automation/LLM users) to see if additional conveniences (e.g., batch CLI mode) are needed.

By following the staged approach above, an LLM agent gains a clear roadmap for implementing selector reuse while preserving existing ergonomics. The result directly addresses the user’s desire for faster multi-section updates by reducing repetitive selector boilerplate.
