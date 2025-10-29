use assert_cmd::Command;
use assert_fs::prelude::*;
use insta::assert_snapshot;
use predicates::prelude::*;
use serde_json::json;

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
        "selector": {
            "select_contains": "Replace me."
        },
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
  selector:
    select_contains: "Status: In Progress"
  content: "Status: **Complete**"
-
  op: delete
  selector:
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

#[test]
fn apply_command_supports_dry_run() {
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("input.md");
    input_file.write_str("# Title\n\nReplace me.\n").unwrap();

    let operations_file = temp.child("ops.json");
    operations_file
        .write_str(
            r#"[
    {
        "op": "replace",
        "selector": {
            "select_contains": "Replace me."
        },
        "content": "Updated content."
    }
]"#,
        )
        .unwrap();

    let original_content = std::fs::read_to_string(input_file.path()).unwrap();

    let output = cmd()
        .arg("--file")
        .arg(input_file.path())
        .arg("apply")
        .arg("--operations-file")
        .arg(operations_file.path())
        .arg("--dry-run")
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout, "# Title\n\nUpdated content.");

    let current_content = std::fs::read_to_string(input_file.path()).unwrap();
    assert_eq!(current_content, original_content);
}

#[test]
fn apply_command_supports_diff_output() {
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("input.md");
    input_file
        .write_str("# Title\n\nReplace me.\nSecond line.\n")
        .unwrap();

    let operations_file = temp.child("ops.json");
    operations_file
        .write_str(
            r#"[
    {
        "op": "replace",
        "selector": {
            "select_contains": "Replace me."
        },
        "content": "Updated content."
    }
]"#,
        )
        .unwrap();

    let original_content = std::fs::read_to_string(input_file.path()).unwrap();

    let output = cmd()
        .arg("--file")
        .arg(input_file.path())
        .arg("apply")
        .arg("--operations-file")
        .arg(operations_file.path())
        .arg("--diff")
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_snapshot!("apply_command_diff_output", stdout);

    let current_content = std::fs::read_to_string(input_file.path()).unwrap();
    assert_eq!(current_content, original_content);
}

#[test]
fn apply_command_supports_inline_operations() {
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("input.md");
    input_file
        .write_str("# Title\n\nReplace me inline.\n")
        .unwrap();

    let operations = json!([
        {
            "op": "replace",
            "selector": { "select_contains": "Replace me inline." },
            "content": "Updated via inline operations.",
        }
    ]);

    cmd()
        .arg("--file")
        .arg(input_file.path())
        .arg("apply")
        .arg("--operations")
        .arg(operations.to_string())
        .assert()
        .success();

    let content = std::fs::read_to_string(input_file.path()).unwrap();
    assert_eq!(content, "# Title\n\nUpdated via inline operations.");
}

#[test]
fn apply_command_supports_until_range() {
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("guide.md");
    input_file
        .write_str("# Guide\n\n## Installation\nStep one.\n\nStep two.\n\n## Usage\nUsage notes.\n")
        .unwrap();

    let operations_file = temp.child("ops.yaml");
    operations_file
        .write_str(
            r#"-
  op: replace
  selector:
    select_type: h2
    select_contains: Installation
  until:
    select_type: h2
    select_contains: Usage
  content: |
    ## Installation
    Updated steps.
"#,
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
    assert!(content.contains("Updated steps."));
    assert!(!content.contains("Step one."));
    assert!(content.contains("## Usage"));
}

#[test]
fn apply_command_supports_scoped_selectors() {
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("roadmap.md");
    input_file
        .write_str(
            "# Roadmap\n\n## Future Features\n- [ ] Task Alpha\n- [ ] Task Beta\n- [ ] Task Gamma\n\n## Done\n- [x] Task Omega\n",
        )
        .unwrap();

    let operations_file = temp.child("ops.yaml");
    operations_file
        .write_str(
            r#"-
  op: delete
  selector:
    select_type: li
    select_contains: Task Beta
    within:
      select_type: h2
      select_contains: Future Features
"#,
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
    assert!(content.contains("Task Alpha"));
    assert!(!content.contains("Task Beta"));
    assert!(content.contains("Task Gamma"));
    assert!(content.contains("Task Omega"));
}

#[test]
fn apply_command_handles_frontmatter_and_body_operations() {
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("doc.md");
    input_file
        .write_str("---\nstatus: draft\nreviewed: false\n---\n# Title\n\nBody text.\n")
        .unwrap();

    let operations_file = temp.child("ops.yaml");
    operations_file
        .write_str(
            r#"-
  op: set_frontmatter
  key: status
  value: approved
-
  op: insert
  selector:
    select_type: h1
  position: after
  content: |
    Summary updated.
"#,
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
    assert!(content.contains("status: approved"));
    assert!(content.contains("Summary updated."));
    assert!(content.contains("Body text."));
}

#[test]
fn apply_command_is_atomic_when_frontmatter_operation_fails() {
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("doc.md");
    input_file
        .write_str("---\nstatus: draft\n---\n# Title\n\nBody text.\n")
        .unwrap();

    let operations_file = temp.child("ops.yaml");
    operations_file
        .write_str(
            r#"-
  op: set_frontmatter
  key: status
  value: approved
-
  op: delete_frontmatter
  key: does_not_exist
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

    assert.stderr(predicate::str::contains(
        "Frontmatter key 'does_not_exist' was not found",
    ));

    let content = std::fs::read_to_string(input_file.path()).unwrap();
    assert!(content.contains("status: draft"));
    assert!(!content.contains("status: approved"));
}

#[test]
fn apply_command_replaces_frontmatter_block() {
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("doc.md");
    input_file
        .write_str("---\nstatus: draft\n---\n# Title\n\nBody text.\n")
        .unwrap();

    let operations_file = temp.child("ops.yaml");
    operations_file
        .write_str(
            r#"-
  op: replace_frontmatter
  format: toml
  content:
    title: "Spec"
    status: approved
    version: 2
"#,
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
    assert!(content.starts_with("+++"));
    assert!(content.contains("title = \"Spec\""));
    assert!(content.contains("version = 2"));
    assert!(content.contains("Body text."));
}
