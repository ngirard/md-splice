use crate::frontmatter::FrontmatterFormat;
use serde::Deserialize;
use serde_yaml::Value as YamlValue;
use std::path::PathBuf;

fn default_select_ordinal() -> usize {
    1
}

fn default_insert_position() -> InsertPosition {
    InsertPosition::After
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Operation {
    Insert(InsertOperation),
    Replace(ReplaceOperation),
    Delete(DeleteOperation),
    SetFrontmatter(SetFrontmatterOperation),
    DeleteFrontmatter(DeleteFrontmatterOperation),
    ReplaceFrontmatter(ReplaceFrontmatterOperation),
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Selector {
    #[serde(default)]
    pub select_type: Option<String>,
    #[serde(default)]
    pub select_contains: Option<String>,
    #[serde(default)]
    pub select_regex: Option<String>,
    #[serde(default = "default_select_ordinal")]
    pub select_ordinal: usize,
    #[serde(default)]
    pub after: Option<Box<Selector>>,
    #[serde(default)]
    pub within: Option<Box<Selector>>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct InsertOperation {
    pub selector: Selector,
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub content_file: Option<PathBuf>,
    #[serde(default = "default_insert_position")]
    pub position: InsertPosition,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct ReplaceOperation {
    pub selector: Selector,
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub content_file: Option<PathBuf>,
    #[serde(default)]
    pub until: Option<Selector>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct DeleteOperation {
    pub selector: Selector,
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(default)]
    pub section: bool,
    #[serde(default)]
    pub until: Option<Selector>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct SetFrontmatterOperation {
    pub key: String,
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(default)]
    pub value: Option<YamlValue>,
    #[serde(default)]
    pub value_file: Option<PathBuf>,
    #[serde(default)]
    pub format: Option<FrontmatterFormat>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct DeleteFrontmatterOperation {
    pub key: String,
    #[serde(default)]
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct ReplaceFrontmatterOperation {
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(default)]
    pub content: Option<YamlValue>,
    #[serde(default)]
    pub content_file: Option<PathBuf>,
    #[serde(default)]
    pub format: Option<FrontmatterFormat>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum InsertPosition {
    Before,
    After,
    PrependChild,
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
