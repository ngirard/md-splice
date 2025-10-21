use md_splice_lib::transaction::{
    Operation, ReplaceOperation, Selector as TxSelector, SetFrontmatterOperation,
};
use md_splice_lib::MarkdownDocument;
use serde_yaml::Value as YamlValue;
use std::str::FromStr;

#[test]
fn load_document_from_string_and_render() {
    let content = "# Title\n\nHello, world.\n";
    let doc = MarkdownDocument::from_str(content).expect("document loads");
    let rendered = doc.render();
    assert_eq!(rendered.trim_end(), content.trim_end());
}

#[test]
fn apply_replace_operation_updates_body() {
    let mut doc =
        MarkdownDocument::from_str("# Tasks\n\nStatus: In Progress.\n").expect("document loads");

    let operations = vec![Operation::Replace(ReplaceOperation {
        selector: TxSelector {
            select_type: None,
            select_contains: Some("Status: In Progress.".to_string()),
            select_regex: None,
            select_ordinal: 1,
            after: None,
            within: None,
        },
        comment: None,
        content: Some("Status: Complete!\n".to_string()),
        content_file: None,
        until: None,
    })];

    doc.apply(operations).expect("apply succeeds");

    let rendered = doc.render();
    assert!(rendered.contains("Status: Complete!"));
    assert!(!rendered.contains("Status: In Progress."));
}

#[test]
fn apply_set_frontmatter_updates_metadata() {
    let initial = "---\nstatus: draft\n---\n\nHello\n";
    let mut doc = MarkdownDocument::from_str(initial).expect("document loads");

    let operations = vec![Operation::SetFrontmatter(SetFrontmatterOperation {
        key: "status".to_string(),
        comment: None,
        value: Some(YamlValue::String("published".to_string())),
        value_file: None,
        format: None,
    })];

    doc.apply(operations).expect("apply succeeds");

    let rendered = doc.render();
    assert!(rendered.contains("status: published"));
    assert!(!rendered.contains("status: draft"));
}
