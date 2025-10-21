//! Contains the logic for finding a target node within the Markdown AST.

use crate::error::SpliceError;
use markdown_ppp::ast::{
    Block, FootnoteDefinition, HeadingKind, Inline, List, ListItem, SetextHeading, Table, TaskState,
};
use regex::Regex;

/// Represents the location of a found block.
#[derive(Debug, PartialEq)]
pub enum FoundNode<'a> {
    Block {
        index: usize,
        block: &'a Block,
    },
    ListItem {
        block_index: usize, // Index of the parent Block::List
        item_index: usize,  // Index of the ListItem within the list
        item: &'a ListItem,
    },
}

/// A set of criteria for selecting a node.
#[derive(Debug, Default)]
pub struct Selector {
    pub select_type: Option<String>,
    pub select_contains: Option<String>,
    pub select_regex: Option<Regex>,
    pub select_ordinal: usize,
    pub after: Option<Box<Selector>>,
    pub within: Option<Box<Selector>>,
}

/// Checks if a type string refers to a list item.
fn is_list_item_type(type_str: &str) -> bool {
    matches!(type_str.to_lowercase().as_str(), "li" | "item" | "listitem")
}

/// Recursively extracts the plain text content from a `ListItem` node.
pub(crate) fn list_item_to_text(item: &ListItem) -> String {
    let body = item
        .blocks
        .iter()
        .map(block_to_text)
        .collect::<Vec<_>>()
        .join("\n");

    match item.task {
        Some(TaskState::Incomplete) => {
            if body.is_empty() {
                "[ ]".to_string()
            } else {
                format!("[ ] {}", body)
            }
        }
        Some(TaskState::Complete) => {
            if body.is_empty() {
                "[x]".to_string()
            } else {
                format!("[x] {}", body)
            }
        }
        None => body,
    }
}

#[derive(Debug, Clone, Copy)]
struct Scope {
    block_start: usize,
    block_end: usize,
    list_restriction: Option<ListRestriction>,
}

