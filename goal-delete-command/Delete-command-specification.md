# Specification: The `delete` Command for `md-splice`

## 1. Command Name and Alias

The new subcommand will be named `delete`. To enhance discoverability and accommodate user preference, it will have a convenient alias: `remove`.

* **Primary:** `md-splice ... delete [OPTIONS]`
* **Alias:** `md-splice ... remove [OPTIONS]`

This provides flexibility while maintaining a clear primary name that aligns with CRUD terminology.

## 2. CLI Definition and Argument Structure

To maintain consistency and prevent user error, the `delete` command will have its own argument structure, separate from `ModificationArgs`. This is a crucial UX decision: it prevents users from nonsensically providing `--content` or `--position` to a delete operation.

The new command will only accept selector arguments.

### Proposed changes to `src/cli.rs`:

```rust
// In src/cli.rs

// ... existing code ...

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Insert new Markdown content at a specified position.
    Insert(ModificationArgs),
    /// Replace a Markdown node with new content.
    Replace(ModificationArgs),
    /// Delete a Markdown node or section.
    #[command(alias = "remove")]
    Delete(DeleteArgs),
}

// ... existing ModificationArgs struct ...

/// Arguments for the `delete` command.
#[derive(Parser, Debug)]
pub struct DeleteArgs {
    // --- Node Selection (copied from ModificationArgs) ---
    /// Select node by type (e.g., 'p', 'h1', 'list', 'table').
    #[arg(long, value_name = "TYPE")]
    pub select_type: Option<String>,

    /// Select node by its text content (fixed string).
    #[arg(long, value_name = "TEXT")]
    pub select_contains: Option<String>,

    /// Select node by its text content (regex pattern).
    #[arg(long, value_name = "REGEX")]
    pub select_regex: Option<String>,

    /// Select the Nth matching node (1-indexed). Default is 1.
    #[arg(long, value_name = "N", default_value_t = 1)]
    pub select_ordinal: usize,

    // --- Delete-specific options ---
    /// When deleting a heading, also delete its entire section.
    /// A section includes all content until the next heading of the same or higher level.
    #[arg(long, requires = "select_type")]
    pub section: bool,
}
```

**Design Rationale:**

* **`DeleteArgs` Struct:** A dedicated struct ensures type safety and a clean CLI. The user's shell completion will not suggest irrelevant flags like `--content`.
* **Re-use of Selectors:** It leverages the existing, powerful selector system without modification. This is the core of `md-splice`'s consistency.
* **The `--section` Flag:** This is the key UX enhancement. A simple `delete` on a heading should only remove the heading itself. However, a very common use case is removing an entire section. This flag makes that powerful operation explicit and discoverable, building on the tool's "intelligent section handling" strength. The `requires = "select_type"` attribute in `clap` can be used to enforce that this flag is only used when a type is specified (we will enforce it's a heading type in the logic).

## 3. Core Functionality

The `delete` command will perform the following actions:

1. **Locate:** Use the provided selector arguments (`--select-*`) to find a single target node (`Block` or `ListItem`) via the existing `locator::locate` function.
2. **Warn on Ambiguity:** If the selector matches multiple nodes, it will operate on the first one (as determined by `--select-ordinal`) and print the same ambiguity warning as `insert` and `replace`.
3. **Remove:**
    * If the target is a `Block`, it will be removed from the document's top-level block list.
    * If the target is a `ListItem`, it will be removed from its parent `List`'s item list. If this leaves the list empty, the entire `List` block will be removed to avoid empty list markers (`- \n- \n`) in the output.
4. **Write:** The modified AST will be rendered and written back to the file atomically, preserving the tool's data integrity guarantees.

## 4. Special Behavior: Section Deletion

The `--section` flag elevates the command from a simple node removal tool to a powerful refactoring utility.

