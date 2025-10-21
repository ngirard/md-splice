//! Core library for md-splice, containing all logic for AST manipulation.

pub mod cli;
pub mod error;
pub mod locator;
pub mod splicer;
pub mod transaction;

use crate::cli::{
    ApplyArgs, Cli, Command, DeleteArgs, GetArgs, InsertPosition as CliInsertPosition,
    ModificationArgs,
};
use crate::error::SpliceError;
use crate::locator::{locate, locate_all, FoundNode, Selector};
use crate::splicer::{
    delete, delete_list_item, delete_section, find_heading_section_end, get_heading_level, insert,
    insert_list_item, replace, replace_list_item,
};
use crate::transaction::{
    DeleteOperation, InsertOperation, InsertPosition as TransactionInsertPosition, Operation,
    ReplaceOperation, Selector as TransactionSelector,
};
use anyhow::{anyhow, Context};
use clap::Parser;
use markdown_ppp::ast::Block;
use markdown_ppp::parser::{parse_markdown, MarkdownParserState};
use markdown_ppp::printer::{config::Config as PrinterConfig, render_markdown};
use regex::Regex;
use similar::TextDiff;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

enum OutputMode {
    Write,
    DryRun,
    Diff,
}

fn compile_optional_regex(pattern: Option<String>, context: &str) -> anyhow::Result<Option<Regex>> {
    pattern
        .map(|pattern| {
            Regex::new(&pattern).with_context(|| format!("Invalid regex pattern for {context}"))
        })
        .transpose()
}

fn build_optional_scope_selector(
    context: &str,
    select_type: Option<String>,
    select_contains: Option<String>,
    select_regex: Option<String>,
    select_ordinal: Option<usize>,
) -> anyhow::Result<Option<Selector>> {
    if select_type.is_none()
        && select_contains.is_none()
        && select_regex.is_none()
        && select_ordinal.is_none()
    {
        return Ok(None);
    }

    let select_regex = compile_optional_regex(select_regex, context)?;

    Ok(Some(Selector {
        select_type,
        select_contains,
        select_regex,
        select_ordinal: select_ordinal.unwrap_or(1),
        after: None,
        within: None,
    }))
}

fn build_primary_selector(
    select_type: Option<String>,
    select_contains: Option<String>,
    select_regex: Option<String>,
    select_ordinal: usize,
    after: Option<Selector>,
    within: Option<Selector>,
) -> anyhow::Result<Selector> {
    let select_regex = compile_optional_regex(select_regex, "--select-regex")?;

    Ok(Selector {
        select_type,
        select_contains,
        select_regex,
        select_ordinal,
        after: after.map(Box::new),
        within: within.map(Box::new),
    })
}

fn build_until_selector(
    select_type: Option<String>,
    select_contains: Option<String>,
    select_regex: Option<String>,
) -> anyhow::Result<Option<Selector>> {
    if select_type.is_none() && select_contains.is_none() && select_regex.is_none() {
        return Ok(None);
    }

    let select_regex = compile_optional_regex(select_regex, "--until-regex")?;

    Ok(Some(Selector {
        select_type,
        select_contains,
        select_regex,
        select_ordinal: 1,
        after: None,
        within: None,
    }))
}

fn compute_range_end(
    blocks: &[Block],
    start_index: usize,
    until_selector: &Selector,
) -> anyhow::Result<usize> {
    if start_index + 1 >= blocks.len() {
        return Ok(blocks.len());
    }

    match locate(&blocks[start_index + 1..], until_selector) {
        Ok((FoundNode::Block { index, .. }, _)) => Ok(start_index + 1 + index),
        Ok((FoundNode::ListItem { .. }, _)) => Err(SpliceError::RangeRequiresBlock.into()),
        Err(SpliceError::NodeNotFound) => Ok(blocks.len()),
        Err(other) => Err(other.into()),
    }
}

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

    let mut output_mode = OutputMode::Write;

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
        Command::Get(args) => {
            process_get(&doc.blocks, args)?;
            return Ok(());
        }
        Command::Apply(args) => {
            output_mode = process_apply_command(&mut doc.blocks, args)?;
        }
    }

    // 5. Render AST to string
    let output_content = render_markdown(&doc, PrinterConfig::default());

    match output_mode {
        OutputMode::DryRun => {
            io::stdout().write_all(output_content.as_bytes())?;
            return Ok(());
        }
        OutputMode::Diff => {
            let diff_output = TextDiff::from_lines(&input_content, &output_content)
                .unified_diff()
                .header("original", "modified")
                .to_string();

            io::stdout().write_all(diff_output.as_bytes())?;
            return Ok(());
        }
        OutputMode::Write => {}
    }

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

