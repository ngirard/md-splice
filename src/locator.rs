//! Contains the logic for finding a target node within the Markdown AST.

use crate::error::SpliceError;
use markdown_ppp::ast::{
    Block, FootnoteDefinition, Heading, HeadingKind, Inline, List, SetextHeading, Table,
};
use regex::Regex;

/// Represents the location of a found block.
#[derive(Debug, PartialEq)]
pub struct FoundBlock<'a> {
    pub index: usize,
    pub block: &'a Block,
}

/// A set of criteria for selecting a node.
#[derive(Debug, Default)]
pub struct Selector {
    pub select_type: Option<String>,
    pub select_contains: Option<String>,
    pub select_regex: Option<Regex>,
    pub select_ordinal: usize,
}

/// Finds the first block in the document that matches all the given selectors.
///
/// # Arguments
///
/// * `blocks`: A slice of `Block` nodes from a `Document`.
/// * `selector`: The selection criteria.
///
/// # Returns
///
/// A `Result` containing a `FoundBlock` on success, or a `SpliceError` if no
/// node is found or if there's an issue with the selectors.
pub fn locate<'a>(
    blocks: &'a [Block],
    selector: &Selector,
) -> Result<FoundBlock<'a>, SpliceError> {
    // The ordinal is 1-indexed, so we must subtract 1 for `nth`.
    // We also must ensure it's not zero to prevent underflow.
    let ordinal_index = selector.select_ordinal.saturating_sub(1);

    let mut matches = blocks.iter().enumerate().filter(|(_i, block)| {
        let type_match = selector
            .select_type
            .as_ref()
            .map_or(true, |t| block_type_matches(block, t));

        let contains_match = selector
            .select_contains
            .as_ref()
            .map_or(true, |text| block_to_text(block).contains(text));

        type_match && contains_match
    });

    matches
        .nth(ordinal_index)
        .map(|(index, block)| FoundBlock { index, block })
        .ok_or(SpliceError::NodeNotFound)
}

/// Checks if a block matches the string representation of its type.
fn block_type_matches(block: &Block, type_str: &str) -> bool {
    match type_str.to_lowercase().as_str() {
        "p" | "paragraph" => matches!(block, Block::Paragraph(_)),
        "h1" => matches!(block, Block::Heading(Heading { kind: HeadingKind::Atx(1) | HeadingKind::Setext(SetextHeading::Level1), .. })),
        "h2" => matches!(block, Block::Heading(Heading { kind: HeadingKind::Atx(2) | HeadingKind::Setext(SetextHeading::Level2), .. })),
        "h3" => matches!(block, Block::Heading(Heading { kind: HeadingKind::Atx(3), .. })),
        "h4" => matches!(block, Block::Heading(Heading { kind: HeadingKind::Atx(4), .. })),
        "h5" => matches!(block, Block::Heading(Heading { kind: HeadingKind::Atx(5), .. })),
        "h6" => matches!(block, Block::Heading(Heading { kind: HeadingKind::Atx(6), .. })),
        "heading" => matches!(block, Block::Heading(_)),
        "list" => matches!(block, Block::List(_)),
        "table" => matches!(block, Block::Table(_)),
        "blockquote" => matches!(block, Block::BlockQuote(_)),
        "code" | "codeblock" => matches!(block, Block::CodeBlock(_)),
        "html" | "htmlblock" => matches!(block, Block::HtmlBlock(_)),
        "thematicbreak" => matches!(block, Block::ThematicBreak),
        "definition" => matches!(block, Block::Definition(_)),
        "footnotedefinition" => matches!(block, Block::FootnoteDefinition(_)),
        "empty" => matches!(block, Block::Empty),
        _ => false,
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
fn block_to_text(block: &Block) -> String {
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
        // Per spec, these blocks have no user-facing text content
        Block::ThematicBreak
        | Block::HtmlBlock(_)
        | Block::Definition(_)
        | Block::Empty => String::new(),
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use markdown_ppp::parser::{parse_markdown, MarkdownParserState};

    const TEST_MARKDOWN: &str = r#"
# A Heading

This is the first paragraph.

- A list item
- Another list item

This is the second paragraph.

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
        let found = result.unwrap();

        // The first paragraph is the second block (index 1) after the H1.
        assert_eq!(
            found,
            FoundBlock {
                index: 1,
                block: &doc.blocks[1]
            }
        );
        assert!(matches!(found.block, Block::Paragraph(_)));
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
        let found = result.unwrap();

        // The markdown has: H1, P, List, P, CodeBlock.
        // So the second paragraph is at index 3.
        assert_eq!(
            found,
            FoundBlock {
                index: 3,
                block: &doc.blocks[3]
            }
        );
        assert!(matches!(found.block, Block::Paragraph(_)));
    }

    #[test]
    fn test_l3_select_heading_by_content_contains() {
        // L3 (Content Contains): Select a heading containing a specific word.
        let doc = parse_markdown(MarkdownParserState::default(), TEST_MARKDOWN).unwrap();
        let selector = Selector {
            select_contains: Some("Heading".to_string()),
            ..Default::default()
        };

        let result = locate(&doc.blocks, &selector);

        assert!(result.is_ok(), "locate should find a matching block");
        let found = result.unwrap();

        // The first block (index 0) is the H1 with content "A Heading".
        assert_eq!(
            found,
            FoundBlock {
                index: 0,
                block: &doc.blocks[0]
            }
        );
        assert!(matches!(found.block, Block::Heading(_)));
    }
}