impl Scope {
    fn entire_document(len: usize) -> Self {
        Self {
            block_start: 0,
            block_end: len,
            list_restriction: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ListRestriction {
    block_index: usize,
    start_item: Option<usize>,
}

fn heading_level(kind: &HeadingKind) -> usize {
    match kind {
        HeadingKind::Atx(level) => usize::from(*level),
        HeadingKind::Setext(SetextHeading::Level1) => 1,
        HeadingKind::Setext(SetextHeading::Level2) => 2,
    }
}

fn find_section_end(blocks: &[Block], heading_index: usize, level: usize) -> usize {
    let mut end = blocks.len();
    for (idx, block) in blocks.iter().enumerate().skip(heading_index + 1) {
        if let Block::Heading(h) = block {
            if heading_level(&h.kind) <= level {
                end = idx;
                break;
            }
        }
    }
    end
}

fn apply_scope<'a>(blocks: &'a [Block], selector: &Selector) -> Result<Scope, SpliceError> {
    if selector.after.is_some() && selector.within.is_some() {
        return Err(SpliceError::ConflictingScopeModifiers);
    }

    if let Some(after_selector) = selector.after.as_ref() {
        let (landmark, _) = locate(blocks, after_selector)?;
        match landmark {
            FoundNode::Block { index, .. } => Ok(Scope {
                block_start: index.saturating_add(1),
                block_end: blocks.len(),
                list_restriction: None,
            }),
            FoundNode::ListItem {
                block_index,
                item_index,
                ..
            } => Ok(Scope {
                block_start: block_index.saturating_add(1),
                block_end: blocks.len(),
                list_restriction: Some(ListRestriction {
                    block_index,
                    start_item: Some(item_index),
                }),
            }),
        }
    } else if let Some(within_selector) = selector.within.as_ref() {
        let (landmark, _) = locate(blocks, within_selector)?;
        match landmark {
            FoundNode::Block { index, block } => match block {
                Block::Heading(heading) => {
                    let level = heading_level(&heading.kind);
                    let start = index.saturating_add(1);
                    let end = find_section_end(blocks, index, level);
                    Ok(Scope {
                        block_start: start,
                        block_end: end,
                        list_restriction: None,
                    })
                }
                Block::List(_) => Ok(Scope {
                    block_start: index,
                    block_end: index + 1,
                    list_restriction: Some(ListRestriction {
                        block_index: index,
                        start_item: None,
                    }),
                }),
                _ => Err(SpliceError::NodeNotFound),
            },
            FoundNode::ListItem { .. } => Err(SpliceError::NodeNotFound),
        }
    } else {
        Ok(Scope::entire_document(blocks.len()))
    }
}

fn block_matches_selector(block: &Block, selector: &Selector) -> bool {
    if let Some(type_str) = &selector.select_type {
        if !block_type_matches(block, type_str) {
            return false;
        }
    }

    if selector.select_contains.is_some() || selector.select_regex.is_some() {
        let text_content = block_to_text(block);

        if let Some(contains_str) = &selector.select_contains {
            if !text_content.contains(contains_str) {
                return false;
            }
        }

        if let Some(re) = &selector.select_regex {
            if !re.is_match(&text_content) {
                return false;
            }
        }
    }

    true
}

fn list_item_matches_filters(selector: &Selector, item: &ListItem) -> bool {
    if selector.select_contains.is_some() || selector.select_regex.is_some() {
        let text_content = list_item_to_text(item);

        if let Some(contains_str) = &selector.select_contains {
            if !text_content.contains(contains_str) {
                return false;
            }
        }

        if let Some(re) = &selector.select_regex {
            if !re.is_match(&text_content) {
                return false;
            }
        }
    }

    true
}

fn collect_scoped_list_items<'a>(
    blocks: &'a [Block],
    selector: &Selector,
    scope: Scope,
) -> Vec<(usize, usize, &'a ListItem)> {
    let mut items = Vec::new();

    if let Some(restriction) = scope.list_restriction {
        if let Some(Block::List(list)) = blocks.get(restriction.block_index) {
            for (item_index, item) in list.items.iter().enumerate() {
                if let Some(start) = restriction.start_item {
                    if item_index <= start {
                        continue;
                    }
                }

                if list_item_matches_filters(selector, item) {
                    items.push((restriction.block_index, item_index, item));
                }
            }
        }
    }

    for block_index in scope.block_start..scope.block_end {
        if let Some(restriction) = scope.list_restriction {
            if restriction.block_index == block_index {
                continue;
            }
        }

        if let Some(Block::List(list)) = blocks.get(block_index) {
            for (item_index, item) in list.items.iter().enumerate() {
                if list_item_matches_filters(selector, item) {
                    items.push((block_index, item_index, item));
                }
            }
        }
    }

    items
}

/// Finds the first node in the document that matches all the given selectors.
///
/// The function can find top-level `Block` nodes or nested `ListItem` nodes.
///
/// # Arguments
///
/// * `blocks`: A slice of `Block` nodes from a `Document`.
/// * `selector`: The selection criteria.
///
/// # Returns
///
/// A `Result` containing a tuple of `(FoundNode, bool)` on success, where the
/// boolean is `true` if more than one node matched the criteria (indicating ambiguity).
/// Returns a `SpliceError` if no node is found.
pub fn locate<'a>(
    blocks: &'a [Block],
    selector: &Selector,
) -> Result<(FoundNode<'a>, bool), SpliceError> {
    let ordinal_index = selector.select_ordinal.saturating_sub(1);
    let scope = apply_scope(blocks, selector)?;

    // --- Search Strategy ---
    // If the selector type is for a list item, we perform a nested search.
    // Otherwise, we perform the standard top-level block search.
    if let Some(type_str) = &selector.select_type {
        if is_list_item_type(type_str) {
            // --- List Item Search Logic ---
            let matches = collect_scoped_list_items(blocks, selector, scope);

            let is_ambiguous = matches.len() > 1;

            return matches
                .get(ordinal_index)
                .map(|(block_index, item_index, item)| {
                    (
                        FoundNode::ListItem {
                            block_index: *block_index,
                            item_index: *item_index,
                            item,
                        },
                        is_ambiguous,
                    )
                })
                .ok_or(SpliceError::NodeNotFound);
        }
    }

    // --- Block Search Logic (default) ---
    let matches: Vec<_> = (scope.block_start..scope.block_end)
        .filter_map(|index| {
            let block = blocks.get(index)?;
            if block_matches_selector(block, selector) {
                Some((index, block))
            } else {
                None
            }
        })
        .collect();

    let is_ambiguous = matches.len() > 1;

    matches
        .get(ordinal_index)
        .map(|(index, block)| {
            (
                FoundNode::Block {
                    index: *index,
                    block,
                },
                is_ambiguous,
            )
        })
        .ok_or(SpliceError::NodeNotFound)
}

