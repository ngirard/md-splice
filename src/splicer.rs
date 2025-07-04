//! Contains the logic for modifying the Markdown AST (inserting/replacing nodes).

use crate::{cli::InsertPosition, error::SpliceError};
use markdown_ppp::ast::{Block, FootnoteDefinition, Heading, HeadingKind, SetextHeading};

/// Replaces a block at a specific index with a new set of blocks.
///
/// # Arguments
///
/// * `doc_blocks`: The mutable vector of blocks from the document to modify.
/// * `index`: The index of the block to be replaced.
/// * `new_blocks`: A vector of blocks to insert in place of the old one.
pub fn replace(doc_blocks: &mut Vec<Block>, index: usize, new_blocks: Vec<Block>) {
    // Vec::splice is the perfect tool here. It can replace a range of elements
    // with a new iterator of elements. By specifying the range `index..=index`,
    // we are targeting the single block at the given index for replacement.
    doc_blocks.splice(index..=index, new_blocks);
}

/// Inserts new blocks into the document relative to a target block.
///
/// # Arguments
///
/// * `doc_blocks`: The mutable vector of blocks from the document to modify.
/// * `index`: The index of the target block.
/// * `new_blocks`: A vector of blocks to insert.
/// * `position`: Where to insert the new blocks relative to the target.
pub fn insert(
    doc_blocks: &mut Vec<Block>,
    index: usize,
    mut new_blocks: Vec<Block>,
    position: InsertPosition,
) -> anyhow::Result<()> {
    match position {
        InsertPosition::Before => {
            // `splice` with an empty range (e.g., `index..index`) inserts at that
            // position without removing any elements.
            doc_blocks.splice(index..index, new_blocks);
        }
        InsertPosition::After => {
            // To insert after the element at `index`, we specify the position `index + 1`.
            let insert_at = index + 1;
            doc_blocks.splice(insert_at..insert_at, new_blocks);
        }
        InsertPosition::PrependChild | InsertPosition::AppendChild => {
            // We match on an immutable reference first to avoid borrowing issues,
            // then get a mutable reference inside the specific match arms.
            match &doc_blocks[index] {
                Block::BlockQuote(_) | Block::FootnoteDefinition(_) => {
                    let target_block = &mut doc_blocks[index];
                    let inner_blocks = match target_block {
                        Block::BlockQuote(b) => b,
                        Block::FootnoteDefinition(fd) => &mut fd.blocks,
                        _ => unreachable!(), // We already matched the valid types.
                    };

                    if position == InsertPosition::PrependChild {
                        inner_blocks.splice(0..0, new_blocks);
                    } else {
                        // AppendChild
                        inner_blocks.append(&mut new_blocks);
                    }
                }
                Block::Heading(_) => {
                    // For headings, we operate on the main `doc_blocks` vector.
                    let target_level = get_heading_level(&doc_blocks[index]).unwrap(); // Safe to unwrap.

                    if position == InsertPosition::PrependChild {
                        // Insert immediately after the heading.
                        let insert_at = index + 1;
                        doc_blocks.splice(insert_at..insert_at, new_blocks);
                    } else {
                        // AppendChild: Find the end of the section and insert there.
                        let end_index = find_heading_section_end(doc_blocks, index, target_level);
                        doc_blocks.splice(end_index..end_index, new_blocks);
                    }
                }
                other_block => {
                    // All other block types are not considered containers for child insertion.
                    return Err(SpliceError::InvalidChildInsertion(
                        block_type_name(other_block).to_string(),
                    )
                    .into());
                }
            }
        }
    }
    Ok(())
}

/// Gets the level (1-6) of a heading block.
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

/// Finds the index that marks the end of a heading section.
/// The section ends just before the next heading of the same or higher level,
/// or at the end of the document.
fn find_heading_section_end(blocks: &[Block], start_index: usize, target_level: u8) -> usize {
    for i in (start_index + 1)..blocks.len() {
        if let Some(level) = get_heading_level(&blocks[i]) {
            if level <= target_level {
                return i; // Found the boundary.
            }
        }
    }
    blocks.len() // Reached the end of the document.
}

