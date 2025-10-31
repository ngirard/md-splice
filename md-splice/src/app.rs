use crate::cli::{
    ApplyArgs, Cli, Command, DeleteArgs, FrontmatterCommand, FrontmatterDeleteArgs,
    FrontmatterFormatArg, FrontmatterGetArgs, FrontmatterOutputFormat, FrontmatterSetArgs, GetArgs,
    InsertPosition as CliInsertPosition, ModificationArgs,
};
use anyhow::{anyhow, Context};
use clap::Parser;
use markdown_ppp::ast::{Block, Heading, HeadingKind, SetextHeading};
use markdown_ppp::parser::{parse_markdown, MarkdownParserState};
use markdown_ppp::printer::render_markdown;
use md_splice_lib::error::SpliceError;
use md_splice_lib::frontmatter::{self, FrontmatterFormat};
use md_splice_lib::locator::{locate, locate_all, FoundNode, Selector};
use md_splice_lib::transaction::{
    DeleteFrontmatterOperation, DeleteOperation, InsertOperation,
    InsertPosition as TxInsertPosition, Operation, ReplaceOperation, Selector as TxSelector,
    SetFrontmatterOperation,
};
use md_splice_lib::{default_printer_config, MarkdownDocument};
use regex::Regex;
use serde_yaml::Value as YamlValue;
use similar::TextDiff;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::str::FromStr;
use tempfile::Builder as TempFileBuilder;

pub fn run() -> anyhow::Result<()> {
    env_logger::init();

    let Cli {
        file,
        output,
        command,
    } = Cli::parse();

    validate_stdin_usage(&file, &command)?;

    let input_content = read_input(file.as_ref())?;

    match command {
        Command::Get(args) => {
            process_get(&input_content, args)?;
            Ok(())
        }
        Command::Frontmatter(FrontmatterCommand::Get(args)) => {
            process_frontmatter_get(&input_content, args)?;
            Ok(())
        }
        Command::Insert(args) => {
            let mut doc = MarkdownDocument::from_str(&input_content)?;
            let operation = Operation::Insert(build_insert_operation(args)?);
            doc.apply(vec![operation]).map_err(map_splice_error)?;
            finalize_output(
                OutputMode::Write,
                &output,
                &file,
                &input_content,
                doc.render(),
            )
        }
        Command::Replace(args) => {
            let mut doc = MarkdownDocument::from_str(&input_content)?;
            let operation = Operation::Replace(build_replace_operation(args)?);
            doc.apply(vec![operation]).map_err(map_splice_error)?;
            finalize_output(
                OutputMode::Write,
                &output,
                &file,
                &input_content,
                doc.render(),
            )
        }
        Command::Delete(args) => {
            let mut doc = MarkdownDocument::from_str(&input_content)?;
            let operation = Operation::Delete(build_delete_operation(args)?);
            doc.apply(vec![operation]).map_err(map_splice_error)?;
            finalize_output(
                OutputMode::Write,
                &output,
                &file,
                &input_content,
                doc.render(),
            )
        }
        Command::Apply(args) => {
            let (operations, mode) = prepare_apply_operations(args)?;
            let mut doc = MarkdownDocument::from_str(&input_content)?;
            doc.apply(operations).map_err(map_splice_error)?;
            finalize_output(mode, &output, &file, &input_content, doc.render())
        }
        Command::Frontmatter(FrontmatterCommand::Set(args)) => {
            let mut doc = MarkdownDocument::from_str(&input_content)?;
            let operation = Operation::SetFrontmatter(build_set_frontmatter_operation(args)?);
            doc.apply(vec![operation]).map_err(map_splice_error)?;
            finalize_output(
                OutputMode::Write,
                &output,
                &file,
                &input_content,
                doc.render(),
            )
        }
        Command::Frontmatter(FrontmatterCommand::Delete(args)) => {
            let mut doc = MarkdownDocument::from_str(&input_content)?;
            let operation = Operation::DeleteFrontmatter(build_delete_frontmatter_operation(args));
            doc.apply(vec![operation]).map_err(map_splice_error)?;
            finalize_output(
                OutputMode::Write,
                &output,
                &file,
                &input_content,
                doc.render(),
            )
        }
    }
}