* **Activation:** The flag is only effective when the selected node is a `heading` (any level from `h1` to `h6`).
* **Behavior:** When active, `md-splice` will:
    1. Find the target heading block.
    2. Identify the range of blocks that constitute its "section". This range starts at the heading and ends just before the next heading of the *same or lesser level*, or at the end of the document. This re-uses the same logic as `insert --position append-child`.
    3. Remove the entire range of blocks.
* **Error Handling:** If `--section` is used but the selected node is not a heading (e.g., `--select-type p --section`), the program will exit with a clear error message.

## 5. Hypothetical CLI Usage Examples

### Example 1: Delete a Specific Paragraph

**File `doc.md`:**
```markdown
# Introduction
This is the intro.

# Obsolete Section
This section is no longer needed.

# Conclusion
This is the conclusion.
```

**Command:**
```sh
md-splice --file doc.md delete --select-contains "no longer needed"
```

**Result `doc.md`:**
```markdown
# Introduction
This is the intro.

# Obsolete Section

# Conclusion
This is the conclusion.
```

### Example 2: Delete a List Item

**File `tasks.md`:**
```markdown
# To-Do
- [x] Buy milk
- [ ] Finish report
- [ ] Call Alice
```

**Command:**
```sh
md-splice --file tasks.md delete --select-type li --select-ordinal 2
```

**Result `tasks.md`:**
```markdown
# To-Do
- [x] Buy milk
- [ ] Call Alice
```

### Example 3: Delete a Heading and its Entire Section (with `--section`)

**File `api.md`:**
```markdown
# Main API
Details about the main API.

# Current Methods
- `GET /items`
- `POST /items`

# Deprecated API
This API is old.
- `GET /old/items`

# New Endpoints
The future is here.
```

**Command:**
```sh
md-splice --file api.md delete \
  --select-type h2 --select-contains "Deprecated API" --section
```

**Result `api.md`:**
```markdown
# Main API
Details about the main API.

# Current Methods
- `GET /items`
- `POST /items`

# New Endpoints
The future is here.
```

### Example 4: Delete Only the Heading (without `--section`)

Using the same `api.md` as above.

**Command:**
```sh
md-splice --file api.md delete \
  --select-type h2 --select-contains "Deprecated API"
```

**Result `api.md`:**
```markdown
# Main API
Details about the main API.

# Current Methods
- `GET /items`
- `POST /items`

This API is old.
- `GET /old/items`

# New Endpoints
The future is here.
```
*(Note: The content of the deprecated section is now associated with the "Current Methods" section, which is the correct and predictable default behavior.)*

## 6. Implementation Outline

1. **`src/cli.rs`:**
    * Add the `Delete(DeleteArgs)` variant to the `Command` enum.
    * Define the new `DeleteArgs` struct as specified above.
2. **`src/lib.rs`:**
    * In the `run` function, add a match arm for `Command::Delete(args)`.
    * This arm will locate the node using the selectors from `args`.
    * It will *not* need to parse any new content.
    * It will call new functions in `splicer.rs` to perform the deletion.
    * It will include logic to validate that `--section` is only used with a heading.
3. **`src/splicer.rs`:**
    * Create `pub fn delete(doc_blocks: &mut Vec<Block>, index: usize)`. This can be a simple one-liner: `doc_blocks.remove(index);`.
    * Create `pub fn delete_section(doc_blocks: &mut Vec<Block>, start_index: usize)`. This function will find the section end (reusing `find_heading_section_end`) and use `doc_blocks.drain(start_index..end_index)` to remove the range.
    * Create `pub(crate) fn delete_list_item(...)`. This will find the parent list and call `list.items.remove(item_index)`. It should also check if the list becomes empty and, if so, remove the parent `Block::List` from `doc_blocks`.
4. **`src/error.rs`:**
    * Add a new error variant, e.g., `InvalidSectionDelete(String)`, for when `--section` is used on a non-heading type.

This specification provides a complete, consistent, and powerful `delete` command that enhances the tool's capabilities while adhering to its existing design principles.