/// Gets a user-friendly name for a block type, used in error messages.
fn block_type_name(block: &Block) -> &'static str {
    match block {
        Block::Paragraph(_) => "Paragraph",
        Block::Heading(_) => "Heading",
        Block::ThematicBreak => "ThematicBreak",
        Block::BlockQuote(_) => "BlockQuote",
        Block::List(_) => "List",
        Block::CodeBlock(_) => "CodeBlock",
        Block::HtmlBlock(_) => "HtmlBlock",
        Block::Definition(_) => "Definition",
        Block::Table(_) => "Table",
        Block::FootnoteDefinition(_) => "FootnoteDefinition",
        Block::Empty => "Empty",
    }
}

#[cfg(test)]
mod tests {
    use super::insert;
    use crate::cli::InsertPosition;
    use crate::error::SpliceError;
    use crate::locator::{locate, Selector};
    use crate::splicer::replace;
    use markdown_ppp::ast::{Block, Document, Inline};
    use markdown_ppp::parser::{parse_markdown, MarkdownParserState};

    const TEST_MARKDOWN: &str = r#"# A Heading

This is the first paragraph.

This is the second paragraph.
"#;

    fn parse_str(text: &str) -> Document {
        parse_markdown(MarkdownParserState::default(), text).unwrap()
    }

    #[test]
    fn test_s1_replace_paragraph() {
        // --- Setup ---
        let mut doc = parse_str(TEST_MARKDOWN);
        let new_content_doc = parse_str("This is the REPLACED paragraph.\n\nIt has two lines now.");

        // Locate the second paragraph to get its index.
        let target_index = {
            let selector = Selector {
                select_type: Some("p".to_string()),
                select_ordinal: 2,
                ..Default::default()
            };
            // We scope the locate call so that `found_block` and its immutable borrow
            // are dropped before we try to mutably borrow `doc.blocks` again.
            let (found_block, _is_ambiguous) = locate(&doc.blocks, &selector).unwrap();
            found_block.index
        };

        // --- Action ---
        replace(&mut doc.blocks, target_index, new_content_doc.blocks);

        // --- Verification ---
        assert_eq!(doc.blocks.len(), 4); // H1, P1, P2(new), P3(new)
        assert!(matches!(&doc.blocks[0], Block::Heading(_)));
        assert!(matches!(&doc.blocks[1], Block::Paragraph(_)));

        // Check the first new paragraph
        let new_p1 = &doc.blocks[2];
        assert!(
            matches!(new_p1, Block::Paragraph(inlines) if matches!(&inlines[0], Inline::Text(t) if t == "This is the REPLACED paragraph."))
        );

        // Check the second new paragraph
        let new_p2 = &doc.blocks[3];
        assert!(
            matches!(new_p2, Block::Paragraph(inlines) if matches!(&inlines[0], Inline::Text(t) if t == "It has two lines now."))
        );
    }

    #[test]
    fn test_s2_insert_before_paragraph() {
        // --- Setup ---
        let mut doc = parse_str(TEST_MARKDOWN);
        let new_content_doc = parse_str("This is an INSERTED paragraph.");

        let target_index = {
            let selector = Selector {
                select_type: Some("p".to_string()),
                select_ordinal: 2,
                ..Default::default()
            };
            let (found_block, _is_ambiguous) = locate(&doc.blocks, &selector).unwrap();
            found_block.index
        };

        // --- Action ---
        insert(
            &mut doc.blocks,
            target_index,
            new_content_doc.blocks,
            InsertPosition::Before,
        )
        .unwrap();

        // --- Verification ---
        // Original: H1, P1, P2. Total 3 blocks.
        // After insert: H1, P1, P_new, P2. Total 4 blocks.
        assert_eq!(doc.blocks.len(), 4);

        // The new block should be at the target index (which was 2).
        let new_p = &doc.blocks[2];
        assert!(
            matches!(new_p, Block::Paragraph(inlines) if matches!(&inlines[0], Inline::Text(t) if t == "This is an INSERTED paragraph."))
        );

        // The original second paragraph should now be at index 3.
        let original_p2 = &doc.blocks[3];
        assert!(
            matches!(original_p2, Block::Paragraph(inlines) if matches!(&inlines[0], Inline::Text(t) if t == "This is the second paragraph."))
        );
    }

