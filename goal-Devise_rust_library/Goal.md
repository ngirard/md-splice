# Goal: Devising a Rust Library (`md-splice-lib`)

The project is already structured with a clear separation between the binary (`src/bin/md-splice.rs`) and the library logic (`src/lib.rs` and its modules). This is the ideal starting point. Formalizing this into a public library, let's call it `md-splice-lib`, has several key benefits and perspectives.

## Benefits of a Formal Rust Library

1. **Formalized, Ergonomic API:** The current `lib.rs` is designed to serve the `clap`-based CLI. A formal library would expose a cleaner, more intentional API. Instead of functions that take CLI argument structs, you would have a primary struct, perhaps `MarkdownDocument`, with methods like:
    * `MarkdownDocument::from_string(content: &str) -> Result<Self>`
    * `doc.apply(operations: Vec<Operation>) -> Result<()>`
    * `doc.find_all(selector: &Selector) -> Result<Vec<FoundNode>>`
    * `doc.render() -> String`
    This is far more reusable and intuitive for other Rust developers than the current internal structure.

2. **Maximum Performance and Safety:** By offering a native Rust library, any other Rust-based application (e.g., static site generators, documentation tools, Rust-based LLM agents) can embed `md-splice`'s logic directly. This provides the highest possible performance and leverages Rust's compile-time safety guarantees for a tool that manipulates critical files.

3. **Core Logic Encapsulation:** The true value of `md-splice` is not in parsing Markdown (it correctly delegates this to `markdown-ppp`), but in its sophisticated `locator` and `splicer` logic. This "secret sauce"—the ability to translate high-level, human-like selectors into precise AST manipulations—is a powerful primitive. A library makes this primitive directly accessible.

4. **Improved Testability:** While the project already has excellent integration tests, a library-first approach encourages more focused unit testing on the API boundary, separate from the concerns of command-line argument parsing and I/O.

## Perspectives & Implementation Strategy

The transition from the current state to a formal library is primarily a refactoring effort:

1. **Decouple from `clap`:** The core functions in `lib.rs` (like `process_insert_or_replace`, `process_apply`) currently consume `clap` structs (`ModificationArgs`, `ApplyArgs`). These would be replaced by library-native structs (`InsertOptions`, `ApplyOptions`, etc.) that are independent of the CLI. The CLI module would then become a translator, converting `clap` structs into these library-native structs before calling the core logic.

2. **Create a Central `MarkdownDocument` Struct:** As seen in the `goal-Frontmatter_support/Strategy.md`, the plan is to introduce a `ParsedDocument` struct. This should be elevated to the central, public-facing object of the library. It would hold the state of the document (frontmatter, AST blocks) and be the entry point for all manipulations.

3. **Refine Error Handling:** The current use of `anyhow::Result` is perfect for a CLI, as it simplifies error propagation to a user-friendly message. A library should expose the specific `SpliceError` enum more directly, allowing programmatic consumers to match on specific failure modes (e.g., `SpliceError::NodeNotFound`) and handle them accordingly.

4. **Publish to Crates.io:** The final step would be to publish `md-splice-lib` as a separate crate, allowing the `md-splice` CLI crate to depend on it. This makes the core logic available to the entire Rust ecosystem.
