# `Strategy.md`

This document outlines the step-by-step strategy to refactor the `md-splice` command-line tool into a formal Rust library (`md-splice-lib`) and a consuming binary crate (`md-splice`), as detailed in `Goal.md`. This strategy employs a test-driven development (TDD) approach to ensure correctness and maintainability throughout the process.

# Step 1: Establish a Cargo Workspace

The first step is to restructure the project from a single crate into a Cargo workspace containing two crates: the `md-splice-lib` library and the `md-splice` binary.

**Action:**

1. Create a new directory `md-splice-lib`.
2. Move the entire existing `src` directory into `md-splice-lib/`.
3. Create a new directory `md-splice` for the binary.
4. Move `src/bin/md-splice.rs` to `md-splice/src/main.rs`.
5. Delete the now-empty `md-splice-lib/src/bin` directory.
6. Create a new `Cargo.toml` at the project root with the following content to define the workspace:
    ```toml
    [workspace]
    members = [
        "md-splice-lib",
        "md-splice",
    ]
    resolver = "2"
    ```
7. Rename the original `Cargo.toml` to `md-splice-lib/Cargo.toml`. Modify it to be a library crate:
    * Remove the `[[bin]]` section if it exists.
    * Add a `[lib]` section: `[lib]\nname = "md_splice_lib"\npath = "src/lib.rs"`
    * Update the package name: `name = "md-splice-lib"`
8. Create a new `md-splice/Cargo.toml` for the binary crate:
    ```toml
    [package]
    name = "md-splice"
    version = "0.5.0" # Increment version for this major refactor
    edition = "2021"

    [dependencies]
    md-splice-lib = { path = "../md-splice-lib", version = "0.5.0" }
    anyhow = "1.0"
    clap = { version = "4.5", features = ["derive"] }
    env_logger = "0.11"
    log = "0.4"

    [dev-dependencies]
    assert_cmd = "2.0"
    assert_fs = "1.1"
    predicates = "3.1"
    rstest = "0.26"
    ```
9. Move the existing `tests` directory to the workspace root. Update its `Cargo.toml` if it exists, or ensure tests can see both crates.

**Verification:**

* Run `cargo build --workspace` from the root directory. The project should compile successfully, though the `md-splice` binary will be broken, which we will fix later.

# Step 2: Define the Public Library API (TDD)

This is the core of the refactoring. We will define a clean, public API in `md-splice-lib` and implement it by refactoring the existing logic.

**Action:**

1. **Create the central `MarkdownDocument` struct.** In `md-splice-lib/src/lib.rs`, define the primary public struct. It should encapsulate the state of a parsed document.
    ```rust
    // In md-splice-lib/src/lib.rs
    
    // ... (pub mod declarations for error, frontmatter, etc.)
    
    use crate::frontmatter::ParsedDocument;
    use markdown_ppp::ast::Document;
    
    pub struct MarkdownDocument {
        parsed: ParsedDocument,
        doc: Document,
    }
    ```

2. **Implement `MarkdownDocument::from_str` (TDD).**
    * **Test:** Create a new test file `md-splice-lib/tests/api.rs`. Write a test that attempts to create a `MarkdownDocument` from a string and fails.
        ```rust
        // In md-splice-lib/tests/api.rs
        #[test]
        fn test_load_document_from_string() {
            let content = "# Title\n\nHello, world.";
            let doc = md_splice_lib::MarkdownDocument::from_str(content).unwrap();
            // Add assertions later
        }
        ```
    * **Implement:** Implement the `from_str` function in `md-splice-lib/src/lib.rs`. This will involve calling `frontmatter::parse` and `markdown_ppp::parser::parse_markdown`.

3. **Implement `MarkdownDocument::render` (TDD).**
    * **Test:** In `md-splice-lib/tests/api.rs`, write a test that loads a document, renders it, and asserts the output is identical to the input.
    * **Implement:** Create a `render(&self) -> String` method on `MarkdownDocument` that combines the frontmatter and the rendered Markdown body.