    #[test]
    fn test_s2_insert_after_paragraph() {
        // --- Setup ---
        let mut doc = parse_str(TEST_MARKDOWN);
        let new_content_doc = parse_str("This is an INSERTED paragraph.");

        let target_index = {
            let selector = Selector {
                select_type: Some("p".to_string()),
                select_ordinal: 2,
                ..Default::default()
            };
            let (found_block, _is_ambiguous) = locate(&doc.blocks, &selector).unwrap();
            found_block.index
        };

        // --- Action ---
        insert(
            &mut doc.blocks,
            target_index,
            new_content_doc.blocks,
            InsertPosition::After,
        )
        .unwrap();

        // --- Verification ---
        // Original: H1, P1, P2. Total 3 blocks.
        // After insert: H1, P1, P2, P_new. Total 4 blocks.
        assert_eq!(doc.blocks.len(), 4);

        // The original second paragraph should still be at index 2.
        let original_p2 = &doc.blocks[2];
        assert!(
            matches!(original_p2, Block::Paragraph(inlines) if matches!(&inlines[0], Inline::Text(t) if t == "This is the second paragraph."))
        );

        // The new block should be at index 3.
        let new_p = &doc.blocks[3];
        assert!(
            matches!(new_p, Block::Paragraph(inlines) if matches!(&inlines[0], Inline::Text(t) if t == "This is an INSERTED paragraph."))
        );
    }

    const BLOCKQUOTE_MARKDOWN: &str = r#"# A Title

> This is the original line in the blockquote.
>
> It has multiple paragraphs.

Another paragraph.
"#;

    #[test]
    fn test_s3_prepend_child_into_blockquote() {
        // --- Setup ---
        let mut doc = parse_str(BLOCKQUOTE_MARKDOWN);
        let new_content_doc = parse_str("A prepended line.");

        let target_index = {
            let selector = Selector {
                select_type: Some("blockquote".to_string()),
                ..Default::default()
            };
            let (found_block, _is_ambiguous) = locate(&doc.blocks, &selector).unwrap();
            found_block.index
        };

        // --- Action ---
        insert(
            &mut doc.blocks,
            target_index,
            new_content_doc.blocks,
            InsertPosition::PrependChild,
        )
        .unwrap();

        // --- Verification ---
        assert_eq!(doc.blocks.len(), 3); // H1, BlockQuote, P
        let blockquote = &doc.blocks[1];
        if let Block::BlockQuote(inner_blocks) = blockquote {
            assert_eq!(inner_blocks.len(), 3); // Prepended P, Original P1, Original P2
            let prepended_p = &inner_blocks[0];
            assert!(
                matches!(prepended_p, Block::Paragraph(inlines) if matches!(&inlines[0], Inline::Text(t) if t == "A prepended line."))
            );
            let original_p1 = &inner_blocks[1];
            assert!(
                matches!(original_p1, Block::Paragraph(inlines) if matches!(&inlines[0], Inline::Text(t) if t == "This is the original line in the blockquote."))
            );
        } else {
            panic!("Expected a BlockQuote at index 1");
        }
    }

    #[test]
    fn test_s3_append_child_into_blockquote() {
        // --- Setup ---
        let mut doc = parse_str(BLOCKQUOTE_MARKDOWN);
        let new_content_doc = parse_str("An appended line.");

        let target_index = {
            let selector = Selector {
                select_type: Some("blockquote".to_string()),
                ..Default::default()
            };
            let (found_block, _is_ambiguous) = locate(&doc.blocks, &selector).unwrap();
            found_block.index
        };

        // --- Action ---
        insert(
            &mut doc.blocks,
            target_index,
            new_content_doc.blocks,
            InsertPosition::AppendChild,
        )
        .unwrap();

        // --- Verification ---
        assert_eq!(doc.blocks.len(), 3); // H1, BlockQuote, P
        let blockquote = &doc.blocks[1];
        if let Block::BlockQuote(inner_blocks) = blockquote {
            assert_eq!(inner_blocks.len(), 3); // Original P1, Original P2, Appended P
            let appended_p = &inner_blocks[2];
            assert!(
                matches!(appended_p, Block::Paragraph(inlines) if matches!(&inlines[0], Inline::Text(t) if t == "An appended line."))
            );
            let original_p2 = &inner_blocks[1];
            assert!(
                matches!(original_p2, Block::Paragraph(inlines) if matches!(&inlines[0], Inline::Text(t) if t == "It has multiple paragraphs."))
            );
        } else {
            panic!("Expected a BlockQuote at index 1");
        }
    }

    const HEADING_SECTION_MARKDOWN: &str = r#"# Level 1 Title

Some content under Level 1.

## Level 2 Subtitle

Some content under Level 2.

### Level 3 Sub-subtitle

Some content under Level 3.

## Another Level 2

Content under the second Level 2.

# Another Level 1

Final content.
"#;

