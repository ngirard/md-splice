# `Strategy.md`

## 1. Objective

This document outlines the development strategy for implementing the **Scoped and Range-Based Selectors** as defined in `Specification.md`. The target audience for this strategy is an LLM agent. The plan must be followed sequentially to ensure a robust, test-driven, and well-documented implementation.

The guiding principles are:
1. **Test-Driven Development (TDD):** For every new piece of logic, a failing test must be written first.
2. **Incremental Changes:** The implementation is broken into logical phases, starting from the core data structures and moving outwards to the user-facing CLI and documentation.
3. **Clarity and Consistency:** The new code and CLI options must feel like a natural extension of the existing `md-splice` architecture.

## 2. Development Phases

The implementation is divided into four distinct phases.

### Phase 1: Enhance Core Logic in `locator.rs`

The foundation of this feature is the ability of the `locator` to understand scoped searches. We will modify the core data structures and search logic first.

**TDD Approach:** All work in this phase will be driven by new tests added to the `tests` module at the bottom of `src/locator.rs`.

1. **Write Failing Tests for Scoped Selection:**
    * Create a new test markdown string that is sufficiently complex to test relational queries (e.g., multiple sections with similar content, nested lists, etc.).
    * Write a test case for an `--after` query: e.g., "find the first paragraph *after* the `h2` containing 'Installation'". This test will fail to compile initially.
    * Write a test case for a `--within` query: e.g., "find the `li` containing 'Task A' *within* the `h2` section 'High Priority'". This will also fail to compile.
    * Write tests for edge cases:
        * Landmark selector (for `after` or `within`) does not match any node. The `locate` function should return `SpliceError::NodeNotFound`.
        * Primary selector does not match any node within the specified scope. The `locate` function should return `SpliceError::NodeNotFound`.
        * `--within` is used on a node type that cannot have children (e.g., a paragraph). This should result in no matches.

2. **Update Data Structures in `src/locator.rs`:**
    * Modify the `Selector` struct to support nesting. This is the key structural change. Use `Box` to prevent infinite struct sizing.
    ```rust
    // In src/locator.rs
    pub struct Selector {
        pub select_type: Option<String>,
        pub select_contains: Option<String>,
        pub select_regex: Option<Regex>,
        pub select_ordinal: usize,
        // New fields for scoped selection
        pub after: Option<Box<Selector>>,
        pub within: Option<Box<Selector>>,
    }
    ```

3. **Refactor `locate` and `locate_all` Functions:**
    * Modify the function signatures to accept the updated `Selector`.
    * Implement the core scoping logic. The function should now operate in stages:
        a. **Resolve Scope:** Check if `selector.after` or `selector.within` is `Some`. If so, perform a recursive call to `locate` to find the landmark node.
        b. **Restrict Search Space:** Based on the landmark node found, create a new, temporary slice of `&Block`s representing the valid search space.
            * For `after`, this slice will be `&blocks[landmark_index + 1..]`.
            * For `within`, this slice will be derived from the landmark's children (for `BlockQuote`, `List`, etc.) or by finding the section end (for `Heading`).
        c. **Perform Primary Search:** Run the existing type, content, and regex matching logic *only on the restricted slice of blocks*.
        d. **Return Result:** Map the index from the restricted slice back to the original document's index.
    * Ensure all new tests written in step 1 now pass.

### Phase 2: Integrate into CLI and Commands

With the `locator` updated, we can now expose this functionality through the CLI.

**TDD Approach:** Create a new integration test file, e.g., `tests/scoped_selectors.rs`, or add to an existing one. Use `assert_cmd` to write end-to-end tests that invoke the `md-splice` binary with the new flags.

1. **Write Failing Integration Tests:**
    * Write a test for a range `delete` operation: `md-splice delete --select-type h2 --select-contains "Deprecated" --until-type h2 --until-contains "Examples"`. Assert that the correct section is removed from the file.
    * Write a test for a scoped `get` operation: `md-splice get --select-type p --after-select-type h1 --after-select-contains "Introduction"`. Assert that the correct paragraph is printed to stdout.
    * Write a test for a scoped `insert` operation: `md-splice insert --content "..." --select-type li --select-ordinal 1 --within-select-type h2 --within-select-contains "Tasks"`. Assert the new list item appears in the correct list.

