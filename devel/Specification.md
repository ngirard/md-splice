# Program Specification: `md-splice`

## 1. Project Goal & Name

* **Name**: `md-splice`
* **Goal**: A command-line tool that leverages the `markdown-ppp` AST to perform structurally-aware insertion and replacement of content within Markdown files. It will prioritize user-friendliness and predictability by operating on logical document sections, not just raw AST nodes.

## 2. Core Workflow

The tool will operate on a "Parse-Locate-Modify-Render" cycle for every invocation.

1. **Parse**: The input Markdown file is read and parsed into a `markdown_ppp::ast::Document` using `markdown_ppp::parser::parse_markdown`. All link definitions and footnotes are indexed for correct rendering later.
2. **Locate**: The `Document`'s `blocks` vector is traversed to find a target `Block` node based on the user-provided selectors. The search stops at the **first match**, and a warning is issued to `stderr` if other potential matches exist.
3. **Modify**: The user-provided content string is parsed into a temporary `Document`. Its `blocks` are then used to modify the main document's `blocks` vector according to the specified operation (`insert` or `replace`) and position. This step contains the core logic, including the "heading section" heuristic.
4. **Render**: The modified `Document` AST is rendered back into a string using `markdown_ppp::printer::render_markdown`.
5. **Output**: The resulting string is written to the destination, which is either the original file (in-place) or a new file.

## 3. Command-Line Interface (CLI) Specification

We will use `clap` with its `derive` feature for a clean and self-documenting CLI.

```rust
// In a cli.rs file or similar
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "md-splice", version, about = "Splice and modify Markdown files with AST-level precision.")]
pub struct Cli {
    /// The Markdown file to modify.
    #[arg(short, long, global = true, value_name = "FILE_PATH")]
    pub file: PathBuf,

    /// Write the output to a new file instead of modifying the original.
    #[arg(short, long, global = true, value_name = "OUTPUT_PATH")]
    pub output: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Insert new Markdown content at a specified position.
    Insert(ModificationArgs),
    /// Replace a Markdown node with new content.
    Replace(ModificationArgs),
}

#[derive(Parser, Debug)]
pub struct ModificationArgs {
    // --- Content to be added ---
    /// The Markdown content to insert or replace with.
    #[arg(short, long, value_name = "MARKDOWN_STRING", conflicts_with = "content_file")]
    pub content: Option<String>,

    /// A file containing the Markdown content to insert or replace with.
    #[arg(long, value_name = "CONTENT_PATH", conflicts_with = "content")]
    pub content_file: Option<PathBuf>,

    // --- Node Selection ---
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

    // --- Insert-specific options ---
    /// Position for the 'insert' operation. [default: after]
    #[arg(short, long, value_enum, default_value_t = InsertPosition::After)]
    pub position: InsertPosition,
}

#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
pub enum InsertPosition {
    /// Insert before the selected node (as a sibling).
    Before,
    /// Insert after the selected node (as a sibling).
    After,
    /// Insert as the first child of the selected node/section.
    PrependChild,
    /// Insert as the last child of the selected node/section.
    AppendChild,
}
```

## 4. Node Selection Logic (The "Locator")

The locator is responsible for finding the target `(index, Block)` tuple in the `Document::blocks` vector.

* **Combining Selectors**: All provided `select-*` flags are **AND-ed** together to form the search criteria.
    * `--select-type h1 --select-contains "Intro"` finds the first `h1` heading containing "Intro".
    * `--select-type p --select-ordinal 3` finds the 3rd paragraph in the document.
* **Selector Implementation**:
    * `--select-type`: Maps strings like "p", "h1", "h2", "list", "table", "blockquote", "code" to their corresponding `Block` enum variants. `h1`..`h6` will match `Block::Heading` with the correct level.
    * `--select-contains`: Performs a simple substring search on the rendered plain text of a block.
    * `--select-regex`: Compiles the regex and searches the rendered plain text of a block.
    * `--select-ordinal`: After filtering by other criteria, this selects the Nth result from the list of matches.
* **Matching Behavior**: The locator will iterate through the document's blocks, checking each one against the criteria. It will stop at the **first** block that satisfies all conditions for the given ordinal.
* **Warning on Ambiguity**: After a match is found and the operation is complete, the locator will continue scanning the rest of the document. If more potential matches are found, a warning will be printed to `stderr`: `Warning: Selector matched multiple nodes. Operation was applied to the first match only.`

## 5. Modification Logic (The "Splicer")

The splicer modifies the `Document::blocks` vector. It receives the index of the target block and the parsed blocks of the new content.

* **For `replace`**:
    1. Find the index `i` of the target block.
    2. Remove the block at index `i`.
    3. Insert the new content blocks at index `i`.
    4. `Vec::splice(i..=i, new_blocks)` is a perfect fit.