fn validate_stdin_usage(file: &Option<PathBuf>, command: &Command) -> anyhow::Result<()> {
    if let Command::Insert(args) | Command::Replace(args) = command {
        let content_from_stdin = args
            .content_file
            .as_deref()
            .is_some_and(|path| path.to_string_lossy() == "-");

        if file.is_none() && content_from_stdin {
            return Err(SpliceError::AmbiguousStdinSource.into());
        }
    }

    if let Command::Frontmatter(FrontmatterCommand::Set(args)) = command {
        let value_from_stdin = args
            .value_file
            .as_deref()
            .is_some_and(|path| path.to_string_lossy() == "-");

        if file.is_none() && value_from_stdin {
            return Err(SpliceError::AmbiguousStdinSource.into());
        }
    }

    Ok(())
}

fn read_input(path: Option<&PathBuf>) -> anyhow::Result<String> {
    if let Some(file_path) = path {
        fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read input file: {}", file_path.display()))
    } else {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        Ok(buf)
    }
}

fn finalize_output(
    mode: OutputMode,
    output_path: &Option<PathBuf>,
    input_path: &Option<PathBuf>,
    original_content: &str,
    rendered_content: String,
) -> anyhow::Result<()> {
    match mode {
        OutputMode::DryRun => {
            io::stdout().write_all(rendered_content.as_bytes())?;
            return Ok(());
        }
        OutputMode::Diff => {
            let diff_output = TextDiff::from_lines(original_content, &rendered_content)
                .unified_diff()
                .header("original", "modified")
                .to_string();

            io::stdout().write_all(diff_output.as_bytes())?;
            return Ok(());
        }
        OutputMode::Write => {}
    }

    if let Some(path) = output_path {
        fs::write(path, &rendered_content)
            .with_context(|| format!("Failed to write to output file: {}", path.display()))?;
        return Ok(());
    }

    if let Some(input_path) = input_path {
        let parent_dir = input_path.parent().ok_or_else(|| {
            anyhow!(
                "Could not determine parent directory of {}",
                input_path.display()
            )
        })?;

        let mut temp_file = TempFileBuilder::new()
            .prefix(".md-splice-")
            .suffix(".tmp")
            .tempfile_in(parent_dir)
            .with_context(|| {
                format!(
                    "Failed to create temporary file in {}",
                    parent_dir.display()
                )
            })?;

        temp_file
            .write_all(rendered_content.as_bytes())
            .with_context(|| "Failed to write to temporary file")?;

        temp_file
            .persist(input_path)
            .with_context(|| format!("Failed to replace original file {}", input_path.display()))?;
    } else {
        io::stdout().write_all(rendered_content.as_bytes())?;
    }

    Ok(())
}

fn build_insert_operation(args: ModificationArgs) -> anyhow::Result<InsertOperation> {
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

    if until_type.is_some() || until_contains.is_some() || until_regex.is_some() {
        return Err(anyhow!(
            "The --until-* flags can only be used with the 'replace' command"
        ));
    }

    let selector = build_transaction_selector(
        select_type,
        select_contains,
        select_regex,
        select_ordinal,
        build_optional_transaction_selector(
            after_select_type,
            after_select_contains,
            after_select_regex,
            after_select_ordinal,
            "--after-select-regex",
        )?,
        build_optional_transaction_selector(
            within_select_type,
            within_select_contains,
            within_select_regex,
            within_select_ordinal,
            "--within-select-regex",
        )?,
    )?;

    Ok(InsertOperation {
        selector: Some(selector),
        selector_ref: None,
        comment: None,
        content,
        content_file,
        position: map_cli_insert_position(position),
    })
}

fn build_replace_operation(args: ModificationArgs) -> anyhow::Result<ReplaceOperation> {
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
        position: _,
    } = args;

    let selector = build_transaction_selector(
        select_type,
        select_contains,
        select_regex,
        select_ordinal,
        build_optional_transaction_selector(
            after_select_type,
            after_select_contains,
            after_select_regex,
            after_select_ordinal,
            "--after-select-regex",
        )?,
        build_optional_transaction_selector(
            within_select_type,
            within_select_contains,
            within_select_regex,
            within_select_ordinal,
            "--within-select-regex",
        )?,
    )?;

    let until_selector = build_optional_transaction_selector(
        until_type,
        until_contains,
        until_regex,
        None,
        "--until-regex",
    )?;

    Ok(ReplaceOperation {
        selector: Some(selector),
        selector_ref: None,
        comment: None,
        content,
        content_file,
        until: until_selector,
        until_ref: None,
    })
}

