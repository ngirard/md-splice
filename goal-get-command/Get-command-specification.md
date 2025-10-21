# Specification: The `get` Command

The `get` command will provide a powerful, read-only interface to extract Markdown content. It will reuse the existing selector logic for maximum consistency and user familiarity, while introducing new flags to handle the specific requirements of data extraction.

## 1. Command Name

The command will be named `get`.

*   **Reasoning:** `get` is a standard, intuitive verb for retrieval in command-line tools (e.g., `kubectl get`, `git config --get`). It's short, clear, and unambiguous.

## 2. Proposed CLI Specification

The `get` command will be a new subcommand at the same level as `insert`, `replace`, and `delete`.

```
Usage: md-splice get [OPTIONS]

Options:
      --select-type <TYPE>      Select node by type (e.g., 'p', 'h1', 'list')
      --select-contains <TEXT>  Select node by its text content (fixed string)
      --select-regex <REGEX>    Select node by its text content (regex pattern)
      --select-ordinal <N>      Select the Nth matching node (1-indexed) [default: 1]
      --select-all              Select all nodes matching the criteria
      --section                 When selecting a heading, get its entire section
      --separator <STRING>      Separator to use between results with --select-all [default: "\n"]
```

## 3. Key Design Decisions & New Flags

### A. Reusing `--section` for Consistency

Your proposal mentioned a new `--include-children` flag. A better approach is to **reuse the existing `--section` flag from the `delete` command.**

*   **UX Rationale:** This creates a powerful and consistent mental model for the user. The `--section` flag means the exact same thing in both contexts: "operate on the entire heading section, not just the heading node itself." This is far more intuitive than introducing a new flag with a similar meaning.
*   **Behavior:** When `get` is used with `--select-type hN` and `--section`, it will find the target heading and print the Markdown for the heading itself plus all content until the next heading of an equal or lesser level, or the end of the document.

### B. Enabling Multi-Node Extraction with `--select-all`

The current selector logic is designed to find a single node (respecting `--select-ordinal`). While useful, the "list all unchecked tasks" use case requires fetching *multiple* nodes. To solve this, we introduce a new flag: `--select-all`.

*   **UX Rationale:** This provides an explicit, unambiguous way to switch from single-node to multi-node retrieval. It avoids complex logic around the meaning of `--select-ordinal 0` or a special value.
*   **Behavior:**
    *   When `--select-all` is present, the command will find *all* nodes that match the `--select-type`, `--select-contains`, and/or `--select-regex` criteria.
    *   The `--select-ordinal` flag will be ignored (or disallowed by `clap`'s `conflicts_with` feature).
    *   The Markdown content for each found node will be printed to standard output, separated by a specific string.

### C. Script-Friendly Output with `--separator`

When using `--select-all`, simply concatenating the results can be ambiguous. We need a reliable way to separate them.

*   **UX Rationale:** This gives the user full control over the output format, making it trivial to pipe into other command-line tools like `xargs`, `awk`, or a script with `read`.
*   **Behavior:**
    *   The `--separator` flag is only active when `--select-all` is used.
    *   It defaults to a newline (`\n`), which is convenient for human-readable output.
    *   For robust scripting, users can specify the null byte (`--separator '\0'`), a common and safe practice in shell scripting.

## 4. Usage Examples

Here is how the new command would work in practice, covering the hypothetical use cases and more.

**Example 1: Get an Entire Heading Section (Your original use case)**

```sh
# Get the content of the second H2 section in the spec
md-splice --file spec.md get \
  --select-type h2 --select-ordinal 2 --section
```
*Output (to stdout):*
```markdown
# Acceptance Criteria

- The tool must accept a file path.
- The tool must support a 'get' command.
```

**Example 2: Get a Single List Item**

```sh
# Get the text of the 3rd list item containing "[ ]"
md-splice --file tasks.md get \
  --select-type li --select-regex "\[ \]" --select-ordinal 3
```
*Output (to stdout):*
```markdown
- [ ] Call the client
```

**Example 3: Get Just the Text of a Heading**

```sh
# Get only the heading node itself, without its section content
md-splice --file README.md get \
  --select-type h2 --select-contains "Installation"
```
*Output (to stdout):*
```markdown
# Installation
```

**Example 4: Extract All Unchecked Tasks (using `--select-all`)**

```sh
# Find all list items that contain "[ ]" and print them
md-splice --file todo.md get \
  --select-type li --select-contains "[ ]" --select-all
```
*Output (to stdout):*
```markdown
- [ ] Write the report
- [ ] Call the client
```

**Example 5: Scripting with Null Separators**

This is a more advanced use case showing how an LLM's agent could safely process multiple results.

```sh
# Get all code blocks and process them one by one
md-splice --file docs.md get \
  --select-type code --select-all --separator '\0' | \
  while IFS= read -r -d '' code_block; do
    echo "--- Found Code Block ---"
    echo "$code_block"
    # An LLM agent could now analyze or modify this block
  done
```

## 5. Implementation Strategy

This feature can be implemented by adding a new `process_get` function and extending the `locator` module.

1.  **`cli.rs`:**
    *   Add a `Get(GetArgs)` variant to the `Command` enum.
    *   Define a new `GetArgs` struct. It will be very similar to `DeleteArgs` but with the addition of `--select-all` and `--separator`. Use `clap` attributes to define conflicts between `--select-ordinal` and `--select-all`.

2.  **`lib.rs`:**
    *   Add a `process_get` function that takes `GetArgs`.
    *   This function will call a new `locate_all` function if `--select-all` is present, or the existing `locate` function otherwise.
    *   It will then iterate through the found nodes, render each one to a Markdown string, and print it to `stdout` followed by the separator.

3.  **`locator.rs`:**
    *   The existing `locate` function is perfect for the single-node case.
    *   A new function, `locate_all`, will be needed. It will be very similar to `locate` but will collect *all* matching nodes into a `Vec<FoundNode>` instead of stopping after the Nth match.

4.  **New `renderer` module (or helpers):**
    *   The existing `render_markdown` function works on an entire `Document`. We will need a way to render a single `Block` or a slice of `Block`s (for the `--section` case).
    *   A new helper function `render_blocks(blocks: &[Block]) -> String` can be created.
    *   For `FoundNode::ListItem`, we'll need a way to render a single `ListItem` back into its text representation (e.g., `- content`). This might involve creating a temporary `Block::List` with a single item and rendering that.

## 6. Error Handling

The implementation should handle these cases gracefully:

*   **Node Not Found:** The existing `SpliceError::NodeNotFound` is perfect for this.
*   **Invalid `--section` Use:** If `--section` is used with a selector that doesn't target a heading, it should return an error similar to `SpliceError::InvalidSectionDelete`. A new, more specific error like `SpliceError::SectionRequiresHeading` would be ideal.
*   **No Output:** If no nodes are found, the command should print nothing to `stdout` and exit with a success code (0). This is standard Unix behavior and allows for easy scripting (`if [ -n "$(md-splice get ...)" ]`).

This specification provides a feature that is not only powerful and solves the immediate problem for LLM integration but is also consistent, predictable, and a natural extension of the tool's existing design.