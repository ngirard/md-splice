# `Strategy.md`: Implementing the `delete` Command

## 1. Objective

This document outlines the step-by-step strategy to implement the `delete` command as specified in `Delete-command-specification.md`. The core methodology is **Test-Driven Development (TDD)**. We will follow a strict "Red-Green-Refactor" cycle for each piece of functionality.

**Guiding Principles:**
* **Test First:** No production code will be written until a failing test that requires it exists.
* **Incremental Steps:** Each step will add a small, verifiable piece of functionality.
* **Consistency:** The new code must align with the existing architecture and coding style.
* **Clarity:** The final implementation should be clear, well-documented, and robust.

## 2. Development Phases

The implementation is broken down into the following phases:

1. **Phase 1: CLI Scaffolding and Recognition**
    * Goal: Make the application recognize the `delete` command and its arguments, without any actual logic.
2. **Phase 2: Basic Block Deletion**
    * Goal: Implement the core logic for deleting a single, top-level block (e.g., a paragraph).
3. **Phase 3: List Item Deletion**
    * Goal: Implement the special-case logic for deleting an item from within a list, including handling empty lists.
4. **Phase 4: Section Deletion**
    * Goal: Implement the powerful `--section` flag for deleting entire heading sections.
5. **Phase 5: Documentation and Finalization**
    * Goal: Update the `README.md` to reflect the new feature and ensure all code is clean.

---

## Phase 1: CLI Scaffolding and Recognition

Our first goal is to make `clap` aware of the new command. The test will verify that the CLI can be invoked with `delete` and its expected arguments.

### **Step 1.1 (Red): Write a failing integration test for the CLI.**

1. Create a new integration test file: `tests/delete_cmd.rs`.
2. Inside this file, add a test case that uses `assert_cmd` to check the `--help` output of the `delete` subcommand.

    ```rust
    // in tests/delete_cmd.rs
    use assert_cmd::prelude::*;
    use std::process::Command;

    #[test]
    fn test_delete_cli_help() {
        let mut cmd = Command::cargo_bin("md-splice").unwrap();
        let assert = cmd.arg("delete").arg("--help").assert();
        assert
            .success()
            .stdout(predicates::str::contains("--select-type"))
            .stdout(predicates::str::contains("--select-contains"))
            .stdout(predicates::str::contains("--select-regex"))
            .stdout(predicates::str::contains("--select-ordinal"))
            .stdout(predicates::str::contains("--section"));
    }
    ```
3. Run `cargo test`. This test will fail because the `delete` subcommand does not exist.

### **Step 1.2 (Green): Implement the minimal CLI changes.**

1. **Modify `src/cli.rs`:**
    * Add a new `Delete(DeleteArgs)` variant to the `Command` enum, including the `#[command(alias = "remove")]` attribute as specified.
    * Create a new `#[derive(Parser, Debug)]` struct named `DeleteArgs`.
    * Copy all the `--select-*` fields from `ModificationArgs` into `DeleteArgs`.
    * Add the new `--section` boolean flag to `DeleteArgs` as specified in `Delete-command-specification.md`.

2. **Modify `src/lib.rs`:**
    * In the `run` function, add a match arm for `Command::Delete(args)`.
    * For now, this arm can simply do nothing and return `Ok(())`. The goal is only to make the CLI parser work.

    ```rust
    // in src/lib.rs
    // ...
    match &cli.command {
        Command::Insert(args) => (args, false, false), // (mod_args, is_replace, is_delete)
        Command::Replace(args) => (args, true, false),
        Command::Delete(args) => {
            // For now, just make it compile. Logic comes later.
            // We will handle the args properly in the next phase.
            return Ok(());
        }
    };
    // ...
    ```
    *You will need to adjust the `match` statement and its return tuple to accommodate the new command. A better approach is to refactor the `run` function to handle the command dispatch more cleanly.*

    **Refined `run` function structure:**
    ```rust
    // in src/lib.rs
    pub fn run() -> anyhow::Result<()> {
        // ... parsing, input reading, etc.
        let cli = Cli::parse();

        // ... handle input file reading ...
        // ... parse markdown to AST ...

        match cli.command {
            Command::Insert(args) => {
                // ... existing insert logic ...
            }
            Command::Replace(args) => {
                // ... existing replace logic ...
            }
            Command::Delete(args) => {
                // Logic to be implemented in Phase 2.
                // For now, we just need the match arm to exist.
            }
        }

        // ... render and write output ...
        Ok(())
    }
    ```

3. Run `cargo test`. The `test_delete_cli_help` test should now pass.

### **Step 1.3 (Refactor):**
The initial changes are minimal. No refactoring is required at this stage.

---

## Phase 2: Basic Block Deletion

