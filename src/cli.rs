//! Defines the command-line interface for the application.

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "md-splice",
    version,
    about = "Splice and modify Markdown files with AST-level precision."
)]
pub struct Cli {
    /// The Markdown file to modify. [default: reads from stdin]
    #[arg(short, long, global = true, value_name = "FILE_PATH")]
    pub file: Option<PathBuf>,

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
    /// Delete a Markdown node or section.
    #[command(alias = "remove")]
    Delete(DeleteArgs),
    /// Read Markdown content matching a selector without modifying the file.
    Get(GetArgs),
    /// Apply a sequence of transactional operations to the document.
    Apply(ApplyArgs),
}

#[derive(Parser, Debug)]
pub struct ModificationArgs {
    // --- Content to be added ---
    /// The Markdown content to insert or replace with.
    #[arg(
        short,
        long,
        value_name = "MARKDOWN_STRING",
        conflicts_with = "content_file",
        allow_hyphen_values = true
    )]
    pub content: Option<String>,

    /// A file containing the Markdown content. Use '-' to read from stdin.
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

    /// Restrict the search to the first match that occurs after another selector.
    #[arg(long = "after-select-type", value_name = "TYPE")]
    pub after_select_type: Option<String>,

    /// Restrict the search to the first match that occurs after another selector.
    #[arg(long = "after-select-contains", value_name = "TEXT")]
    pub after_select_contains: Option<String>,

    /// Restrict the search to the first match that occurs after another selector.
    #[arg(long = "after-select-regex", value_name = "REGEX")]
    pub after_select_regex: Option<String>,

    /// Choose the Nth landmark match for the `--after` selector (1-indexed).
    #[arg(long = "after-select-ordinal", value_name = "N")]
    pub after_select_ordinal: Option<usize>,

    /// Restrict the search to nodes contained within another selector.
    #[arg(long = "within-select-type", value_name = "TYPE")]
    pub within_select_type: Option<String>,

    /// Restrict the search to nodes contained within another selector.
    #[arg(long = "within-select-contains", value_name = "TEXT")]
    pub within_select_contains: Option<String>,

    /// Restrict the search to nodes contained within another selector.
    #[arg(long = "within-select-regex", value_name = "REGEX")]
    pub within_select_regex: Option<String>,

    /// Choose the Nth landmark match for the `--within` selector (1-indexed).
    #[arg(long = "within-select-ordinal", value_name = "N")]
    pub within_select_ordinal: Option<usize>,

    /// Select nodes up to (but not including) another selector.
    #[arg(long = "until-type", value_name = "TYPE")]
    pub until_type: Option<String>,

    /// Select nodes up to (but not including) another selector.
    #[arg(long = "until-contains", value_name = "TEXT")]
    pub until_contains: Option<String>,

    /// Select nodes up to (but not including) another selector.
    #[arg(long = "until-regex", value_name = "REGEX")]
    pub until_regex: Option<String>,

    // --- Insert-specific options ---
    /// Position for the 'insert' operation.
    #[arg(short, long, value_enum, default_value_t = InsertPosition::After)]
    pub position: InsertPosition,
}

/// Arguments for the `delete` command.
#[derive(Parser, Debug)]
pub struct DeleteArgs {
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

    /// Restrict the search to the first match that occurs after another selector.
    #[arg(long = "after-select-type", value_name = "TYPE")]
    pub after_select_type: Option<String>,

    /// Restrict the search to the first match that occurs after another selector.
    #[arg(long = "after-select-contains", value_name = "TEXT")]
    pub after_select_contains: Option<String>,

    /// Restrict the search to the first match that occurs after another selector.
    #[arg(long = "after-select-regex", value_name = "REGEX")]
    pub after_select_regex: Option<String>,

    /// Choose the Nth landmark match for the `--after` selector (1-indexed).
    #[arg(long = "after-select-ordinal", value_name = "N")]
    pub after_select_ordinal: Option<usize>,

    /// Restrict the search to nodes contained within another selector.
    #[arg(long = "within-select-type", value_name = "TYPE")]
    pub within_select_type: Option<String>,

    /// Restrict the search to nodes contained within another selector.
    #[arg(long = "within-select-contains", value_name = "TEXT")]
    pub within_select_contains: Option<String>,