4. **Implement `MarkdownDocument::apply` (TDD).**
    * **Test:** In `md-splice-lib/tests/api.rs`, write a test for a simple `replace` operation. Create a `MarkdownDocument`, define a `transaction::Operation`, call `doc.apply()`, render the result, and assert the content was replaced correctly. The test will fail.
    * **Implement:** Create an `apply(&mut self, operations: Vec<transaction::Operation>) -> Result<(), error::SpliceError>` method. Refactor the logic from the old `process_apply` function into this method. It should operate on `&mut self.doc.blocks` and `&mut self.parsed`.
    * **Iterate:** Add more tests for `insert`, `delete`, range operations, and frontmatter operations, ensuring each passes after implementation.

5. **Refine Error Handling.**
    * Modify the public functions (`from_str`, `apply`, etc.) to return `Result<T, error::SpliceError>` instead of `anyhow::Result`. This provides specific, catchable errors for library consumers.

**Verification:**

* All new unit tests in `md-splice-lib/tests/api.rs` must pass.
* Run `cargo clippy --workspace` to ensure the new API adheres to Rust conventions.

# Step 3: Refactor the CLI to Consume the Library

With a functional library API, we will now refactor the `md-splice` binary to be a simple client of `md-splice-lib`.

**Action:**

1. **Gut the old `md-splice` logic.** The `md-splice/src/main.rs` file should be simplified. The old `lib.rs` and its modules are now in the library crate.
2. **Rewrite `md-splice/src/main.rs`.**
    * The `main` function will parse `clap` arguments.
    * It will read the input file content into a string.
    * It will create a `md_splice_lib::MarkdownDocument` instance using `from_str`.
    * It will translate the `clap` arguments into the library's `transaction::Operation` structs. For simple commands like `insert`, this means creating a `vec!` with a single operation.
    * It will call `doc.apply()` with the created operations.
    * It will call `doc.render()` to get the final string.
    * It will write the output to the correct destination (stdout, new file, or in-place).
    * Error handling will wrap the library's `SpliceError` using `anyhow` for user-friendly output.

**Verification:**

* All existing integration tests in the root `tests/` directory must be run and **must pass without modification**. This confirms that the CLI's external behavior has not changed despite the internal refactoring. Run `cargo test --workspace`.

# Step 4: Update All User-Facing Documentation

The refactoring is complete, but the project is not done until the documentation reflects the changes.

**Action:**

1. **Update `README.md`:**
    * Add a new top-level section titled "**Using as a Library**".
    * Provide a clear Rust code example demonstrating how to add `md-splice-lib` as a dependency and use the `MarkdownDocument` API to perform a simple transaction.
    * Review all existing CLI examples to ensure they are still accurate.

2. **Add Crate-Level Documentation:**
    * In `md-splice-lib/src/lib.rs`, add a comprehensive crate-level doc comment (`//!`) at the top of the file.
    * This comment should explain the library's purpose, its core concepts (AST-based manipulation, selectors, transactions), and include a concise usage example (which can be the same as the one in the README).

3. **Add Public API Documentation:**
    * Add `#[doc = "..."]` comments to all public structs, enums, and methods in `md-splice-lib`. This includes `MarkdownDocument`, its methods, and the structs in `transaction.rs`.
    * Focus on explaining *what* each function does, its parameters, and what it returns, especially any potential errors.

**Verification:**

* Run `cargo doc --workspace --no-deps --open` from the project root.
* Review the generated HTML documentation. It should be professional, complete, and easy for a new user to understand. The examples should be clear and correct.

# Step 5: Finalization and Cleanup

Perform final quality checks before concluding the task.

**Action:**

1. Run `cargo fmt --workspace` to ensure all code is correctly formatted.
2. Run `cargo clippy --workspace -- -D warnings` to catch any remaining lints.
3. Run `cargo test --workspace` one last time to ensure all unit and integration tests are passing.
4. Delete any old, unused files (e.g., the original `md-splice/src/lib.rs` if it was not repurposed).

**Verification:**

* The workspace is clean, fully tested, documented, and builds successfully in release mode with `cargo build --workspace --release`. The project structure now clearly separates the core library from the command-line interface.