Now we implement the primary function: deleting a single block.

### **Step 2.1 (Red): Write a failing test for deleting a paragraph.**

1. In `tests/delete_cmd.rs`, add a new test.
2. Use `assert_fs` to create a temporary markdown file with known content.
3. Use `assert_cmd` to run the `delete` command, selecting a specific paragraph.
4. Use `insta` to snapshot the resulting file content.

    ```rust
    // in tests/delete_cmd.rs
    use assert_fs::prelude::*;
    use predicates::prelude::*;

    #[test]
    fn test_delete_paragraph_by_content() {
        let file = assert_fs::NamedTempFile::new("test.md").unwrap();
        file.write_str("# Title\n\nFirst paragraph.\n\nSecond paragraph to delete.\n\nThird paragraph.\n").unwrap();

        let mut cmd = Command::cargo_bin("md-splice").unwrap();
        cmd.arg("--file")
            .arg(file.path())
            .arg("delete")
            .arg("--select-contains")
            .arg("Second paragraph");

        cmd.assert().success();

        let result = std::fs::read_to_string(file.path()).unwrap();
        insta::assert_snapshot!(result, @r###"
        # Title

        First paragraph.

        Third paragraph.
        "###);
    }
    ```
5. Run `cargo test`. This test will fail because the delete logic is not implemented (the file will be unchanged).

### **Step 2.2 (Green): Implement the block deletion logic.**

1. **Modify `src/splicer.rs`:**
    * Create a new public function: `pub fn delete(doc_blocks: &mut Vec<Block>, index: usize)`.
    * The implementation is a single line: `doc_blocks.remove(index);`.

2. **Modify `src/lib.rs`:**
    * Flesh out the `Command::Delete(args)` match arm.
    * Build the `Selector` from `args`.
    * Call `locator::locate` to find the node.
    * Handle the `FoundNode::Block { index, .. }` case by calling `splicer::delete(&mut doc.blocks, index)`.
    * For now, `panic!` or `unimplemented!()` in the `FoundNode::ListItem` case.

3. Run `cargo test`. The `test_delete_paragraph_by_content` test should now pass.

### **Step 2.3 (Refactor):**
The code is straightforward. No refactoring is needed yet.

---

## Phase 3: List Item Deletion

This requires special handling since the item is nested inside a `Block::List`.

### **Step 3.1 (Red): Write a failing test for deleting a list item.**

1. In `tests/delete_cmd.rs`, add a test for deleting a list item by its ordinal position.
2. Also, add a test for deleting the *last* item in a list, which should cause the entire list block to be removed.

    ```rust
    // in tests/delete_cmd.rs

    #[test]
    fn test_delete_list_item() {
        let file = assert_fs::NamedTempFile::new("test.md").unwrap();
        file.write_str("- one\n- two\n- three\n").unwrap();

        let mut cmd = Command::cargo_bin("md-splice").unwrap();
        cmd.arg("--file")
            .arg(file.path())
            .arg("delete")
            .arg("--select-type")
            .arg("li")
            .arg("--select-ordinal")
            .arg("2");

        cmd.assert().success();
        let result = std::fs::read_to_string(file.path()).unwrap();
        insta::assert_snapshot!(result, @r###"
        - one
        - three
        "###);
    }

    #[test]
    fn test_delete_last_list_item_removes_list() {
        let file = assert_fs::NamedTempFile::new("test.md").unwrap();
        file.write_str("# Title\n\n- The only item\n\nAnother paragraph.\n").unwrap();

        let mut cmd = Command::cargo_bin("md-splice").unwrap();
        cmd.arg("--file")
            .arg(file.path())
            .arg("delete")
            .arg("--select-type")
            .arg("li");

        cmd.assert().success();
        let result = std::fs::read_to_string(file.path()).unwrap();
        insta::assert_snapshot!(result, @r###"
        # Title

        Another paragraph.
        "###);
    }
    ```
3. Run `cargo test`. The `test_delete_list_item` test will fail because the `ListItem` case panics.

### **Step 3.2 (Green): Implement list item deletion logic.**

1. **Modify `src/splicer.rs`:**
    * Create a new function: `pub(crate) fn delete_list_item(doc_blocks: &mut Vec<Block>, block_index: usize, item_index: usize) -> anyhow::Result<()>`.
    * Inside, get a mutable reference to the `Block::List` at `block_index`.
    * Remove the item at `item_index` from `list.items`.
    * After removing, check if `list.items.is_empty()`. If it is, remove the entire `Block::List` from `doc_blocks` at `block_index`. *Correction*: This is difficult due to mutable borrowing rules. A better pattern is to return a boolean indicating if the list became empty.
    * **Revised `splicer.rs` function signature:** `pub(crate) fn delete_list_item(...) -> anyhow::Result<bool>` where `bool` is `list_became_empty`.

