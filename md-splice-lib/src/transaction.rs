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
    /// Optional alias assigned to this selector for later reuse.
    pub alias: Option<String>,
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
    /// Narrows the search to nodes appearing after a referenced selector alias.
    pub after_ref: Option<String>,
    #[serde(default)]
    /// Narrows the search to nodes contained within another selector's scope.
    pub within: Option<Box<Selector>>,
    #[serde(default)]
    /// Narrows the search to nodes contained within a referenced selector alias.
    pub within_ref: Option<String>,
}

impl Default for Selector {
    fn default() -> Self {
        Self {
            alias: None,
            select_type: None,
            select_contains: None,
            select_regex: None,
            select_ordinal: default_select_ordinal(),
            after: None,
            after_ref: None,
            within: None,
            within_ref: None,
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Clone, Default)]
/// Describes where and how new content should be inserted relative to a selector.
pub struct InsertOperation {
    #[serde(default)]
    /// The selector that identifies the insertion anchor.
    pub selector: Option<Selector>,
    #[serde(default)]
    /// Reference to a selector alias that identifies the insertion anchor.
    pub selector_ref: Option<String>,
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
    #[serde(default)]
    /// The selector that identifies the content to replace.
    pub selector: Option<Selector>,
    #[serde(default)]
    /// Reference to a selector alias identifying the content to replace.
    pub selector_ref: Option<String>,
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
    #[serde(default)]
    /// Reference to an alias delimiting the end of a multi-block replacement.
    pub until_ref: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Default)]
/// Describes deletion of content matched by a selector.
pub struct DeleteOperation {
    #[serde(default)]
    /// The selector identifying content to delete.
    pub selector: Option<Selector>,
    #[serde(default)]
    /// Reference to a selector alias identifying content to delete.
    pub selector_ref: Option<String>,
    #[serde(default)]
    /// Optional human-readable note recorded alongside the operation.
    pub comment: Option<String>,
    #[serde(default)]
    /// Deletes the entire section when targeting a heading selector.
    pub section: bool,
    #[serde(default)]
    /// Optional selector delimiting the end of a multi-block deletion.
    pub until: Option<Selector>,
    #[serde(default)]
    /// Reference to an alias delimiting the end of a multi-block deletion.
    pub until_ref: Option<String>,
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
    #[serde(alias = "prepend-child")]
    PrependChild,
    /// Insert as the last child of the selector node.
    #[serde(alias = "append-child")]
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
                let selector = op.selector.as_ref().expect("selector should be present");
                assert_eq!(
                    selector.select_contains.as_deref(),
                    Some("Status: In Progress")
                );
                assert_eq!(op.content.as_deref(), Some("Status: **Complete**"));
                assert!(op.content_file.is_none());
                assert!(selector.after.is_none());
                assert!(op.until.is_none());
            }
            other => panic!("expected replace operation, got {other:?}"),
        }

        match &operations[1] {
            Operation::Insert(op) => {
                let selector = op.selector.as_ref().expect("selector should be present");
                assert_eq!(selector.select_type.as_deref(), Some("li"));
                assert_eq!(
                    selector.select_contains.as_deref(),
                    Some("Write documentation")
                );
                assert_eq!(op.position, InsertPosition::Before);
                assert_eq!(op.content.as_deref(), Some("- [ ] Implement unit tests"));
                assert!(selector.after.is_none());
            }
            other => panic!("expected insert operation, got {other:?}"),
        }

        match &operations[2] {
            Operation::Delete(op) => {
                let selector = op.selector.as_ref().expect("selector should be present");
                assert_eq!(selector.select_type.as_deref(), Some("h2"));
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

        let selector = op.selector.as_ref().expect("selector should be present");
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

    #[test]
    fn deserialize_insert_position_hyphenated_aliases() {
        let data = r#"
        [
            {
                "op": "insert",
                "selector": {
                    "select_type": "li"
                },
                "position": "append-child",
                "content": "- nested"
            },
            {
                "op": "insert",
                "selector": {
                    "select_type": "li"
                },
                "position": "prepend-child",
                "content": "- nested"
            }
        ]
        "#;

        let operations: Vec<Operation> = serde_json::from_str(data).unwrap();

        match &operations[0] {
            Operation::Insert(op) => assert_eq!(op.position, InsertPosition::AppendChild),
            other => panic!("expected insert operation, got {other:?}"),
        }

        match &operations[1] {
            Operation::Insert(op) => assert_eq!(op.position, InsertPosition::PrependChild),
            other => panic!("expected insert operation, got {other:?}"),
        }
    }

    #[test]
    fn deserialize_operations_with_selector_alias_handles() {
        let data = r###"
        - op: replace
          selector:
            alias: intro_h2
            select_type: h2
            select_contains: Introduction
          content: "## Introduction"
        - op: replace
          selector:
            alias: changelog_h2
            select_type: h2
            select_contains: Changelog
            after_ref: intro_h2
          content: "## Changelog"
          until:
            alias: outro_h2
            select_type: h2
            select_contains: Outro
        - op: insert
          selector_ref: changelog_h2
          position: append_child
          content: "- Added entry"
        - op: delete
          selector:
            select_type: li
            select_contains: Legacy
            within_ref: changelog_h2
          until_ref: outro_h2
        "###;

        let operations: Vec<Operation> = serde_yaml::from_str(data).unwrap();
        assert_eq!(operations.len(), 4);

        let Operation::Replace(intro_replace) = &operations[0] else {
            panic!("expected replace operation for intro heading");
        };
        let intro_selector = intro_replace
            .selector
            .as_ref()
            .expect("inline selector must exist");
        assert_eq!(intro_selector.alias.as_deref(), Some("intro_h2"));
        assert_eq!(intro_selector.select_type.as_deref(), Some("h2"));
        assert_eq!(intro_selector.select_contains.as_deref(), Some("Introduction"));

        let Operation::Replace(changelog_replace) = &operations[1] else {
            panic!("expected replace operation for changelog heading");
        };
        let changelog_selector = changelog_replace
            .selector
            .as_ref()
            .expect("inline selector must exist");
        assert_eq!(changelog_selector.alias.as_deref(), Some("changelog_h2"));
        assert_eq!(changelog_selector.after_ref.as_deref(), Some("intro_h2"));
        let until_selector = changelog_replace
            .until
            .as_ref()
            .expect("inline until selector must exist");
        assert_eq!(until_selector.alias.as_deref(), Some("outro_h2"));

        let Operation::Insert(insert_using_ref) = &operations[2] else {
            panic!("expected insert operation using selector_ref");
        };
        assert!(insert_using_ref.selector.is_none());
        assert_eq!(insert_using_ref.selector_ref.as_deref(), Some("changelog_h2"));
        assert_eq!(insert_using_ref.position, InsertPosition::AppendChild);

        let Operation::Delete(delete_within_ref) = &operations[3] else {
            panic!("expected delete operation with within_ref");
        };
        let delete_selector = delete_within_ref
            .selector
            .as_ref()
            .expect("inline selector must exist");
        assert_eq!(delete_selector.within_ref.as_deref(), Some("changelog_h2"));
        assert_eq!(delete_within_ref.until_ref.as_deref(), Some("outro_h2"));
    }
}