/// Finds all nodes matching the selector criteria.
pub fn locate_all<'a>(
    blocks: &'a [Block],
    selector: &Selector,
) -> Result<Vec<FoundNode<'a>>, SpliceError> {
    let scope = apply_scope(blocks, selector)?;

    if let Some(type_str) = &selector.select_type {
        if is_list_item_type(type_str) {
            let matches = collect_scoped_list_items(blocks, selector, scope)
                .into_iter()
                .map(|(block_index, item_index, item)| FoundNode::ListItem {
                    block_index,
                    item_index,
                    item,
                })
                .collect();

            return Ok(matches);
        }
    }

    let matches = (scope.block_start..scope.block_end)
        .filter_map(|index| {
            let block = blocks.get(index)?;
            if block_matches_selector(block, selector) {
                Some(FoundNode::Block { index, block })
            } else {
                None
            }
        })
        .collect();

    Ok(matches)
}

/// Checks if a block matches the string representation of its type.
/// This version is more explicit and robust for handling heading levels.
fn block_type_matches(block: &Block, type_str: &str) -> bool {
    let type_str = type_str.to_lowercase();
    match block {
        Block::Paragraph(_) => type_str == "p" || type_str == "paragraph",
        Block::Heading(h) => {
            let level = match h.kind {
                HeadingKind::Atx(level) => level,
                HeadingKind::Setext(SetextHeading::Level1) => 1,
                HeadingKind::Setext(SetextHeading::Level2) => 2,
            };
            match type_str.as_str() {
                "heading" => true,
                "h1" if level == 1 => true,
                "h2" if level == 2 => true,
                "h3" if level == 3 => true,
                "h4" if level == 4 => true,
                "h5" if level == 5 => true,
                "h6" if level == 6 => true,
                _ => false,
            }
        }
        Block::List(_) => type_str == "list",
        Block::Table(_) => type_str == "table",
        Block::BlockQuote(_) => type_str == "blockquote",
        Block::CodeBlock(_) => type_str == "code" || type_str == "codeblock",
        Block::HtmlBlock(_) => type_str == "html" || type_str == "htmlblock",
        Block::ThematicBreak => type_str == "thematicbreak",
        Block::Definition(_) => type_str == "definition",
        Block::FootnoteDefinition(_) => type_str == "footnotedefinition",
        Block::GitHubAlert(alert) => {
            let alert_type = match alert.alert_type {
                markdown_ppp::ast::GitHubAlertType::Note => "note",
                markdown_ppp::ast::GitHubAlertType::Tip => "tip",
                markdown_ppp::ast::GitHubAlertType::Important => "important",
                markdown_ppp::ast::GitHubAlertType::Warning => "warning",
                markdown_ppp::ast::GitHubAlertType::Caution => "caution",
            };

            type_str == "githubalert"
                || type_str == "alert"
                || type_str == alert_type
                || type_str == format!("alert-{}", alert_type)
        }
        Block::Empty => type_str == "empty",
    }
}