fn build_delete_operation(args: DeleteArgs) -> anyhow::Result<DeleteOperation> {
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

    let selector = build_transaction_selector(
        select_type,
        select_contains,
        select_regex,
        select_ordinal,
        build_optional_transaction_selector(
            after_select_type,
            after_select_contains,
            after_select_regex,
            after_select_ordinal,
            "--after-select-regex",
        )?,
        build_optional_transaction_selector(
            within_select_type,
            within_select_contains,
            within_select_regex,
            within_select_ordinal,
            "--within-select-regex",
        )?,
    )?;

    let until_selector = build_optional_transaction_selector(
        until_type,
        until_contains,
        until_regex,
        None,
        "--until-regex",
    )?;

    Ok(DeleteOperation {
        selector: Some(selector),
        selector_ref: None,
        comment: None,
        section,
        until: until_selector,
        until_ref: None,
    })
}

fn build_set_frontmatter_operation(
    args: FrontmatterSetArgs,
) -> anyhow::Result<SetFrontmatterOperation> {
    let FrontmatterSetArgs {
        key,
        value,
        value_file,
        format,
    } = args;

    let value = if let Some(inline) = value {
        Some(parse_yaml_value(&inline)?)
    } else {
        None
    };

    Ok(SetFrontmatterOperation {
        key,
        comment: None,
        value,
        value_file,
        format: format.map(map_frontmatter_format),
    })
}

fn build_delete_frontmatter_operation(args: FrontmatterDeleteArgs) -> DeleteFrontmatterOperation {
    let FrontmatterDeleteArgs { key } = args;
    DeleteFrontmatterOperation { key, comment: None }
}

fn prepare_apply_operations(args: ApplyArgs) -> anyhow::Result<(Vec<Operation>, OutputMode)> {
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
        (Some(_), Some(_)) => unreachable!("clap enforces mutual exclusivity"),
        (None, None) => {
            return Err(anyhow!(
                "Either --operations-file or --operations must be provided."
            ));
        }
    };

    let operations: Vec<Operation> = serde_yaml::from_str(&operations_data)
        .with_context(|| "Failed to parse operations data as JSON or YAML")?;

    let mode = if diff {
        OutputMode::Diff
    } else if dry_run {
        OutputMode::DryRun
    } else {
        OutputMode::Write
    };

    Ok((operations, mode))
}

