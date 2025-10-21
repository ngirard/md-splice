# Specification: Frontmatter Support for `md-splice`

## 1. Executive Summary

This specification details the addition of first-class frontmatter support to `md-splice`. The goal is to enable reading and writing YAML/TOML metadata with the same precision, safety, and transactional integrity as the existing AST-based operations.

This feature introduces a new `frontmatter` subcommand namespace (`md-splice frontmatter <COMMAND>`) for direct CLI interaction and integrates new operation types (`set_frontmatter`, `delete_frontmatter`, etc.) into the powerful `apply` command, making it a vital tool for document lifecycle management by LLM agents.

## 2. Guiding Principles

1. **Consistency:** The new commands and options should mirror the existing CLI structure. Naming conventions (`--key`, `--value` vs. `--select-contains`, `--content`), file handling, and operational logic will feel familiar.
2. **Clarity of Scope:** Operations on frontmatter are distinct from operations on the Markdown body. The CLI will enforce this separation to prevent ambiguity. A user is either targeting the frontmatter *or* the Markdown AST, never both in a single simple command.
3. **Power & Flexibility:** The tool must handle common frontmatter value types (strings, numbers, booleans, arrays) gracefully. Key selection should support nested objects via dot-notation (e.g., `author.name`).
4. **Transactional Integrity:** All frontmatter modifications, especially within an `apply` transaction, must be atomic. If any operation in a transaction fails (whether on frontmatter or the body), the entire file remains untouched.
5. **Intelligence:** The tool should automatically detect existing frontmatter format (YAML vs. TOML) and preserve it. When creating new frontmatter, it should default to a sensible choice (YAML) but allow user override.

## 3. Proposed CLI Structure

To maintain a clear separation of concerns, all frontmatter operations will be nested under a new `frontmatter` subcommand.

```
md-splice frontmatter <SUBCOMMAND> [OPTIONS]
```

This creates a clean namespace and avoids polluting the top-level commands with potentially conflicting flags.

### 3.1. `md-splice frontmatter get`

Reads a value from the frontmatter and prints it to stdout.

**Usage:**
```sh
md-splice --file <PATH> frontmatter get [OPTIONS]
```

**Options:**
* `--key <KEY>`: The key to retrieve. Supports dot-notation for nested values (e.g., `metadata.version`). If omitted, the entire frontmatter block is retrieved.
* `--output-format <FORMAT>`: Specifies the output format.
    * `string`: (Default) Prints the raw value. For complex types like arrays or objects, this will be a YAML representation.
    * `json`: Prints the value as a JSON string or object.
    * `yaml`: Prints the value as a YAML string or object.

**Examples:**
```sh
# Get the value of the 'status' key
$ md-splice -f spec.md frontmatter get --key status
draft

# Get a nested key and format as JSON
$ md-splice -f spec.md frontmatter get --key reviewers[0].name --output-format json
"Alice"

# Get the entire frontmatter block as YAML
$ md-splice -f spec.md frontmatter get --output-format yaml
status: draft
reviewers:
  - name: Alice
    email: alice@example.com
```

### 3.2. `md-splice frontmatter set`

Sets a key to a given value in the frontmatter. If the key exists, it is updated. If it does not exist, it is created. If no frontmatter exists in the file, it will be created.

**Usage:**
```sh
md-splice --file <PATH> frontmatter set --key <KEY> [OPTIONS]
```

**Options:**
* `--key <KEY>`: (Required) The key to set, supporting dot-notation.
* `--value <VALUE_STRING>`: (Required, conflicts with `--value-file`) The value to set. **The value is parsed as YAML**, allowing for native types.
    * `--value "in-review"` (string)
    * `--value 42` (number)
    * `--value true` (boolean)
    * `--value "['alice', 'bob']"` (array)
* `--value-file <PATH>`: (Required, conflicts with `--value`) A file containing the value to set. The content is parsed as YAML.
* `--format <yaml|toml>`: (Optional) When creating a *new* frontmatter block, specifies the format. Defaults to `yaml`. Ignored if frontmatter already exists.

**Examples:**
```sh
# Set a simple string value
$ md-splice -f spec.md frontmatter set --key status --value published

# Set a numeric value
$ md-splice -f spec.md frontmatter set --key version --value 1.2

# Add a new reviewer to a nested list (assuming 'reviewers' exists as a list)
$ md-splice -f spec.md frontmatter set --key 'reviewers[+].name' --value "Charlie" 
# Note: A syntax like `[+]` for array append is a powerful extension to consider.

# Create frontmatter if it doesn't exist
$ md-splice -f new.md frontmatter set --key title --value "New Document"
```

