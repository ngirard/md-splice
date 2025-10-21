use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::process::Command;

fn fixture_document() -> &'static str {
    "---\nstatus: draft\ntitle: Sample\n---\n# Heading\n\nBody text.\n"
}

#[test]
fn get_top_level_key_from_yaml() {
    let file = assert_fs::NamedTempFile::new("doc.md").unwrap();
    file.write_str(fixture_document()).unwrap();

    let mut cmd = Command::cargo_bin("md-splice").unwrap();
    cmd.arg("--file")
        .arg(file.path())
        .arg("frontmatter")
        .arg("get")
        .arg("--key")
        .arg("status");

    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout, "draft\n");
}

#[test]
fn get_nested_key_with_array_index() {
    let file = assert_fs::NamedTempFile::new("nested.md").unwrap();
    file.write_str(
        "---\nmetadata:\n  reviewers:\n    - name: Alice\n      email: alice@example.com\n    - name: Bob\n      email: bob@example.com\n---\n# Doc\n",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("md-splice").unwrap();
    cmd.arg("--file")
        .arg(file.path())
        .arg("frontmatter")
        .arg("get")
        .arg("--key")
        .arg("metadata.reviewers[0].email");

    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout, "alice@example.com\n");
}

#[test]
fn get_entire_frontmatter_defaults_to_yaml_string() {
    let file = assert_fs::NamedTempFile::new("doc.md").unwrap();
    file.write_str(fixture_document()).unwrap();

    let mut cmd = Command::cargo_bin("md-splice").unwrap();
    cmd.arg("--file")
        .arg(file.path())
        .arg("frontmatter")
        .arg("get");

    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout, "status: draft\ntitle: Sample\n");
}

#[test]
fn get_value_as_json() {
    let file = assert_fs::NamedTempFile::new("doc.md").unwrap();
    file.write_str(fixture_document()).unwrap();

    let mut cmd = Command::cargo_bin("md-splice").unwrap();
    cmd.arg("--file")
        .arg(file.path())
        .arg("frontmatter")
        .arg("get")
        .arg("--key")
        .arg("title")
        .arg("--output-format")
        .arg("json");

    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout, "\"Sample\"\n");
}

#[test]
fn missing_key_produces_error() {
    let file = assert_fs::NamedTempFile::new("doc.md").unwrap();
    file.write_str(fixture_document()).unwrap();

    let mut cmd = Command::cargo_bin("md-splice").unwrap();
    cmd.arg("--file")
        .arg(file.path())
        .arg("frontmatter")
        .arg("get")
        .arg("--key")
        .arg("missing");

    cmd.assert().failure().stderr(predicate::str::contains(
        "Frontmatter key 'missing' was not found.",
    ));
}

#[test]
fn gracefully_handles_missing_frontmatter_when_no_key() {
    let file = assert_fs::NamedTempFile::new("no-frontmatter.md").unwrap();
    file.write_str("# No metadata\n").unwrap();

    let mut cmd = Command::cargo_bin("md-splice").unwrap();
    cmd.arg("--file")
        .arg(file.path())
        .arg("frontmatter")
        .arg("get");

    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(stdout.is_empty());
}

#[test]
fn missing_frontmatter_with_key_errors() {
    let file = assert_fs::NamedTempFile::new("no-frontmatter.md").unwrap();
    file.write_str("# No metadata\n").unwrap();

    let mut cmd = Command::cargo_bin("md-splice").unwrap();
    cmd.arg("--file")
        .arg(file.path())
        .arg("frontmatter")
        .arg("get")
        .arg("--key")
        .arg("status");

    cmd.assert().failure().stderr(predicate::str::contains(
        "No frontmatter exists in the document.",
    ));
}
