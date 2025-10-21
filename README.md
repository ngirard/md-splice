# md-splice

A command-line tool for precise, AST-aware insertion, replacement, deletion, and retrieval of content within Markdown files.

`md-splice` parses Markdown into an Abstract Syntax Tree (AST), allowing you to select and modify logical document elements (like headings, paragraphs, or lists) instead of relying on fragile text or regex matching. It supports atomic in-place file updates to prevent data loss.

## Core features

* **Structurally-aware modifications**: Operates on the Markdown AST, not plain text.
* **Insert, replace, delete, or get**: Supports inserting new content, replacing existing nodes, deleting nodes and sections entirely, or reading Markdown without modifying the file.
* **Powerful node selection**: Select elements by type (`h1`, `p`, `list`), text content (fixed string or regex), and ordinal position (e.g., the 3rd paragraph).
* **Heading section logic**: Intelligently handles insertions relative to a heading, correctly identifying the "section" of content that belongs to it.
* **Safe file handling**: Performs atomic in-place writes to prevent file corruption on error. Can also write to a new file or standard output.

## Installation

Install directly from crates.io using `cargo`:

```sh
cargo install md-splice
```

Alternatively, install the latest version directly from the repository:

```sh
cargo install --git https://github.com/ngirard/md-splice.git
```

## Usage

### Basic command structure

```sh
md-splice --file <PATH> [COMMAND] [OPTIONS]
```

* `--file <PATH>`: The Markdown file to modify.
* `[COMMAND]`: One of `insert`, `replace`, `delete` (alias: `remove`), or `get`.
* `[OPTIONS]`: Selectors and content options.

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
  -p, --position <POSITION>        Position for the 'insert' operation [default: after]
```

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
      --select-all              Select all nodes matching the criteria
      --section                 When selecting a heading, get its entire section
      --separator <STRING>      Separator to use between results with --select-all [default: "\n"]
```

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
