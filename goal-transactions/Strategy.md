# `Strategy.md`: Implementing Multi-Operation Support

This document outlines the development strategy for implementing the multi-operation support feature as specified in `Transactions-specification.md`. The target audience for this guide is a Large Language Model (LLM) developer, and the process is designed to be incremental, verifiable, and robust.

A **Test-Driven Development (TDD)** approach is mandatory. Each phase will begin with writing a failing test that defines the desired functionality before any implementation code is written.

## Guiding Principles

1. **TDD is Non-Negotiable:** Follow the Red-Green-Refactor cycle for every functional change.
2. **Incremental Progress:** Each phase builds upon the last, ensuring a solid foundation. We will not implement all features at once.
3. **Reuse Existing Logic:** The core `locator` and `splicer` modules are well-tested and robust. The new transaction logic should be a coordinator that calls these existing functions.
4. **Clarity and Modularity:** We will introduce new data structures and functions in a logical way to keep the codebase clean.
5. **Refer to the Specification:** All CLI and data structure decisions must align with `Transactions-specification.md`.

## Phase 1: The Skeleton - CLI and Data Structures

**Goal:** Set up the necessary command-line interface and the Rust data structures for deserializing the operations file. No business logic will be implemented yet.

### Step 1.1: Add the `apply` Subcommand

* **RED (Write a failing test):**
    * Create a new integration test file, e.g., `tests/apply.rs`.
    * Write a test using `assert_cmd` that attempts to run `md-splice apply --operations-file foo.json`.
    * This test will fail because `apply` is not a valid subcommand.

* **GREEN (Make the test pass):**
    * Modify `src/cli.rs`.
    * Add an `Apply(ApplyArgs)` variant to the `Command` enum.
    * Define a new `ApplyArgs` struct using `clap::Parser`.
    * As per `Transactions-specification.md`, this struct should include:
        * `#[arg(short = 'O', long)] pub operations_file: Option<PathBuf>`
        * `#[arg(long)] pub operations: Option<String>`
        * `#[arg(long)] pub dry_run: bool`
        * `#[arg(long)] pub diff: bool`
    * The test should now fail for a different reason (e.g., file not found or unimplemented logic), which means the CLI parsing part is working.

### Step 1.2: Define the Operation Data Structures

* **RED (Write a failing test):**
    * Create a new module `src/transaction.rs`.
    * In a `#[cfg(test)]` block within that file, write a unit test that attempts to deserialize a JSON string representing a list of operations (use the example from `Transactions-specification.md`).
    * Use `serde_json::from_str`. The test will fail to compile because the target structs do not exist.

* **GREEN (Make the test pass):**
    * In `src/transaction.rs`, define the public structs that model the operations file.
    * Use `serde::Deserialize`.
    * This will likely involve an enum `Operation` with `#[serde(tag = "op")]` and variants for `Insert`, `Replace`, and `Delete`, each containing its specific fields.
    * Ensure field names match the `snake_case` convention from the specification (e.g., `select_type`, `content_file`). Use `#[serde(default)]` for optional fields like `select_ordinal` and `position`.
    * The unit test should now pass, confirming that your structs can correctly deserialize the operations file format.

## Phase 2: The Brains - The Transaction Runner

**Goal:** Create the core logic loop that iterates through operations and applies them to the AST. We will start by implementing only the `replace` operation.

### Step 2.1: Create the `process_apply` function

* **RED (Write a failing test):**
    * In `src/lib.rs` (or a new test file), write a unit test for a new function, `process_apply`.
    * This test should:
        1. Create a sample `Vec<Block>` representing a simple Markdown document.
        2. Create a `Vec<Operation>` containing a single `replace` operation that targets a node in your sample AST.
        3. Call `process_apply` with the mutable blocks and the operations.
        4. Assert that the `Vec<Block>` was modified as expected.
    * The test will fail to compile because `process_apply` does not exist.

