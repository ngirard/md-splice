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

#[test]
fn set_updates_existing_key_in_yaml() {
    let file = assert_fs::NamedTempFile::new("doc.md").unwrap();
    file.write_str(fixture_document()).unwrap();

    let mut cmd = Command::cargo_bin("md-splice").unwrap();
    cmd.arg("--file")
        .arg(file.path())
        .arg("frontmatter")
        .arg("set")
        .arg("--key")
        .arg("status")
        .arg("--value")
        .arg("published");

    cmd.assert().success();

    file.assert(predicate::str::contains("status: published"));
    file.assert(predicate::str::contains("# Heading"));
}

#[test]
fn set_creates_frontmatter_when_missing() {
    let file = assert_fs::NamedTempFile::new("new.md").unwrap();
    file.write_str("# Fresh document\n").unwrap();

    let mut cmd = Command::cargo_bin("md-splice").unwrap();
    cmd.arg("--file")
        .arg(file.path())
        .arg("frontmatter")
        .arg("set")
        .arg("--key")
        .arg("title")
        .arg("--value")
        .arg("Fresh document");

    cmd.assert().success();

    file.assert(predicate::str::starts_with("---"));
    file.assert(predicate::str::contains("title: Fresh document"));
}

#[test]
fn set_respects_requested_format_for_new_toml_frontmatter() {
    let file = assert_fs::NamedTempFile::new("toml.md").unwrap();
    file.write_str("# Fresh document\n").unwrap();

    let mut cmd = Command::cargo_bin("md-splice").unwrap();
    cmd.arg("--file")
        .arg(file.path())
        .arg("frontmatter")
        .arg("set")
        .arg("--key")
        .arg("title")
        .arg("--value")
        .arg("Fresh document")
        .arg("--format")
        .arg("toml");

    cmd.assert().success();

    file.assert(predicate::str::starts_with("+++"));
    file.assert(predicate::str::contains("title = \"Fresh document\""));
}

#[test]
fn set_reads_value_from_file() {
    let file = assert_fs::NamedTempFile::new("doc.md").unwrap();
    file.write_str(fixture_document()).unwrap();

    let value_file = assert_fs::NamedTempFile::new("value.yaml").unwrap();
    value_file
        .write_str("reviewers:\n  - name: Dana\n")
        .unwrap();

    let mut cmd = Command::cargo_bin("md-splice").unwrap();
    cmd.arg("--file")
        .arg(file.path())
        .arg("frontmatter")
        .arg("set")
        .arg("--key")
        .arg("metadata")
        .arg("--value-file")
        .arg(value_file.path());

    cmd.assert().success();

    file.assert(predicate::str::contains(
        "metadata:\n  reviewers:\n  - name: Dana",
    ));
}

#[test]
fn delete_removes_key_and_frontmatter_block_when_empty() {
    let file = assert_fs::NamedTempFile::new("doc.md").unwrap();
    file.write_str("---\nstatus: draft\n---\n# Heading\n")
        .unwrap();

    let mut cmd = Command::cargo_bin("md-splice").unwrap();
    cmd.arg("--file")
        .arg(file.path())
        .arg("frontmatter")
        .arg("delete")
        .arg("--key")
        .arg("status");

    cmd.assert().success();

    file.assert(predicate::str::starts_with("# Heading"));
}

#[test]
fn delete_missing_key_reports_error() {
    let file = assert_fs::NamedTempFile::new("doc.md").unwrap();
    file.write_str(fixture_document()).unwrap();

    let mut cmd = Command::cargo_bin("md-splice").unwrap();
    cmd.arg("--file")
        .arg(file.path())
        .arg("frontmatter")
        .arg("delete")
        .arg("--key")
        .arg("missing");

    cmd.assert().failure().stderr(predicate::str::contains(
        "Frontmatter key 'missing' was not found.",
    ));
}
