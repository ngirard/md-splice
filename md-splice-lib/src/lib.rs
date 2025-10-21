//! `md-splice-lib` exposes the AST-aware Markdown editing primitives that power
//! the `md-splice` CLI.
//!
//! The library parses Markdown into a structured syntax tree, applies
//! transactional operations against that tree, and renders the resulting
//! document while preserving frontmatter. You can use it directly to perform
//! automated documentation maintenance without shelling out to the CLI.
//!
//! # Example
//!
//! ```rust
//! use std::str::FromStr;
//!
//! use md_splice_lib::transaction::{InsertOperation, InsertPosition, Operation, Selector};
//! use md_splice_lib::MarkdownDocument;
//!
//! # fn demo() -> Result<(), md_splice_lib::error::SpliceError> {
//! let mut document = MarkdownDocument::from_str(
//!     "---\nstatus: pending\n---\n\n## Tasks\n\n- [ ] Write docs\n",
//! )?;
//!
//! let selector = Selector {
//!     select_type: Some("list".into()),
//!     within: Some(Box::new(Selector {
//!         select_type: Some("h2".into()),
//!         select_contains: Some("Tasks".into()),
//!         ..Selector::default()
//!     })),
//!     ..Selector::default()
//! };
//!
//! let operation = Operation::Insert(InsertOperation {
//!     selector,
//!     position: InsertPosition::AppendChild,
//!     content: Some("- [ ] Review open issues".into()),
//!     ..InsertOperation::default()
//! });
//!
//! document.apply(vec![operation])?;
//! assert!(document.render().contains("Review open issues"));
//! # Ok(())
//! # }
//! ```

pub mod error;
pub mod frontmatter;
pub mod locator;
pub mod splicer;
pub mod transaction;

use crate::error::SpliceError;
use crate::frontmatter::{refresh_frontmatter_block, FrontmatterFormat, ParsedDocument};
use crate::locator::{locate, FoundNode, Selector};
use crate::splicer::{
    delete, delete_list_item, delete_section, insert, insert_list_item, replace, replace_list_item,
};
use crate::transaction::{
    DeleteFrontmatterOperation, DeleteOperation, InsertOperation, Operation,
    ReplaceFrontmatterOperation, ReplaceOperation, Selector as TransactionSelector,
    SetFrontmatterOperation,
};
use anyhow::{anyhow, Context};
use markdown_ppp::ast::Block;
use markdown_ppp::ast::Document;
use markdown_ppp::parser::{parse_markdown, MarkdownParserState};
use markdown_ppp::printer::{config::Config as PrinterConfig, render_markdown};
use regex::Regex;
use serde_yaml::{Mapping, Value as YamlValue};
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::str::FromStr;

/// Represents an in-memory Markdown document that can be manipulated using
/// AST-aware operations.
pub struct MarkdownDocument {
    parsed: ParsedDocument,
    doc: Document,
}

impl MarkdownDocument {
    /// Applies a list of transactional operations to the document.
    ///
    /// Operations are executed sequentially against a temporary copy of the
    /// document. If every step succeeds, the working copy replaces the
    /// original. If any step fails (e.g., because a selector matches nothing),
    /// the document is left untouched and a [`SpliceError`] is returned.
    pub fn apply(&mut self, operations: Vec<Operation>) -> Result<(), SpliceError> {
        let frontmatter_mutated =
            apply_operations(&mut self.doc.blocks, &mut self.parsed, operations)?;

        if frontmatter_mutated {
            refresh_frontmatter_block(&mut self.parsed)
                .map_err(|err| SpliceError::FrontmatterSerialize(err.to_string()))?;
        }

        Ok(())
    }

    /// Renders the document, including frontmatter, back to a Markdown string.
    ///
    /// The output preserves the original frontmatter delimiter style and uses
    /// `markdown-ppp`'s default printer configuration for the body.
    pub fn render(&self) -> String {
        let mut output = String::new();

        if let Some(prefix) = self.parsed.frontmatter_block.as_deref() {
            output.push_str(prefix);
        }

        let body_output = render_markdown(&self.doc, PrinterConfig::default());
        output.push_str(&body_output);

        output
    }

