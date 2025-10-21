# `Strategy.md`: Implementing the `get` Command

## 1. Objective

This document outlines the step-by-step strategy to implement the `get` command as defined in `Get-command-specification.md`. The primary goal is to create a robust, well-tested, and user-friendly read-only query feature for `md-splice`.

## 2. Guiding Principles

*   **Test-Driven Development (TDD):** Every piece of new functionality will begin with a failing test. We will follow the Red-Green-Refactor cycle for each step. Integration tests using `assert_cmd` and `insta` snapshots are the preferred method.
*   **Incremental Implementation:** The feature will be built in small, logical, and verifiable phases. This minimizes complexity and ensures the codebase remains stable at each step.
*   **Code Reuse:** We will leverage existing modules like `locator.rs` and `splicer.rs` wherever possible to maintain consistency and avoid duplication.
*   **Consistency:** The new command's interface and behavior must feel like a natural extension of the existing `insert`, `replace`, and `delete` commands.

## 3. Implementation Phases

The implementation is broken down into four distinct phases. Each phase builds upon the previous one.

### Phase 1: Scaffolding and Basic Single-Node `get`

**Goal:** Implement the simplest version of the `get` command: finding a single node by its selector and printing its Markdown content to stdout.

1.  **CLI Scaffolding (`cli.rs`):**
    *   Define a new `GetArgs` struct. It will contain the standard selector arguments: `select_type`, `select_contains`, `select_regex`, and `select_ordinal`.
    *   Add the new flags specified in the spec: `select_all` (boolean flag) and `separator` (string option with a default value).
    *   Use `clap`'s `conflicts_with` attribute to ensure `--select_all` and `--select_ordinal` cannot be used together.
    *   Add a `Get(GetArgs)` variant to the `Command` enum.

2.  **Main Logic Scaffolding (`lib.rs`):**
    *   In `run()`, add a `Command::Get(args)` match arm.
    *   This arm will call a new, initially empty function: `process_get(&mut doc.blocks, args)`.
    *   Modify the output logic in `run()`: if the command is `Get`, the program should *not* write back to any file. All output from `get` goes to `stdout`. The existing logic for writing to `--output`, in-place, or `stdout` should be bypassed for the `get` command.

3.  **TDD Cycle for Single-Node `get`:**
    *   **RED (Write a failing test):**
        *   Create a new integration test file, `tests/get.rs`.
        *   Add a test case that attempts to get a specific paragraph from a test Markdown file (e.g., `md-splice --file test.md get --select-type p --select-ordinal 2`).
        *   Use `assert_cmd` to assert that the command succeeds.
        *   Use `insta` to snapshot the `stdout` of the command. The initial snapshot will be empty. This test will fail.

    *   **GREEN (Make the test pass):**
        *   Implement the `process_get` function.
        *   It will first call the existing `locator::locate()` function with the selectors from `GetArgs`.
        *   Create a new private helper function, `render_found_node(node: &FoundNode) -> String`. This function will be responsible for converting a `FoundNode` back into its Markdown string representation.
            *   For `FoundNode::Block`, create a temporary `Document` containing only that block and render it using the existing `render_markdown` function.
            *   For `FoundNode::ListItem`, create a temporary `Document` containing a `Block::List` with only that single item, then render it.
        *   In `process_get`, call this new renderer with the located node and print the resulting string to `stdout`.
        *   Run the test. It should now pass, and `cargo insta review` will show the correct Markdown content in the snapshot.

    *   **REFACTOR:**
        *   Review the `render_found_node` helper. Is it clean and efficient? Can its logic be simplified? Ensure error handling for `NodeNotFound` is clean (prints nothing, exits 0).

### Phase 2: Implementing `--section` Support

**Goal:** Add the ability to get an entire heading section, not just the heading node itself.

1.  **TDD Cycle for `--section`:**
    *   **RED (Write failing tests):**
        *   Add a new integration test that gets a heading section (e.g., `... get --select-type h2 --select-ordinal 1 --section`). Snapshot the output, which should initially only contain the heading itself.
        *   Add another test that asserts the command fails if `--section` is used with a non-heading selector (e.g., `--select-type p`).

    *   **GREEN (Make the tests pass):**
        *   In `process_get`, add a check: `if args.section`.
        *   Inside this block, verify that the located node is a `Block::Heading`. If not, return an appropriate error (e.g., a new `SpliceError::SectionRequiresHeading`).
        *   Reuse the logic from `splicer::find_heading_section_end` to determine the range of blocks that constitute the section.
        *   Create a new helper `render_blocks(blocks: &[Block]) -> String` that takes a slice of blocks, wraps them in a temporary `Document`, and renders them.
        *   Call this renderer with the slice of blocks representing the section and print the result.
        *   Run tests. They should now pass.

    *   **REFACTOR:**
        *   Examine the section-finding logic. If it was copied from `splicer`, consider moving it to a shared location to adhere to the DRY (Don't Repeat Yourself) principle.

### Phase 3: Implementing `--select-all` and `--separator`

**Goal:** Enable the retrieval of multiple nodes matching a selector.

1.  **New Locator Logic (`locator.rs`):**
    *   Create a new function `locate_all<'a>(blocks: &'a [Block], selector: &Selector) -> Result<Vec<FoundNode<'a>>, SpliceError>`.
    *   This function's implementation will be very similar to `locate`, but instead of finding the Nth match, it will filter and collect *all* matches into a `Vec`. It will ignore `selector.select_ordinal`.

2.  **TDD Cycle for `--select-all`:**
    *   **RED (Write failing tests):**
        *   Add a test that uses `--select-all` to find all list items in a file. Snapshot the output, which should be empty or incorrect.
        *   Add a second test that does the same but also uses a custom separator, e.g., `--separator "---"`. Snapshot this output as well.

    *   **GREEN (Make the tests pass):**
        *   In `process_get`, refactor the main logic to branch: `if args.select_all`.
        *   In the `true` branch, call the new `locator::locate_all()` function.
        *   Iterate over the returned `Vec<FoundNode>`.
        *   In the loop, call the `render_found_node` helper from Phase 1 for each node.
        *   Print the rendered string. If it's not the last item in the vector, print the `args.separator` string.
        *   The `else` branch will contain the existing single-node logic from Phase 1 & 2.
        *   Run tests. They should now pass.

    *   **REFACTOR:**
        *   Review the loop for printing with the separator. Is the logic clean? Does it correctly handle the case of zero or one found nodes (i.e., no separator printed)?

### Phase 4: Finalization and Documentation

**Goal:** Polish the implementation and update user-facing documentation.

1.  **Code Review:**
    *   Run `cargo fmt` and `cargo clippy -- -D warnings` to ensure code quality and style.
    *   Read through all new code. Add comments where the logic is complex. Ensure function and variable names are clear and descriptive.
    *   Verify that all `clap` help messages in `cli.rs` for the new command and its arguments are clear and accurate.

2.  **Documentation (`README.md`):**
    *   Add a new top-level section for the `get` command.
    *   Provide a clear description of its purpose.
    *   Include the CLI usage block for `md-splice get`.
    *   Add at least three compelling examples:
        1.  A simple single-node `get`.
        2.  Getting a heading section with `--section`.
        3.  Getting multiple nodes with `--select-all` and explaining the use of `--separator` for scripting.

By following this phased, test-driven strategy, we will produce a high-quality implementation that meets all requirements of the specification while integrating seamlessly into the existing `md-splice` project.
