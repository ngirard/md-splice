# md-splice

A command-line tool for precise, AST-aware insertion, replacement, deletion, and retrieval of content within Markdown files.

`md-splice` parses Markdown into an Abstract Syntax Tree (AST), allowing you to select and modify logical document elements (like headings, paragraphs, or lists) instead of relying on fragile text or regex matching. It supports atomic in-place file updates to prevent data loss.

## Core features

* **Structurally-aware modifications**: Operates on the Markdown AST, not plain text.
* **Insert, replace, delete, or get**: Supports inserting new content, replacing existing nodes, deleting nodes and sections entirely, or reading Markdown without modifying the file.
* **Powerful node selection**: Select elements by type (`h1`, `p`, `list`), text content (fixed string or regex), ordinal position, relational landmarks (`--after-*`, `--within-*`), and ranges (`--until-*`).
* **Heading section logic**: Intelligently handles insertions relative to a heading, correctly identifying the "section" of content that belongs to it.
* **Safe file handling**: Performs atomic in-place writes to prevent file corruption on error. Can also write to a new file or standard output.
* **Multi-operation transactions**: Execute a sequence of inserts, replacements, and deletes atomically with a single command.
* **Frontmatter-aware metadata editing**: Read, write, or delete YAML/TOML frontmatter, either directly through dedicated subcommands or as part of transactional `apply` workflows.

## Installation

Install directly from crates.io using `cargo`:

```sh
cargo install md-splice
```

Alternatively, install the latest version directly from the repository:

```sh
cargo install --git https://github.com/ngirard/md-splice.git
```

## Using as a Library

`md-splice-lib` exposes the same AST-aware primitives that power the CLI. Add it
as a dependency in your own crate to perform transactional Markdown updates
programmatically:

```toml
[dependencies]
md-splice-lib = "0.5"
```

The snippet below loads a document, inserts a checklist item under a scoped
section, and then renders the updated Markdown:

```rust
use md_splice_lib::{
    transaction::{InsertOperation, InsertPosition, Operation, Selector},
    MarkdownDocument,
};

fn append_task(markdown: &str) -> Result<String, md_splice_lib::error::SpliceError> {
    let mut document = MarkdownDocument::from_str(markdown)?;

    let operation = Operation::Insert(InsertOperation {
        selector: Selector {
            select_type: Some("list".into()),
            within: Some(Box::new(Selector {
                select_type: Some("h2".into()),
                select_contains: Some("High Priority".into()),
                ..Selector::default()
            })),
            ..Selector::default()
        },
        position: InsertPosition::AppendChild,
        content: Some("- [ ] Review newly filed issues".into()),
        ..InsertOperation::default()
    });

    document.apply(vec![operation])?;
    Ok(document.render())
}
```

Every operation is applied atomically. If a selector fails to match or an
insertion would be ambiguous, `apply` returns a `SpliceError` and the original
document remains unchanged.

## Multi-operation transactions with `apply`

Complex document updates often require multiple coordinated inserts, replacements, deletes, or metadata edits. Running each command
individually is fragile and inefficient because selectors must be recomputed after every modification. The `apply`
subcommand solves this by accepting a list of operations, applying them against the Markdown AST and frontmatter in memory, and only
writing the file if every operation succeeds.

Key advantages:

* **Atomicity:** All operations succeed or none do. If any selector fails, the original file remains unchanged.
* **Selector stability:** Later operations see the AST after earlier modifications, preventing positional drift.
* **Fast feedback:** Use `--dry-run` to preview the resulting Markdown or `--diff` to emit a unified diff to stdout.

Operations can be provided through `--operations-file <PATH>` (supports JSON or YAML and accepts `-` for stdin) or inline
via `--operations '<JSON>'`. The CLI automatically detects JSON vs. YAML when reading from a file.

To keep everything in one shell command, you can pipe a YAML transaction directly with a heredoc. The example below appends a
task under the "Tasks" heading and rewrites the "Notes" section without creating any intermediate files:

```sh
md-splice --file project.md apply --operations-file - <<'YAML'
- op: insert
  selector:
    select_type: h2
    select_contains: "Tasks"
  position: append_child
  content: "- [ ] Schedule kickoff meeting"
- op: replace
  selector:
    select_type: h2
    select_contains: "Notes"
  until:
    select_type: h2
  content: |
    ## Notes

    Updated notes go here.
YAML
```

Because `--operations-file -` reads from standard input, the heredoc content is parsed as if it were in an external YAML file.

Frontmatter operations (`set_frontmatter`, `delete_frontmatter`, and `replace_frontmatter`) follow the same YAML parsing rules as the standalone `frontmatter` subcommands, so values can come from inline YAML or external files. These operations can be freely mixed with body edits inside a single transaction while preserving atomicity.

Example operations file (`changes.yaml`):

```yaml
- op: replace
  selector:
    select_type: h2
    select_contains: "Deprecated API"
  until:
    select_type: h2
    select_contains: "Examples"
  content: |
    ## Deprecated API
    This section has been removed.
- op: insert
  selector:
    select_type: list
    within:
      select_type: h2
      select_contains: "High Priority"
  position: append_child
  content: "- [ ] Implement unit tests"
- op: delete
  selector:
    select_type: p
    select_contains: "Legacy notice"
    after:
      select_type: h2
      select_contains: "Deprecated API"
```

Run the transaction and preview the diff without touching the file:

```sh
md-splice --file TODO.md apply --operations-file changes.yaml --diff
```

When `--diff` is supplied, `md-splice` prints a unified diff (with `original`/`modified` headers) and exits without writing.
`--dry-run` behaves similarly but prints the rendered Markdown instead of a diff.

### Operations file structure

Each transaction file is an array of operation objects. Every object includes an `op` field (`insert`, `replace`, or `delete`)
and a nested `selector` object describing the primary match (`select_type`, `select_contains`, `select_regex`, `select_ordinal`).
Selectors can optionally include their own `after` or `within` selector objects to scope the search before the primary match is
resolved. Range-based operations supply an optional top-level `until` selector that marks the exclusive end of the span.

Operation variants accept additional fields:

* `replace`: `content` or `content_file`, plus optional `until` to replace a span of blocks.
* `insert`: `content`/`content_file` plus optional `position` (`before`, `after`, `prepend_child`, `append_child`).
* `delete`: optional `section` to remove an entire heading section, or `until` to delete a range of blocks.

See [`goal-transactions/Transactions-specification.md`](goal-transactions/Transactions-specification.md) for the complete
schema, examples, and behavioral guarantees.

## Frontmatter operations

`md-splice` automatically detects YAML (`---`) and TOML (`+++`) frontmatter blocks at the top of a Markdown file, preserving the original format when metadata is updated. Keys accept dot and array notation such as `author.name` or `reviewers[0].email`, and nested maps are created on demand when writing values.

### Read metadata with `frontmatter get`

Use `md-splice frontmatter get` to print metadata without touching the Markdown body. Omit `--key` to render the entire frontmatter block, or provide a path to drill into a nested value.

```sh
md-splice --file spec.md frontmatter get --key status
md-splice --file spec.md frontmatter get --key reviewers[0].email --output-format json
md-splice --file spec.md frontmatter get --output-format yaml
```

The `--output-format` flag controls how the result is rendered (`string`, `json`, or `yaml`). Complex structures default to YAML even when `string` is selected.

### Write metadata with `frontmatter set`

Use `md-splice frontmatter set --key <PATH>` with either `--value <YAML>` or `--value-file <PATH>` to create or update metadata. Values are parsed as YAML, so native types (numbers, booleans, arrays, objects) are preserved. When creating a new frontmatter block, the `--format` flag selects between YAML and TOML; otherwise the existing format is reused.

```sh
# Inline YAML value
md-splice --file spec.md frontmatter set --key status --value published

# Nested update sourced from a file
md-splice --file spec.md frontmatter set --key reviewers[0] --value-file reviewer.yaml

# Create frontmatter from scratch in TOML
md-splice --file empty.md frontmatter set --key title --value "Launch Plan" --format toml
```

Provide `--value-file -` to read the value from standard input, which is useful when another tool streams YAML to `md-splice`.

### Remove metadata with `frontmatter delete`