2. **Update `src/cli.rs`:**
    * Define new structs to hold the selector arguments to avoid repetition.
    ```rust
    // In src/cli.rs
    #[derive(Parser, Debug)]
    pub struct SelectorArgs {
        #[arg(long, value_name = "TYPE")]
        pub select_type: Option<String>,
        // ... other select_* args
    }

    #[derive(Parser, Debug)]
    pub struct ScopedSelectorArgs {
        #[clap(flatten, help_heading = "Primary Selector")]
        pub primary: SelectorArgs,

        #[clap(flatten, prefix = "after-", help_heading = "After Landmark Selector")]
        pub after: Option<SelectorArgs>,

        #[clap(flatten, prefix = "within-", help_heading = "Within Landmark Selector")]
        pub within: Option<SelectorArgs>,
    }
    ```
    * Update `ModificationArgs`, `DeleteArgs`, and `GetArgs` to use these new structs.
    * Add the `--until-*` flags to the relevant command structs (`DeleteArgs`, `GetArgs`, `ModificationArgs` for `replace`). These can be modeled as another optional `SelectorArgs` struct with a `until-` prefix.

3. **Update Command Handlers in `src/lib.rs`:**
    * Modify `process_delete`, `process_get`, `process_insert_or_replace` to accept the new CLI arguments.
    * Inside these functions, create the nested `locator::Selector` structs from the flattened CLI arguments.
    * **Implement Range Logic:** For commands with `--until-*` flags:
        a. Call `locate()` with the primary selector to find the `start_node`.
        b. If an `until` selector is present, call `locate()` again, starting the search *after* the `start_node`, to find the `end_node`.
        c. Calculate the range of block indices (`start_index..end_index`). If no `end_node` is found, the range extends to the end of the document.
        d. Apply the operation (e.g., `doc_blocks.drain()`, `render_blocks()`) to this entire range.
    * Ensure all integration tests from step 1 now pass.

### Phase 3: Integrate into `apply` Transactional Command

Now, extend the powerful `apply` command to support the new selector syntax in its YAML/JSON format.

**TDD Approach:** Add tests to `src/transaction.rs` for deserialization and to the `tests` module in `src/lib.rs` for the `process_apply` logic.

1. **Write Failing Tests:**
    * In `src/transaction.rs`, write a test to deserialize a YAML/JSON string containing the nested `selector` structure as defined in `Specification.md`. Assert that the `after`, `within`, and `until` fields are correctly parsed into the Rust structs.
    * In `src/lib.rs`, write a new `process_apply` test that uses a multi-operation transaction with scoped and range-based selectors. Verify the final rendered markdown is correct.

2. **Update Data Structures in `src/transaction.rs`:**
    * Modify the `Selector` struct to mirror the changes in `locator::Selector`, adding `pub after: Option<Selector>` and `pub within: Option<Selector>`.
    * Modify `DeleteOperation`, `ReplaceOperation`, etc., to include an optional `pub until: Option<Selector>` field.

3. **Update `apply` Logic in `src/lib.rs`:**
    * Modify the `apply_*_operation` helper functions.
    * The logic will be very similar to that added in Phase 2, but it will construct the `locator::Selector` from the deserialized transaction structs instead of the `clap` structs.
    * Implement the same range logic for `delete` and `replace` operations that have an `until` field.
    * Ensure all tests from step 1 now pass.

### Phase 4: Update User-Facing Documentation

The final and most critical step is to ensure users (and LLM agents) know how to use these powerful new features.

1. **Update `README.md`:**
    * Add a new top-level section titled **"Scoped and Range-Based Selections"**.
    * In this section, explain the concepts of `--after-*`, `--within-*`, and `--until-*`.
    * Provide clear, practical examples for each, demonstrating their power. Use the examples from `Specification.md` as a starting point.
    * **Update the `apply` section:** Modify the example `changes.yaml` to showcase a more complex operation using the new nested selector syntax. Explain the new structure.
    * **Update the Command-Line Reference:** Systematically go through the help text for `replace`, `insert`, `delete`, and `get` and add all the new flags (`--after-select-type`, `--within-select-contains`, `--until-type`, etc.).

2. **Final Review:**
    * Run `cargo make check-all` to ensure all tests, formatting, and linting checks pass.
    * Manually review the `README.md` for clarity, correctness, and completeness. Ensure the new documentation is consistent with the existing style.

## 3. Verification Checklist

Upon completion, the LLM must verify the following:

- [ ] All new unit tests in `src/locator.rs` are passing.
- [ ] All new integration tests for the CLI commands are passing.
- [ ] All new tests for the `apply` command (deserialization and logic) are passing.
- [ ] The `md-splice --help` output for `insert`, `replace`, `delete`, and `get` shows the new flags.
- [ ] `README.md` contains the new "Scoped and Range-Based Selections" section with examples.
- [ ] `README.md` contains an updated example for the `apply` command.
- [ ] `README.md`'s "Command-Line Reference" section is fully updated with all new flags.
- [ ] The entire project passes `cargo make check-all`.
