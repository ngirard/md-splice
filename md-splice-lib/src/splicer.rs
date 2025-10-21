//! Contains the logic for modifying the Markdown AST (inserting/replacing nodes).

use crate::{error::SpliceError, transaction::InsertPosition};
use markdown_ppp::ast::{Block, Heading, HeadingKind, ListItem, SetextHeading};

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

/// Deletes a block at the specified index from the document.
pub fn delete(doc_blocks: &mut Vec<Block>, index: usize) {
    doc_blocks.remove(index);
}

/// Extracts a vector of `ListItem`s from a vector of `Block`s.
///
/// This function expects the input blocks to represent a single list. It will fail
/// if the blocks contain anything other than one `Block::List` (ignoring empty blocks).
fn extract_list_items_from_blocks(mut blocks: Vec<Block>) -> Result<Vec<ListItem>, SpliceError> {
    // The parser might produce empty blocks around the list. We filter them out.
    blocks.retain(|b| !matches!(b, Block::Empty));

    if blocks.len() == 1 {
        if let Some(Block::List(list)) = blocks.into_iter().next() {
            return Ok(list.items);
        }
    }
    Err(SpliceError::InvalidListItemContent)
}

/// Replaces a list item at a specific index with one or more new list items.
pub(crate) fn replace_list_item(
    doc_blocks: &mut [Block],
    block_index: usize,
    item_index: usize,
    new_blocks: Vec<Block>,
) -> anyhow::Result<()> {
    let new_items = extract_list_items_from_blocks(new_blocks)?;

    if let Some(Block::List(list)) = doc_blocks.get_mut(block_index) {
        // Ensure the target item exists before splicing.
        if item_index < list.items.len() {
            list.items.splice(item_index..=item_index, new_items);
            Ok(())
        } else {
            anyhow::bail!(
                "Internal error: item index {} is out of bounds for list with {} items",
                item_index,
                list.items.len()
            )
        }
    } else {
        anyhow::bail!(
            "Internal error: block at index {} is not a list",
            block_index
        )
    }
}

/// Inserts new content relative to a target list item.
pub(crate) fn insert_list_item(
    doc_blocks: &mut [Block],
    block_index: usize,
    item_index: usize,
    mut new_blocks: Vec<Block>,
    position: InsertPosition,
) -> anyhow::Result<()> {
    match position {
        InsertPosition::Before | InsertPosition::After => {
            let new_items = extract_list_items_from_blocks(new_blocks)?;
            if let Some(Block::List(list)) = doc_blocks.get_mut(block_index) {
                let insert_at = if position == InsertPosition::Before {
                    item_index
                } else {
                    item_index + 1
                };
                list.items.splice(insert_at..insert_at, new_items);
            } else {
                anyhow::bail!(
                    "Internal error: block at index {} is not a list",
                    block_index
                )
            }
        }
        InsertPosition::PrependChild | InsertPosition::AppendChild => {
            if let Some(Block::List(list)) = doc_blocks.get_mut(block_index) {
                if let Some(item) = list.items.get_mut(item_index) {
                    if position == InsertPosition::PrependChild {
                        item.blocks.splice(0..0, new_blocks);
                    } else {
                        // AppendChild
                        item.blocks.append(&mut new_blocks);
                    }
                } else {
                    anyhow::bail!(
                        "Internal error: item at index {} not found in list",
                        item_index
                    )
                }
            } else {
                anyhow::bail!(
                    "Internal error: block at index {} is not a list",
                    block_index
                )
            }
        }
    }
    Ok(())
}

/// Deletes a list item and reports whether the parent list became empty.
pub(crate) fn delete_list_item(
    doc_blocks: &mut [Block],
    block_index: usize,
    item_index: usize,
) -> anyhow::Result<bool> {
    if let Some(Block::List(list)) = doc_blocks.get_mut(block_index) {
        if item_index < list.items.len() {
            list.items.remove(item_index);
            Ok(list.items.is_empty())
        } else {
            anyhow::bail!(
                "Internal error: item index {} is out of bounds for list with {} items",
                item_index,
                list.items.len()
            );
        }
    } else {
        anyhow::bail!(
            "Internal error: block at index {} is not a list",
            block_index
        );
    }
}

/// Deletes a heading and all blocks in its section.
pub fn delete_section(doc_blocks: &mut Vec<Block>, start_index: usize) {
    if let Some(level) = get_heading_level(&doc_blocks[start_index]) {
        let end_index = find_heading_section_end(doc_blocks, start_index, level);
        doc_blocks.drain(start_index..end_index);
    }
}