2. **Modify `src/lib.rs`:**
    * Implement the `FoundNode::ListItem` match arm.
    * Call `splicer::delete_list_item`.
    * Check the boolean return value. If `true`, call `splicer::delete` on the parent list block.

3. Run `cargo test`. Both list item deletion tests should now pass.

### **Step 3.3 (Refactor):**
Review the logic in `lib.rs` for handling the boolean return. Ensure it's clear and doesn't introduce complexity. The current approach is sound.

---

## Phase 4: Section Deletion

This is the most advanced feature, building on existing section logic.

### **Step 4.1 (Red): Write failing tests for section deletion.**

1. In `tests/delete_cmd.rs`, add two tests:
    * One for the "happy path": deleting a heading with `--section` removes it and all its content until the next heading of the same/higher level.
    * One for the error path: using `--section` with a non-heading selector (e.g., `p`) should fail with a specific error message.

    ```rust
    // in tests/delete_cmd.rs

    #[test]
    fn test_delete_heading_with_section_flag() {
        let file = assert_fs::NamedTempFile::new("test.md").unwrap();
        let content = "# Title\n\n## Section to Delete\n\n- item 1\n- item 2\n\n### A subsection\n\n## Next Section\n\nThis should remain.\n";
        file.write_str(content).unwrap();

        let mut cmd = Command::cargo_bin("md-splice").unwrap();
        cmd.arg("--file")
            .arg(file.path())
            .arg("delete")
            .arg("--select-type")
            .arg("h2")
            .arg("--select-contains")
            .arg("Section to Delete")
            .arg("--section");

        cmd.assert().success();
        let result = std::fs::read_to_string(file.path()).unwrap();
        insta::assert_snapshot!(result, @r###"
        # Title

        ## Next Section

        This should remain.
        "###);
    }

    #[test]
    fn test_delete_with_section_flag_on_non_heading_fails() {
        let file = assert_fs::NamedTempFile::new("test.md").unwrap();
        file.write_str("A paragraph.\n").unwrap();

        let mut cmd = Command::cargo_bin("md-splice").unwrap();
        cmd.arg("--file")
            .arg(file.path())
            .arg("delete")
            .arg("--select-type")
            .arg("p")
            .arg("--section");

        cmd.assert()
            .failure()
            .stderr(predicates::str::contains("Error: The --section flag can only be used when deleting a heading"));
    }
    ```
2. Run `cargo test`. Both tests will fail.

### **Step 4.2 (Green): Implement section deletion logic.**

1. **Modify `src/error.rs`:**
    * Add a new error variant: `#[error("The --section flag can only be used when deleting a heading (h1-h6).")] InvalidSectionDelete`.

2. **Modify `src/splicer.rs`:**
    * Create a new public function: `pub fn delete_section(doc_blocks: &mut Vec<Block>, start_index: usize)`.
    * Inside this function, get the heading level from `doc_blocks[start_index]`.
    * Reuse the existing `find_heading_section_end` function to find the end index.
    * Use `doc_blocks.drain(start_index..end_index);` to remove the entire range of blocks.

3. **Modify `src/lib.rs`:**
    * In the `Command::Delete` arm, after locating the node, check `if args.section`.
    * If `true`:
        * Verify that the `found_node` is a `FoundNode::Block` and that the block is a `Block::Heading`. If not, return the `InvalidSectionDelete` error.
        * Call `splicer::delete_section` with the heading's index.
    * If `false`, proceed with the normal single-node deletion logic from Phases 2 & 3.

4. Run `cargo test`. Both section deletion tests should now pass.

### **Step 4.3 (Refactor):**
The `Command::Delete` arm in `lib.rs` now has conditional logic. Review it for clarity. Ensure the flow (`--section` check first) is logical and easy to follow.

---

## Phase 5: Documentation and Finalization

With all functionality implemented and tested, the final step is to update user-facing documentation.

1. **Update `README.md`:**
    * Add a new section under "Examples" showcasing the `delete` command. Include examples for simple deletion, list item deletion, and section deletion. Use the same examples from the tests for consistency.
    * Update the "Command-Line Reference" section. Add a new `#### delete` block, documenting the command, its alias (`remove`), and its arguments, especially the `--section` flag.
2. **Code Cleanup:**
    * Run `cargo fmt` to ensure consistent formatting.
    * Run `cargo clippy -- -D warnings` to catch any final lints.
    * Review all new code, adding doc comments where necessary, especially for the new public functions in `splicer.rs`.
3. **Final Test Run:**
    * Run `cargo test --all-features` one last time to ensure everything is working correctly.
