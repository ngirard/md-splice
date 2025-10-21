use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
}

#[test]
fn apply_subcommand_requires_operations_source() {
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("input.md");
    input_file
        .write_str("# Title\n\nSome content to transform.\n")
        .unwrap();

    cmd()
        .arg("--file")
        .arg(input_file.path())
        .arg("apply")
        .arg("--operations-file")
        .arg("foo.json")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No such file or directory"));
}

#[test]
fn apply_command_applies_replace_operation() {
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("input.md");
    input_file.write_str("# Title\n\nReplace me.\n").unwrap();

    let operations_file = temp.child("ops.json");
    operations_file
        .write_str(
            r#"[
    {
        "op": "replace",
        "select_contains": "Replace me.",
        "content": "Updated content."
    }
]"#,
        )
        .unwrap();

    cmd()
        .arg("--file")
        .arg(input_file.path())
        .arg("apply")
        .arg("--operations-file")
        .arg(operations_file.path())
        .assert()
        .success();

    let content = std::fs::read_to_string(input_file.path()).unwrap();
    assert_eq!(content, "# Title\n\nUpdated content.");
}

#[test]
fn apply_command_is_atomic_when_operation_fails() {
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("input.md");
    input_file
        .write_str("# Title\n\nStatus: In Progress\n")
        .unwrap();

    let operations_file = temp.child("ops.yaml");
    operations_file
        .write_str(
            r#"-
  op: replace
  select_contains: "Status: In Progress"
  content: "Status: **Complete**"
-
  op: delete
  select_type: h2
  select_contains: "Does Not Exist"
"#,
        )
        .unwrap();

    let assert = cmd()
        .arg("--file")
        .arg(input_file.path())
        .arg("apply")
        .arg("--operations-file")
        .arg(operations_file.path())
        .assert()
        .failure();

    assert.stderr(predicate::str::contains("Selector did not match any nodes"));

    let content = std::fs::read_to_string(input_file.path()).unwrap();
    assert_eq!(content, "# Title\n\nStatus: In Progress\n");
}