fn process_get(content: &str, args: GetArgs) -> anyhow::Result<()> {
    let parsed = frontmatter::parse(content)?;
    let doc = parse_markdown(MarkdownParserState::default(), &parsed.body)
        .map_err(|e| anyhow!("Failed to parse input markdown: {}", e))?;
    let blocks = doc.blocks;

    let selector = build_locator_selector_from_args(
        args.select_type,
        args.select_contains,
        args.select_regex,
        args.select_ordinal,
        args.after_select_type,
        args.after_select_contains,
        args.after_select_regex,
        args.after_select_ordinal,
        args.within_select_type,
        args.within_select_contains,
        args.within_select_regex,
        args.within_select_ordinal,
    )?;

    let until_selector = build_optional_locator_selector_from_args(
        "--until-regex",
        args.until_type,
        args.until_contains,
        args.until_regex,
        None,
    )?;

    if args.select_all {
        let matches = locate_all(&blocks, &selector)?;
        if matches.is_empty() {
            return Ok(());
        }

        let mut had_trailing_newline = false;
        let mut rendered_items = Vec::with_capacity(matches.len());
        for found in &matches {
            let rendered = if args.section {
                render_heading_section(&blocks, found)?
            } else {
                render_found_node(&blocks, found)?
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

        let mut output = normalized.join(&args.separator);
        if had_trailing_newline && args.separator.ends_with('\n') {
            output.push('\n');
        }

        let mut stdout = io::stdout().lock();
        stdout.write_all(output.as_bytes())?;
        stdout.flush()?;
        return Ok(());
    }

    let (found_node, _) = locate(&blocks, &selector)?;
    let mut stdout = io::stdout().lock();
    let rendered = match &found_node {
        FoundNode::Block { index, .. } => {
            if let Some(until_selector) = until_selector.as_ref() {
                let end_index = compute_range_end(&blocks, *index, until_selector)?;
                render_blocks(&blocks[*index..end_index])
            } else if args.section {
                render_heading_section(&blocks, &found_node)?
            } else {
                render_found_node(&blocks, &found_node)?
            }
        }
        FoundNode::ListItem { .. } => {
            if until_selector.is_some() {
                return Err(SpliceError::RangeRequiresBlock.into());
            }
            render_found_node(&blocks, &found_node)?
        }
    };
    stdout.write_all(rendered.as_bytes())?;
    stdout.flush()?;

    Ok(())
}

fn process_frontmatter_get(content: &str, args: FrontmatterGetArgs) -> anyhow::Result<()> {
    let parsed = frontmatter::parse(content)?;

    let Some(frontmatter) = parsed.frontmatter else {
        if args.key.is_some() {
            return Err(SpliceError::FrontmatterMissing.into());
        }
        return Ok(());
    };

    if let Some(key) = args.key {
        let segments = parse_frontmatter_path(&key)?;
        if let Some(value) = resolve_frontmatter_path(&frontmatter, &segments) {
            print_frontmatter_value(value, args.output_format)?;
        } else {
            return Err(SpliceError::FrontmatterKeyNotFound(key).into());
        }
    } else {
        match args.output_format {
            FrontmatterOutputFormat::String | FrontmatterOutputFormat::Yaml => {
                let rendered = frontmatter::serialize_yaml_value(&frontmatter)?;
                println!("{}", rendered);
            }
            FrontmatterOutputFormat::Json => {
                let json = serde_json::to_string_pretty(&frontmatter)?;
                println!("{}", json);
            }
        }
    }

    Ok(())
}

fn build_transaction_selector(
    select_type: Option<String>,
    select_contains: Option<String>,
    select_regex: Option<String>,
    select_ordinal: usize,
    after: Option<TxSelector>,
    within: Option<TxSelector>,
) -> anyhow::Result<TxSelector> {
    if let Some(pattern) = &select_regex {
        Regex::new(pattern)
            .with_context(|| "Invalid regex pattern for --select-regex".to_string())?;
    }

    Ok(TxSelector {
        alias: None,
        select_type,
        select_contains,
        select_regex,
        select_ordinal,
        after: after.map(Box::new),
        after_ref: None,
        within: within.map(Box::new),
        within_ref: None,
    })
}

fn build_optional_transaction_selector(
    select_type: Option<String>,
    select_contains: Option<String>,
    select_regex: Option<String>,
    select_ordinal: Option<usize>,
    regex_context: &str,
) -> anyhow::Result<Option<TxSelector>> {
    if select_type.is_none() && select_contains.is_none() && select_regex.is_none() {
        return Ok(None);
    }

    if let Some(pattern) = &select_regex {
        Regex::new(pattern)
            .with_context(|| format!("Invalid regex pattern for {regex_context}"))?;
    }

    Ok(Some(TxSelector {
        alias: None,
        select_type,
        select_contains,
        select_regex,
        select_ordinal: select_ordinal.unwrap_or(1),
        after: None,
        after_ref: None,
        within: None,
        within_ref: None,
    }))
}

#[allow(clippy::too_many_arguments)]
fn build_locator_selector_from_args(
    select_type: Option<String>,
    select_contains: Option<String>,
    select_regex: Option<String>,
    select_ordinal: usize,
    after_select_type: Option<String>,
    after_select_contains: Option<String>,
    after_select_regex: Option<String>,
    after_select_ordinal: Option<usize>,
    within_select_type: Option<String>,
    within_select_contains: Option<String>,
    within_select_regex: Option<String>,
    within_select_ordinal: Option<usize>,
) -> anyhow::Result<Selector> {
    let after = build_optional_locator_selector_from_args(
        "--after-select-regex",
        after_select_type,
        after_select_contains,
        after_select_regex,
        after_select_ordinal,
    )?;
    let within = build_optional_locator_selector_from_args(
        "--within-select-regex",
        within_select_type,
        within_select_contains,
        within_select_regex,
        within_select_ordinal,
    )?;

    build_primary_selector(
        select_type,
        select_contains,
        select_regex,
        select_ordinal,
        after,
        within,
    )
}

fn build_optional_locator_selector_from_args(
    context: &str,
    select_type: Option<String>,
    select_contains: Option<String>,
    select_regex: Option<String>,
    select_ordinal: Option<usize>,
) -> anyhow::Result<Option<Selector>> {
    if select_type.is_none() && select_contains.is_none() && select_regex.is_none() {
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

fn compile_optional_regex(pattern: Option<String>, context: &str) -> anyhow::Result<Option<Regex>> {
    pattern
        .map(|pattern| {
            Regex::new(&pattern).with_context(|| format!("Invalid regex pattern for {context}"))
        })
        .transpose()
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
    let temp_doc = markdown_ppp::ast::Document {
        blocks: blocks.to_vec(),
    };
    let mut rendered = render_markdown(&temp_doc, default_printer_config());
    if !rendered.is_empty() && !rendered.ends_with('\n') {
        rendered.push('\n');
    }
    rendered
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
                for next in chars.by_ref() {
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
    value: &'a YamlValue,
    segments: &[FrontmatterPathSegment],
) -> Option<&'a YamlValue> {
    let mut cursor = value;
    for segment in segments {
        match segment {
            FrontmatterPathSegment::Key(key) => {
                let mapping = cursor.as_mapping()?;
                let mut next_value = None;
                for (map_key, map_value) in mapping {
                    if map_key.as_str() == Some(key.as_str()) {
                        next_value = Some(map_value);
                        break;
                    }
                }
                cursor = next_value?;
            }
            FrontmatterPathSegment::Index(index) => {
                let sequence = cursor.as_sequence()?;
                cursor = sequence.get(*index)?;
            }
        }
    }
    Some(cursor)
}

fn print_frontmatter_value(
    value: &YamlValue,
    format: FrontmatterOutputFormat,
) -> anyhow::Result<()> {
    match format {
        FrontmatterOutputFormat::String => match value {
            YamlValue::Null => {}
            YamlValue::Bool(v) => println!("{}", v),
            YamlValue::Number(v) => println!("{}", v),
            YamlValue::String(v) => println!("{}", v),
            other => {
                let rendered = frontmatter::serialize_yaml_value(other)?;
                println!("{}", rendered);
            }
        },
        FrontmatterOutputFormat::Json => {
            let json = serde_json::to_string_pretty(value)?;
            println!("{}", json);
        }
        FrontmatterOutputFormat::Yaml => {
            let rendered = frontmatter::serialize_yaml_value(value)?;
            println!("{}", rendered);
        }
    }

    Ok(())
}

fn parse_yaml_value(content: &str) -> anyhow::Result<YamlValue> {
    serde_yaml::from_str(content)
        .with_context(|| "Failed to parse value as YAML for frontmatter set operation")
}

fn map_frontmatter_format(arg: FrontmatterFormatArg) -> FrontmatterFormat {
    match arg {
        FrontmatterFormatArg::Yaml => FrontmatterFormat::Yaml,
        FrontmatterFormatArg::Toml => FrontmatterFormat::Toml,
    }
}

#[derive(Debug)]
enum FrontmatterPathSegment {
    Key(String),
    Index(usize),
}

fn map_cli_insert_position(position: CliInsertPosition) -> TxInsertPosition {
    match position {
        CliInsertPosition::Before => TxInsertPosition::Before,
        CliInsertPosition::After => TxInsertPosition::After,
        CliInsertPosition::PrependChild => TxInsertPosition::PrependChild,
        CliInsertPosition::AppendChild => TxInsertPosition::AppendChild,
    }
}

fn get_heading_level(block: &Block) -> Option<u8> {
    if let Block::Heading(Heading { kind, .. }) = block {
        Some(match kind {
            HeadingKind::Atx(level) => *level,
            HeadingKind::Setext(SetextHeading::Level1) => 1,
            HeadingKind::Setext(SetextHeading::Level2) => 2,
        })
    } else {
        None
    }
}

fn find_heading_section_end(blocks: &[Block], start_index: usize, target_level: u8) -> usize {
    for (i, block) in blocks.iter().enumerate().skip(start_index + 1) {
        if let Some(level) = get_heading_level(block) {
            if level <= target_level {
                return i;
            }
        }
    }
    blocks.len()
}

fn map_splice_error(err: SpliceError) -> anyhow::Error {
    match err {
        SpliceError::OperationFailed(message) => anyhow!(message),
        other => anyhow!(other),
    }
}

#[derive(Clone, Copy)]
enum OutputMode {
    Write,
    DryRun,
    Diff,
}
