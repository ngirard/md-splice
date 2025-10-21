use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use insta::assert_snapshot;
use predicates::str::{contains, is_empty};
use std::process::Command;

#[test]
fn delete_section_using_until_flag() {
    let file = assert_fs::NamedTempFile::new("doc.md").unwrap();
    file.write_str(
        "# Guide\n\n## Deprecated Methods\n\nOld API details.\n\n## Examples\n\nUseful samples.\n",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("md-splice").unwrap();
    cmd.arg("--file")
        .arg(file.path())
        .arg("delete")
        .arg("--select-type")
        .arg("h2")
        .arg("--select-contains")
        .arg("Deprecated Methods")
        .arg("--until-type")
        .arg("h2")
        .arg("--until-contains")
        .arg("Examples");

    cmd.assert().success();

    let result = std::fs::read_to_string(file.path()).unwrap();
    assert_snapshot!(result, @r###"# Guide

## Examples

Useful samples.
"###);
}

#[test]
fn get_paragraph_after_heading() {
    let file = assert_fs::NamedTempFile::new("doc.md").unwrap();
    file.write_str("# Introduction\n\nWelcome paragraph.\n\n# Details\n\nFurther reading.\n")
        .unwrap();

    let mut cmd = Command::cargo_bin("md-splice").unwrap();
    cmd.arg("--file")
        .arg(file.path())
        .arg("get")
        .arg("--select-type")
        .arg("p")
        .arg("--after-select-type")
        .arg("h1")
        .arg("--after-select-contains")
        .arg("Introduction");

    cmd.assert()
        .success()
        .stdout(contains("Welcome paragraph."))
        .stderr(is_empty());
}

#[test]
fn insert_task_within_section() {
    let file = assert_fs::NamedTempFile::new("doc.md").unwrap();
    file.write_str(
        "# High Priority\n\n- [ ] Upgrade dependencies\n\n# Backlog\n\n- [ ] Investigate new feature\n",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("md-splice").unwrap();
    cmd.arg("--file")
        .arg(file.path())
        .arg("insert")
        .arg("--content")
        .arg("- [ ] Address security vulnerability")
        .arg("--select-type")
        .arg("li")
        .arg("--select-ordinal")
        .arg("1")
        .arg("--within-select-type")
        .arg("h1")
        .arg("--within-select-contains")
        .arg("High Priority");

    cmd.assert().success();

    let result = std::fs::read_to_string(file.path()).unwrap();
    assert_snapshot!(result, @r###"# High Priority

 - [ ] Upgrade dependencies
 - [ ] Address security vulnerability

# Backlog

 - [ ] Investigate new feature
"###);
}
