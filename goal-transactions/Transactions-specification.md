# Specification: Multi-Operation Support for `md-splice`

This document outlines the design for a new feature enabling `md-splice` to perform a sequence of operations within a single, atomic transaction.

## 1. Executive Summary

The proposed feature introduces a new subcommand, `apply`, which executes a series of splice operations defined in a structured data file (JSON or YAML). This addresses the core problems of inefficiency and fragility when making multiple changes to a document.

By parsing the source Markdown only once and applying all changes to the in-memory AST before writing the result, this feature provides:

* **Atomicity:** All specified operations succeed, or none are applied. The target file is never left in a partially modified state.
* **Efficiency:** Eliminates the overhead of repeatedly parsing the same file for each operation.
* **Robustness:** Selectors for later operations act on the AST as modified by earlier operations, preventing positional selectors from becoming invalid.

This enhancement transforms `md-splice` from a single-purpose tool into a powerful document transformation engine, ideal for complex, automated workflows driven by LLMs or CI/CD systems.

## 2. Proposed CLI Design

To maintain consistency with the existing command structure, we will introduce a new subcommand: `apply`.

### New `apply` Command

The `apply` command reads a list of operations and executes them sequentially on the target file.

```
USAGE:
    md-splice [GLOBAL OPTIONS] apply [OPTIONS]

GLOBAL OPTIONS:
    -f, --file <FILE_PATH>      The Markdown file to modify. [default: reads from stdin]
    -o, --output <OUTPUT_PATH>  Write to a new file instead of modifying the original.

APPLY OPTIONS:
    -O, --operations-file <PATH>  Path to a JSON or YAML file containing the operations. Use '-' to read from stdin.
        --operations <JSON_STRING>  A JSON string containing the array of operations.
        --dry-run                   Process all operations and print the final Markdown to stdout without writing to any file.
        --diff                      Show a unified diff of the changes instead of writing the file.
```

### Key UX Decisions:

1.  **Command Name:** `apply` is a clear, action-oriented verb that accurately describes applying a set of changes.
2.  **Input Flexibility:**
    * `--operations-file <PATH>` is the primary, recommended method for clarity and version control. Passing `-` reads operations from `stdin`.
    * `--operations <JSON_STRING>` provides a convenient inline option for simple scripts or programmatic calls where creating a file is cumbersome.
    * Exactly one of `--operations-file` or `--operations` must be supplied. Providing neither should produce a user-facing error message.
3.  **Safety and Introspection:**
    * `--dry-run` is a critical safety feature, allowing users to preview the rendered Markdown result before committing any changes.
    * `--diff` provides immediate, actionable feedback on what the transaction *will do* or *did*, which is invaluable for debugging and verification. The diff uses the unified format with `original`/`modified` headers and never writes to disk.

### Example Usage

```sh
# Apply a set of changes from a YAML file to spec.md in-place
md-splice --file spec.md apply --operations-file changes.yaml

# Preview the changes without modifying the file
md-splice --file spec.md apply --operations-file changes.yaml --dry-run

# Show a diff of the pending changes
md-splice --file spec.md apply --operations-file changes.yaml --diff

# Read markdown from stdin, apply inline JSON operations, and write to stdout
cat README.md | md-splice apply --operations '[{"op":"delete","select_contains":"Old Section"}]'
```

## 3. Operations File Specification

The operations file will be a JSON or YAML document containing a top-level array of *operation objects*. The tool will auto-detect the format. For field names, we will use `snake_case` as it is more conventional for configuration files than the CLI's `kebab-case`.

Each object in the array must contain an `op` field specifying the action (`insert`, `replace`, or `delete`) and the relevant selector and content fields.

### Common Fields

These fields are available in every operation object.

| Field             | Type   | Description                                                                                             |
| :---------------- | :----- | :------------------------------------------------------------------------------------------------------ |
| `op`              | String | **Required.** The operation to perform. Must be one of `insert`, `replace`, or `delete`.                |
| `select_type`     | String | Selects a node by type (e.g., `p`, `h2`, `li`). Corresponds to `--select-type`.                           |
| `select_contains` | String | Selects a node by its text content (fixed string). Corresponds to `--select-contains`.                  |
| `select_regex`    | String | Selects a node by its text content (regex pattern). Corresponds to `--select-regex`.                      |
| `select_ordinal`  | Number | Selects the Nth matching node (1-indexed). Defaults to `1`. Corresponds to `--select-ordinal`.            |
| `comment`         | String | *Optional.* A user-defined comment to describe the operation's purpose. Ignored by the tool.            |

