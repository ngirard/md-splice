use crate::frontmatter::FrontmatterFormat;
use serde::Deserialize;
use serde_yaml::Value as YamlValue;
use std::path::PathBuf;

fn default_select_ordinal() -> usize {
    1
}

#[derive(Debug, Deserialize, PartialEq)]
/// A single atomic mutation that can be applied to a [`MarkdownDocument`](crate::MarkdownDocument).
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Operation {
    /// Insert content relative to a matched selector.
    Insert(InsertOperation),
    /// Replace the matched selector (optionally spanning until another selector).
    Replace(ReplaceOperation),
    /// Delete the matched selector (optionally spanning until another selector).
    Delete(DeleteOperation),
    /// Assign or update a value within document frontmatter.
    SetFrontmatter(SetFrontmatterOperation),
    /// Remove a key from document frontmatter.
    DeleteFrontmatter(DeleteFrontmatterOperation),
    /// Replace the entire frontmatter block.
    ReplaceFrontmatter(ReplaceFrontmatterOperation),
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
/// Criteria describing a node to match in the Markdown AST.
pub struct Selector {
    #[serde(default)]
    /// Restricts matches to nodes of a given HTML-like element type (e.g., `h2`).
    pub select_type: Option<String>,
    #[serde(default)]
    /// Restricts matches to nodes whose rendered text contains the provided substring.
    pub select_contains: Option<String>,
    #[serde(default)]
    /// Restricts matches to nodes whose rendered text satisfies the provided regex.
    pub select_regex: Option<String>,
    #[serde(default = "default_select_ordinal")]
    /// Selects the _n_th match (1-indexed) when multiple nodes satisfy the selector.
    pub select_ordinal: usize,
    #[serde(default)]
    /// Narrows the search to nodes appearing after another selector.
    pub after: Option<Box<Selector>>,
    #[serde(default)]
    /// Narrows the search to nodes contained within another selector's scope.
    pub within: Option<Box<Selector>>,
}

impl Default for Selector {
    fn default() -> Self {
        Self {
            select_type: None,
            select_contains: None,
            select_regex: None,
            select_ordinal: default_select_ordinal(),
            after: None,
            within: None,
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Clone, Default)]
/// Describes where and how new content should be inserted relative to a selector.
pub struct InsertOperation {
    /// The selector that identifies the insertion anchor.
    pub selector: Selector,
    #[serde(default)]
    /// Optional human-readable note recorded alongside the operation.
    pub comment: Option<String>,
    #[serde(default)]
    /// Inline Markdown content to insert.
    pub content: Option<String>,
    #[serde(default)]
    /// Path to a file whose contents should be inserted.
    pub content_file: Option<PathBuf>,
    #[serde(default)]
    /// Placement relative to the selector.
    pub position: InsertPosition,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Default)]
/// Describes a replacement of existing content matched by a selector.
pub struct ReplaceOperation {
    /// The selector that identifies the content to replace.
    pub selector: Selector,
    #[serde(default)]
    /// Optional human-readable note recorded alongside the operation.
    pub comment: Option<String>,
    #[serde(default)]
    /// Inline Markdown content that replaces the selection.
    pub content: Option<String>,
    #[serde(default)]
    /// Path to a file providing replacement Markdown content.
    pub content_file: Option<PathBuf>,
    #[serde(default)]
    /// Optional selector delimiting the end of a multi-block replacement.
    pub until: Option<Selector>,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Default)]
/// Describes deletion of content matched by a selector.
pub struct DeleteOperation {
    /// The selector identifying content to delete.
    pub selector: Selector,
    #[serde(default)]
    /// Optional human-readable note recorded alongside the operation.
    pub comment: Option<String>,
    #[serde(default)]
    /// Deletes the entire section when targeting a heading selector.
    pub section: bool,
    #[serde(default)]
    /// Optional selector delimiting the end of a multi-block deletion.
    pub until: Option<Selector>,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Default)]
/// Assigns a value to a frontmatter key path.
pub struct SetFrontmatterOperation {
    /// The YAML path to assign.
    pub key: String,
    #[serde(default)]
    /// Optional human-readable note recorded alongside the operation.
    pub comment: Option<String>,
    #[serde(default)]
    /// Inline YAML value to assign.
    pub value: Option<YamlValue>,
    #[serde(default)]
    /// Path to a file providing the YAML value to assign.
    pub value_file: Option<PathBuf>,
    #[serde(default)]
    /// Overrides the frontmatter serialization format when creating a new block.
    pub format: Option<FrontmatterFormat>,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Default)]
/// Removes a frontmatter key path.
pub struct DeleteFrontmatterOperation {
    /// The YAML path to remove.
    pub key: String,
    #[serde(default)]
    /// Optional human-readable note recorded alongside the operation.
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Default)]
/// Replaces the entire frontmatter block with new content.
pub struct ReplaceFrontmatterOperation {
    #[serde(default)]
    /// Optional human-readable note recorded alongside the operation.
    pub comment: Option<String>,
    #[serde(default)]
    /// Inline YAML content to use as the new frontmatter block.
    pub content: Option<YamlValue>,
    #[serde(default)]
    /// Path to a file providing replacement YAML content.
    pub content_file: Option<PathBuf>,
    #[serde(default)]
    /// Overrides the frontmatter serialization format when creating the block.
    pub format: Option<FrontmatterFormat>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy, Default)]
