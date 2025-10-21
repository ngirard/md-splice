use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use insta::assert_snapshot;
use predicates::str::contains;
use std::process::Command;

#[test]
fn get_paragraph_by_type_and_ordinal() {
    let file = assert_fs::NamedTempFile::new("sample.md").unwrap();
    file.write_str(
        "# Title\n\nFirst paragraph.\n\nSecond paragraph to fetch.\n\nThird paragraph.\n",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("md-splice").unwrap();
    cmd.arg("--file")
        .arg(file.path())
        .arg("get")
        .arg("--select-type")
        .arg("p")
        .arg("--select-ordinal")
        .arg("2");

    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(stdout.ends_with('\n'));
    assert_snapshot!(stdout.trim_end_matches('\n'), @"Second paragraph to fetch.");

    // The original file should remain unchanged by the get command.
    let file_contents = std::fs::read_to_string(file.path()).unwrap();
    assert_snapshot!(file_contents.as_str(), @r###"# Title

First paragraph.

Second paragraph to fetch.

Third paragraph.
"###);
}

#[test]
fn get_heading_section_with_section_flag() {
    let file = assert_fs::NamedTempFile::new("sample.md").unwrap();
    file.write_str(
        "# Title\n\n## Section\n\nContent line one.\n\n### Subsection\n\nMore text.\n\n## Next\n\nTail.\n",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("md-splice").unwrap();
    cmd.arg("--file")
        .arg(file.path())
        .arg("get")
        .arg("--select-type")
        .arg("h2")
        .arg("--section");

    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert_snapshot!(stdout, @r###"## Section

Content line one.

### Subsection

More text.
"###);
}

#[test]
fn get_section_flag_requires_heading() {
    let file = assert_fs::NamedTempFile::new("sample.md").unwrap();
    file.write_str("A paragraph only.\n").unwrap();

    let mut cmd = Command::cargo_bin("md-splice").unwrap();
    cmd.arg("--file")
        .arg(file.path())
        .arg("get")
        .arg("--select-type")
        .arg("p")
        .arg("--section");

    cmd.assert().failure().stderr(contains(
        "The --section flag can only be used when targeting a heading (h1-h6).",
    ));
}

#[test]
fn get_all_list_items_with_select_all() {
    let file = assert_fs::NamedTempFile::new("tasks.md").unwrap();
    file.write_str("- [ ] One\n- [x] Two\n- [ ] Three\n")
        .unwrap();

    let mut cmd = Command::cargo_bin("md-splice").unwrap();
    cmd.arg("--file")
        .arg(file.path())
        .arg("get")
        .arg("--select-type")
        .arg("li")
        .arg("--select-contains")
        .arg("[ ]")
        .arg("--select-all");

    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert_snapshot!(stdout, @r###" - [ ] One
 - [ ] Three
"###);
}

#[test]
fn get_all_list_items_with_custom_separator() {
    let file = assert_fs::NamedTempFile::new("tasks.md").unwrap();
    file.write_str("- [ ] One\n- [ ] Two\n").unwrap();

    let mut cmd = Command::cargo_bin("md-splice").unwrap();
    cmd.arg("--file")
        .arg(file.path())
        .arg("get")
        .arg("--select-type")
        .arg("li")
        .arg("--select-all")
        .arg("--separator")
        .arg("---");

    let assert = cmd.assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert_snapshot!(stdout, @" - [ ] One--- - [ ] Two");
}