Use `md-splice frontmatter delete --key <PATH>` to remove keys or array elements. Attempting to delete a missing key results in an error, keeping the frontmatter unchanged. Empty frontmatter blocks are automatically removed from the document.

```sh
md-splice --file spec.md frontmatter delete --key draft_notes
md-splice --file spec.md frontmatter delete --key reviewers[1]
```

### Frontmatter edits in transactions

Transactions support three metadata operations:

* `set_frontmatter` — assign or overwrite a value at the provided key path.
* `delete_frontmatter` — remove a key or array index, failing if it does not exist.
* `replace_frontmatter` — swap the entire frontmatter block with new content.

These operations accept inline YAML via `value` / `content` fields or external files (`value_file` / `content_file`), matching the CLI behavior.

```yaml
# approve.yaml
- op: set_frontmatter
  comment: "Mark the spec as approved"
  key: status
  value: approved
- op: set_frontmatter
  comment: "Capture the approval date"
  key: last_updated
  value: 2025-10-21
- op: delete_frontmatter
  comment: "Drop obsolete deadline metadata"
  key: review_deadline
- op: insert
  comment: "Append an approval notice to the Summary section"
  selector:
    select_type: h2
    select_contains: Summary
  position: append_child
  content: |
    > **Approved:** This specification was approved on 2025-10-21.
```

Run `md-splice --file spec.md apply --operations-file approve.yaml` to apply all updates atomically. If any step fails—such as attempting to delete a missing key—the Markdown body and frontmatter remain untouched.

## Scoped and Range-Based Selections

Selectors can be refined with relational context to express intent unambiguously. Every command that accepts selectors (`replace`,
`insert`, `delete`, `get`, and transactional operations) supports the same modifiers:

### Landmark scoping with `--after-*`

Use `--after-select-*` flags to locate a landmark node first, then search for the primary match that appears after it. This is
useful for commands like "the first paragraph after Installation":

```sh
md-splice --file README.md get \
  --select-type p \
  --after-select-type h2 \
  --after-select-contains "Installation"
```

### Section scoping with `--within-*`

Use `--within-select-*` flags to restrict the search to nodes contained by another selector. When the landmark is a heading, the
search is limited to that heading's section; for lists and block quotes the child nodes are searched.

```sh
md-splice --file ROADMAP.md delete \
  --select-type li --select-contains "[ ] Task Beta" \
  --within-select-type h2 --within-select-contains "Future Features"
```

### Range selection with `--until-*`

Range selectors extend an operation from the starting node to the node matched by the `--until-*` flags (exclusive). When the
ending selector is not found, the range extends to the end of the document.

```sh
md-splice --file docs/api.md replace \
  --select-type h2 --select-contains "Deprecated API" \
  --until-type h2 --until-contains "Examples" \
  --content "## Deprecated API\nThis section has been removed."
```

Scoped selectors and range selectors can be composed. For example, the `apply` transaction below finds the first list item after
"Task Alpha" within the "Future Features" section and deletes everything from the "Deprecated API" heading up to "Examples":

```yaml
- op: delete
  selector:
    select_type: li
    select_contains: Task Beta
    after:
      select_type: li
      select_contains: Task Alpha
    within:
      select_type: h2
      select_contains: Future Features
- op: replace
  selector:
    select_type: h2
    select_contains: Deprecated API
  until:
    select_type: h2
    select_contains: Examples
  content: |
    ## Deprecated API
    This section has been removed.
```

## Usage

### Basic command structure

```sh
md-splice --file <PATH> [COMMAND] [OPTIONS]
```

* `--file <PATH>`: The Markdown file to modify.
* `[COMMAND]`: One of `insert`, `replace`, `delete` (alias: `remove`), `get`, `frontmatter`, or `apply`.
* `[OPTIONS]`: Selector flags, content inputs, and command-specific options.

When using `apply`, the selector and content options come from a structured operations file (JSON or YAML) or inline JSON. The command reads all operations, applies them to the in-memory Markdown AST, and only writes the result after every operation has succeeded.

### Examples

#### 1. Replace a Paragraph

Given `report.md`:

```markdown
# Weekly Report

Status: In Progress

This is a summary of the week's events.
```