    #[test]
    fn test_s4_prepend_child_into_heading_section() {
        // --- Setup ---
        let mut doc = parse_str(HEADING_SECTION_MARKDOWN);
        let new_content_doc = parse_str("* A prepended paragraph.");

        let target_index = {
            let selector = Selector {
                select_type: Some("h2".to_string()),
                select_contains: Some("Level 2 Subtitle".to_string()),
                ..Default::default()
            };
            let (found_block, _is_ambiguous) = locate(&doc.blocks, &selector).unwrap();
            found_block.index
        };

        // --- Action ---
        insert(
            &mut doc.blocks,
            target_index,
            new_content_doc.blocks,
            InsertPosition::PrependChild,
        )
        .unwrap();

        // --- Verification ---
        // Original: H1, P, H2, P, H3, P, H2, P, H1, P. Total 10 blocks.
        // After insert: 11 blocks.
        assert_eq!(doc.blocks.len(), 11);

        // The target H2 is at index 2. The new block should be inserted at index 3.
        let new_block = &doc.blocks[3];
        assert!(
            matches!(new_block, Block::List(_)), // "* A prepended paragraph.*" parses as a list
            "The new block should be a List"
        );

        // The block that was originally at index 3 (the paragraph "Some content under Level 2.")
        // should now be at index 4.
        let original_p = &doc.blocks[4];
        assert!(
            matches!(original_p, Block::Paragraph(inlines) if matches!(&inlines[0], Inline::Text(t) if t.starts_with("Some content under Level 2."))),
            "The original paragraph should be shifted"
        );
    }

    #[test]
    fn test_s4_append_child_into_heading_section() {
        // --- Setup ---
        let mut doc = parse_str(HEADING_SECTION_MARKDOWN);
        let new_content_doc = parse_str("* An appended paragraph.");

        let target_index = {
            let selector = Selector {
                select_type: Some("h2".to_string()),
                select_contains: Some("Level 2 Subtitle".to_string()),
                ..Default::default()
            };
            let (found_block, _is_ambiguous) = locate(&doc.blocks, &selector).unwrap();
            found_block.index
        };

        // --- Action ---
        insert(
            &mut doc.blocks,
            target_index,
            new_content_doc.blocks,
            InsertPosition::AppendChild,
        )
        .unwrap();

        // --- Verification ---
        // Original: H1, P, H2(idx 2), P, H3, P, H2(idx 6), P, H1, P. Total 10 blocks.
        // The section for "## Level 2 Subtitle" ends just before "## Another Level 2".
        // The insertion point should be at index 6.
        // After insert: 11 blocks.
        assert_eq!(doc.blocks.len(), 11);

        // The new block should be at index 6.
        let new_block = &doc.blocks[6];
        assert!(
            matches!(new_block, Block::List(_)), // "* An appended paragraph.*" parses as a list
            "The new block should be a List"
        );

        // The block that was originally at index 6 ("## Another Level 2")
        // should now be at index 7.
        let original_h2 = &doc.blocks[7];
        assert!(
            matches!(original_h2, Block::Heading(h) if matches!(&h.content[0], Inline::Text(t) if t == "Another Level 2")),
            "The next H2 should be shifted"
        );
    }

    #[test]
    fn test_s5_invalid_child_insertion_on_paragraph() {
        // --- Setup ---
        let mut doc = parse_str(TEST_MARKDOWN);
        let new_content_doc = parse_str("This should fail.");

        let target_index = {
            let selector = Selector {
                select_type: Some("p".to_string()),
                select_ordinal: 1,
                ..Default::default()
            };
            let (found_block, _is_ambiguous) = locate(&doc.blocks, &selector).unwrap();
            found_block.index
        };

        // --- Action ---
        let result = insert(
            &mut doc.blocks,
            target_index,
            new_content_doc.blocks,
            InsertPosition::PrependChild,
        );

        // --- Verification ---
        assert!(result.is_err());
        let err = result.unwrap_err();
        // Use downcast_ref to check the specific error type from anyhow::Error
        let splice_error = err.downcast_ref::<SpliceError>();
        assert!(
            matches!(splice_error, Some(SpliceError::InvalidChildInsertion(type_name)) if type_name == "Paragraph"),
            "Expected InvalidChildInsertion error for Paragraph, but got: {:?}",
            splice_error
        );
    }
}
