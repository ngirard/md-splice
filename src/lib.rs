//! Core library for md-splice, containing all logic for AST manipulation.

pub mod cli;
pub mod error;
pub mod locator;
pub mod splicer;

use crate::cli::{Cli, Command, DeleteArgs, ModificationArgs};
use crate::error::SpliceError;
use crate::locator::{locate, FoundNode, Selector};
use crate::splicer::{
    delete, delete_list_item, delete_section, insert, insert_list_item, replace, replace_list_item,
};
use anyhow::{anyhow, Context};
use clap::Parser;
use markdown_ppp::ast::Block;
use markdown_ppp::parser::{parse_markdown, MarkdownParserState};
use markdown_ppp::printer::{config::Config as PrinterConfig, render_markdown};
use regex::Regex;
use std::fs;
use std::io::{self, Read, Write};

/// The main entry point for the application logic.
pub fn run() -> anyhow::Result<()> {
    // Initialize the logger. This will be configured by the RUST_LOG environment variable.
    env_logger::init();

    // 1. Parse CLI args
    let cli = Cli::parse();
    let Cli {
        file,
        output,
        command,
    } = cli;

    // 2. Check for ambiguous STDIN sources before any I/O
    if let Command::Insert(args) | Command::Replace(args) = &command {
        let content_from_stdin = args
            .content_file
            .as_deref()
            .is_some_and(|p| p.to_string_lossy() == "-");

        if file.is_none() && content_from_stdin {
            return Err(SpliceError::AmbiguousStdinSource.into());
        }
    }

    // 3. Read input content from file or stdin
    let input_content = if let Some(file_path) = &file {
        fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read input file: {}", file_path.display()))?
    } else {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        buf
    };

    // 4. Parse markdown to AST
    let mut doc = parse_markdown(MarkdownParserState::default(), &input_content)
        .map_err(|e| anyhow!("Failed to parse input markdown: {}", e))?;

    match command {
        Command::Insert(args) => {
            process_insert_or_replace(&mut doc.blocks, args, false)?;
        }
        Command::Replace(args) => {
            process_insert_or_replace(&mut doc.blocks, args, true)?;
        }
        Command::Delete(args) => {
            process_delete(&mut doc.blocks, args)?;
        }
    }

    // 5. Render AST to string
    let output_content = render_markdown(&doc, PrinterConfig::default());

    // 12. Write to output (file, in-place, or stdout)
    if let Some(output_path) = &output {
        fs::write(output_path, output_content).with_context(|| {
            format!("Failed to write to output file: {}", output_path.display())
        })?;
    } else if let Some(file_path) = &file {
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
        temp_file
            .write_all(output_content.as_bytes())
            .with_context(|| "Failed to write to temporary file")?;

        // 3. Atomically replace the original file with the temporary file.
        // `persist` handles the atomic rename/move operation.
        temp_file
            .persist(file_path)
            .with_context(|| format!("Failed to replace original file {}", file_path.display()))?;
    } else {
        // Input was from stdin, so output to stdout.
        io::stdout().write_all(output_content.as_bytes())?;
    }

    Ok(())
}

fn process_insert_or_replace(
    doc_blocks: &mut Vec<Block>,
    args: ModificationArgs,
    is_replace: bool,
) -> anyhow::Result<()> {
    let ModificationArgs {
        content,
        content_file,
        select_type,
        select_contains,
        select_regex,
        select_ordinal,
        position,
    } = args;

    let select_regex = if let Some(pattern) = select_regex {
        Some(Regex::new(&pattern).context("Invalid regex pattern for --select-regex")?)
    } else {
        None
    };

    let selector = Selector {
        select_type,
        select_contains,
        select_regex,
        select_ordinal,
    };

    let (found_node, is_ambiguous) = locate(&*doc_blocks, &selector)?;

    if is_ambiguous {
        log::warn!(
            "Warning: Selector matched multiple nodes. Operation was applied to the first match only."
        );
    }

    let new_content_str = match (content, content_file) {
        (Some(content), None) => Ok(content),
        (None, Some(path)) => {
            if path.to_string_lossy() == "-" {
                let mut buf = String::new();
                io::stdin().read_to_string(&mut buf)?;
                Ok(buf)
            } else {
                fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read content file: {}", path.display()))
            }
        }
        (None, None) => Err(SpliceError::NoContent.into()),
        (Some(_), Some(_)) => unreachable!("clap's conflicts_with should prevent this"),
    }?;

    let new_content_doc = parse_markdown(MarkdownParserState::default(), &new_content_str)
        .map_err(|e| anyhow!("Failed to parse content markdown: {}", e))?;
    let new_blocks = new_content_doc.blocks;

    match found_node {
        FoundNode::Block { index, .. } => {
            if is_replace {
                replace(doc_blocks, index, new_blocks);
            } else {
                insert(doc_blocks, index, new_blocks, position)?;
            }
        }
        FoundNode::ListItem {
            block_index,
            item_index,
            ..
        } => {
            if is_replace {
                replace_list_item(doc_blocks, block_index, item_index, new_blocks)?;
            } else {
                insert_list_item(doc_blocks, block_index, item_index, new_blocks, position)?;
            }
        }
    }

    Ok(())
}

fn process_delete(doc_blocks: &mut Vec<Block>, args: DeleteArgs) -> anyhow::Result<()> {
    let DeleteArgs {
        select_type,
        select_contains,
        select_regex,
        select_ordinal,
        section,
    } = args;

    let select_regex = if let Some(pattern) = select_regex {
        Some(Regex::new(&pattern).context("Invalid regex pattern for --select-regex")?)
    } else {
        None
    };

    let selector = Selector {
        select_type,
        select_contains,
        select_regex,
        select_ordinal,
    };

    let (found_node, is_ambiguous) = locate(&*doc_blocks, &selector)?;

    if is_ambiguous {
        log::warn!(
            "Warning: Selector matched multiple nodes. Operation was applied to the first match only."
        );
    }

    match found_node {
        FoundNode::Block { index, block } => {
            if section {
                if matches!(block, Block::Heading(_)) {
                    delete_section(doc_blocks, index);
                } else {
                    return Err(SpliceError::InvalidSectionDelete.into());
                }
            } else {
                delete(doc_blocks, index);
            }
        }
        FoundNode::ListItem {
            block_index,
            item_index,
            ..
        } => {
            if section {
                return Err(SpliceError::InvalidSectionDelete.into());
            }
            let list_became_empty = delete_list_item(doc_blocks, block_index, item_index)?;
            if list_became_empty {
                delete(doc_blocks, block_index);
            }
        }
    }

    Ok(())
}