To replace the status paragraph, select it by its content and provide the new content. This modifies `report.md` in-place.

```sh
md-splice --file report.md replace \
  --select-contains "Status: In Progress" \
  --content "Status: **Complete**"
```

Resulting `report.md`:

```markdown
# Weekly Report

Status: **Complete**

This is a summary of the week's events.
```

#### 2. Insert Content After a Node

Given `doc.md`:

```markdown
# Chapter 1

This is the first paragraph.
```

To insert a new section *after* the first paragraph:

```sh
md-splice --file doc.md insert \
  --select-type p --select-ordinal 1 \
  --position after \
  --content "This is the second paragraph."
```

Resulting `doc.md`:

```markdown
# Chapter 1

This is the first paragraph.

This is the second paragraph.
```

#### 3. Append Content to a Heading Section

The `--position append-child` option is powerful when used with a heading. It inserts content at the end of the "section" owned by that heading (i.e., just before the next heading of the same or higher level).

Given `README.md`:

```markdown
# Project Title

## Installation

Instructions here.

## Usage

Examples here.
```

To add a "Troubleshooting" subsection to the end of the `Installation` section:

```sh
md-splice --file README.md insert \
  --select-type h2 --select-contains "Installation" \
  --position append-child \
  --content "### Troubleshooting\n\nIf you encounter issues, ..."
```

Resulting `README.md`:

```markdown
# Project Title

## Installation

Instructions here.

### Troubleshooting

If you encounter issues, ...

## Usage

Examples here.
```

#### 4. Replace Content from a File

You can source the new content from a file instead of a command-line string.

Given `input.md`:
```markdown
# Data

[DATA_TABLE]
```
And `new_table.md`:
```markdown
| Header 1 | Header 2 |
|----------|----------|
| Data A   | Data B   |
```

Use `--content-file` to replace the placeholder:

```sh
md-splice --file input.md --output output.md replace \
  --select-contains "[DATA_TABLE]" \
  --content-file new_table.md
```

#### 5. Modify Individual List Items

By setting `--select-type` to `li` (or `listitem`), you can apply selectors directly to items within a list.

Given `todo.md`:
```markdown
# My Tasks
- [x] Buy milk
- [ ] Write the report
- [ ] Call the client
```

To replace an item **by its content**:
```sh
md-splice --file todo.md replace \
  --select-type li --select-contains "Write the report" \
  --content "- [x] Write and **submit** the report"
```

Resulting `todo.md`:
```markdown
# My Tasks
- [x] Buy milk
- [x] Write and **submit** the report
- [ ] Call the client
```

To insert a new item *before* the third list item **by its position**:
```sh
md-splice --file todo.md insert \
  --select-type li --select-ordinal 3 \
  --position before \
  --content "- [ ] Prepare for meeting"
```

Resulting `todo.md`:
```markdown
# My Tasks
- [x] Buy milk
- [x] Write and **submit** the report
- [ ] Prepare for meeting
- [ ] Call the client
```

To add a **nested list** to an item, use `--position append-child`:

```sh
md-splice --file todo.md insert \
  --select-type li --select-contains "Write the report" \
  --position append-child \
  --content "  - [ ] Write the first section"
```

Resulting `todo.md`:
```markdown
# My Tasks
- [x] Buy milk
- [ ] Write the report
  - [ ] Write the first section
- [ ] Call the client
```

#### 6. Read Markdown with `get`

Use the read-only `get` command when you want to inspect nodes without modifying the file. The selectors behave exactly the same as they do for `insert`, `replace`, and `delete`.

**Read a paragraph by ordinal:**

```sh
md-splice --file report.md get \
  --select-type p --select-ordinal 2
```

**Capture an entire heading section:**

```sh
md-splice --file docs.md get \
  --select-type h2 --select-contains "Installation" --section
```

**List every unchecked task with a custom separator:**

```sh
md-splice --file todo.md get \
  --select-type li --select-contains "[ ]" \
  --select-all --separator '\0'
```

#### 7. Delete Content

The `delete` command removes nodes from the document using the same selector system. It also supports an optional `--section` f
lag for heading-aware deletions.

**Remove a specific paragraph:**

Given `doc.md`:

```markdown
# Title

First paragraph.

Second paragraph to delete.

Third paragraph.
```

Delete the middle paragraph by matching its contents:

```sh
md-splice --file doc.md delete --select-contains "Second paragraph"
```

Resulting `doc.md`:

```markdown
# Title

First paragraph.

Third paragraph.
```

**Delete a list item:**

```sh
md-splice --file tasks.md delete --select-type li --select-ordinal 2
```

This removes the second list item from `tasks.md`. If a list becomes empty, `md-splice` deletes the entire list block to avoid l
eaving empty markers behind.

**Delete an entire heading section:**

```sh
md-splice --file api.md delete \
  --select-type h2 --select-contains "Deprecated API" --section
```

When `--section` is supplied, the selected heading and all content up to the next heading of the same or higher level is remove
d. Using the command above deletes the "Deprecated API" section while leaving the rest of the document intact.

#### 8. Apply multiple operations atomically

Create an operations file describing the desired changes:

```yaml
- op: replace
  selector:
    select_contains: "Status: In Progress"
  content: "Status: **Complete**"
- op: insert
  selector:
    select_type: li
    select_contains: "Write documentation"
  position: before
  content: "- [ ] Implement unit tests"
```

Apply both operations in a single, atomic transaction:

```sh
md-splice --file TODO.md apply --operations-file changes.yaml
```

Add `--dry-run` to preview the resulting Markdown or `--diff` to review a unified diff without modifying the file.

## Command-Line Reference

### Global Options

* `-f, --file <FILE_PATH>`: The Markdown file to modify.
* `-o, --output <OUTPUT_PATH>`: Write the output to a new file instead of modifying the original. If omitted, the input file is modified in-place.

### Commands

#### `replace`

Replaces the selected node with new content.

```
Usage: md-splice replace [OPTIONS]

Options:
  -c, --content <MARKDOWN_STRING>  The Markdown content to replace with
      --content-file <CONTENT_PATH>  A file containing the Markdown content
      --select-type <TYPE>           Select node by type (e.g., 'p', 'h1', 'list')
      --select-contains <TEXT>       Select node by its text content (fixed string)
      --select-regex <REGEX>         Select node by its text content (regex pattern)
      --select-ordinal <N>           Select the Nth matching node (1-indexed) [default: 1]
      --after-select-type <TYPE>     Restrict the search to matches that occur after another selector
      --after-select-contains <TEXT> Restrict the search to matches that occur after another selector
      --after-select-regex <REGEX>   Restrict the search to matches that occur after another selector
      --after-select-ordinal <N>     Choose the Nth landmark match for the `--after` selector (1-indexed)
      --within-select-type <TYPE>    Restrict the search to nodes contained within another selector
      --within-select-contains <TEXT>
                                    Restrict the search to nodes contained within another selector
      --within-select-regex <REGEX>  Restrict the search to nodes contained within another selector
      --within-select-ordinal <N>    Choose the Nth landmark match for the `--within` selector (1-indexed)
      --until-type <TYPE>            Extend the operation up to (but not including) another selector
      --until-contains <TEXT>        Extend the operation up to (but not including) another selector
      --until-regex <REGEX>          Extend the operation up to (but not including) another selector
```

#### `insert`

Inserts new Markdown content at a specified position relative to the selected node.

```
Usage: md-splice insert [OPTIONS]

Options:
  -c, --content <MARKDOWN_STRING>  The Markdown content to insert
      --content-file <CONTENT_PATH>  A file containing the Markdown content
      --select-type <TYPE>           Select node by type (e.g., 'p', 'h1', 'list')
      --select-contains <TEXT>       Select node by its text content (fixed string)
      --select-regex <REGEX>         Select node by its text content (regex pattern)
  --select-ordinal <N>           Select the Nth matching node (1-indexed) [default: 1]
      --after-select-type <TYPE>     Restrict the search to matches that occur after another selector
      --after-select-contains <TEXT> Restrict the search to matches that occur after another selector
      --after-select-regex <REGEX>   Restrict the search to matches that occur after another selector
      --after-select-ordinal <N>     Choose the Nth landmark match for the `--after` selector (1-indexed)
      --within-select-type <TYPE>    Restrict the search to nodes contained within another selector
      --within-select-contains <TEXT>
                                    Restrict the search to nodes contained within another selector
      --within-select-regex <REGEX>  Restrict the search to nodes contained within another selector
      --within-select-ordinal <N>    Choose the Nth landmark match for the `--within` selector (1-indexed)
  -p, --position <POSITION>        Position for the 'insert' operation [default: after]
```

