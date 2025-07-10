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

    // --- Insert-specific options ---
    /// Position for the 'insert' operation.
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
