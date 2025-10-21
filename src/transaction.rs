use serde::Deserialize;
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
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct InsertOperation {
    #[serde(flatten)]
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
    #[serde(flatten)]
    pub selector: Selector,
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub content_file: Option<PathBuf>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct DeleteOperation {
    #[serde(flatten)]
    pub selector: Selector,
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(default)]
    pub section: bool,
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
                "select_contains": "Status: In Progress",
                "content": "Status: **Complete**"
            },
            {
                "op": "insert",
                "select_type": "li",
                "select_contains": "Write documentation",
                "position": "before",
                "content": "- [ ] Implement unit tests"
            },
            {
                "op": "delete",
                "select_type": "h2",
                "select_contains": "Low Priority",
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
            }
            other => panic!("expected insert operation, got {other:?}"),
        }

        match &operations[2] {
            Operation::Delete(op) => {
                assert_eq!(op.selector.select_type.as_deref(), Some("h2"));
                assert!(op.section);
            }
            other => panic!("expected delete operation, got {other:?}"),
        }
    }
}
