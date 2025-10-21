//! Core library for md-splice, containing all logic for AST manipulation.

pub mod cli;
pub mod error;
pub mod frontmatter;
pub mod locator;
pub mod splicer;
pub mod transaction;

use crate::cli::{
    ApplyArgs, Cli, Command, DeleteArgs, FrontmatterCommand, FrontmatterDeleteArgs,
    FrontmatterFormatArg, FrontmatterGetArgs, FrontmatterOutputFormat, FrontmatterSetArgs, GetArgs,
    InsertPosition as CliInsertPosition, ModificationArgs,
};
use crate::error::SpliceError;
use crate::frontmatter::{refresh_frontmatter_block, FrontmatterFormat, ParsedDocument};
use crate::locator::{locate, locate_all, FoundNode, Selector};
use crate::splicer::{
    delete, delete_list_item, delete_section, find_heading_section_end, get_heading_level, insert,
    insert_list_item, replace, replace_list_item,
};
use crate::transaction::{
    DeleteFrontmatterOperation, DeleteOperation, InsertOperation,
    InsertPosition as TransactionInsertPosition, Operation, ReplaceFrontmatterOperation,
    ReplaceOperation, Selector as TransactionSelector, SetFrontmatterOperation,
};
use anyhow::{anyhow, Context};
use clap::Parser;
use markdown_ppp::ast::Block;
use markdown_ppp::parser::{parse_markdown, MarkdownParserState};
use markdown_ppp::printer::{config::Config as PrinterConfig, render_markdown};
use regex::Regex;
use serde_yaml::{Mapping, Value as YamlValue};
use similar::TextDiff;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

enum OutputMode {
    Write,
    DryRun,
    Diff,
}

enum FrontmatterCommandMode {
    Unchanged,
    ReadOnly,
    Mutated,
}

impl From<FrontmatterFormatArg> for FrontmatterFormat {
    fn from(arg: FrontmatterFormatArg) -> Self {
        match arg {
            FrontmatterFormatArg::Yaml => FrontmatterFormat::Yaml,
            FrontmatterFormatArg::Toml => FrontmatterFormat::Toml,
        }
    }
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