    /// Restrict the search to nodes contained within another selector.
    #[arg(long = "within-select-regex", value_name = "REGEX")]
    pub within_select_regex: Option<String>,

    /// Choose the Nth landmark match for the `--within` selector (1-indexed).
    #[arg(long = "within-select-ordinal", value_name = "N")]
    pub within_select_ordinal: Option<usize>,

    /// Select nodes up to (but not including) another selector.
    #[arg(long = "until-type", value_name = "TYPE")]
    pub until_type: Option<String>,

    /// Select nodes up to (but not including) another selector.
    #[arg(long = "until-contains", value_name = "TEXT")]
    pub until_contains: Option<String>,

    /// Select nodes up to (but not including) another selector.
    #[arg(long = "until-regex", value_name = "REGEX")]
    pub until_regex: Option<String>,

    // --- Delete-specific options ---
    /// When deleting a heading, also delete its entire section.
    #[arg(long, requires = "select_type")]
    pub section: bool,
}

/// Arguments for the `get` command.
#[derive(Parser, Debug)]
pub struct GetArgs {
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
    #[arg(
        long,
        value_name = "N",
        default_value_t = 1,
        conflicts_with = "select_all"
    )]
    pub select_ordinal: usize,

    /// Restrict the search to the first match that occurs after another selector.
    #[arg(long = "after-select-type", value_name = "TYPE")]
    pub after_select_type: Option<String>,

    /// Restrict the search to the first match that occurs after another selector.
    #[arg(long = "after-select-contains", value_name = "TEXT")]
    pub after_select_contains: Option<String>,

    /// Restrict the search to the first match that occurs after another selector.
    #[arg(long = "after-select-regex", value_name = "REGEX")]
    pub after_select_regex: Option<String>,

    /// Choose the Nth landmark match for the `--after` selector (1-indexed).
    #[arg(long = "after-select-ordinal", value_name = "N")]
    pub after_select_ordinal: Option<usize>,

    /// Restrict the search to nodes contained within another selector.
    #[arg(long = "within-select-type", value_name = "TYPE")]
    pub within_select_type: Option<String>,

    /// Restrict the search to nodes contained within another selector.
    #[arg(long = "within-select-contains", value_name = "TEXT")]
    pub within_select_contains: Option<String>,

    /// Restrict the search to nodes contained within another selector.
    #[arg(long = "within-select-regex", value_name = "REGEX")]
    pub within_select_regex: Option<String>,

    /// Choose the Nth landmark match for the `--within` selector (1-indexed).
    #[arg(long = "within-select-ordinal", value_name = "N")]
    pub within_select_ordinal: Option<usize>,

    /// Select nodes up to (but not including) another selector.
    #[arg(
        long = "until-type",
        value_name = "TYPE",
        conflicts_with = "select_all"
    )]
    pub until_type: Option<String>,

    /// Select nodes up to (but not including) another selector.
    #[arg(
        long = "until-contains",
        value_name = "TEXT",
        conflicts_with = "select_all"
    )]
    pub until_contains: Option<String>,

    /// Select nodes up to (but not including) another selector.
    #[arg(
        long = "until-regex",
        value_name = "REGEX",
        conflicts_with = "select_all"
    )]
    pub until_regex: Option<String>,

    /// When selecting a heading, include the entire section.
    #[arg(long, requires = "select_type")]
    pub section: bool,

    /// Select all nodes matching the criteria instead of a single node.
    #[arg(long)]
    pub select_all: bool,

    /// Separator to print between results when --select-all is used. [default: "\n"]
    #[arg(
        long,
        default_value = "\n",
        requires = "select_all",
        allow_hyphen_values = true
    )]
    pub separator: String,
}

/// Arguments for the `apply` command.
#[derive(Parser, Debug)]
pub struct ApplyArgs {
    /// Path to a JSON or YAML file containing the operations. Use '-' for stdin.
    #[arg(short = 'O', long, value_name = "PATH", conflicts_with = "operations")]
    pub operations_file: Option<PathBuf>,

    /// JSON string describing the operations inline.
    #[arg(long, value_name = "JSON_STRING", conflicts_with = "operations_file")]
    pub operations: Option<String>,

    /// Preview the result without writing any files.
    #[arg(long)]
    pub dry_run: bool,

    /// Show a diff of the pending changes instead of writing files.
    #[arg(long)]
    pub diff: bool,
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
