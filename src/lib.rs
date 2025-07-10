//! Core library for md-splice, containing all logic for AST manipulation.

pub mod cli;
pub mod error;
pub mod locator;
pub mod splicer;

use crate::cli::{Cli, Command};
use crate::error::SpliceError;
use crate::locator::{locate, FoundNode, Selector};
use crate::splicer::{insert, insert_list_item, replace, replace_list_item};
use anyhow::{anyhow, Context};
use clap::Parser;
use markdown_ppp::parser::{parse_markdown, MarkdownParserState};
use markdown_ppp::printer::{config::Config as PrinterConfig, render_markdown};
use regex::Regex;
use std::fs;

/// The main entry point for the application logic.
pub fn run() -> anyhow::Result<()> {
    // Initialize the logger. This will be configured by the RUST_LOG environment variable.
    env_logger::init();
    
    // 1. Parse CLI args
    let cli = Cli::parse();
    // Note: Logger initialization will be added in a later step (I6).

    // 2. Get file path and ensure it's provided
    let file_path = cli
        .file
        .ok_or_else(|| anyhow!("--file argument is required"))?;

    // 3. Read input file
    let input_content = fs::read_to_string(&file_path)
        .with_context(|| format!("Failed to read input file: {}", file_path.display()))?;

    // 4. Parse markdown to AST
    let mut doc = parse_markdown(MarkdownParserState::default(), &input_content)
        .map_err(|e| anyhow!("Failed to parse input markdown: {}", e))?;

    // 5. Extract modification args and command type
    let (mod_args, is_replace) = match &cli.command {
        Command::Insert(args) => (args, false),
        Command::Replace(args) => (args, true),
    };

    // 6. Build Selector from modification args
    let selector = Selector {
        select_type: mod_args.select_type.clone(),
        select_contains: mod_args.select_contains.clone(),
        select_regex: mod_args
            .select_regex
            .as_ref()
            .map(|s| Regex::new(s))
            .transpose()
            .context("Invalid regex pattern for --select-regex")?,
        select_ordinal: mod_args.select_ordinal,
    };

    // 7. Locate target node
    let (found_node, is_ambiguous) = locate(&doc.blocks, &selector)?;

    // 8. Handle ambiguity warning (for I6)
    if is_ambiguous {
        // This will only be visible if a logger is initialized.
        log::warn!("Warning: Selector matched multiple nodes. Operation was applied to the first match only.");
    }

    // 9. Get content to splice in
    let new_content_str = match (&mod_args.content, &mod_args.content_file) {
        (Some(content), None) => Ok(content.clone()),
        (None, Some(path)) => fs::read_to_string(path)
            .with_context(|| format!("Failed to read content file: {}", path.display())),
        (None, None) => Err(SpliceError::NoContent.into()),
        (Some(_), Some(_)) => unreachable!("clap's conflicts_with should prevent this"),
    }?;

    // 10. Parse content markdown
    let new_content_doc = parse_markdown(MarkdownParserState::default(), &new_content_str)
        .map_err(|e| anyhow!("Failed to parse content markdown: {}", e))?;
    let new_blocks = new_content_doc.blocks;

    // 11. Splice/modify AST
    match found_node {
        FoundNode::Block { index, .. } => {
            if is_replace {
                replace(&mut doc.blocks, index, new_blocks);
            } else {
                insert(
                    &mut doc.blocks,
                    index,
                    new_blocks,
                    mod_args.position.clone(),
                )?;
            }
        }
        FoundNode::ListItem {
            block_index,
            item_index,
            ..
        } => {
            if is_replace {
                replace_list_item(&mut doc.blocks, block_index, item_index, new_blocks)?;
            } else {
                insert_list_item(
                    &mut doc.blocks,
                    block_index,
                    item_index,
                    new_blocks,
                    mod_args.position.clone(),
                )?;
            }
        }
    }

    // 12. Render AST to string
    let output_content = render_markdown(&doc, PrinterConfig::default());

    // 13. Write to output (handles I2, I3 will handle the `else` case)
    if let Some(output_path) = &cli.output {
        fs::write(output_path, output_content)
            .with_context(|| format!("Failed to write to output file: {}", output_path.display()))?;
        } else {
        // In-place modification.
        // 1. Create a named temporary file in the same directory as the original file.
        // This is crucial for ensuring an atomic rename operation later.
        let parent_dir = file_path.parent().ok_or_else(|| {
            anyhow!(
                "Could not determine parent directory of {}",
                file_path.display()
            )
        })?;

        let mut temp_file = tempfile::Builder::new()
            .prefix(".md-splice-")
            .suffix(".tmp")
            .tempfile_in(parent_dir)
            .with_context(|| {
                format!(
                    "Failed to create temporary file in {}",
                    parent_dir.display()
                )
            })?;

        // 2. Write the rendered content to the temporary file.
        use std::io::Write;
        temp_file
            .write_all(output_content.as_bytes())
            .with_context(|| "Failed to write to temporary file")?;

        // 3. Atomically replace the original file with the temporary file.
        // `persist` handles the atomic rename/move operation.
        temp_file
            .persist(&file_path)
            .with_context(|| format!("Failed to replace original file {}", file_path.display()))?;
    }

    Ok(())
}
