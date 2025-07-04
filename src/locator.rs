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
/// A `Result` containing a tuple of `(FoundBlock, bool)` on success, where the
/// boolean is `true` if more than one node matched the criteria (indicating ambiguity).
/// Returns a `SpliceError` if no node is found.
pub fn locate<'a>(
    blocks: &'a [Block],
    selector: &Selector,
) -> Result<(FoundBlock<'a>, bool), SpliceError> {
    // The ordinal is 1-indexed, so we must subtract 1 for indexing.
    // We also must ensure it's not zero to prevent underflow.
    let ordinal_index = selector.select_ordinal.saturating_sub(1);

    let matches: Vec<_> = blocks
        .iter()
        .enumerate()
        .filter(|(_i, block)| {
            // First, check the type selector. This is fast.
            let type_match = selector
                .select_type
                .as_ref()
                .map_or(true, |t| block_type_matches(block, t));

            if !type_match {
                return false;
            }

            // If there are no text-based selectors, we have a match.
            if selector.select_contains.is_none() && selector.select_regex.is_none() {
                return true;
            }

            // Only now, if we have text selectors, do we compute the text content.
            let text_content = block_to_text(block);

            let contains_match = selector
                .select_contains
                .as_ref()
                .map_or(true, |text| text_content.contains(text));

            let regex_match = selector
                .select_regex
                .as_ref()
                .map_or(true, |re| re.is_match(&text_content));

            // The final result is the AND of the text-based checks.
            contains_match && regex_match
        })
        .collect();

    let is_ambiguous = matches.len() > 1;

    matches
        .get(ordinal_index)
        .map(|(index, block)| {
            (
                FoundBlock {
                    index: *index,
                    block,
                },
                is_ambiguous,
            )
        })
        .ok_or(SpliceError::NodeNotFound)
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
        assert_eq!(
            found,
            FoundBlock {
                index: 1,
                block: &doc.blocks[1]
            }
        );
        assert!(matches!(found.block, Block::Paragraph(_)));
        assert!(is_ambiguous, "should detect ambiguity as there are two paragraphs");
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
        assert_eq!(
            found,
            FoundBlock {
                index: 3,
                block: &doc.blocks[3]
            }
        );
        assert!(matches!(found.block, Block::Paragraph(_)));
        assert!(is_ambiguous, "should detect ambiguity as there are two paragraphs");
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
        assert_eq!(
            found,
            FoundBlock {
                index: 0,
                block: &doc.blocks[0]
            }
        );
        assert!(matches!(found.block, Block::Heading(_)));
        assert!(!is_ambiguous, "should not detect ambiguity as there is only one heading");
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
        assert_eq!(
            found,
            FoundBlock {
                index: 4,
                block: &doc.blocks[4]
            }
        );
        assert!(matches!(found.block, Block::CodeBlock(_)));
        assert!(!is_ambiguous, "should not detect ambiguity as there is only one code block");
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

        // Blocks are: H1, P, P, P, P.
        // Paragraphs with "Note" are at indices 2 and 4.
        // The second one is at index 4.
        assert_eq!(
            found,
            FoundBlock {
                index: 4,
                block: &doc.blocks[4]
            }
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
        assert_eq!(found.index, 2);
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
        assert_eq!(found.index, 0);
        assert!(
            !is_ambiguous,
            "Expected ambiguity to be false when only one node matches"
        );
    }
}
