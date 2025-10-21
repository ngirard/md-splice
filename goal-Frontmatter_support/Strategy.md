# `Strategy.md`

## 1. Objective

This document outlines the implementation strategy for adding frontmatter support to `md-splice`, as detailed in `Specification.md`. The primary audience for this strategy is an LLM developer tasked with the implementation.

The core tenets of this strategy are:
1. **Test-Driven Development (TDD):** Every functional change must be preceded or accompanied by a corresponding test. We will heavily rely on integration tests (`assert_cmd`, `assert_fs`) to validate CLI behavior from a user's perspective.
2. **Incremental Implementation:** The feature will be built in logical, sequential phases, ensuring a stable foundation before adding complexity.
3. **Documentation First:** User-facing documentation (`README.md`, CLI help text) is not an afterthought; it is an integral part of the implementation process for each feature.

## 2. Overall Plan

The implementation is broken down into five distinct phases. Each phase builds upon the previous one, ensuring a robust and verifiable result at every stage.

* **Phase 1: Foundational - Parsing & Data Structures:** Isolate and parse frontmatter without modifying any CLI logic.
* **Phase 2: Read-Only Operations:** Implement the `md-splice frontmatter get` command.
* **Phase 3: Write Operations:** Implement the `md-splice frontmatter set` and `delete` commands.
* **Phase 4: Transactional Integration:** Integrate `set_frontmatter`, `delete_frontmatter`, and `replace_frontmatter` operations into the `apply` command.
* **Phase 5: Documentation & Finalization:** Update all user-facing documentation and perform a final review.

---

## Phase 1: Foundational - Parsing & Data Structures

**Goal:** Reliably separate frontmatter from the Markdown body and parse it into a structured, in-memory representation. This phase introduces no new CLI commands.

1. **Create Test Cases:**
    * In a new `tests/fixtures/frontmatter` directory, create several sample Markdown files:
        * `yaml_simple.md`: Basic YAML frontmatter.
        * `toml_simple.md`: Basic TOML frontmatter.
        * `no_frontmatter.md`: A standard Markdown file.
        * `malformed.md`: A file with invalid YAML/TOML in its frontmatter block.
        * `empty_frontmatter.md`: A file with `---` delimiters but no content.

2. **Define Data Structures:**
    * Create a new module, `src/frontmatter.rs`.
    * Inside it, define an enum `FrontmatterFormat { Yaml, Toml }`.
    * Define a struct `ParsedDocument` to hold the result of parsing:
        ```rust
        pub struct ParsedDocument {
            pub frontmatter: Option<serde_yaml::Value>,
            pub body: String,
            pub format: Option<FrontmatterFormat>,
        }
        ```
        Using `serde_yaml::Value` provides a flexible way to represent both YAML and TOML data structures.

3. **Implement Parser:**
    * In `src/frontmatter.rs`, create a `parse` function: `pub fn parse(content: &str) -> anyhow::Result<ParsedDocument>`.
    * This function will:
        * Use regex or string splitting to identify `---` (YAML) or `+++` (TOML) delimiters at the start of the file.
        * If found, extract the frontmatter string and the body string.
        * Use `serde_yaml::from_str` or `toml::from_str` to parse the frontmatter into a `serde_yaml::Value`.
        * Return a `ParsedDocument` instance.
        * If no frontmatter is found, return `ParsedDocument` with `frontmatter: None`.
        * Return an error for malformed frontmatter.

4. **Write Unit Tests:**
    * Create a `tests` mod in `src/frontmatter.rs`.
    * Write unit tests for the `parse` function using the fixture files created in step 1. Assert that each file is parsed into the correct `ParsedDocument` structure.

5. **Integrate into `lib.rs`:**
    * Modify the entry point of `run()` in `src/lib.rs`. Instead of immediately calling `markdown_ppp::parser::parse_markdown`, first call your new `frontmatter::parse`.
    * The `markdown_ppp` parser should now operate on the `body` of the `ParsedDocument`.
    * The `ParsedDocument` struct will be passed through the program to hold the complete state of the file.

## Phase 2: Read-Only Operations (`frontmatter get`)

**Goal:** Implement the `get` subcommand as specified in `Specification.md`. This validates the parsing logic and establishes the new CLI structure.

1. **Update `cli.rs`:**
    * Add a new `Frontmatter(FrontmatterCommand)` variant to the main `Command` enum.
    * Define `FrontmatterCommand` as an enum with a `Get(FrontmatterGetArgs)` variant.
    * Define the `FrontmatterGetArgs` struct with `--key` and `--output-format` options.

2. **Write Integration Tests:**
    * In a new `tests/cmd_frontmatter.rs` file, write tests for `md-splice frontmatter get`.
    * Use `assert_cmd` to test:
        * Getting a top-level key from a YAML file.
        * Getting a nested key using dot-notation (e.g., `author.name`).
        * Getting an entire frontmatter block (no `--key`).
        * Correct output for `--output-format json`.
        * An error is produced when a key does not exist.
        * The command exits gracefully for a file with no frontmatter.