#[serde(rename_all = "snake_case")]
/// Specifies where to place newly inserted content relative to the selector.
pub enum InsertPosition {
    /// Insert before the selector node.
    Before,
    /// Insert after the selector node.
    #[default]
    After,
    /// Insert as the first child of the selector node.
    PrependChild,
    /// Insert as the last child of the selector node.
    AppendChild,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_operations_example() {
        let data = r#"
        [
            {
                "op": "replace",
                "selector": {
                    "select_contains": "Status: In Progress"
                },
                "content": "Status: **Complete**"
            },
            {
                "op": "insert",
                "selector": {
                    "select_type": "li",
                    "select_contains": "Write documentation"
                },
                "position": "before",
                "content": "- [ ] Implement unit tests"
            },
            {
                "op": "delete",
                "selector": {
                    "select_type": "h2",
                    "select_contains": "Low Priority"
                },
                "section": true
            }
        ]
        "#;

        let operations: Vec<Operation> = serde_json::from_str(data).unwrap();
        assert_eq!(operations.len(), 3);

        match &operations[0] {
            Operation::Replace(op) => {
                assert_eq!(
                    op.selector.select_contains.as_deref(),
                    Some("Status: In Progress")
                );
                assert_eq!(op.content.as_deref(), Some("Status: **Complete**"));
                assert!(op.content_file.is_none());
                assert!(op.selector.after.is_none());
                assert!(op.until.is_none());
            }
            other => panic!("expected replace operation, got {other:?}"),
        }

        match &operations[1] {
            Operation::Insert(op) => {
                assert_eq!(op.selector.select_type.as_deref(), Some("li"));
                assert_eq!(
                    op.selector.select_contains.as_deref(),
                    Some("Write documentation")
                );
                assert_eq!(op.position, InsertPosition::Before);
                assert_eq!(op.content.as_deref(), Some("- [ ] Implement unit tests"));
                assert!(op.selector.after.is_none());
            }
            other => panic!("expected insert operation, got {other:?}"),
        }

        match &operations[2] {
            Operation::Delete(op) => {
                assert_eq!(op.selector.select_type.as_deref(), Some("h2"));
                assert!(op.section);
                assert!(op.until.is_none());
            }
            other => panic!("expected delete operation, got {other:?}"),
        }
    }

    #[test]
    fn deserialize_nested_scoped_selectors() {
        let data = r#"
        [
            {
                "op": "delete",
                "selector": {
                    "select_type": "p",
                    "after": {
                        "select_type": "h2",
                        "select_contains": "Installation"
                    },
                    "within": {
                        "select_type": "h1",
                        "select_contains": "Guide"
                    }
                },
                "until": {
                    "select_type": "p",
                    "select_contains": "Next Steps"
                }
            }
        ]
        "#;

        let operations: Vec<Operation> = serde_yaml::from_str(data).unwrap();
        assert_eq!(operations.len(), 1);

        let Operation::Delete(op) = &operations[0] else {
            panic!("expected delete operation");
        };

        let selector = &op.selector;
        assert_eq!(selector.select_type.as_deref(), Some("p"));
        assert!(selector.select_contains.is_none());

        let after = selector
            .after
            .as_ref()
            .expect("after selector should be present");
        assert_eq!(after.select_type.as_deref(), Some("h2"));
        assert_eq!(after.select_contains.as_deref(), Some("Installation"));

        let within = selector
            .within
            .as_ref()
            .expect("within selector should be present");
        assert_eq!(within.select_type.as_deref(), Some("h1"));
        assert_eq!(within.select_contains.as_deref(), Some("Guide"));

        let until = op.until.as_ref().expect("until selector should be present");
        assert_eq!(until.select_type.as_deref(), Some("p"));
        assert_eq!(until.select_contains.as_deref(), Some("Next Steps"));
    }

    #[test]
    fn deserialize_frontmatter_operations() {
        let data = r#"
        - op: set_frontmatter
          key: status
          value: approved
        - op: delete_frontmatter
          key: legacy_id
        - op: replace_frontmatter
          format: toml
          content:
            title: "Spec"
            version: 2
        "#;

        let operations: Vec<Operation> = serde_yaml::from_str(data).unwrap();
        assert_eq!(operations.len(), 3);

        match &operations[0] {
            Operation::SetFrontmatter(op) => {
                assert_eq!(op.key, "status");
                assert_eq!(op.value, Some(YamlValue::String("approved".to_string())));
                assert!(op.value_file.is_none());
                assert!(op.format.is_none());
            }
            other => panic!("expected set_frontmatter operation, got {other:?}"),
        }

        match &operations[1] {
            Operation::DeleteFrontmatter(op) => {
                assert_eq!(op.key, "legacy_id");
            }
            other => panic!("expected delete_frontmatter operation, got {other:?}"),
        }

        match &operations[2] {
            Operation::ReplaceFrontmatter(op) => {
                assert_eq!(op.format, Some(FrontmatterFormat::Toml));
                let Some(content) = op.content.as_ref() else {
                    panic!("expected inline content value");
                };
                let mapping = content.as_mapping().expect("expected mapping value");
                assert_eq!(mapping.len(), 2);
            }
            other => panic!("expected replace_frontmatter operation, got {other:?}"),
        }
    }
}