fn process_apply_command(
    doc_blocks: &mut Vec<Block>,
    args: ApplyArgs,
) -> anyhow::Result<OutputMode> {
    let ApplyArgs {
        operations_file,
        operations,
        dry_run,
        diff,
    } = args;

    let operations_data = match (operations_file, operations) {
        (Some(path), None) => {
            if path.to_string_lossy() == "-" {
                let mut buf = String::new();
                io::stdin().read_to_string(&mut buf)?;
                buf
            } else {
                fs::read_to_string(&path).with_context(|| {
                    format!("Failed to read operations file: {}", path.display())
                })?
            }
        }
        (None, Some(inline)) => inline,
        (Some(_), Some(_)) => unreachable!("clap's conflicts_with should prevent this"),
        (None, None) => {
            return Err(anyhow!(
                "Either --operations-file or --operations must be provided."
            ));
        }
    };

    let operations: Vec<Operation> = serde_yaml::from_str(&operations_data)
        .with_context(|| "Failed to parse operations data as JSON or YAML")?;

    process_apply(doc_blocks, operations)?;

    if diff {
        return Ok(OutputMode::Diff);
    }

    if dry_run {
        return Ok(OutputMode::DryRun);
    }

    Ok(OutputMode::Write)
}

#[allow(dead_code)]
fn process_apply(doc_blocks: &mut Vec<Block>, operations: Vec<Operation>) -> anyhow::Result<()> {
    let mut working_blocks = doc_blocks.clone();

    for operation in operations {
        match operation {
            Operation::Replace(replace_op) => {
                apply_replace_operation(&mut working_blocks, replace_op)?
            }
            Operation::Insert(insert_op) => apply_insert_operation(&mut working_blocks, insert_op)?,
            Operation::Delete(delete_op) => apply_delete_operation(&mut working_blocks, delete_op)?,
        }
    }

    *doc_blocks = working_blocks;

    Ok(())
}

#[allow(dead_code)]
fn apply_replace_operation(
    doc_blocks: &mut Vec<Block>,
    operation: ReplaceOperation,
) -> anyhow::Result<()> {
    let selector = build_locator_selector(&operation.selector)?;
    let (found_node, is_ambiguous) = locate(&*doc_blocks, &selector)?;

    if is_ambiguous {
        log::warn!(
            "Warning: Selector matched multiple nodes. Operation was applied to the first match only."
        );
    }

    let content_str = resolve_operation_content(operation.content, operation.content_file)?;
    let new_content_doc = parse_markdown(MarkdownParserState::default(), &content_str)
        .map_err(|e| anyhow!("Failed to parse content markdown: {}", e))?;
    let new_blocks = new_content_doc.blocks;

    match found_node {
        FoundNode::Block { index, .. } => {
            replace(doc_blocks, index, new_blocks);
        }
        FoundNode::ListItem {
            block_index,
            item_index,
            ..
        } => {
            replace_list_item(doc_blocks, block_index, item_index, new_blocks)?;
        }
    }

    Ok(())
}