/// Gets the level (1-6) of a heading block.
pub(crate) fn get_heading_level(block: &Block) -> Option<u8> {
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
pub(crate) fn find_heading_section_end(
    blocks: &[Block],
    start_index: usize,
    target_level: u8,
) -> usize {
    // We skip to the block after the starting one and find the first block
    // that meets the end-of-section criteria.
    for (i, block) in blocks.iter().enumerate().skip(start_index + 1) {
        if let Some(level) = get_heading_level(block) {
            if level <= target_level {
                return i; // Found the boundary, return its index.
            }
        }
    }
    blocks.len() // Reached the end of the document, return the length as the end index.
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
        Block::GitHubAlert(_) => "GitHubAlert",
        Block::Empty => "Empty",
    }
}

#[cfg(test)]
mod tests {
    use super::insert;
    use crate::error::SpliceError;
    use crate::locator::{list_item_to_text, locate, FoundNode, Selector};
    use crate::splicer::{insert_list_item, replace, replace_list_item};
    use crate::transaction::InsertPosition;
    use markdown_ppp::ast::{Block, Document, Inline};
    use markdown_ppp::parser::{parse_markdown, MarkdownParserState};

    const TEST_MARKDOWN: &str = r#"# A Heading

This is the first paragraph.

This is the second paragraph.
"#;

    fn parse_str(text: &str) -> Document {
        parse_markdown(MarkdownParserState::default(), text).unwrap()
    }

    /// Helper to extract the block index from a FoundNode, panicking if it's not a block.
    fn get_block_index(found_node: FoundNode) -> usize {
        if let FoundNode::Block { index, .. } = found_node {
            index
        } else {
            panic!("Test setup error: Expected to find a Block node");
        }
    }