* **GREEN (Make the test pass):**
    * In `src/lib.rs`, create the function `fn process_apply(doc_blocks: &mut Vec<Block>, operations: Vec<Operation>) -> anyhow::Result<()>`.
    * Implement a loop over the `operations` vector.
    * Inside the loop, use a `match` statement on the operation type. For now, only implement the `Replace` arm.
    * **Crucially, reuse existing code:**
        1. Construct a `locator::Selector` from the fields in your operation struct.
        2. Call `locator::locate()` to find the target node.
        3. Parse the `content` string into new `Block`s.
        4. Call `splicer::replace()` or `splicer::replace_list_item()` based on the `FoundNode` variant.
    * The unit test should now pass.

### Step 2.2: Integrate `process_apply` into the CLI flow

* **RED (Write a failing integration test):**
    * In `tests/apply.rs`, write a full integration test.
    * Use `assert_fs` to create a temporary Markdown file and an operations file (JSON or YAML) with a single `replace` operation.
    * Run `md-splice --file ... apply --operations-file ...`.
    * Assert that the content of the Markdown file has been correctly modified.
    * This test will fail because the `Apply` command arm in `src/lib.rs` is not yet implemented.

* **GREEN (Make the test pass):**
    * In `src/lib.rs`, in the main `run` function, add a `match` arm for `Command::Apply(args)`.
    * Inside this arm:
        1. Read and parse the operations file (handle both JSON and YAML, perhaps using the `serde_yaml` crate which can also handle JSON).
        2. Call your new `process_apply` function.
    * The integration test should now pass.

## Phase 3: The Muscles - Implementing All Operations

**Goal:** Extend the `process_apply` function to handle `insert` and `delete` operations.

### Step 3.1: Implement the `insert` operation

* **RED (Write a failing test):**
    * Extend the unit test for `process_apply`. Add a new test case with an `insert` operation.
    * Assert that the `Vec<Block>` is modified correctly (a new block is inserted).
    * The test will fail because the `Insert` arm in the `match` statement is unimplemented.

* **GREEN (Make the test pass):**
    * Implement the `Insert` arm in `process_apply`.
    * This will be very similar to the `Replace` arm but will call `splicer::insert()` or `splicer::insert_list_item()`.

### Step 3.2: Implement the `delete` operation

* **RED (Write a failing test):**
    * Extend the unit test for `process_apply` again. Add a test case with a `delete` operation.
    * Assert that the target block is removed from the `Vec<Block>`.
    * Include a test case for deleting a heading with `section: true`.
    * The test will fail.

* **GREEN (Make the test pass):**
    * Implement the `Delete` arm in `process_apply`.
    * It should call `splicer::delete()` or `splicer::delete_list_item()`.
    * Add logic to check for the `section` flag and call `splicer::delete_section()` when appropriate.

## Phase 4: The Safety Net - Atomicity and Error Handling

**Goal:** Ensure that if any operation fails, the entire transaction is aborted and no changes are written.

* **RED (Write a failing test):**
    * Write a new integration test in `tests/apply.rs`.
    * Create a Markdown file and an operations file with two operations: one valid, and one that will fail (e.g., its selector will find no nodes).
    * Run the `apply` command.
    * Assert two things:
        1. The command exits with a non-zero status code.
        2. The original Markdown file is **completely unchanged**.
    * This test may pass by accident, so write a more specific unit test for `process_apply`:
        * Pass it a list of operations where the *first* one fails.
        * Assert that the function returns an `Err`.
        * Assert that the `Vec<Block>` passed to it is identical to its original state.

* **GREEN (Make the test pass):**
    * Review the `process_apply` function. Ensure that all fallible calls (like `locator::locate`) use the `?` operator to propagate errors immediately.
    * Because we are passing a mutable reference `&mut Vec<Block>` to `process_apply`, and Rust's error handling will cause an early return, the modifications from previous successful operations within the same failed transaction will persist on the in-memory AST.
    * To ensure full atomicity, the `process_apply` function should **clone** the incoming `doc_blocks` at the beginning. All modifications should be performed on the clone. If the entire loop completes successfully, the original `doc_blocks` can be replaced with the modified clone. If an error occurs, the clone is discarded, and the original `doc_blocks` remains untouched.

## Phase 5: The Polish - UX Enhancements

**Goal:** Implement the `--dry-run` and `--diff` flags as specified.

### Step 5.1: Implement `--dry-run`