### Operation-Specific Fields

#### `op: "replace"`

| Field          | Type   | Description                                                                                             |
| :------------- | :----- | :------------------------------------------------------------------------------------------------------ |
| `content`      | String | The Markdown content to replace the node with. Mutually exclusive with `content_file`.                  |
| `content_file` | String | A path to a file containing the Markdown content. Path is relative to the current working directory.    |

#### `op: "insert"`

| Field          | Type   | Description                                                                                             |
| :------------- | :----- | :------------------------------------------------------------------------------------------------------ |
| `content`      | String | The Markdown content to insert. Mutually exclusive with `content_file`.                                 |
| `content_file` | String | A path to a file containing the Markdown content. Path is relative to the current working directory.    |
| `position`     | String | Position for insertion. One of `before`, `after`, `prepend_child`, `append_child`. Defaults to `after`. |

#### `op: "delete"`

| Field     | Type    | Description                                                                                             |
| :-------- | :------ | :------------------------------------------------------------------------------------------------------ |
| `section` | Boolean | If `true` and the target is a heading, deletes the entire section. Defaults to `false`.                 |

---

### Complete Example

Imagine we want to perform three modifications on a `TODO.md` file.

**Initial `TODO.md`:**

```markdown
# Project Tasks

Status: In Progress

# High Priority
- [ ] Design the API
- [ ] Write documentation

# Low Priority
- [ ] Refactor old module
```

**Operations File (`changes.yaml`):**

```yaml
- op: replace
  comment: "Update the project status to Complete."
  select_contains: "Status: In Progress"
  content: "Status: **Complete**"

- op: insert
  comment: "Add a new high-priority task for testing."
  select_type: li
  select_contains: "Write documentation"
  position: before
  content: "- [ ] Implement unit tests"

- op: delete
  comment: "Remove the entire 'Low Priority' section as it's no longer needed."
  select_type: h2
  select_contains: "Low Priority"
  section: true
```

**Equivalent JSON (`changes.json`):**

```json
[
  {
    "op": "replace",
    "comment": "Update the project status to Complete.",
    "select_contains": "Status: In Progress",
    "content": "Status: **Complete**"
  },
  {
    "op": "insert",
    "comment": "Add a new high-priority task for testing.",
    "select_type": "li",
    "select_contains": "Write documentation",
    "position": "before",
    "content": "- [ ] Implement unit tests"
  },
  {
    "op": "delete",
    "comment": "Remove the entire 'Low Priority' section as it's no longer needed.",
    "select_type": "h2",
    "select_contains": "Low Priority",
    "section": true
  }
]
```

**Command:**

```sh
md-splice --file TODO.md apply --operations-file changes.yaml
```

**Final `TODO.md`:**

```markdown
# Project Tasks

Status: **Complete**

# High Priority
- [ ] Design the API
- [ ] Implement unit tests
- [ ] Write documentation
```

## 4. Key Behaviors

* **Sequential Execution:** Operations are executed in the order they appear in the array. The selector for each subsequent operation is resolved against the AST that has been modified by all preceding operations in the same transaction.
* **Transactional Integrity:** If any operation fails (e.g., its selector does not find a matching node), the entire `apply` command will fail. No changes will be written, and the original file will remain untouched, thus guaranteeing atomicity. The tool will exit with a non-zero status code and print an error message indicating which operation failed and why.
* **Selector Consistency:** All selector logic (`locate`, `locate_all`) will be reused, ensuring that the behavior of `select_type`, `select_contains`, etc., is identical to their CLI counterparts.

## 5. Future Enhancements

While not part of the initial scope, this design opens the door for future improvements:

* **Variables:** Introduce a top-level `vars` map in the operations file and support `${VAR_NAME}` substitution in `content` and selector fields, allowing for templated transformations.
* **Conditional Operations:** Add an optional `if_exists` selector to an operation, causing it to be skipped gracefully if the condition isn't met, rather than failing the transaction.