Range selectors (`--until-*`) are only valid with the `replace` command.

#### `delete`

Deletes the selected node. When the target is a heading, the optional `--section` flag deletes the entire section owned by that
heading.

```
Usage: md-splice delete [OPTIONS]

Options:
      --select-type <TYPE>      Select node by type (e.g., 'p', 'h1', 'list')
      --select-contains <TEXT>  Select node by its text content (fixed string)
      --select-regex <REGEX>    Select node by its text content (regex pattern)
      --select-ordinal <N>      Select the Nth matching node (1-indexed) [default: 1]
      --after-select-type <TYPE>     Restrict the search to matches that occur after another selector
      --after-select-contains <TEXT> Restrict the search to matches that occur after another selector
      --after-select-regex <REGEX>   Restrict the search to matches that occur after another selector
      --after-select-ordinal <N>     Choose the Nth landmark match for the `--after` selector (1-indexed)
      --within-select-type <TYPE>    Restrict the search to nodes contained within another selector
      --within-select-contains <TEXT>
                                    Restrict the search to nodes contained within another selector
      --within-select-regex <REGEX>  Restrict the search to nodes contained within another selector
      --within-select-ordinal <N>    Choose the Nth landmark match for the `--within` selector (1-indexed)
      --until-type <TYPE>            Extend the delete up to (but not including) another selector
      --until-contains <TEXT>        Extend the delete up to (but not including) another selector
      --until-regex <REGEX>          Extend the delete up to (but not including) another selector
      --section                 When deleting a heading, also delete its entire section
```

#### `get`

Reads Markdown nodes that match the selector flags and prints them to `stdout` without modifying the source document.

```
Usage: md-splice get [OPTIONS]

Options:
      --select-type <TYPE>      Select node by type (e.g., 'p', 'h1', 'list')
      --select-contains <TEXT>  Select node by its text content (fixed string)
      --select-regex <REGEX>    Select node by its text content (regex pattern)
      --select-ordinal <N>      Select the Nth matching node (1-indexed) [default: 1]
      --after-select-type <TYPE>     Restrict the search to matches that occur after another selector
      --after-select-contains <TEXT> Restrict the search to matches that occur after another selector
      --after-select-regex <REGEX>   Restrict the search to matches that occur after another selector
      --after-select-ordinal <N>     Choose the Nth landmark match for the `--after` selector (1-indexed)
      --within-select-type <TYPE>    Restrict the search to nodes contained within another selector
      --within-select-contains <TEXT>
                                    Restrict the search to nodes contained within another selector
      --within-select-regex <REGEX>  Restrict the search to nodes contained within another selector
      --within-select-ordinal <N>    Choose the Nth landmark match for the `--within` selector (1-indexed)
      --until-type <TYPE>            Extend the read up to (but not including) another selector
      --until-contains <TEXT>        Extend the read up to (but not including) another selector
      --until-regex <REGEX>          Extend the read up to (but not including) another selector
      --select-all              Select all nodes matching the criteria
      --section                 When selecting a heading, get its entire section
      --separator <STRING>      Separator to use between results with --select-all [default: "\n"]
```

#### `frontmatter`

Inspect or modify the document frontmatter without touching the Markdown body.

```
Usage: md-splice frontmatter <COMMAND> [OPTIONS]

Commands:
  get     Read metadata values from the frontmatter block
  set     Create or update frontmatter keys
  delete  Remove frontmatter keys or array elements
```

`md-splice` automatically preserves the existing frontmatter format (YAML or TOML). When creating a new block, use `--format yaml|toml` with `frontmatter set` to choose the delimiter style.

`frontmatter get` accepts an optional `--key` (dot and array notation) and `--output-format` (`string`, `json`, or `yaml`). `frontmatter set` requires `--key` alongside either `--value <YAML>` or `--value-file <PATH>` (use `-` to read from stdin). `frontmatter delete` removes the specified key and deletes the entire block automatically when it becomes empty.