### 3.3. `md-splice frontmatter delete`

Removes a key-value pair from the frontmatter.

**Usage:**
```sh
md-splice --file <PATH> frontmatter delete --key <KEY>
```

**Options:**
* `--key <KEY>`: (Required) The key to delete, supporting dot-notation.

**Example:**
```sh
# Remove the 'draft_notes' key
$ md-splice -f spec.md frontmatter delete --key draft_notes
```

## 4. `apply` Command Integration

This is the core feature for LLM agents. We will introduce new operation types specifically for frontmatter, which can be mixed with existing body operations in a single atomic transaction.

### 4.1. New Operation: `set_frontmatter`

Adds or updates a key-value pair.

**Schema:**
```yaml
- op: set_frontmatter
  key: string # Dot-notation supported
  value: any # Value is parsed as YAML/JSON
  comment: string (optional)
```

**Example:**
```yaml
- op: set_frontmatter
  key: "status"
  value: "published"
- op: set_frontmatter
  key: "last_updated"
  value: "2024-05-21"
- op: set_frontmatter
  key: "approved"
  value: true
- op: set_frontmatter
  key: "reviewers"
  value: ["Alice", "Bob", "Charlie"]
```

### 4.2. New Operation: `delete_frontmatter`

Removes a key-value pair.

**Schema:**
```yaml
- op: delete_frontmatter
  key: string # Dot-notation supported
  comment: string (optional)
```

**Example:**
```yaml
- op: delete_frontmatter
  key: "legacy_id"
```

### 4.3. New Operation: `replace_frontmatter`

Replaces the entire frontmatter block with new content. This is useful for complete metadata overhauls.

**Schema:**
```yaml
- op: replace_frontmatter
  content: object # The new frontmatter content as a YAML/JSON object
  content_file: path (optional)
  comment: string (optional)
```

**Example:**
```yaml
- op: replace_frontmatter
  content:
    title: "Final Specification"
    status: "approved"
    version: 2.0
```

## 5. Example Walkthrough: An LLM Agent's Task

**Task:** "Mark `spec-A.md` as 'Approved', set today's date as `last_updated`, remove the `review_deadline` field, and add a final approval notice to the 'Summary' section."

**`spec-A.md` (Before):**
```markdown
---
title: "Specification for Feature X"
status: "in-review"
review_deadline: "2024-05-20"
---
# Specification A

# Summary

This document outlines the requirements for Feature X.
```

**`operations.yaml`:**
```yaml
# Operations for approving Spec-A
- op: set_frontmatter
  comment: "Update status to approved"
  key: "status"
  value: "approved"

- op: set_frontmatter
  comment: "Set the last updated date"
  key: "last_updated"
  value: "2024-05-21" # Assuming today's date

- op: delete_frontmatter
  comment: "Remove the now-irrelevant deadline"
  key: "review_deadline"

- op: insert
  comment: "Add an approval notice to the summary section"
  selector:
    select_type: h2
    select_contains: "Summary"
  position: append_child
  content: |
    > **Approved:** This specification was approved on 2024-05-21.
```

**Command:**
```sh
md-splice --file spec-A.md apply --operations-file operations.yaml --diff
```
The `--diff` flag allows the agent (or a human supervisor) to review the changes before committing them. If the diff is correct, the command is re-run without `--diff`.

**`spec-A.md` (After):**
```markdown
---
title: "Specification for Feature X"
status: "approved"
last_updated: "2024-05-21"
---
# Specification A

# Summary

This document outlines the requirements for Feature X.

> **Approved:** This specification was approved on 2024-05-21.
```

This example demonstrates the seamless, atomic combination of frontmatter and body modifications in a single, declarative transaction.

## 6. Implementation Considerations

* **Parsing:** A robust frontmatter parsing library (like `gray_matter` for Node.js, or a Rust equivalent) should be used to reliably separate the frontmatter from the Markdown body and parse it into a `serde_yaml::Value` or similar structure.
* **Serialization:** After modifications, the frontmatter data structure must be serialized back to its original format (YAML or TOML) and prepended to the rendered Markdown body, ensuring correct delimiters (`---` or `+++`).
* **Error Handling:**
    * Malformed frontmatter in the source file should result in a clear error.
    * Attempting to `get` or `delete` a non-existent key should fail with a distinct error, consistent with `SpliceError::NodeNotFound`.
    * In `apply` transactions, any frontmatter operation failure must abort the entire transaction.
* **Dependencies:** This will likely add `serde_yaml` and potentially a TOML parser (`toml`) to the project's dependencies. The existing `serde_yaml` can be leveraged for both parsing and value representation.
