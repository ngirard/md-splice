# Proposed Specification: Scoped and Range-Based Selectors

The core design principle is to treat these new features not as entirely new types of selectors, but as **modifiers** to the existing, well-understood selector system. This maintains consistency and allows for powerful composition.

We will introduce two new concepts:

1. **Scoped Selectors:** These flags restrict the *search space* where the primary `--select-*` flags operate. They answer the question, "Where should I look?"
2. **Range Selectors:** These flags define a *span* of nodes for an operation, starting from the node found by the primary `--select-*` flags and ending at a node defined by the range selector. They answer the question, "How far should the operation apply?"

## 1. Scoped Selectors: `--after` and `--within`

These flags establish a "landmark" node first, and then run the primary selection logic relative to that landmark. To avoid inventing a new selector syntax (e.g., `--after "type:h2,contains:Foo"`) and to maintain consistency, we will use prefixed versions of the existing selector flags.

### `--after-*`: Searching After a Landmark

This modifier finds the first node matching the primary `--select-*` criteria that appears *after* a landmark node.

**New Flags:**

* `--after-select-type <TYPE>`
* `--after-select-contains <TEXT>`
* `--after-select-regex <REGEX>`
* `--after-select-ordinal <N>` (Defaults to 1)

**Behavior:**

1. The locator first finds the landmark node matching the `--after-select-*` criteria. If not found, the entire operation fails.
2. The search for the primary `--select-*` node then begins on the block *immediately following* the landmark node.
3. The `--select-ordinal` of the primary selector applies to the results found *within this restricted search space*.

**Example Use-Case:** Find the first paragraph after the "Installation" section.

```sh
# Given a README.md with "Installation" and "Usage" sections
md-splice --file README.md get \
  --select-type p \
  --select-ordinal 1 \
  --after-select-type h2 \
  --after-select-contains "Installation"
```

This is vastly more readable and less ambiguous than trying to chain commands. It directly expresses the relational intent.

### `--within-*`: Searching Inside a Landmark

This modifier restricts the search for the primary `--select-*` node to the children of a landmark node.

**New Flags:**

* `--within-select-type <TYPE>`
* `--within-select-contains <TEXT>`
* `--within-select-regex <REGEX>`
* `--within-select-ordinal <N>` (Defaults to 1)

**Behavior:**

1. The locator finds the landmark node matching the `--within-select-*` criteria. If not found, the operation fails.
2. The definition of "within" depends on the landmark node type:
    * **Heading:** The search space is the heading's "section" (all nodes between it and the next heading of the same or higher level). This is a powerful extension of the existing `--section` logic.
    * **List / Blockquote / etc.:** The search space is the list of child blocks directly contained by the node.
    * **Paragraph / Code Block / etc.:** These nodes cannot contain other blocks, so using them as a `--within` landmark with a block-level primary selector will result in an error (or simply no matches).
3. The primary `--select-*` search is performed only on nodes in this restricted space.

**Example Use-Case:** Delete the first to-do list item inside the "Future Features" section.

```sh
md-splice --file ROADMAP.md delete \
  --select-type li \
  --select-contains "[ ]" \
  --within-select-type h2 \
  --within-select-contains "Future Features"
```

**Constraint:** `--after-*` and `--within-*` flags are mutually exclusive to prevent ambiguous queries like "find a paragraph after the installation section within the usage section."

## 2. Range Selectors: `--until-*`

This modifier extends an operation from a starting node to an ending node. It pairs with the primary `--select-*` flags to define the range's start and the new `--until-*` flags to define the (exclusive) end. This is a more generalized and powerful version of the existing `--section` flag.

**New Flags:**

* `--until-type <TYPE>`
* `--until-contains <TEXT>`
* `--until-regex <REGEX>`

**Behavior:**

1. The primary `--select-*` flags locate the **start node** of the range (inclusive).
2. Starting from the node *after* the start node, the locator searches for the first node that matches the `--until-*` criteria. This is the **end node** of the range (exclusive).
3. The operation (`delete`, `get`, `replace`) applies to all nodes from the start node up to (but not including) the end node.
4. If no node matches the `--until-*` criteria, the range extends to the end of the document.
5. This feature is applicable to `delete`, `get`, and `replace` commands.

**Example Use-Case:** Delete the "Deprecated Methods" subsection, which ends right before the "Examples" subsection.

```sh
# This is the CLI equivalent of the original proposal, but far more ergonomic.
md-splice --file api.md delete \
  --select-type h2 --select-contains "Deprecated Methods" \
  --until-type h2 --until-contains "Examples"
```

**Example Use-Case:** Get all content from the "Usage" section until the end of the document.

```sh
md-splice --file README.md get \
  --select-type h2 --select-contains "Usage" \
  --until-type h1 --select-contains "License" # Or omit --until-* to go to EOF
```

**Composition:** Range selectors can be combined with Scoped Selectors. For instance, you can select a range that exists *after* a certain landmark.

```sh
# Replace the first two paragraphs after the introduction heading
md-splice --file doc.md replace --content "..." \
  --select-type p --select-ordinal 1 \
  --after-select-type h1 --after-select-contains "Introduction" \
  --until-type p --select-ordinal 3 # The range ends before the 3rd paragraph
```

## 3. Integration with `apply` (Transactions)

The true power of this system is unlocked in the `apply` command's YAML/JSON format. The flat `snake_case` keys can be extended to support nested selector objects.

**Proposed Schema Extension:**

An operation's `selector` object can now contain optional `after` and `within` keys, which are themselves selector objects. The operation object itself can contain an optional `until` key, which is also a selector object.

**Example `operations.yaml`:**

```yaml
# Operation 1: Delete the entire "Deprecated API" section.
- op: delete
  comment: "Remove the deprecated API section, which is between H2s"
  selector:
    select_type: h2
    select_contains: "Deprecated API"
  until:
    select_type: h2
    select_contains: "Examples"

# Operation 2: Add a new task to the "High Priority" list, but not any other list.
- op: insert
  comment: "Add a critical task to the main high-priority list"
  selector:
    # Select the list itself to append a child to it.
    select_type: list
    # Scope the search for the list to be within the "High Priority" section.
    within:
      select_type: h2
      select_contains: "High Priority"
  position: append_child
  content: "- [ ] Address security vulnerability CVE-2024-12345"

# Operation 3: Replace the first paragraph located *after* the table of contents.
- op: replace
  comment: "Update the introductory paragraph that follows the ToC"
  selector:
    select_type: p
    select_ordinal: 1
    after:
      select_type: list # Assuming the ToC is the first list in the doc
      select_ordinal: 1
  content: "This is the new, updated introductory paragraph."
```

This structure is clean, highly expressive, and directly maps the conceptual model to the configuration format. It allows an LLM to construct sophisticated, multi-step refactoring plans that can be executed in a single, atomic transaction.

## Summary of UX Advantages

* **Consistency:** Reuses the established `--select-*` pattern with prefixes (`--after-`, `--within-`, `--until-`) instead of introducing a new syntax.
* **Composability:** Scoped selectors and range selectors can be combined to express very precise and complex queries.
* **Readability:** The flags read like natural language: `delete --select-type h2 --contains "Old" --until-type h2 --contains "New"`.
* **Power:** Unlocks the full range of relational and range-based operations requested, providing a robust tool for complex document manipulation.
* **Transaction-Ready:** The design translates beautifully to a structured format like YAML, making the `apply` command the ultimate power-user feature.