    /// Provides read-only access to the Markdown AST blocks.
    pub fn blocks(&self) -> &[Block] {
        &self.doc.blocks
    }

    /// Returns the parsed frontmatter value, if present.
    pub fn frontmatter(&self) -> Option<&YamlValue> {
        self.parsed.frontmatter.as_ref()
    }

    /// Returns the serialization format of the frontmatter, if known.
    pub fn frontmatter_format(&self) -> Option<FrontmatterFormat> {
        self.parsed.format
    }
}

impl FromStr for MarkdownDocument {
    type Err = SpliceError;

    /// Parses Markdown (including optional YAML/TOML frontmatter) into a
    /// [`MarkdownDocument`].
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let parsed = frontmatter::parse(content)
            .map_err(|err| SpliceError::FrontmatterParse(err.to_string()))?;
        let doc = parse_markdown(MarkdownParserState::default(), &parsed.body)
            .map_err(|err| SpliceError::MarkdownParse(err.to_string()))?;

        Ok(Self { parsed, doc })
    }
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

fn apply_operations(
    doc_blocks: &mut Vec<Block>,
    parsed_document: &mut ParsedDocument,
    operations: Vec<Operation>,
) -> Result<bool, SpliceError> {
    let mut working_blocks = doc_blocks.clone();
    let mut working_document = parsed_document.clone();
    let mut frontmatter_mutated = false;

    for operation in operations {
        match operation {
            Operation::Replace(replace_op) => {
                apply_replace_operation(&mut working_blocks, replace_op)
                    .map_err(|err| SpliceError::OperationFailed(err.to_string()))?
            }
            Operation::Insert(insert_op) => apply_insert_operation(&mut working_blocks, insert_op)
                .map_err(|err| SpliceError::OperationFailed(err.to_string()))?,
            Operation::Delete(delete_op) => apply_delete_operation(&mut working_blocks, delete_op)
                .map_err(|err| SpliceError::OperationFailed(err.to_string()))?,
            Operation::SetFrontmatter(set_op) => {
                apply_set_frontmatter_operation(&mut working_document, set_op)
                    .map_err(|err| SpliceError::OperationFailed(err.to_string()))?;
                frontmatter_mutated = true;
            }
            Operation::DeleteFrontmatter(delete_op) => {
                apply_delete_frontmatter_operation(&mut working_document, delete_op)
                    .map_err(|err| SpliceError::OperationFailed(err.to_string()))?;
                frontmatter_mutated = true;
            }
            Operation::ReplaceFrontmatter(replace_op) => {
                apply_replace_frontmatter_operation(&mut working_document, replace_op)
                    .map_err(|err| SpliceError::OperationFailed(err.to_string()))?;
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
    let position = operation.position;

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
    selector.map(build_locator_selector).transpose()
}

#[allow(dead_code)]
fn resolve_operation_content(
    content: Option<String>,
    content_file: Option<PathBuf>,
) -> anyhow::Result<String> {
    match (content, content_file) {
        (Some(inline), None) => Ok(inline),
        (None, Some(path)) => {
            if path.to_string_lossy() == "-" {
                let mut buf = String::new();
                io::stdin()
                    .read_to_string(&mut buf)
                    .with_context(|| "Failed to read content from stdin")?;
                Ok(buf)
            } else {
                fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read content file: {}", path.display()))
            }
        }
        (Some(_), Some(_)) => Err(anyhow!(
            "Operation cannot specify both inline content and a content_file"
        )),
        (None, None) => Err(anyhow!(
            "Operation must provide inline content or a content_file"
        )),
    }
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

        let frontmatter_changed = apply_operations(&mut blocks, &mut parsed_document, operations)
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

        let frontmatter_changed = apply_operations(&mut blocks, &mut parsed_document, operations)
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

        let frontmatter_changed = apply_operations(&mut blocks, &mut parsed_document, operations)
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

        let frontmatter_changed = apply_operations(&mut blocks, &mut parsed_document, operations)
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

        let frontmatter_changed = apply_operations(&mut blocks, &mut parsed_document, operations)
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

        let result = apply_operations(&mut blocks, &mut parsed_document, operations);

        assert!(
            result.is_err(),
            "apply_operations should fail when a selector does not match"
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