    #[test]
    fn test_s1_replace_paragraph() {
        // --- Setup ---
        let mut doc = parse_str(TEST_MARKDOWN);
        let new_content_doc = parse_str("This is the REPLACED paragraph.\n\nIt has two lines now.");

        let target_index = {
            let selector = Selector {
                select_type: Some("p".to_string()),
                select_ordinal: 2,
                ..Default::default()
            };
            let (found_node, _is_ambiguous) = locate(&doc.blocks, &selector).unwrap();
            get_block_index(found_node)
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
            let (found_node, _is_ambiguous) = locate(&doc.blocks, &selector).unwrap();
            get_block_index(found_node)
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
        assert_eq!(doc.blocks.len(), 4);
        let new_p = &doc.blocks[2];
        assert!(
            matches!(new_p, Block::Paragraph(inlines) if matches!(&inlines[0], Inline::Text(t) if t == "This is an INSERTED paragraph."))
        );
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
            let (found_node, _is_ambiguous) = locate(&doc.blocks, &selector).unwrap();
            get_block_index(found_node)
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
        assert_eq!(doc.blocks.len(), 4);
        let original_p2 = &doc.blocks[2];
        assert!(
            matches!(original_p2, Block::Paragraph(inlines) if matches!(&inlines[0], Inline::Text(t) if t == "This is the second paragraph."))
        );
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
            let (found_node, _is_ambiguous) = locate(&doc.blocks, &selector).unwrap();
            get_block_index(found_node)
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
        assert_eq!(doc.blocks.len(), 3);
        let blockquote = &doc.blocks[1];
        if let Block::BlockQuote(inner_blocks) = blockquote {
            assert_eq!(inner_blocks.len(), 3);
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
            let (found_node, _is_ambiguous) = locate(&doc.blocks, &selector).unwrap();
            get_block_index(found_node)
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
        assert_eq!(doc.blocks.len(), 3);
        let blockquote = &doc.blocks[1];
        if let Block::BlockQuote(inner_blocks) = blockquote {
            assert_eq!(inner_blocks.len(), 3);
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
            let (found_node, _is_ambiguous) = locate(&doc.blocks, &selector).unwrap();
            get_block_index(found_node)
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
        assert_eq!(doc.blocks.len(), 11);
        let new_block = &doc.blocks[3];
        assert!(
            matches!(new_block, Block::List(_)),
            "The new block should be a List"
        );
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
            let (found_node, _is_ambiguous) = locate(&doc.blocks, &selector).unwrap();
            get_block_index(found_node)
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
        assert_eq!(doc.blocks.len(), 11);
        let new_block = &doc.blocks[6];
        assert!(
            matches!(new_block, Block::List(_)),
            "The new block should be a List"
        );
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
            let (found_node, _is_ambiguous) = locate(&doc.blocks, &selector).unwrap();
            get_block_index(found_node)
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
        let splice_error = err.downcast_ref::<SpliceError>();
        assert!(
            matches!(splice_error, Some(SpliceError::InvalidChildInsertion(type_name)) if type_name == "Paragraph"),
            "Expected InvalidChildInsertion error for Paragraph, but got: {:?}",
            splice_error
        );
    }

    // --- Tests for Phase 4.2: List Item Splicing ---

    const LIST_ITEM_MARKDOWN: &str = r#"# List Document

- First item
- Second item
- Third item with

  a nested paragraph.

1. Ordered item 1
2. Ordered item 2
"#;

    /// Helper to extract list item indices from a FoundNode.
    fn get_list_item_indices(found_node: FoundNode) -> (usize, usize) {
        if let FoundNode::ListItem {
            block_index,
            item_index,
            ..
        } = found_node
        {
            (block_index, item_index)
        } else {
            panic!("Test setup error: Expected to find a ListItem node");
        }
    }

    #[test]
    fn test_ls1_replace_list_item() {
        // --- Setup ---
        let mut doc = parse_str(LIST_ITEM_MARKDOWN);
        // The new content must be a valid list from which we can extract items.
        let new_content_doc = parse_str("- Replaced item");

        let (block_index, item_index) = {
            let selector = Selector {
                select_type: Some("li".to_string()),
                select_ordinal: 2, // "Second item"
                ..Default::default()
            };
            let (found_node, _is_ambiguous) = locate(&doc.blocks, &selector).unwrap();
            get_list_item_indices(found_node)
        };

        // --- Action ---
        replace_list_item(
            &mut doc.blocks,
            block_index,
            item_index,
            new_content_doc.blocks,
        )
        .unwrap();

        // --- Verification ---
        let list_block = &doc.blocks[1];
        if let Block::List(list) = list_block {
            assert_eq!(list.items.len(), 3);
            let replaced_item = &list.items[1];
            let text = list_item_to_text(replaced_item);
            assert_eq!(text, "Replaced item");
        } else {
            panic!("Expected a list block");
        }
    }

    #[test]
    fn test_ls4_replace_one_list_item_with_many() {
        // --- Setup ---
        let mut doc = parse_str(LIST_ITEM_MARKDOWN);
        let new_content_doc = parse_str("- Replaced item 1\n- Replaced item 2");

        let (block_index, item_index) = {
            let selector = Selector {
                select_type: Some("li".to_string()),
                select_ordinal: 2, // "Second item"
                ..Default::default()
            };
            let (found_node, _is_ambiguous) = locate(&doc.blocks, &selector).unwrap();
            get_list_item_indices(found_node)
        };

        // --- Action ---
        replace_list_item(
            &mut doc.blocks,
            block_index,
            item_index,
            new_content_doc.blocks,
        )
        .unwrap();

        // --- Verification ---
        let list_block = &doc.blocks[1];
        if let Block::List(list) = list_block {
            assert_eq!(list.items.len(), 4); // 3 (original) - 1 (replaced) + 2 (new) = 4
            let text1 = list_item_to_text(&list.items[1]);
            assert_eq!(text1, "Replaced item 1");
            let text2 = list_item_to_text(&list.items[2]);
            assert_eq!(text2, "Replaced item 2");
            let text3 = list_item_to_text(&list.items[3]);
            assert!(text3.starts_with("Third item"));
        } else {
            panic!("Expected a list block");
        }
    }

    #[test]
    fn test_ls2_insert_list_item_before() {
        // --- Setup ---
        let mut doc = parse_str(LIST_ITEM_MARKDOWN);
        let new_content_doc = parse_str("- Inserted item");

        let (block_index, item_index) = {
            let selector = Selector {
                select_type: Some("li".to_string()),
                select_ordinal: 2, // "Second item"
                ..Default::default()
            };
            let (found_node, _is_ambiguous) = locate(&doc.blocks, &selector).unwrap();
            get_list_item_indices(found_node)
        };

        // --- Action ---
        insert_list_item(
            &mut doc.blocks,
            block_index,
            item_index,
            new_content_doc.blocks,
            InsertPosition::Before,
        )
        .unwrap();

        // --- Verification ---
        let list_block = &doc.blocks[1];
        if let Block::List(list) = list_block {
            assert_eq!(list.items.len(), 4);
            let text = list_item_to_text(&list.items[1]);
            assert_eq!(text, "Inserted item");
            let text_orig = list_item_to_text(&list.items[2]);
            assert_eq!(text_orig, "Second item");
        } else {
            panic!("Expected a list block");
        }
    }

    #[test]
    fn test_ls2_insert_list_item_after() {
        // --- Setup ---
        let mut doc = parse_str(LIST_ITEM_MARKDOWN);
        let new_content_doc = parse_str("- Inserted item");

        let (block_index, item_index) = {
            let selector = Selector {
                select_type: Some("li".to_string()),
                select_ordinal: 2, // "Second item"
                ..Default::default()
            };
            let (found_node, _is_ambiguous) = locate(&doc.blocks, &selector).unwrap();
            get_list_item_indices(found_node)
        };

        // --- Action ---
        insert_list_item(
            &mut doc.blocks,
            block_index,
            item_index,
            new_content_doc.blocks,
            InsertPosition::After,
        )
        .unwrap();

        // --- Verification ---
        let list_block = &doc.blocks[1];
        if let Block::List(list) = list_block {
            assert_eq!(list.items.len(), 4);
            let text_orig = list_item_to_text(&list.items[1]);
            assert_eq!(text_orig, "Second item");
            let text = list_item_to_text(&list.items[2]);
            assert_eq!(text, "Inserted item");
        } else {
            panic!("Expected a list block");
        }
    }

    #[test]
    fn test_ls3_append_child_to_list_item() {
        // --- Setup ---
        let mut doc = parse_str(LIST_ITEM_MARKDOWN);
        // This content is just a block, not a list item.
        let new_content_doc = parse_str("* A nested list item");

        let (block_index, item_index) = {
            let selector = Selector {
                select_type: Some("li".to_string()),
                select_ordinal: 1, // "First item"
                ..Default::default()
            };
            let (found_node, _is_ambiguous) = locate(&doc.blocks, &selector).unwrap();
            get_list_item_indices(found_node)
        };

        // --- Action ---
        insert_list_item(
            &mut doc.blocks,
            block_index,
            item_index,
            new_content_doc.blocks,
            InsertPosition::AppendChild,
        )
        .unwrap();

        // --- Verification ---
        let list_block = &doc.blocks[1];
        if let Block::List(list) = list_block {
            assert_eq!(list.items.len(), 3);
            let modified_item = &list.items[0];
            // The item should now have 2 blocks: the original paragraph and the new list.
            assert_eq!(modified_item.blocks.len(), 2);
            assert!(matches!(modified_item.blocks[0], Block::Paragraph(_)));
            assert!(matches!(modified_item.blocks[1], Block::List(_)));
            let text = list_item_to_text(modified_item);
            assert_eq!(text, "First item\nA nested list item");
        } else {
            panic!("Expected a list block");
        }
    }

    #[test]
    fn test_ls3_prepend_child_to_list_item() {
        // --- Setup ---
        let mut doc = parse_str(LIST_ITEM_MARKDOWN);
        let new_content_doc = parse_str("A prepended paragraph.");

        let (block_index, item_index) = {
            let selector = Selector {
                select_type: Some("li".to_string()),
                select_ordinal: 1, // "First item"
                ..Default::default()
            };
            let (found_node, _is_ambiguous) = locate(&doc.blocks, &selector).unwrap();
            get_list_item_indices(found_node)
        };

        // --- Action ---
        insert_list_item(
            &mut doc.blocks,
            block_index,
            item_index,
            new_content_doc.blocks,
            InsertPosition::PrependChild,
        )
        .unwrap();

        // --- Verification ---
        let list_block = &doc.blocks[1];
        if let Block::List(list) = list_block {
            assert_eq!(list.items.len(), 3);
            let modified_item = &list.items[0];
            assert_eq!(modified_item.blocks.len(), 2);
            assert!(matches!(modified_item.blocks[0], Block::Paragraph(_)));
            assert!(matches!(modified_item.blocks[1], Block::Paragraph(_)));
            let text = list_item_to_text(modified_item);
            assert_eq!(text, "A prepended paragraph.\nFirst item");
        } else {
            panic!("Expected a list block");
        }
    }

    #[test]
    fn test_error_on_replace_list_item_with_non_list_content() {
        // --- Setup ---
        let mut doc = parse_str(LIST_ITEM_MARKDOWN);
        // This content is a paragraph, not a list.
        let new_content_doc = parse_str("This is not a list.");

        let (block_index, item_index) = {
            let selector = Selector {
                select_type: Some("li".to_string()),
                select_ordinal: 2,
                ..Default::default()
            };
            let (found_node, _is_ambiguous) = locate(&doc.blocks, &selector).unwrap();
            get_list_item_indices(found_node)
        };

        // --- Action ---
        let result = replace_list_item(
            &mut doc.blocks,
            block_index,
            item_index,
            new_content_doc.blocks,
        );

        // --- Verification ---
        assert!(result.is_err());
        let err = result.unwrap_err();
        let splice_error = err.downcast_ref::<SpliceError>();
        assert!(
            matches!(splice_error, Some(SpliceError::InvalidListItemContent)),
            "Expected InvalidListItemContent error, but got: {:?}",
            splice_error
        );
    }
}