#### `apply`

Executes a series of operations defined in an external file or inline JSON, applying them atomically to the target document.

```
Usage: md-splice apply [OPTIONS]

Options:
  -O, --operations-file <PATH>  Path to a JSON or YAML file describing the operations (use '-' for stdin)
      --operations <JSON>       Inline JSON array of operations
      --dry-run                 Render the resulting Markdown to stdout without writing files
      --diff                    Emit a unified diff to stdout instead of writing files
```

At least one of `--operations-file` or `--operations` must be supplied. When `--diff` is set, the command prints a diff with
`original` and `modified` headers and exits without mutating the file system.

### Selector Options

All provided `--select-*` flags are combined with **AND** logic. For example, `--select-type p --select-contains "foo"` will only match paragraphs that contain the text "foo".

* `--select-type <TYPE>`: Matches a node by its type. This can be a top-level block or a nested element like a list item. The following types are supported:

	| Type String(s)           | Markdown Construct                    | Scope  |
	| :----------------------- | :------------------------------------ | :----- |
	| `p`, `paragraph`         | A standard paragraph of text.         | Block  |
	| `heading`                | Any heading, regardless of level.     | Block  |
	| `h1` - `h6`              | A heading of a specific level.        | Block  |
	| `list`                   | An entire ordered or unordered list.  | Block  |
	| `li`, `item`, `listitem` | An individual item within a list.     | Nested |
  | `table`                  | A GFM-style table.                    | Block  |
  | `blockquote`             | A block quote (`> ...`).              | Block  |
  | `code`, `codeblock`      | A fenced or indented code block.      | Block  |
  | `html`, `htmlblock`      | A block of raw HTML.                  | Block  |
  | `githubalert`, `alert`, `note`, `tip`, `important`, `warning`, `caution`, `alert-note`, `alert-tip`, `alert-important`, `alert-warning`, `alert-caution` | A GitHub-flavored Markdown callout rendered with the "[!TYPE]" syntax. | Block  |
  | `thematicbreak`          | A horizontal rule (`---`, `***`, etc.).            | Block  |
  | `definition`             | A link reference definition, e.g., `[label]: url`.  | Block  |
  | `footnotedefinition`     | A footnote definition, e.g., `[^label]: text`.  | Block  |

  GitHub callouts (also known as GitHub Alerts) can be targeted using any of the strings above.
  For example, `--select-type alert-warning` matches a callout declared with `[!WARNING]`, while `--select-type note` matches any `[!NOTE]` block regardless of its specific label text.

* `--select-contains <TEXT>`: Matches if the node's text content includes the given string.
* `--select-regex <REGEX>`: Matches if the node's text content matches the given regular expression.
* `--select-ordinal <N>`: After all other selectors have produced a list of matching nodes, this selects the Nth node from that list (1-indexed).

### Insert Position Options

Used with the `insert` command to specify where new content should go.

* `before`: Inserts the new content as a sibling *before* the selected node.
* `after`: Inserts the new content as a sibling *after* the selected node.
* `prepend-child`: Inserts the new content as the *first child* of the selected node. This is only valid for container nodes like `blockquote` or `list`, and has special behavior for `heading` nodes (see example 3).
* `append-child`: Inserts the new content as the *last child* of the selected node.

## Development

### Setup

This project uses `cargo-make` as a task runner. A bootstrap script is provided to install all necessary development tools.

1. Ensure you have Rust and `cargo` installed, preferably via `rustup`.
2. Run the bootstrap script:
    ```sh
    ./scripts/bootstrap.sh
    ```
    This will install `cargo-make` and other development dependencies.

### Common Tasks

Tasks are defined in `Makefile.toml` and run with `cargo make`.

* **Run all checks and tests**:
    ```sh
    cargo make check-all
    ```
* **Run tests**:
    ```sh
    cargo make test
    ```
* **Review or update test snapshots**:
    ```sh
    cargo insta review
    ```
* **Format and lint the code**:
    ```sh
    cargo make fmt
    cargo make clippy
    ```
