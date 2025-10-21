use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use insta::assert_snapshot;
use predicates::str::contains;
use std::process::Command;

#[test]
fn delete_help_lists_expected_flags() {
    let mut cmd = Command::cargo_bin("md-splice").unwrap();
    cmd.arg("delete")
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("--select-type"))
        .stdout(contains("--select-contains"))
        .stdout(contains("--select-regex"))
        .stdout(contains("--select-ordinal"))
        .stdout(contains("--section"));
}

#[test]
fn delete_list_item() {
    let file = assert_fs::NamedTempFile::new("test.md").unwrap();
    file.write_str("- one\n- two\n- three\n").unwrap();

    let mut cmd = Command::cargo_bin("md-splice").unwrap();
    cmd.arg("--file")
        .arg(file.path())
        .arg("delete")
        .arg("--select-type")
        .arg("li")
        .arg("--select-ordinal")
        .arg("2");

    cmd.assert().success();

    let result = std::fs::read_to_string(file.path()).unwrap();
    assert_snapshot!(result, @r###"- one
- three
"###);
}

#[test]
fn delete_last_list_item_removes_list() {
    let file = assert_fs::NamedTempFile::new("test.md").unwrap();
    file.write_str("# Title\n\n- The only item\n\nAnother paragraph.\n")
        .unwrap();

    let mut cmd = Command::cargo_bin("md-splice").unwrap();
    cmd.arg("--file")
        .arg(file.path())
        .arg("delete")
        .arg("--select-type")
        .arg("li");

    cmd.assert().success();

    let result = std::fs::read_to_string(file.path()).unwrap();
    assert_snapshot!(result, @r###"# Title

Another paragraph.
"###);
}

#[test]
fn delete_heading_with_section_flag() {
    let file = assert_fs::NamedTempFile::new("test.md").unwrap();
    let content = "# Title\n\n## Section to Delete\n\n- item 1\n- item 2\n\n### A subsection\n\n## Next Section\n\nThis should remain.\n";
    file.write_str(content).unwrap();

    let mut cmd = Command::cargo_bin("md-splice").unwrap();
    cmd.arg("--file")
        .arg(file.path())
        .arg("delete")
        .arg("--select-type")
        .arg("h2")
        .arg("--select-contains")
        .arg("Section to Delete")
        .arg("--section");

    cmd.assert().success();

    let result = std::fs::read_to_string(file.path()).unwrap();
    assert_snapshot!(result, @r###"# Title

## Next Section

This should remain.
"###);
}

#[test]
fn delete_with_section_flag_on_non_heading_fails() {
    let file = assert_fs::NamedTempFile::new("test.md").unwrap();
    file.write_str("A paragraph.\n").unwrap();

    let mut cmd = Command::cargo_bin("md-splice").unwrap();
    cmd.arg("--file")
        .arg(file.path())
        .arg("delete")
        .arg("--select-type")
        .arg("p")
        .arg("--section");

    cmd.assert().failure().stderr(contains(
        "The --section flag can only be used when deleting a heading",
    ));
}

#[test]
fn delete_paragraph_by_content() {
    let file = assert_fs::NamedTempFile::new("test.md").unwrap();
    file.write_str(
        "# Title\n\nFirst paragraph.\n\nSecond paragraph to delete.\n\nThird paragraph.\n",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("md-splice").unwrap();
    cmd.arg("--file")
        .arg(file.path())
        .arg("delete")
        .arg("--select-contains")
        .arg("Second paragraph");

    cmd.assert().success();

    let result = std::fs::read_to_string(file.path()).unwrap();
    assert_snapshot!(result, @r###"# Title

First paragraph.

Third paragraph.
"###);
}