* **RED (Write a failing test):**
    * Write an integration test that uses the `--dry-run` flag.
    * Assert that:
        1. The original file's content remains unchanged.
        2. The command's stdout contains the fully rendered, modified Markdown.

* **GREEN (Make the test pass):**
    * In `src/lib.rs`, after a successful call to `process_apply`, check if the `dry_run` flag is set.
    * If it is, render the modified AST to a string, print it to stdout, and return `Ok(())` early, skipping the file-writing logic.

### Step 5.2: Implement `--diff`

* **RED (Write a failing test):**
    * Write an integration test that uses the `--diff` flag.
    * Use `insta` to create a snapshot test of the command's stdout.
    * The test will fail because the output is not a diff.

* **GREEN (Make the test pass):**
    * Add a suitable diffing crate as a dependency (e.g., `similar`).
    * In `src/lib.rs`, check for the `diff` flag.
    * If present, render the modified AST to a string.
    * Generate a textual diff between the original input content and the new rendered content.
    * Print the diff to stdout and return early.

## Phase 6: The Manual - Documentation Updates

**Goal:** Ensure that all user-facing documentation is updated to reflect the new `apply` command and its associated options, as per `Transactions-specification.md`.

### Step 6.1: Update `README.md`

* **RED (Write a failing test):**
    * This phase is primarily manual, but we can simulate a "failing test" by checking for the *absence* of information.
    * Mentally (or programmatically, if you have a documentation checker) verify that `README.md` does *not* currently mention:
        * The `apply` subcommand.
        * The `--operations-file`, `--operations`, `--dry-run`, and `--diff` flags.
        * The concept of multi-operation transactions.
        * Any examples of using the `apply` command.

* **GREEN (Make the test pass):**
    * **Add a new section for "Multi-Operation Support" or "Transactions".**
        * Explain the problem this feature solves (inefficiency, fragility of multiple calls).
        * Introduce the `apply` subcommand and its purpose.
        * Clearly state the benefits: atomicity, efficiency, robustness.
    * **Update the "Usage" section:**
        * Add the `apply` command to the basic command structure.
        * Provide clear examples of how to use `md-splice ... apply ...` with `--operations-file`, `--dry-run`, and `--diff`.
        * Include an example demonstrating the use of `--operations` for inline JSON.
    * **Add a "Operations File Specification" subsection.**
        * Explain the structure of the JSON/YAML operations file.
        * Detail the common fields (`op`, `select_type`, `select_contains`, `select_regex`, `select_ordinal`, `comment`).
        * Detail the operation-specific fields for `replace`, `insert`, and `delete` (including `position` for `insert` and `section` for `delete`).
        * Refer explicitly to `Transactions-specification.md` for the definitive schema.
    * **Review existing examples:** Ensure they are still relevant and don't conflict with the new capabilities. Add new examples if necessary to illustrate how multi-operation can simplify complex tasks.

### Step 6.2: Update `Transactions-specification.md` (Self-Correction/Refinement)

* **RED (Write a failing test):**
    * Review the `Transactions-specification.md` document itself. Does it accurately reflect the implemented CLI arguments and data structures? Are there any ambiguities or missing details that would hinder an LLM's understanding?

* **GREEN (Make the test pass):**
    * **Refine the specification:**
        * Ensure consistency between the CLI arguments (`kebab-case`) and the operations file fields (`snake_case`), clearly explaining this mapping.
        * Clarify any edge cases or behaviors (e.g., how errors are reported, the exact behavior of `--diff` when no changes occur).
        * Add any necessary details about supported file formats (JSON, YAML) and auto-detection.
        * Ensure the example in the specification is comprehensive and accurate.

### Step 6.3: Update Command-Line Reference

* **RED (Write a failing test):**
    * Check the "Command-Line Reference" section in `README.md`.
    * Verify that the `apply` command and its options are not listed or are listed incorrectly.

* **GREEN (Make the test pass):**
    * Add a new entry for the `apply` command under the "Commands" section.
    * List all its options (`--operations-file`, `--operations`, `--dry-run`, `--diff`) with their descriptions and short flags.
    * Ensure the global options (`--file`, `--output`) are still mentioned as applicable.