/// Recursively extracts the plain text from a slice of `Inline` nodes.
fn inlines_to_text(inlines: &[Inline]) -> String {
    inlines
        .iter()
        .map(|inline| -> String {
            match inline {
                Inline::Text(s) | Inline::Code(s) => s.clone(),
                Inline::Link(link) => inlines_to_text(&link.children),
                Inline::Image(image) => image.alt.clone(),
                Inline::Emphasis(children)
                | Inline::Strong(children)
                | Inline::Strikethrough(children) => inlines_to_text(children),
                Inline::LinkReference(link_ref) => inlines_to_text(&link_ref.text),
                // Per spec, other inlines do not contribute to text content
                Inline::LineBreak
                | Inline::Html(_)
                | Inline::Autolink(_)
                | Inline::FootnoteReference(_)
                | Inline::Empty => String::new(),
            }
        })
        .collect()
}

/// Recursively extracts the plain text content from a `Block` node.
pub(crate) fn block_to_text(block: &Block) -> String {
    match block {
        Block::Paragraph(inlines) => inlines_to_text(inlines),
        Block::Heading(heading) => inlines_to_text(&heading.content),
        Block::BlockQuote(blocks) => blocks
            .iter()
            .map(block_to_text)
            .collect::<Vec<_>>()
            .join("\n"),
        Block::List(List { items, .. }) => items
            .iter()
            .map(|item| {
                item.blocks
                    .iter()
                    .map(block_to_text)
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .collect::<Vec<_>>()
            .join("\n"),
        Block::CodeBlock(code_block) => code_block.literal.clone(),
        Block::Table(Table { rows, .. }) => rows
            .iter()
            .map(|row| {
                row.iter()
                    .map(|cell| inlines_to_text(cell))
                    .collect::<Vec<_>>()
                    .join("\t")
            })
            .collect::<Vec<_>>()
            .join("\n"),
        Block::FootnoteDefinition(FootnoteDefinition { blocks, .. }) => blocks
            .iter()
            .map(block_to_text)
            .collect::<Vec<_>>()
            .join("\n"),
        Block::GitHubAlert(alert) => alert
            .blocks
            .iter()
            .map(block_to_text)
            .collect::<Vec<_>>()
            .join("\n"),
        // Per spec, these blocks have no user-facing text content
        Block::ThematicBreak | Block::HtmlBlock(_) | Block::Definition(_) | Block::Empty => {
            String::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use markdown_ppp::parser::{parse_markdown, MarkdownParserState};

    const TEST_MARKDOWN: &str = r#"# A Heading

This is the first paragraph.

- A list item
- Another list item

This is the second paragraph. Note the content.

```rust
fn main() {
    println!("Hello, World!");
}
```
"#;

    #[test]
    fn test_l1_select_first_paragraph_by_type() {
        // L1 (Simple Type): Select the first paragraph.
        let doc = parse_markdown(MarkdownParserState::default(), TEST_MARKDOWN).unwrap();
        let selector = Selector {
            select_type: Some("p".to_string()),
            select_ordinal: 1,
            ..Default::default()
        };

        let result = locate(&doc.blocks, &selector);

        assert!(result.is_ok(), "locate should find a matching block");
        let (found, is_ambiguous) = result.unwrap();

        // The first paragraph is the second block (index 1) after the H1.
        assert!(
            matches!(found, FoundNode::Block { index, .. } if index == 1),
            "Expected to find block at index 1"
        );
        if let FoundNode::Block { block, .. } = found {
            assert!(matches!(block, Block::Paragraph(_)));
        }
        assert!(
            is_ambiguous,
            "should detect ambiguity as there are two paragraphs"
        );
    }

    #[test]
    fn test_l2_select_second_paragraph_by_type_and_ordinal() {
        // L2 (Type & Ordinal): Select the *second* paragraph.
        let doc = parse_markdown(MarkdownParserState::default(), TEST_MARKDOWN).unwrap();
        let selector = Selector {
            select_type: Some("paragraph".to_string()),
            select_ordinal: 2,
            ..Default::default()
        };

        let result = locate(&doc.blocks, &selector);

        assert!(result.is_ok(), "locate should find a matching block");
        let (found, is_ambiguous) = result.unwrap();

        // The markdown has: H1, P, List, P, CodeBlock.
        // So the second paragraph is at index 3.
        assert!(
            matches!(found, FoundNode::Block { index, .. } if index == 3),
            "Expected to find block at index 3"
        );
        if let FoundNode::Block { block, .. } = found {
            assert!(matches!(block, Block::Paragraph(_)));
        }
        assert!(
            is_ambiguous,
            "should detect ambiguity as there are two paragraphs"
        );
    }

    #[test]
    fn test_l3_select_heading_by_content_contains() {
        // L3 (Content Contains): Select a heading containing a specific word.
        let doc = parse_markdown(MarkdownParserState::default(), TEST_MARKDOWN).unwrap();
        let selector = Selector {
            select_contains: Some("Heading".to_string()),
            select_ordinal: 1,
            ..Default::default()
        };

        let result = locate(&doc.blocks, &selector);

        assert!(result.is_ok(), "locate should find a matching block");
        let (found, is_ambiguous) = result.unwrap();

        // The first block (index 0) is the H1 with content "A Heading".
        assert!(
            matches!(found, FoundNode::Block { index, .. } if index == 0),
            "Expected to find block at index 0"
        );
        if let FoundNode::Block { block, .. } = found {
            assert!(matches!(block, Block::Heading(_)));
        }
        assert!(
            !is_ambiguous,
            "should not detect ambiguity as there is only one heading"
        );
    }

    #[test]
    fn test_l4_select_code_block_by_content_regex() {
        // L4 (Content Regex): Select a code block matching a regex.
        let doc = parse_markdown(MarkdownParserState::default(), TEST_MARKDOWN).unwrap();
        let selector = Selector {
            select_regex: Some(Regex::new(r"Hello, World!").unwrap()),
            select_ordinal: 1,
            ..Default::default()
        };

        let result = locate(&doc.blocks, &selector);

        assert!(result.is_ok(), "locate should find a matching block");
        let (found, is_ambiguous) = result.unwrap();

        // The code block is the last block (index 4).
        assert!(
            matches!(found, FoundNode::Block { index, .. } if index == 4),
            "Expected to find block at index 4"
        );
        if let FoundNode::Block { block, .. } = found {
            assert!(matches!(block, Block::CodeBlock(_)));
        }
        assert!(
            !is_ambiguous,
            "should not detect ambiguity as there is only one code block"
        );
    }

    const AMBIGUOUS_MARKDOWN: &str = r#"# Title

A paragraph.

Another paragraph with a Note.

A list.

A final paragraph, also with a Note.
"#;

    #[test]
    fn test_l5_select_combined_selectors() {
        // L5 (Combined Selectors): Select the 2nd paragraph that contains "Note".
        let doc = parse_markdown(MarkdownParserState::default(), AMBIGUOUS_MARKDOWN).unwrap();
        let selector = Selector {
            select_type: Some("p".to_string()),
            select_contains: Some("Note".to_string()),
            select_ordinal: 2,
            ..Default::default()
        };

        let result = locate(&doc.blocks, &selector);

        assert!(result.is_ok(), "locate should find a matching block");
        let (found, is_ambiguous) = result.unwrap();

        // Blocks are: H1, P, P, List, P.
        // Paragraphs with "Note" are at indices 2 and 4.
        // The second one is at index 4.
        assert!(
            matches!(found, FoundNode::Block { index, .. } if index == 4),
            "Expected to find block at index 4"
        );
        assert!(is_ambiguous, "should have detected ambiguity");
    }

    #[test]
    fn test_l6_no_match_returns_error() {
        // L6 (No Match): Verify SpliceError::NodeNotFound.
        let doc = parse_markdown(MarkdownParserState::default(), TEST_MARKDOWN).unwrap();
        let selector = Selector {
            select_type: Some("h2".to_string()), // No h2 in TEST_MARKDOWN
            select_ordinal: 1,
            ..Default::default()
        };

        let result = locate(&doc.blocks, &selector);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SpliceError::NodeNotFound));
    }

    #[test]
    fn test_l7_ambiguity_detection() {
        // L7 (Ambiguity Warning): Verify the ambiguity flag is set correctly.
        let doc = parse_markdown(MarkdownParserState::default(), AMBIGUOUS_MARKDOWN).unwrap();

        // Case 1: Ambiguous selector
        let selector_ambiguous = Selector {
            select_type: Some("p".to_string()),
            select_contains: Some("Note".to_string()),
            select_ordinal: 1,
            ..Default::default()
        };

        let result_ambiguous = locate(&doc.blocks, &selector_ambiguous);
        assert!(result_ambiguous.is_ok());
        let (found, is_ambiguous) = result_ambiguous.unwrap();
        // First match is at index 2
        assert!(matches!(found, FoundNode::Block { index, .. } if index == 2));
        assert!(
            is_ambiguous,
            "Expected ambiguity to be true when multiple nodes match criteria"
        );

        // Case 2: Unambiguous selector
        let selector_unambiguous = Selector {
            select_type: Some("h1".to_string()),
            select_ordinal: 1,
            ..Default::default()
        };

        let result_unambiguous = locate(&doc.blocks, &selector_unambiguous);
        assert!(result_unambiguous.is_ok());
        let (found, is_ambiguous) = result_unambiguous.unwrap();
        assert!(matches!(found, FoundNode::Block { index, .. } if index == 0));
        assert!(
            !is_ambiguous,
            "Expected ambiguity to be false when only one node matches"
        );
    }

    // --- Tests for Phase 4: List Item Selection ---

    const LIST_ITEM_MARKDOWN: &str = r#"# List Document

- First item
- Second item

A paragraph.

1. Third item
2. Fourth item
"#;

    const SCOPED_MARKDOWN: &str = r#"# Project Overview

Introduction paragraph.

## Installation
Overview of installation.
Additional installation notes.

- Step zero
- Step one
- Step two

Closing installation paragraph.

## Backlog
- [ ] Idea 1
- [ ] Idea 2

## Future Features
Intro to future features.

- [ ] Task Alpha
- [ ] Task Beta
- [x] Task Gamma

## Usage
Usage introduction.
More usage guidance.
"#;

    #[test]
    fn test_ll1_select_list_item_by_type_and_ordinal() {
        // LL1: Select the 3rd list item in a document containing multiple lists.
        let doc = parse_markdown(MarkdownParserState::default(), LIST_ITEM_MARKDOWN).unwrap();
        let selector = Selector {
            select_type: Some("li".to_string()),
            select_ordinal: 3,
            ..Default::default()
        };

        let result = locate(&doc.blocks, &selector);
        assert!(result.is_ok(), "locate should find a matching list item");
        let (found, is_ambiguous) = result.unwrap();

        // The flat list of items is [First, Second, Third, Fourth].
        // The 3rd item is "Third item".
        // It's in the second list (block index 3), and is the first item (item index 0).
        if let FoundNode::ListItem {
            block_index,
            item_index,
            item,
        } = found
        {
            assert_eq!(block_index, 3);
            assert_eq!(item_index, 0);
            let text = list_item_to_text(item);
            assert!(text.starts_with("Third item"));
            assert!(is_ambiguous, "Should detect 4 total matching 'li' nodes");
        } else {
            panic!("Expected to find a ListItem, but found {:?}", found);
        }
    }

    #[test]
    fn test_ll2_select_list_item_by_content() {
        // LL2: Select a list item using --select-contains.
        let doc = parse_markdown(MarkdownParserState::default(), LIST_ITEM_MARKDOWN).unwrap();
        let selector = Selector {
            select_type: Some("listitem".to_string()),
            select_contains: Some("Fourth".to_string()),
            ..Default::default()
        };

        let result = locate(&doc.blocks, &selector);
        assert!(result.is_ok());
        let (found, is_ambiguous) = result.unwrap();

        // The item "Fourth item" is the only match.
        if let FoundNode::ListItem {
            block_index,
            item_index,
            item,
        } = found
        {
            assert_eq!(block_index, 3);
            assert_eq!(item_index, 1);
            let text = list_item_to_text(item);
            assert!(text.starts_with("Fourth item"));
            assert!(
                !is_ambiguous,
                "Should not detect ambiguity for a unique match"
            );
        } else {
            panic!("Expected to find a ListItem, but found {:?}", found);
        }
    }

    #[test]
    fn test_ll3_select_list_item_by_regex() {
        // LL3: Select a list item using --select-regex.
        let doc = parse_markdown(MarkdownParserState::default(), LIST_ITEM_MARKDOWN).unwrap();
        let selector = Selector {
            select_type: Some("item".to_string()),
            select_regex: Some(Regex::new(r"(?i)second").unwrap()), // case-insensitive
            ..Default::default()
        };

        let result = locate(&doc.blocks, &selector);
        assert!(result.is_ok());
        let (found, is_ambiguous) = result.unwrap();

        // The item "Second item" is the only match.
        if let FoundNode::ListItem {
            block_index,
            item_index,
            item,
        } = found
        {
            assert_eq!(block_index, 1);
            assert_eq!(item_index, 1);
            let text = list_item_to_text(item);
            assert!(text.starts_with("Second item"));
            assert!(
                !is_ambiguous,
                "Should not detect ambiguity for a unique match"
            );
        } else {
            panic!("Expected to find a ListItem, but found {:?}", found);
        }
    }

    #[test]
    fn test_ll4_no_match_list_item() {
        // LL4: Verify SpliceError::NodeNotFound when a list item selector finds nothing.
        let doc = parse_markdown(MarkdownParserState::default(), LIST_ITEM_MARKDOWN).unwrap();
        let selector = Selector {
            select_type: Some("li".to_string()),
            select_contains: Some("Non-existent".to_string()),
            ..Default::default()
        };

        let result = locate(&doc.blocks, &selector);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SpliceError::NodeNotFound));
    }

    #[test]
    fn test_ll5_ambiguity_list_item() {
        // LL5: Verify the ambiguity warning is triggered when a selector matches multiple list items.
        let doc = parse_markdown(MarkdownParserState::default(), LIST_ITEM_MARKDOWN).unwrap();
        let selector = Selector {
            select_type: Some("li".to_string()),
            select_contains: Some("item".to_string()),
            select_ordinal: 1,
            ..Default::default()
        };

        let result = locate(&doc.blocks, &selector);
        assert!(result.is_ok());
        let (_found, is_ambiguous) = result.unwrap();

        assert!(
            is_ambiguous,
            "Expected ambiguity to be true when multiple list items match"
        );
    }

    #[test]
    fn test_scoped_after_heading_paragraph_selection() {
        let doc = parse_markdown(MarkdownParserState::default(), SCOPED_MARKDOWN).unwrap();
        let selector = Selector {
            select_type: Some("p".to_string()),
            select_ordinal: 1,
            after: Some(Box::new(Selector {
                select_type: Some("h2".to_string()),
                select_contains: Some("Installation".to_string()),
                select_ordinal: 1,
                ..Default::default()
            })),
            ..Default::default()
        };

        let result =
            locate(&doc.blocks, &selector).expect("Expected paragraph after Installation heading");
        let (found, is_ambiguous) = result;

        assert!(
            matches!(found, FoundNode::Block { index, .. } if index == 3),
            "Expected to find the first paragraph after the Installation heading"
        );
        assert!(
            is_ambiguous,
            "Multiple paragraphs exist after the Installation heading, ambiguity should be detected"
        );
    }

    #[test]
    fn test_scoped_within_heading_limits_search_space() {
        let doc = parse_markdown(MarkdownParserState::default(), SCOPED_MARKDOWN).unwrap();
        let selector = Selector {
            select_type: Some("li".to_string()),
            select_contains: Some("Task Beta".to_string()),
            within: Some(Box::new(Selector {
                select_type: Some("h2".to_string()),
                select_contains: Some("Future Features".to_string()),
                ..Default::default()
            })),
            ..Default::default()
        };

        let (found, is_ambiguous) = locate(&doc.blocks, &selector)
            .expect("Expected to find Task Beta within Future Features");

        if let FoundNode::ListItem {
            block_index,
            item_index,
            item,
        } = found
        {
            assert_eq!(
                block_index, 10,
                "Future Features list should be at block index 10"
            );
            assert_eq!(
                item_index, 1,
                "Task Beta should be the second item in the list"
            );
            assert!(
                list_item_to_text(item).contains("Task Beta"),
                "Selected item should contain Task Beta"
            );
            assert!(
                !is_ambiguous,
                "Only one item should match Task Beta within the scoped section"
            );
        } else {
            panic!("Expected to find a list item within Future Features");
        }
    }

    #[test]
    fn test_scoped_after_missing_landmark_errors() {
        let doc = parse_markdown(MarkdownParserState::default(), SCOPED_MARKDOWN).unwrap();
        let selector = Selector {
            select_type: Some("p".to_string()),
            select_ordinal: 1,
            after: Some(Box::new(Selector {
                select_type: Some("h2".to_string()),
                select_contains: Some("Does Not Exist".to_string()),
                ..Default::default()
            })),
            ..Default::default()
        };

        let result = locate(&doc.blocks, &selector);
        assert!(matches!(result, Err(SpliceError::NodeNotFound)));
    }

    #[test]
    fn test_scoped_within_missing_primary_errors() {
        let doc = parse_markdown(MarkdownParserState::default(), SCOPED_MARKDOWN).unwrap();
        let selector = Selector {
            select_type: Some("p".to_string()),
            select_contains: Some("Non-existent".to_string()),
            within: Some(Box::new(Selector {
                select_type: Some("h2".to_string()),
                select_contains: Some("Future Features".to_string()),
                ..Default::default()
            })),
            ..Default::default()
        };

        let result = locate(&doc.blocks, &selector);
        assert!(matches!(result, Err(SpliceError::NodeNotFound)));
    }

    #[test]
    fn test_scoped_within_invalid_target_errors() {
        let doc = parse_markdown(MarkdownParserState::default(), SCOPED_MARKDOWN).unwrap();
        let selector = Selector {
            select_type: Some("p".to_string()),
            select_ordinal: 1,
            within: Some(Box::new(Selector {
                select_type: Some("p".to_string()),
                select_contains: Some("Introduction paragraph.".to_string()),
                ..Default::default()
            })),
            ..Default::default()
        };

        let result = locate(&doc.blocks, &selector);
        assert!(matches!(result, Err(SpliceError::NodeNotFound)));
    }

    #[test]
    fn test_scoped_conflicting_modifiers_error() {
        let doc = parse_markdown(MarkdownParserState::default(), SCOPED_MARKDOWN).unwrap();
        let selector = Selector {
            select_type: Some("p".to_string()),
            select_ordinal: 1,
            after: Some(Box::new(Selector {
                select_type: Some("h2".to_string()),
                select_contains: Some("Installation".to_string()),
                ..Default::default()
            })),
            within: Some(Box::new(Selector {
                select_type: Some("h2".to_string()),
                select_contains: Some("Usage".to_string()),
                ..Default::default()
            })),
            ..Default::default()
        };

        let result = locate(&doc.blocks, &selector);
        assert!(matches!(
            result,
            Err(SpliceError::ConflictingScopeModifiers)
        ));
    }

    #[test]
    fn test_scoped_after_list_item_selects_following_item() {
        let doc = parse_markdown(MarkdownParserState::default(), SCOPED_MARKDOWN).unwrap();
        let selector = Selector {
            select_type: Some("li".to_string()),
            select_ordinal: 1,
            after: Some(Box::new(Selector {
                select_type: Some("li".to_string()),
                select_contains: Some("Step zero".to_string()),
                ..Default::default()
            })),
            ..Default::default()
        };

        let (found, _) =
            locate(&doc.blocks, &selector).expect("Expected to find list item after Step zero");

        if let FoundNode::ListItem {
            block_index,
            item_index,
            item,
        } = found
        {
            assert_eq!(
                block_index, 4,
                "Installation checklist should be at block index 4"
            );
            assert_eq!(
                item_index, 1,
                "First item after Step zero should be Step one"
            );
            assert!(
                list_item_to_text(item).contains("Step one"),
                "Expected to select the list item immediately after Step zero"
            );
        } else {
            panic!("Expected to find a list item after Step zero");
        }
    }
}