#[allow(dead_code)]
fn apply_insert_operation(
    doc_blocks: &mut Vec<Block>,
    operation: InsertOperation,
) -> anyhow::Result<()> {
    let selector = build_locator_selector(&operation.selector)?;
    let (found_node, is_ambiguous) = locate(&*doc_blocks, &selector)?;

    if is_ambiguous {
        log::warn!(
            "Warning: Selector matched multiple nodes. Operation was applied to the first match only."
        );
    }

    let content_str = resolve_operation_content(operation.content, operation.content_file)?;
    let new_content_doc = parse_markdown(MarkdownParserState::default(), &content_str)
        .map_err(|e| anyhow!("Failed to parse content markdown: {}", e))?;
    let new_blocks = new_content_doc.blocks;
    let position = map_transaction_insert_position(operation.position);

    match found_node {
        FoundNode::Block { index, .. } => {
            insert(doc_blocks, index, new_blocks, position)?;
        }
        FoundNode::ListItem {
            block_index,
            item_index,
            ..
        } => {
            insert_list_item(doc_blocks, block_index, item_index, new_blocks, position)?;
        }
    }

    Ok(())
}

#[allow(dead_code)]
fn apply_delete_operation(
    doc_blocks: &mut Vec<Block>,
    operation: DeleteOperation,
) -> anyhow::Result<()> {
    let selector = build_locator_selector(&operation.selector)?;
    let (found_node, is_ambiguous) = locate(&*doc_blocks, &selector)?;

    if is_ambiguous {
        log::warn!(
            "Warning: Selector matched multiple nodes. Operation was applied to the first match only."
        );
    }

    match found_node {
        FoundNode::Block { index, block } => {
            if operation.section {
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
            if operation.section {
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

#[allow(dead_code)]
fn build_locator_selector(selector: &TransactionSelector) -> anyhow::Result<Selector> {
    let select_regex = if let Some(pattern) = &selector.select_regex {
        Some(
            Regex::new(pattern)
                .with_context(|| "Invalid regex pattern in operation selector".to_string())?,
        )
    } else {
        None
    };

    Ok(Selector {
        select_type: selector.select_type.clone(),
        select_contains: selector.select_contains.clone(),
        select_regex,
        select_ordinal: selector.select_ordinal,
        ..Default::default()
    })
}

#[allow(dead_code)]
fn resolve_operation_content(
    content: Option<String>,
    content_file: Option<PathBuf>,
) -> anyhow::Result<String> {
    match (content, content_file) {
        (Some(inline), None) => Ok(inline),
        (None, Some(path)) => fs::read_to_string(&path)
            .with_context(|| format!("Failed to read content file: {}", path.display())),
        (Some(_), Some(_)) => Err(anyhow!(
            "Operation cannot specify both inline content and a content_file"
        )),
        (None, None) => Err(anyhow!(
            "Operation must provide inline content or a content_file"
        )),
    }
}

fn map_transaction_insert_position(position: TransactionInsertPosition) -> CliInsertPosition {
    match position {
        TransactionInsertPosition::Before => CliInsertPosition::Before,
        TransactionInsertPosition::After => CliInsertPosition::After,
        TransactionInsertPosition::PrependChild => CliInsertPosition::PrependChild,
        TransactionInsertPosition::AppendChild => CliInsertPosition::AppendChild,
    }
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
        after_select_type,
        after_select_contains,
        after_select_regex,
        after_select_ordinal,
        within_select_type,
        within_select_contains,
        within_select_regex,
        within_select_ordinal,
        until_type,
        until_contains,
        until_regex,
        position,
    } = args;

    if !is_replace && (until_type.is_some() || until_contains.is_some() || until_regex.is_some()) {
        return Err(anyhow!(
            "The --until-* flags can only be used with the 'replace' command"
        ));
    }

    let after_selector = build_optional_scope_selector(
        "--after-select-regex",
        after_select_type,
        after_select_contains,
        after_select_regex,
        after_select_ordinal,
    )?;

    let within_selector = build_optional_scope_selector(
        "--within-select-regex",
        within_select_type,
        within_select_contains,
        within_select_regex,
        within_select_ordinal,
    )?;

    let selector = build_primary_selector(
        select_type,
        select_contains,
        select_regex,
        select_ordinal,
        after_selector,
        within_selector,
    )?;

    let until_selector = build_until_selector(until_type, until_contains, until_regex)?;

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
                if let Some(until_selector) = until_selector.as_ref() {
                    let end_index = compute_range_end(doc_blocks, index, until_selector)?;
                    doc_blocks.splice(index..end_index, new_blocks);
                } else {
                    replace(doc_blocks, index, new_blocks);
                }
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
                if until_selector.is_some() {
                    return Err(SpliceError::RangeRequiresBlock.into());
                }
                replace_list_item(doc_blocks, block_index, item_index, new_blocks)?;
            } else {
                insert_list_item(doc_blocks, block_index, item_index, new_blocks, position)?;
            }
        }
    }

    Ok(())
}

fn process_get(doc_blocks: &[Block], args: GetArgs) -> anyhow::Result<()> {
    let GetArgs {
        select_type,
        select_contains,
        select_regex,
        select_ordinal,
        after_select_type,
        after_select_contains,
        after_select_regex,
        after_select_ordinal,
        within_select_type,
        within_select_contains,
        within_select_regex,
        within_select_ordinal,
        until_type,
        until_contains,
        until_regex,
        section,
        select_all,
        separator,
    } = args;

    let after_selector = build_optional_scope_selector(
        "--after-select-regex",
        after_select_type,
        after_select_contains,
        after_select_regex,
        after_select_ordinal,
    )?;

    let within_selector = build_optional_scope_selector(
        "--within-select-regex",
        within_select_type,
        within_select_contains,
        within_select_regex,
        within_select_ordinal,
    )?;

    let selector = build_primary_selector(
        select_type,
        select_contains,
        select_regex,
        select_ordinal,
        after_selector,
        within_selector,
    )?;

    let until_selector = build_until_selector(until_type, until_contains, until_regex)?;

    if select_all {
        let matches = locate_all(doc_blocks, &selector)?;
        if matches.is_empty() {
            return Ok(());
        }

        let mut had_trailing_newline = false;
        let mut rendered_items = Vec::with_capacity(matches.len());
        for found in &matches {
            let rendered = if section {
                render_heading_section(doc_blocks, found)?
            } else {
                render_found_node(doc_blocks, found)?
            };

            if rendered.ends_with('\n') {
                had_trailing_newline = true;
            }
            rendered_items.push(rendered);
        }

        let normalized: Vec<String> = rendered_items
            .into_iter()
            .map(|s| s.trim_end_matches('\n').to_string())
            .collect();

        let mut output = normalized.join(&separator);
        if had_trailing_newline && separator.ends_with('\n') {
            output.push('\n');
        }

        let mut stdout = io::stdout().lock();
        stdout.write_all(output.as_bytes())?;
        stdout.flush()?;
        return Ok(());
    }

    let (found_node, _) = locate(doc_blocks, &selector)?;
    let mut stdout = io::stdout().lock();
    let rendered = match &found_node {
        FoundNode::Block { index, .. } => {
            if let Some(until_selector) = until_selector.as_ref() {
                let end_index = compute_range_end(doc_blocks, *index, until_selector)?;
                render_blocks(&doc_blocks[*index..end_index])
            } else if section {
                render_heading_section(doc_blocks, &found_node)?
            } else {
                render_found_node(doc_blocks, &found_node)?
            }
        }
        FoundNode::ListItem { .. } => {
            if until_selector.is_some() {
                return Err(SpliceError::RangeRequiresBlock.into());
            }
            render_found_node(doc_blocks, &found_node)?
        }
    };
    stdout.write_all(rendered.as_bytes())?;
    stdout.flush()?;

    Ok(())
}

fn render_heading_section(doc_blocks: &[Block], found: &FoundNode) -> anyhow::Result<String> {
    if let FoundNode::Block { index, block } = found {
        if let Some(level) = get_heading_level(block) {
            let end_index = find_heading_section_end(doc_blocks, *index, level);
            return Ok(render_blocks(&doc_blocks[*index..end_index]));
        }
    }

    Err(SpliceError::SectionRequiresHeading.into())
}

fn render_found_node(doc_blocks: &[Block], found: &FoundNode) -> anyhow::Result<String> {
    match found {
        FoundNode::Block { block, .. } => Ok(render_blocks(std::slice::from_ref(block))),
        FoundNode::ListItem {
            block_index, item, ..
        } => match doc_blocks.get(*block_index) {
            Some(Block::List(list)) => {
                let mut single_list = list.clone();
                single_list.items = vec![(*item).clone()];
                Ok(render_blocks(std::slice::from_ref(&Block::List(
                    single_list,
                ))))
            }
            _ => Err(anyhow!(
                "Internal error: block at index {} is not a list",
                block_index
            )),
        },
    }
}

fn render_blocks(blocks: &[Block]) -> String {
    use markdown_ppp::ast::Document;

    let temp_doc = Document {
        blocks: blocks.to_vec(),
    };
    let mut rendered = render_markdown(&temp_doc, PrinterConfig::default());
    if !rendered.is_empty() && !rendered.ends_with('\n') {
        rendered.push('\n');
    }
    rendered
}

fn process_delete(doc_blocks: &mut Vec<Block>, args: DeleteArgs) -> anyhow::Result<()> {
    let DeleteArgs {
        select_type,
        select_contains,
        select_regex,
        select_ordinal,
        after_select_type,
        after_select_contains,
        after_select_regex,
        after_select_ordinal,
        within_select_type,
        within_select_contains,
        within_select_regex,
        within_select_ordinal,
        until_type,
        until_contains,
        until_regex,
        section,
    } = args;

    let after_selector = build_optional_scope_selector(
        "--after-select-regex",
        after_select_type,
        after_select_contains,
        after_select_regex,
        after_select_ordinal,
    )?;

    let within_selector = build_optional_scope_selector(
        "--within-select-regex",
        within_select_type,
        within_select_contains,
        within_select_regex,
        within_select_ordinal,
    )?;

    let selector = build_primary_selector(
        select_type,
        select_contains,
        select_regex,
        select_ordinal,
        after_selector,
        within_selector,
    )?;

    let until_selector = build_until_selector(until_type, until_contains, until_regex)?;

    let (found_node, is_ambiguous) = locate(&*doc_blocks, &selector)?;

    if is_ambiguous {
        log::warn!(
            "Warning: Selector matched multiple nodes. Operation was applied to the first match only."
        );
    }

    match found_node {
        FoundNode::Block { index, block } => {
            if let Some(until_selector) = until_selector.as_ref() {
                let end_index = compute_range_end(doc_blocks, index, until_selector)?;
                doc_blocks.drain(index..end_index);
            } else if section {
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
            if until_selector.is_some() {
                return Err(SpliceError::RangeRequiresBlock.into());
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::{
        DeleteOperation, InsertOperation, InsertPosition as TxInsertPosition, Operation,
        ReplaceOperation, Selector as TxSelector,
    };
    use markdown_ppp::ast::Document;
    use markdown_ppp::parser::{parse_markdown, MarkdownParserState};
    use markdown_ppp::printer::{config::Config as PrinterConfig, render_markdown};

    #[test]
    fn process_apply_replaces_matching_block() {
        let initial = "# Project Tasks\n\nStatus: In Progress\n";
        let doc = parse_markdown(MarkdownParserState::default(), initial).unwrap();
        let mut blocks = doc.blocks;

        let operations = vec![Operation::Replace(ReplaceOperation {
            selector: TxSelector {
                select_type: None,
                select_contains: Some("Status: In Progress".to_string()),
                select_regex: None,
                select_ordinal: 1,
            },
            comment: None,
            content: Some("Status: **Complete**".to_string()),
            content_file: None,
        })];

        process_apply(&mut blocks, operations).expect("replace operation succeeds");

        let rendered = render_markdown(
            &Document {
                blocks: blocks.clone(),
            },
            PrinterConfig::default(),
        );

        assert!(rendered.contains("Status: **Complete**"));
        assert!(!rendered.contains("Status: In Progress"));
    }

    #[test]
    fn process_apply_inserts_list_item_before_target() {
        let initial = "# Tasks\n\n- [ ] Write documentation\n";
        let doc = parse_markdown(MarkdownParserState::default(), initial).unwrap();
        let mut blocks = doc.blocks;

        let operations = vec![Operation::Insert(InsertOperation {
            selector: TxSelector {
                select_type: Some("li".to_string()),
                select_contains: Some("Write documentation".to_string()),
                select_regex: None,
                select_ordinal: 1,
            },
            comment: None,
            content: Some("- [ ] Implement unit tests".to_string()),
            content_file: None,
            position: TxInsertPosition::Before,
        })];

        process_apply(&mut blocks, operations).expect("insert operation succeeds");

        let rendered = render_markdown(
            &Document {
                blocks: blocks.clone(),
            },
            PrinterConfig::default(),
        );

        let unit_index = rendered
            .find("- [ ] Implement unit tests")
            .expect("inserted item present");
        let docs_index = rendered
            .find("- [ ] Write documentation")
            .expect("original item present");
        assert!(
            unit_index < docs_index,
            "inserted item should appear before original item"
        );
    }

    #[test]
    fn process_apply_deletes_list_item_and_section() {
        let initial = "# Project Tasks\n\n- [ ] Write documentation\n\n## Low Priority\n- [ ] Old task\n- [ ] Another task\n";
        let doc = parse_markdown(MarkdownParserState::default(), initial).unwrap();
        let mut blocks = doc.blocks;

        let operations = vec![
            Operation::Delete(DeleteOperation {
                selector: TxSelector {
                    select_type: Some("li".to_string()),
                    select_contains: Some("Old task".to_string()),
                    select_regex: None,
                    select_ordinal: 1,
                },
                comment: None,
                section: false,
            }),
            Operation::Delete(DeleteOperation {
                selector: TxSelector {
                    select_type: Some("h2".to_string()),
                    select_contains: Some("Low Priority".to_string()),
                    select_regex: None,
                    select_ordinal: 1,
                },
                comment: None,
                section: true,
            }),
        ];

        process_apply(&mut blocks, operations).expect("delete operations succeed");

        let rendered = render_markdown(
            &Document {
                blocks: blocks.clone(),
            },
            PrinterConfig::default(),
        );

        assert!(!rendered.contains("Old task"));
        assert!(!rendered.contains("Low Priority"));
        assert!(!rendered.contains("Another task"));
        assert!(rendered.contains("Write documentation"));
    }

    #[test]
    fn process_apply_is_atomic_when_operation_fails() {
        let initial = "# Project Tasks\n\nStatus: In Progress\n";
        let doc = parse_markdown(MarkdownParserState::default(), initial).unwrap();
        let mut blocks = doc.blocks;
        let original_blocks = blocks.clone();

        let operations = vec![
            Operation::Replace(ReplaceOperation {
                selector: TxSelector {
                    select_type: None,
                    select_contains: Some("Status: In Progress".to_string()),
                    select_regex: None,
                    select_ordinal: 1,
                },
                comment: None,
                content: Some("Status: **Complete**".to_string()),
                content_file: None,
            }),
            Operation::Delete(DeleteOperation {
                selector: TxSelector {
                    select_type: Some("h2".to_string()),
                    select_contains: Some("Does Not Exist".to_string()),
                    select_regex: None,
                    select_ordinal: 1,
                },
                comment: None,
                section: false,
            }),
        ];

        let result = process_apply(&mut blocks, operations);

        assert!(
            result.is_err(),
            "process_apply should fail when a selector does not match"
        );
        assert_eq!(
            blocks, original_blocks,
            "document blocks should remain unchanged on failure"
        );
    }
}