* **For `insert`**:
    * **`--position before`**:
        1. Find the index `i` of the target block.
        2. Insert the new content blocks at index `i`.
    * **`--position after`**:
        1. Find the index `i` of the target block.
        2. Insert the new content blocks at index `i + 1`.

    * **`--position prepend-child` / `append-child`**: This is where the logic branches based on the target node's type.
        * **Case 1: True Container Nodes** (e.g., `Block::BlockQuote`, `Block::ListItem`).
            * The new content blocks are inserted directly into the target block's own `blocks: Vec<Block>` field at the beginning (`prepend-child`) or end (`append-child`).
        * **Case 2: Semantic Container Node** (i.e., `Block::Heading`).
            1. Find the target `Heading` at index `i` and note its level `L`.
            2. Scan the `Document::blocks` vector starting from `i + 1`.
            3. Find the index `j` of the next `Block::Heading` with a level `<= L`.
            4. If no such heading is found, `j` is the end of the `blocks` vector (`blocks.len()`).
            5. The "section" is the slice of blocks from `i + 1` to `j - 1`.
            6. For `prepend-child`, insert the new content at index `i + 1`.
            7. For `append-child`, insert the new content at index `j`.
        * **Case 3: Non-Container Nodes** (e.g., `Block::Paragraph`, `Block::ThematicBreak`).
            * The operation is invalid. The tool will exit with a clear error message: `Error: Cannot insert child content into a 'Paragraph'. Use --position 'before' or 'after' to insert as a sibling.`

## 6. File Handling

* **Default (In-place)**: If `--output` is not specified, the tool will first render the modified content to an in-memory buffer. If rendering is successful, it will overwrite the original file. This is an atomic operation to prevent data loss on error.
* **With `--output`**: The rendered content is written directly to the specified output file path. The original file is not touched.

## 7. Error Handling

The tool will exit with a non-zero status code and a descriptive error message on `stderr` for any of the following conditions:
* Input file not found or not readable.
* Content file not found or not readable.
* Markdown parsing error in either the input file or the content string/file.
* Selector did not match any nodes in the document.
* An invalid operation was attempted (e.g., `prepend-child` on a paragraph).
* Filesystem error when writing the output.

## 8. Proposed Code Structure

```
md-splice/
├── Cargo.toml
└── src/
    ├── main.rs         # Entry point, orchestrates the workflow
    ├── cli.rs          # clap CLI structure definitions
    ├── locator.rs      # Logic for finding nodes based on selectors
    ├── splicer.rs      # Logic for modifying the AST (insert/replace)
    └── error.rs        # Custom error types and handling
```

## Detailed Logic and Heuristics

This section details the core algorithms and data transformation logic required for `md-splice` to function correctly.

### 1. Text Extraction for Node Matching

To implement `--select-contains` and `--select-regex`, a `Block` from the AST must be converted into a single, continuous string of its textual content. This is **not** the same as rendering it back to Markdown.

A function `fn block_to_text(block: &Block) -> String` will be implemented with the following recursive logic:

-   **`Block::Paragraph(inlines)`**: Concatenate the text from all `Inline` elements.
-   **`Block::Heading(heading)`**: Concatenate the text from the `heading.content` inlines.
-   **`Block::BlockQuote(blocks)`**: Recursively call `block_to_text` on each inner block and join the results with a newline.
-   **`Block::List(list)`**: Recursively process each `ListItem`. For each item, process its `blocks` and join them. Join the text of all items with newlines.
-   **`Block::CodeBlock(code_block)`**: Return the `code_block.literal` directly.
-   **`Block::Table(table)`**: For each cell in each row, concatenate the text of its `Inline` elements. Join cells with a tab `\t` and rows with a newline `\n`.
-   **`Block::FootnoteDefinition(def)`**: Recursively call `block_to_text` on each inner block.
-   **`Block::ThematicBreak`, `Block::HtmlBlock`, `Block::Definition`, `Block::Empty`**: Return an empty string, as they have no user-facing text content to match against.

For `Inline` elements within these blocks, the logic is:
-   **`Inline::Text(s)`**: Return `s`.
-   **`Inline::Code(s)`**: Return `s`.
-   **`Inline::Link(link)`**: Return the text from `link.children`.
-   **`Inline::Image(image)`**: Return the `image.alt` text.
-   All other `Inline` variants return an empty string.

### 2. Heading Section Heuristic Algorithm

When an `insert` operation uses `--position prepend-child` or `append-child` on a `Block::Heading`, the following algorithm determines the bounds of the "section" to operate on.

1. **Input**: The `Document::blocks` vector, and the index `i` of the target `Block::Heading`.
2. **Get Level**: Determine the level `L` of the heading at `blocks[i]`.
3. **Find Section End**: Iterate through the blocks from index `i + 1` to the end of the vector.
    -   Let the iterator index be `j`.
    -   If `blocks[j]` is a `Block::Heading` with a level `L_next <= L`, then the section ends *before* this block. The end index is `j`.
    -   If the loop completes without finding such a heading, the section extends to the end of the document. The end index is `blocks.len()`.
4. **Perform Insertion**:
    -   For `--position prepend-child`, the new content is inserted at index `i + 1`.
    -   For `--position append-child`, the new content is inserted at the calculated end index.

**Edge Cases:**
-   If the target heading is the last block in the document, both `prepend-child` and `append-child` insert the new content at `i + 1`.
-   If two headings that define a section boundary are adjacent (e.g., an `h2` immediately followed by another `h2`), the section is empty. `prepend-child` and `append-child` will insert the new content between them.

### 3. Safe In-Place File Writing

When `--output` is not specified, the following procedure must be used to prevent data loss:

1. Render the modified AST to an in-memory string buffer.
2. If rendering is successful, use a crate like `tempfile` to create a temporary file *in the same directory* as the original file.
3. Write the buffer's contents to the temporary file.
4. Atomically rename/move the temporary file to replace the original file. Using `std::fs::rename` is typically atomic on POSIX systems when the source and destination are on the same filesystem. This ensures the original file is not corrupted if the write is interrupted.