3. **Implement Logic in `lib.rs`:**
    * Add the `Frontmatter` command match arm in `run()`.
    * Implement the `process_frontmatter_get` function.
    * This function will:
        * Use a helper to traverse the `serde_yaml::Value` based on the dot-notated key.
        * Serialize the resulting value based on the `--output-format` flag.
        * Print the result to stdout.

## Phase 3: Write Operations (`frontmatter set`/`delete`)

**Goal:** Implement the `set` and `delete` subcommands, introducing mutation and serialization logic.

1. **Update `cli.rs`:**
    * Add `Set(FrontmatterSetArgs)` and `Delete(FrontmatterDeleteArgs)` variants to the `FrontmatterCommand` enum.
    * Define the corresponding argument structs as per `Specification.md`.

2. **Write Integration Tests:**
    * In `tests/cmd_frontmatter.rs`, add tests for `set` and `delete`.
    * Use `assert_fs` to create temporary files and verify their contents after the command runs.
    * Test cases:
        * `set`: Adding a new key.
        * `set`: Updating an existing key's value.
        * `set`: Creating frontmatter from scratch in a file.
        * `set`: Adding a nested key, ensuring parent maps are created.
        * `set`: Using `--value` with different types (string, number, boolean).
        * `delete`: Removing a key.
        * Verify that the original format (YAML/TOML) is preserved on modification.
        * Verify that the existing atomic file-write mechanism is used.

3. **Implement Logic:**
    * Implement `process_frontmatter_set` and `process_frontmatter_delete` functions.
    * These will modify the `serde_yaml::Value` in the `ParsedDocument`.
    * Create a `serialize` function in `src/frontmatter.rs` that takes a `ParsedDocument` and reconstructs the full file content as a string. It must use the stored `format` to serialize the `frontmatter` value back to the correct format (YAML or TOML) with the correct delimiters.
    * The main `run()` function will call this `serialize` function at the end of a successful operation and use the result for writing to a file.

## Phase 4: Transactional Integration (`apply`)

**Goal:** Integrate frontmatter operations into the `apply` command for atomic, multi-step modifications.

1. **Update `transaction.rs`:**
    * Add `SetFrontmatter(SetFrontmatterOperation)`, `DeleteFrontmatter(DeleteFrontmatterOperation)`, and `ReplaceFrontmatter(ReplaceFrontmatterOperation)` variants to the `Operation` enum.
    * Define the corresponding structs, ensuring they are deserializable from YAML/JSON as specified.

2. **Write `transaction.rs` Unit Tests:**
    * Add tests to deserialize YAML/JSON files containing the new operation types to ensure the schemas are correct.

3. **Write `apply` Integration Tests:**
    * In `tests/cmd_apply.rs`, create new tests using operations files (`.yaml`).
    * Test a transaction with only frontmatter operations.
    * **Crucially, test a mixed transaction:** one that modifies both the frontmatter (`set_frontmatter`) and the Markdown body (`insert` or `replace`).
    * Test atomicity: Create a transaction where a body operation succeeds but a subsequent frontmatter operation fails (e.g., invalid key). Assert that the original file is **completely unchanged**.
    * Test the `--diff` and `--dry-run` flags with frontmatter changes.

4. **Implement Logic in `lib.rs`:**
    * Modify the `process_apply` function. It will now need to manage both the `ParsedDocument` (for frontmatter) and the `Vec<Block>` (for the body) throughout the transaction.
    * Add match arms for the new operation types. These will call the same underlying logic developed for the standalone `set`/`delete` commands, but will operate on the in-memory `ParsedDocument`.
    * Ensure that if any operation fails, the function returns an error immediately, preventing the final file write.

## Phase 5: Documentation & Finalization

**Goal:** Ensure the new feature is clearly documented for end-users and the codebase is clean.

1. **Update `README.md`:**
    * Add a new top-level section titled "Frontmatter Operations".
    * Document the `frontmatter get`, `set`, and `delete` subcommands with clear examples, referencing the specification.
    * Update the "Multi-operation transactions with `apply`" section.
    * Add a new subsection explaining the `set_frontmatter`, `delete_frontmatter`, and `replace_frontmatter` operations.
    * Provide the full LLM agent walkthrough example from `Specification.md`.

2. **Verify CLI Help Text:**
    * Run `md-splice --help`, `md-splice frontmatter --help`, `md-splice frontmatter set --help`, etc.
    * Ensure all help messages, generated by `clap`, are clear, accurate, and reflect the new functionality. Add doc comments in `cli.rs` as needed to improve them.

3. **Code Cleanup and Final Review:**
    * Run `cargo make fmt` and `cargo make clippy` to ensure code quality.
    * Run the entire test suite with `cargo make test` to confirm no regressions were introduced.
    * Review the code for clarity, comments, and adherence to Rust best practices.

## Definition of Done

The frontmatter feature is considered complete when:
* All tests for all five phases are implemented and passing.
* All commands and operations defined in `Specification.md` are fully implemented.
* The `README.md` and CLI help text are fully updated and accurate.
* The entire project passes all checks (`cargo make check-all`).