    if let Command::Frontmatter(FrontmatterCommand::Set(args)) = &command {
        let value_from_stdin = args
            .value_file
            .as_deref()
            .is_some_and(|p| p.to_string_lossy() == "-");

        if file.is_none() && value_from_stdin {
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

    // 4. Parse frontmatter and markdown body
    let mut parsed_document = crate::frontmatter::parse(&input_content)?;

    let mut doc = parse_markdown(MarkdownParserState::default(), &parsed_document.body)
        .map_err(|e| anyhow!("Failed to parse input markdown: {}", e))?;

    let mut output_mode = OutputMode::Write;

    let mut frontmatter_mode = FrontmatterCommandMode::Unchanged;

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
        Command::Frontmatter(frontmatter_command) => {
            frontmatter_mode =
                process_frontmatter_command(&mut parsed_document, frontmatter_command)?;
            if matches!(frontmatter_mode, FrontmatterCommandMode::ReadOnly) {
                return Ok(());
            }
        }
        Command::Apply(args) => {
            let (mode, frontmatter_changed) =
                process_apply_command(&mut doc.blocks, &mut parsed_document, args)?;
            output_mode = mode;
            if frontmatter_changed {
                frontmatter_mode = FrontmatterCommandMode::Mutated;
            }
        }
    }

    // 5. Render AST to string
    let body_output = render_markdown(&doc, PrinterConfig::default());
    let mut output_content = String::new();

    if matches!(frontmatter_mode, FrontmatterCommandMode::Mutated) {
        refresh_frontmatter_block(&mut parsed_document)?;
    }

    if let Some(prefix) = parsed_document.frontmatter_block.as_deref() {
        output_content.push_str(prefix);
    }

    output_content.push_str(&body_output);

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
    parsed_document: &mut ParsedDocument,
    args: ApplyArgs,
) -> anyhow::Result<(OutputMode, bool)> {
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

    let frontmatter_mutated = process_apply(doc_blocks, parsed_document, operations)?;

    let mode = if diff {
        OutputMode::Diff
    } else if dry_run {
        OutputMode::DryRun
    } else {
        OutputMode::Write
    };

    Ok((mode, frontmatter_mutated))
}

#[allow(dead_code)]
fn process_apply(
    doc_blocks: &mut Vec<Block>,
    parsed_document: &mut ParsedDocument,
    operations: Vec<Operation>,
) -> anyhow::Result<bool> {
    let mut working_blocks = doc_blocks.clone();
    let mut working_document = parsed_document.clone();
    let mut frontmatter_mutated = false;

    for operation in operations {
        match operation {
            Operation::Replace(replace_op) => {
                apply_replace_operation(&mut working_blocks, replace_op)?
            }
            Operation::Insert(insert_op) => apply_insert_operation(&mut working_blocks, insert_op)?,
            Operation::Delete(delete_op) => apply_delete_operation(&mut working_blocks, delete_op)?,
            Operation::SetFrontmatter(set_op) => {
                apply_set_frontmatter_operation(&mut working_document, set_op)?;
                frontmatter_mutated = true;
            }
            Operation::DeleteFrontmatter(delete_op) => {
                apply_delete_frontmatter_operation(&mut working_document, delete_op)?;
                frontmatter_mutated = true;
            }
            Operation::ReplaceFrontmatter(replace_op) => {
                apply_replace_frontmatter_operation(&mut working_document, replace_op)?;
                frontmatter_mutated = true;
            }
        }
    }

    *doc_blocks = working_blocks;
    *parsed_document = working_document;

    Ok(frontmatter_mutated)
}

#[allow(dead_code)]
fn apply_replace_operation(
    doc_blocks: &mut Vec<Block>,
    operation: ReplaceOperation,
) -> anyhow::Result<()> {
    let ReplaceOperation {
        selector,
        comment: _,
        content,
        content_file,
        until,
    } = operation;

    let selector = build_locator_selector(&selector)?;
    let until_selector = build_optional_locator_selector(until.as_ref())?;

    let (found_node, is_ambiguous) = locate(&*doc_blocks, &selector)?;

    if is_ambiguous {
        log::warn!(
            "Warning: Selector matched multiple nodes. Operation was applied to the first match only."
        );
    }

    let content_str = resolve_operation_content(content, content_file)?;
    let new_content_doc = parse_markdown(MarkdownParserState::default(), &content_str)
        .map_err(|e| anyhow!("Failed to parse content markdown: {}", e))?;
    let new_blocks = new_content_doc.blocks;

    match found_node {
        FoundNode::Block { index, .. } => {
            if let Some(until_selector) = until_selector.as_ref() {
                let end_index = compute_range_end(doc_blocks, index, until_selector)?;
                doc_blocks.splice(index..end_index, new_blocks);
            } else {
                replace(doc_blocks, index, new_blocks);
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
    let DeleteOperation {
        selector,
        comment: _,
        section,
        until,
    } = operation;

    let selector = build_locator_selector(&selector)?;
    let until_selector = build_optional_locator_selector(until.as_ref())?;
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

fn apply_set_frontmatter_operation(
    parsed_document: &mut ParsedDocument,
    operation: SetFrontmatterOperation,
) -> anyhow::Result<()> {
    let SetFrontmatterOperation {
        key,
        comment: _,
        value,
        value_file,
        format,
    } = operation;

    let new_value = resolve_frontmatter_operation_value(value, value_file, "value")?;
    let segments = parse_frontmatter_path(&key)?;
    assign_frontmatter_value(parsed_document, &segments, &key, format, new_value)
}

fn apply_delete_frontmatter_operation(
    parsed_document: &mut ParsedDocument,
    operation: DeleteFrontmatterOperation,
) -> anyhow::Result<()> {
    let DeleteFrontmatterOperation { key, comment: _ } = operation;
    let segments = parse_frontmatter_path(&key)?;
    remove_frontmatter_value(parsed_document, &segments, &key)
}

fn apply_replace_frontmatter_operation(
    parsed_document: &mut ParsedDocument,
    operation: ReplaceFrontmatterOperation,
) -> anyhow::Result<()> {
    let ReplaceFrontmatterOperation {
        comment: _,
        content,
        content_file,
        format,
    } = operation;

    let new_value = resolve_frontmatter_operation_value(content, content_file, "content")?;
    replace_entire_frontmatter(parsed_document, new_value, format)
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

    let after = build_optional_locator_selector(selector.after.as_deref())?;
    let within = build_optional_locator_selector(selector.within.as_deref())?;

    Ok(Selector {
        select_type: selector.select_type.clone(),
        select_contains: selector.select_contains.clone(),
        select_regex,
        select_ordinal: selector.select_ordinal,
        after: after.map(Box::new),
        within: within.map(Box::new),
    })
}

fn build_optional_locator_selector(
    selector: Option<&TransactionSelector>,
) -> anyhow::Result<Option<Selector>> {
    selector.map(|sel| build_locator_selector(sel)).transpose()
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

fn process_frontmatter_command(
    parsed_document: &mut ParsedDocument,
    command: FrontmatterCommand,
) -> anyhow::Result<FrontmatterCommandMode> {
    match command {
        FrontmatterCommand::Get(args) => {
            process_frontmatter_get(parsed_document, args)?;
            Ok(FrontmatterCommandMode::ReadOnly)
        }
        FrontmatterCommand::Set(args) => {
            process_frontmatter_set(parsed_document, args)?;
            Ok(FrontmatterCommandMode::Mutated)
        }
        FrontmatterCommand::Delete(args) => {
            process_frontmatter_delete(parsed_document, args)?;
            Ok(FrontmatterCommandMode::Mutated)
        }
    }
}

fn process_frontmatter_get(
    parsed_document: &ParsedDocument,
    args: FrontmatterGetArgs,
) -> anyhow::Result<()> {
    let FrontmatterGetArgs { key, output_format } = args;

    let Some(frontmatter) = parsed_document.frontmatter.as_ref() else {
        if key.is_some() {
            return Err(SpliceError::FrontmatterMissing.into());
        }
        return Ok(());
    };

    let value = if let Some(path) = key {
        let segments = parse_frontmatter_path(&path)?;
        resolve_frontmatter_path(frontmatter, &segments)
            .ok_or_else(|| SpliceError::FrontmatterKeyNotFound(path))?
    } else {
        frontmatter
    };

    let mut rendered = match output_format {
        FrontmatterOutputFormat::String => render_frontmatter_as_string(value)?,
        FrontmatterOutputFormat::Json => serde_json::to_string(value)?,
        FrontmatterOutputFormat::Yaml => crate::frontmatter::serialize_yaml_value(value)?,
    };

    if !rendered.ends_with(['\n', '\r']) {
        rendered.push('\n');
    }

    let mut stdout = io::stdout().lock();
    stdout.write_all(rendered.as_bytes())?;
    stdout.flush()?;

    Ok(())
}

fn process_frontmatter_set(
    parsed_document: &mut ParsedDocument,
    args: FrontmatterSetArgs,
) -> anyhow::Result<()> {
    let FrontmatterSetArgs {
        key,
        value,
        value_file,
        format,
    } = args;

    let new_value = resolve_frontmatter_value(value, value_file)?;
    let segments = parse_frontmatter_path(&key)?;
    let format_hint = format.map(FrontmatterFormat::from);
    assign_frontmatter_value(parsed_document, &segments, &key, format_hint, new_value)?;

    Ok(())
}

fn process_frontmatter_delete(
    parsed_document: &mut ParsedDocument,
    args: FrontmatterDeleteArgs,
) -> anyhow::Result<()> {
    let FrontmatterDeleteArgs { key } = args;

    let segments = parse_frontmatter_path(&key)?;
    remove_frontmatter_value(parsed_document, &segments, &key)?;

    Ok(())
}

#[derive(Debug)]
enum FrontmatterPathSegment {
    Key(String),
    Index(usize),
}

fn parse_frontmatter_path(path: &str) -> anyhow::Result<Vec<FrontmatterPathSegment>> {
    if path.trim().is_empty() {
        return Err(anyhow!("Frontmatter key cannot be empty"));
    }

    let mut segments = Vec::new();
    let mut buffer = String::new();
    let mut chars = path.chars();
    let mut last_was_separator = true;

    while let Some(ch) = chars.next() {
        match ch {
            '.' => {
                if last_was_separator {
                    return Err(anyhow!(
                        "Invalid frontmatter path `{}`: consecutive '.' or leading '.' detected",
                        path
                    ));
                }
                if !buffer.is_empty() {
                    segments.push(FrontmatterPathSegment::Key(std::mem::take(&mut buffer)));
                }
                last_was_separator = true;
            }
            '[' => {
                if !buffer.is_empty() {
                    segments.push(FrontmatterPathSegment::Key(std::mem::take(&mut buffer)));
                }

                let mut index_buf = String::new();
                let mut closed = false;
                while let Some(next) = chars.next() {
                    if next == ']' {
                        closed = true;
                        break;
                    }
                    index_buf.push(next);
                }

                if !closed {
                    return Err(anyhow!(
                        "Invalid frontmatter path `{}`: missing closing ']'",
                        path
                    ));
                }

                if index_buf.is_empty() {
                    return Err(anyhow!(
                        "Invalid frontmatter path `{}`: empty array index",
                        path
                    ));
                }

                let index = index_buf.parse::<usize>().map_err(|_| {
                    anyhow!(
                        "Invalid frontmatter path `{}`: array index `{}` is not a non-negative integer",
                        path, index_buf
                    )
                })?;

                segments.push(FrontmatterPathSegment::Index(index));
                last_was_separator = false;
            }
            ']' => {
                return Err(anyhow!(
                    "Invalid frontmatter path `{}`: unexpected ']'",
                    path
                ));
            }
            _ => {
                buffer.push(ch);
                last_was_separator = false;
            }
        }
    }

    if !buffer.is_empty() {
        segments.push(FrontmatterPathSegment::Key(buffer));
        last_was_separator = false;
    }

    if segments.is_empty() {
        return Err(anyhow!("Frontmatter key cannot be empty"));
    }

    if last_was_separator {
        return Err(anyhow!(
            "Invalid frontmatter path `{}`: trailing '.' detected",
            path
        ));
    }

    Ok(segments)
}

fn resolve_frontmatter_path<'a>(
    mut value: &'a YamlValue,
    segments: &[FrontmatterPathSegment],
) -> Option<&'a YamlValue> {
    for segment in segments {
        match segment {
            FrontmatterPathSegment::Key(key) => {
                let mapping = value.as_mapping()?;
                let mut next_value = None;
                for (map_key, map_value) in mapping {
                    if map_key.as_str() == Some(key.as_str()) {
                        next_value = Some(map_value);
                        break;
                    }
                }
                value = next_value?;
            }
            FrontmatterPathSegment::Index(index) => {
                let sequence = value.as_sequence()?;
                value = sequence.get(*index)?;
            }
        }
    }

    Some(value)
}

fn render_frontmatter_as_string(value: &YamlValue) -> anyhow::Result<String> {
    Ok(match value {
        YamlValue::Null => String::new(),
        YamlValue::Bool(b) => b.to_string(),
        YamlValue::Number(n) => n.to_string(),
        YamlValue::String(s) => s.clone(),
        _ => crate::frontmatter::serialize_yaml_value(value)?,
    })
}

fn resolve_frontmatter_value(
    value: Option<String>,
    value_file: Option<PathBuf>,
) -> anyhow::Result<YamlValue> {
    match (value, value_file) {
        (Some(inline), None) => parse_yaml_value(&inline),
        (None, Some(path)) => {
            if path.to_string_lossy() == "-" {
                let mut buf = String::new();
                io::stdin().read_to_string(&mut buf)?;
                parse_yaml_value(&buf)
            } else {
                let contents = fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read value file: {}", path.display()))?;
                parse_yaml_value(&contents)
            }
        }
        (Some(_), Some(_)) => unreachable!("clap should enforce value/value-file exclusivity"),
        (None, None) => Err(anyhow!(
            "Either --value or --value-file must be provided for frontmatter set"
        )),
    }
}

fn parse_yaml_value(content: &str) -> anyhow::Result<YamlValue> {
    serde_yaml::from_str(content)
        .with_context(|| "Failed to parse value as YAML for frontmatter set operation")
}

fn set_value_at_path(
    current: &mut YamlValue,
    segments: &[FrontmatterPathSegment],
    new_value: YamlValue,
) -> anyhow::Result<()> {
    let mut cursor = current;
    let path_display = join_segments(segments);

    for (index, segment) in segments.iter().enumerate() {
        let is_last = index == segments.len() - 1;
        match segment {
            FrontmatterPathSegment::Key(key) => {
                if !cursor.is_mapping() {
                    if cursor.is_null() {
                        *cursor = YamlValue::Mapping(Mapping::new());
                    } else {
                        return Err(anyhow!(
                            "Frontmatter path '{}' expects a mapping at '{}' but found {}",
                            path_display,
                            key,
                            yaml_type_name(cursor),
                        ));
                    }
                }

                let mapping = cursor.as_mapping_mut().expect("validated mapping");
                let key_node = YamlValue::String(key.clone());

                if is_last {
                    mapping.insert(key_node, new_value);
                    return Ok(());
                }

                if !mapping.contains_key(&key_node) {
                    mapping.insert(key_node.clone(), YamlValue::Null);
                }

                cursor = mapping
                    .get_mut(&key_node)
                    .expect("entry inserted or existed");
            }
            FrontmatterPathSegment::Index(position) => {
                let sequence_kind = yaml_type_name(cursor);
                let sequence = cursor.as_sequence_mut().ok_or_else(|| {
                    anyhow!(
                        "Frontmatter path '{}' expects an array but found {}",
                        path_display,
                        sequence_kind
                    )
                })?;

                if *position >= sequence.len() {
                    return Err(anyhow!(
                        "Array index {} out of bounds for frontmatter path '{}'",
                        position,
                        path_display
                    ));
                }

                if is_last {
                    sequence[*position] = new_value;
                    return Ok(());
                }

                cursor = sequence
                    .get_mut(*position)
                    .ok_or_else(|| anyhow!("Invalid array index while traversing frontmatter"))?;
            }
        }
    }

    Ok(())
}

fn delete_value_at_path(
    current: &mut YamlValue,
    segments: &[FrontmatterPathSegment],
) -> anyhow::Result<bool> {
    if segments.is_empty() {
        return Ok(false);
    }

    match segments.first().unwrap() {
        FrontmatterPathSegment::Key(key) => {
            let Some(mapping) = current.as_mapping_mut() else {
                return Ok(false);
            };

            let key_node = YamlValue::String(key.clone());

            if segments.len() == 1 {
                Ok(mapping.remove(&key_node).is_some())
            } else if let Some(next) = mapping.get_mut(&key_node) {
                let removed = delete_value_at_path(next, &segments[1..])?;
                if removed && yaml_value_is_empty(next) {
                    mapping.remove(&key_node);
                }
                Ok(removed)
            } else {
                Ok(false)
            }
        }
        FrontmatterPathSegment::Index(position) => {
            let Some(sequence) = current.as_sequence_mut() else {
                return Ok(false);
            };

            if *position >= sequence.len() {
                return Ok(false);
            }

            if segments.len() == 1 {
                sequence.remove(*position);
                Ok(true)
            } else {
                let removed = delete_value_at_path(&mut sequence[*position], &segments[1..])?;
                if removed && yaml_value_is_empty(&sequence[*position]) {
                    sequence.remove(*position);
                }
                Ok(removed)
            }
        }
    }
}

fn resolve_frontmatter_operation_value(
    value: Option<YamlValue>,
    value_file: Option<PathBuf>,
    value_label: &str,
) -> anyhow::Result<YamlValue> {
    let file_label = format!("{}_file", value_label);
    match (value, value_file) {
        (Some(inline), None) => Ok(inline),
        (None, Some(path)) => {
            let mut content = String::new();
            if path.as_os_str() == "-" {
                io::stdin()
                    .read_to_string(&mut content)
                    .with_context(|| format!("Failed to read {value_label} from stdin"))?;
            } else {
                content = fs::read_to_string(&path).with_context(|| {
                    format!(
                        "Failed to read {} file for frontmatter operation: {}",
                        file_label,
                        path.display()
                    )
                })?;
            }

            parse_yaml_value(&content)
        }
        (Some(_), Some(_)) => Err(anyhow!(
            "Specify either `{}` or `{}` for frontmatter operation, not both",
            value_label,
            file_label
        )),
        (None, None) => Err(anyhow!(
            "Frontmatter operation requires either `{}` or `{}`",
            value_label,
            file_label
        )),
    }
}

fn assign_frontmatter_value(
    parsed_document: &mut ParsedDocument,
    segments: &[FrontmatterPathSegment],
    key_display: &str,
    format_hint: Option<FrontmatterFormat>,
    new_value: YamlValue,
) -> anyhow::Result<()> {
    if segments.is_empty() {
        return Err(anyhow!("Frontmatter key cannot be empty"));
    }

    if parsed_document.frontmatter.is_none() {
        match segments.first().unwrap() {
            FrontmatterPathSegment::Key(_) => {
                parsed_document.frontmatter = Some(YamlValue::Mapping(Mapping::new()));
            }
            FrontmatterPathSegment::Index(_) => {
                return Err(anyhow!(
                    "Cannot set array index `{}` because document frontmatter is empty",
                    key_display
                ));
            }
        }
    }

    let format_to_use = match (parsed_document.format, format_hint) {
        (Some(existing), _) => existing,
        (None, Some(hint)) => hint,
        (None, None) => FrontmatterFormat::Yaml,
    };

    parsed_document.format = Some(format_to_use);

    let frontmatter_value = parsed_document
        .frontmatter
        .get_or_insert_with(|| YamlValue::Mapping(Mapping::new()));

    set_value_at_path(frontmatter_value, segments, new_value)?;

    Ok(())
}

fn remove_frontmatter_value(
    parsed_document: &mut ParsedDocument,
    segments: &[FrontmatterPathSegment],
    key_display: &str,
) -> anyhow::Result<()> {
    let Some(frontmatter) = parsed_document.frontmatter.as_mut() else {
        return Err(SpliceError::FrontmatterMissing.into());
    };

    let removed = delete_value_at_path(frontmatter, segments)?;

    if !removed {
        return Err(SpliceError::FrontmatterKeyNotFound(key_display.to_string()).into());
    }

    if yaml_value_is_empty(frontmatter) {
        parsed_document.frontmatter = None;
        parsed_document.frontmatter_block = None;
        parsed_document.format = None;
    }

    Ok(())
}

fn replace_entire_frontmatter(
    parsed_document: &mut ParsedDocument,
    new_value: YamlValue,
    format_hint: Option<FrontmatterFormat>,
) -> anyhow::Result<()> {
    if new_value.is_null() {
        parsed_document.frontmatter = None;
        parsed_document.frontmatter_block = None;
        parsed_document.format = None;
        return Ok(());
    }

    parsed_document.frontmatter = Some(new_value);

    let format_to_use = match (format_hint, parsed_document.format) {
        (Some(hint), _) => hint,
        (None, Some(existing)) => existing,
        (None, None) => FrontmatterFormat::Yaml,
    };

    parsed_document.format = Some(format_to_use);

    Ok(())
}

fn yaml_value_is_empty(value: &YamlValue) -> bool {
    match value {
        YamlValue::Null => true,
        YamlValue::Mapping(map) => map.is_empty(),
        YamlValue::Sequence(seq) => seq.is_empty(),
        _ => false,
    }
}

fn join_segments(segments: &[FrontmatterPathSegment]) -> String {
    let mut parts = Vec::new();
    for segment in segments {
        match segment {
            FrontmatterPathSegment::Key(key) => parts.push(key.clone()),
            FrontmatterPathSegment::Index(index) => parts.push(format!("[{}]", index)),
        }
    }
    parts.join(".").replace(".[", "[")
}

fn yaml_type_name(value: &YamlValue) -> &'static str {
    match value {
        YamlValue::Null => "null",
        YamlValue::Bool(_) => "bool",
        YamlValue::Number(_) => "number",
        YamlValue::String(_) => "string",
        YamlValue::Sequence(_) => "array",
        YamlValue::Mapping(_) => "mapping",
        YamlValue::Tagged(_) => "tagged value",
    }
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
        let mut parsed_document = ParsedDocument {
            frontmatter: None,
            body: initial.to_string(),
            format: None,
            frontmatter_block: None,
        };

        let operations = vec![Operation::Replace(ReplaceOperation {
            selector: TxSelector {
                select_type: None,
                select_contains: Some("Status: In Progress".to_string()),
                select_regex: None,
                select_ordinal: 1,
                after: None,
                within: None,
            },
            comment: None,
            content: Some("Status: **Complete**".to_string()),
            content_file: None,
            until: None,
        })];

        let frontmatter_changed = process_apply(&mut blocks, &mut parsed_document, operations)
            .expect("replace operation succeeds");
        assert!(!frontmatter_changed);

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
        let mut parsed_document = ParsedDocument {
            frontmatter: None,
            body: initial.to_string(),
            format: None,
            frontmatter_block: None,
        };

        let operations = vec![Operation::Insert(InsertOperation {
            selector: TxSelector {
                select_type: Some("li".to_string()),
                select_contains: Some("Write documentation".to_string()),
                select_regex: None,
                select_ordinal: 1,
                after: None,
                within: None,
            },
            comment: None,
            content: Some("- [ ] Implement unit tests".to_string()),
            content_file: None,
            position: TxInsertPosition::Before,
        })];

        let frontmatter_changed = process_apply(&mut blocks, &mut parsed_document, operations)
            .expect("insert operation succeeds");
        assert!(!frontmatter_changed);

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
        let mut parsed_document = ParsedDocument {
            frontmatter: None,
            body: initial.to_string(),
            format: None,
            frontmatter_block: None,
        };

        let operations = vec![
            Operation::Delete(DeleteOperation {
                selector: TxSelector {
                    select_type: Some("li".to_string()),
                    select_contains: Some("Old task".to_string()),
                    select_regex: None,
                    select_ordinal: 1,
                    after: None,
                    within: None,
                },
                comment: None,
                section: false,
                until: None,
            }),
            Operation::Delete(DeleteOperation {
                selector: TxSelector {
                    select_type: Some("h2".to_string()),
                    select_contains: Some("Low Priority".to_string()),
                    select_regex: None,
                    select_ordinal: 1,
                    after: None,
                    within: None,
                },
                comment: None,
                section: true,
                until: None,
            }),
        ];

        let frontmatter_changed = process_apply(&mut blocks, &mut parsed_document, operations)
            .expect("delete operations succeed");
        assert!(!frontmatter_changed);

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
    fn process_apply_replace_uses_until_range() {
        let initial =
            "# Guide\n\n## Installation\nStep one.\n\nStep two.\n\n## Usage\nUsage notes.\n";
        let doc = parse_markdown(MarkdownParserState::default(), initial).unwrap();
        let mut blocks = doc.blocks;
        let mut parsed_document = ParsedDocument {
            frontmatter: None,
            body: initial.to_string(),
            format: None,
            frontmatter_block: None,
        };

        let operations = vec![Operation::Replace(ReplaceOperation {
            selector: TxSelector {
                select_type: Some("h2".to_string()),
                select_contains: Some("Installation".to_string()),
                select_regex: None,
                select_ordinal: 1,
                after: None,
                within: None,
            },
            comment: None,
            content: Some("## Installation\nUpdated steps.\n".to_string()),
            content_file: None,
            until: Some(TxSelector {
                select_type: Some("h2".to_string()),
                select_contains: Some("Usage".to_string()),
                select_regex: None,
                select_ordinal: 1,
                after: None,
                within: None,
            }),
        })];

        let frontmatter_changed = process_apply(&mut blocks, &mut parsed_document, operations)
            .expect("replace range succeeds");
        assert!(!frontmatter_changed);

        let rendered = render_markdown(
            &Document {
                blocks: blocks.clone(),
            },
            PrinterConfig::default(),
        );

        assert!(rendered.contains("Updated steps."));
        assert!(!rendered.contains("Step one."));
        assert!(rendered.contains("## Usage"));
    }

    #[test]
    fn process_apply_delete_respects_scoped_selectors() {
        let initial = "# Roadmap\n\n## Future Features\n- [ ] Task Alpha\n- [ ] Task Beta\n- [ ] Task Gamma\n\n## Done\n- [x] Task Omega\n";
        let doc = parse_markdown(MarkdownParserState::default(), initial).unwrap();
        let mut blocks = doc.blocks;
        let mut parsed_document = ParsedDocument {
            frontmatter: None,
            body: initial.to_string(),
            format: None,
            frontmatter_block: None,
        };

        let operations = vec![Operation::Delete(DeleteOperation {
            selector: TxSelector {
                select_type: Some("li".to_string()),
                select_contains: Some("Task Beta".to_string()),
                select_regex: None,
                select_ordinal: 1,
                after: None,
                within: Some(Box::new(TxSelector {
                    select_type: Some("h2".to_string()),
                    select_contains: Some("Future Features".to_string()),
                    select_regex: None,
                    select_ordinal: 1,
                    after: None,
                    within: None,
                })),
            },
            comment: None,
            section: false,
            until: None,
        })];

        let frontmatter_changed = process_apply(&mut blocks, &mut parsed_document, operations)
            .expect("scoped delete succeeds");
        assert!(!frontmatter_changed);

        let rendered = render_markdown(
            &Document {
                blocks: blocks.clone(),
            },
            PrinterConfig::default(),
        );

        assert!(rendered.contains("Task Alpha"));
        assert!(!rendered.contains("Task Beta"));
        assert!(rendered.contains("Task Gamma"));
        assert!(rendered.contains("Task Omega"));
    }

    #[test]
    fn process_apply_is_atomic_when_operation_fails() {
        let initial = "# Project Tasks\n\nStatus: In Progress\n";
        let doc = parse_markdown(MarkdownParserState::default(), initial).unwrap();
        let mut blocks = doc.blocks;
        let mut parsed_document = ParsedDocument {
            frontmatter: None,
            body: initial.to_string(),
            format: None,
            frontmatter_block: None,
        };
        let original_blocks = blocks.clone();
        let original_document = parsed_document.clone();

        let operations = vec![
            Operation::Replace(ReplaceOperation {
                selector: TxSelector {
                    select_type: None,
                    select_contains: Some("Status: In Progress".to_string()),
                    select_regex: None,
                    select_ordinal: 1,
                    after: None,
                    within: None,
                },
                comment: None,
                content: Some("Status: **Complete**".to_string()),
                content_file: None,
                until: None,
            }),
            Operation::Delete(DeleteOperation {
                selector: TxSelector {
                    select_type: Some("h2".to_string()),
                    select_contains: Some("Does Not Exist".to_string()),
                    select_regex: None,
                    select_ordinal: 1,
                    after: None,
                    within: None,
                },
                comment: None,
                section: false,
                until: None,
            }),
        ];

        let result = process_apply(&mut blocks, &mut parsed_document, operations);

        assert!(
            result.is_err(),
            "process_apply should fail when a selector does not match"
        );
        assert_eq!(
            blocks, original_blocks,
            "document blocks should remain unchanged on failure"
        );
        assert_eq!(
            parsed_document, original_document,
            "parsed document should remain unchanged on failure"
        );
    }
}
