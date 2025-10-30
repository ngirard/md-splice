use assert_cmd::Command;
use assert_fs::prelude::*;
use insta::assert_snapshot;
use predicates::ord::eq;
use regex::Regex;

fn cmd() -> Command {
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
}

/// Replaces the version number (e.g., "0.1.0") in a string with a static placeholder.
fn redact_version(text: &str) -> String {
    let re = Regex::new(r"\d+\.\d+\.\d+").unwrap();
    re.replace_all(text, "[VERSION]").to_string()
}

#[test]
fn test_i1_version_flag() {
    let output = cmd().arg("--version").output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_snapshot!("i1_version", redact_version(&stdout));
}

#[test]
fn test_i1_help_flag() {
    let output = cmd().arg("--help").output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_snapshot!("i1_help", redact_version(&stdout));
}

#[test]
fn test_i1_help_flag_insert() {
    let output = cmd().args(["insert", "--help"]).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_snapshot!("i1_help_insert", redact_version(&stdout));
}

#[test]
fn test_i1_help_flag_replace() {
    let output = cmd().args(["replace", "--help"]).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_snapshot!("i1_help_replace", redact_version(&stdout));
}

#[test]
fn test_i1_help_flag_apply() {
    let output = cmd().args(["apply", "--help"]).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_snapshot!("i1_help_apply", redact_version(&stdout));
}

#[test]
fn test_i2_file_io_replace_with_output() {
    // Setup: Create a temporary directory and an input file.
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("input.md");
    input_file
        .write_str("# Title\n\nThis is a paragraph to be replaced.\n\nAnother paragraph.\n")
        .unwrap();
    let output_file = temp.child("output.md");

    // Run the command
    cmd()
        .arg("--file")
        .arg(input_file.path())
        .arg("--output")
        .arg(output_file.path())
        .arg("replace")
        .arg("--select-type")
        .arg("p")
        .arg("--select-ordinal")
        .arg("1")
        .arg("--content")
        .arg("**This is the new content.**")
        .assert()
        .success();

    // Verify the output file content with a snapshot.
    let output_content = std::fs::read_to_string(output_file.path()).unwrap();
    insta::assert_snapshot!("i2_replace_output", output_content);

    // Verify the input file was not changed.
    let original_content = "# Title\n\nThis is a paragraph to be replaced.\n\nAnother paragraph.\n";
    input_file.assert(original_content);
}

#[test]
fn test_i2_file_io_insert_with_output() {
    // Setup
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("input.md");
    input_file
        .write_str("# Title\n\nFirst paragraph.\n\nSecond paragraph.\n")
        .unwrap();
    let output_file = temp.child("output.md");

    // Run
    cmd()
        .arg("--file")
        .arg(input_file.path())
        .arg("--output")
        .arg(output_file.path())
        .arg("insert")
        .arg("--select-type")
        .arg("p")
        .arg("--select-ordinal")
        .arg("1")
        .arg("--position")
        .arg("after")
        .arg("--content")
        .arg("## A New Subheading\n\n*And a list item.*")
        .assert()
        .success();

    // Verify output file
    let output_content = std::fs::read_to_string(output_file.path()).unwrap();
    insta::assert_snapshot!("i2_insert_output", output_content);

    // Verify input file is unchanged
    let original_content = "# Title\n\nFirst paragraph.\n\nSecond paragraph.\n";
    input_file.assert(original_content);
}

#[test]
fn test_i3_in_place_edit() {
    // Setup: Create a temporary directory and an input file.
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("inplace.md");
    let original_content = "# In-Place Edit\n\nThis content will be replaced.\n";
    input_file.write_str(original_content).unwrap();

    // Run the command without --output to trigger in-place modification.
    cmd()
        .arg("--file")
        .arg(input_file.path())
        .arg("replace")
        .arg("--select-type")
        .arg("p")
        .arg("--content")
        .arg("The content was successfully replaced in-place.")
        .assert()
        .success();

    // Verify the file was modified.
    // The markdown-ppp renderer does not add a trailing newline to the whole document.
    let expected_content = "# In-Place Edit\n\nThe content was successfully replaced in-place.";
    input_file.assert(eq(expected_content));
}

#[test]
fn test_i4_content_file() {
    // Setup: Create a temporary directory, an input file, and a content file.
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("input.md");
    input_file
        .write_str("# Title\n\nThis is the target paragraph.\n")
        .unwrap();
    let content_file = temp.child("content.md");
    content_file
        .write_str("This content comes from a file.")
        .unwrap();
    let output_file = temp.child("output.md");

    // Run the command using --content-file
    cmd()
        .arg("--file")
        .arg(input_file.path())
        .arg("--output")
        .arg(output_file.path())
        .arg("replace")
        .arg("--select-type")
        .arg("p")
        .arg("--content-file")
        .arg(content_file.path())
        .assert()
        .success();

    // Verify the output file content with a snapshot.
    let output_content = std::fs::read_to_string(output_file.path()).unwrap();
    insta::assert_snapshot!("i4_content_file_output", output_content);

    // Verify the input file was not changed.
    input_file.assert("# Title\n\nThis is the target paragraph.\n");
}

#[test]
fn test_i5_error_reporting_node_not_found() {
    // Setup: Create a temporary directory and an input file.
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("input.md");
    input_file
        .write_str("# A Simple File\n\nJust one paragraph.\n")
        .unwrap();

    // Run the command with a selector that is guaranteed to fail.
    cmd()
        .arg("--file")
        .arg(input_file.path())
        .arg("replace")
        .arg("--select-type")
        .arg("blockquote") // This type does not exist in the input file.
        .arg("--content")
        .arg("some content")
        .assert()
        .failure() // Assert non-zero exit code.
        .stderr(predicates::str::contains(
            "Error: Selector did not match any nodes in the document",
        ));
}

#[test]
fn test_i6_logging_ambiguity_warning() {
    // Setup: Create a file with ambiguous matches.
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("ambiguous.md");
    input_file
        .write_str(
            "# Ambiguous Selectors\n\nThis is the first target.\n\nThis is the second target.\n",
        )
        .unwrap();

    // Run the command. It should succeed but print a warning to stderr.
    // We must enable logging via the RUST_LOG env var for the warning to appear.
    cmd()
        .env("RUST_LOG", "warn")
        .arg("--file")
        .arg(input_file.path())
        .arg("replace")
        .arg("--select-type")
        .arg("p") // This selector matches two paragraphs.
        .arg("--content")
        .arg("New content")
        .assert()
        .success() // The operation itself should succeed.
        .stderr(predicates::str::contains(
            "Warning: Selector matched multiple nodes. Operation was applied to the first match only.",
        ));
}

#[test]
fn test_li1_end_to_end_replace_list_item() {
    // LI1: Use the CLI to replace a list item by its content.
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("input.md");
    input_file
        .write_str("# A List\n\n- Item One\n- Item Two\n- Item Three\n")
        .unwrap();
    let output_file = temp.child("output.md");

    cmd()
        .arg("--file")
        .arg(input_file.path())
        .arg("--output")
        .arg(output_file.path())
        .arg("replace")
        .arg("--select-type")
        .arg("li")
        .arg("--select-contains")
        .arg("Item Two")
        .arg("--content")
        .arg("- Item 2 (Replaced)")
        .assert()
        .success();

    let output_content = std::fs::read_to_string(output_file.path()).unwrap();
    insta::assert_snapshot!("li1_replace_list_item", output_content);
}

#[test]
fn test_li2_end_to_end_insert_list_item() {
    // LI2: Use the CLI to insert a new list item before another, selected by ordinal.
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("input.md");
    input_file
        .write_str("# A List\n\n- Item One\n- Item Two\n- Item Three\n")
        .unwrap();
    let output_file = temp.child("output.md");

    cmd()
        .arg("--file")
        .arg(input_file.path())
        .arg("--output")
        .arg(output_file.path())
        .arg("insert")
        .arg("--select-type")
        .arg("li")
        .arg("--select-ordinal")
        .arg("3") // Target "Item Three"
        .arg("--position")
        .arg("before")
        .arg("--content")
        .arg("- Item 2.5 (Inserted)")
        .assert()
        .success();

    let output_content = std::fs::read_to_string(output_file.path()).unwrap();
    insta::assert_snapshot!("li2_insert_list_item", output_content);
}

#[test]
fn test_li3_end_to_end_error_invalid_list_item_content() {
    // LI3: Verify a non-zero exit code when trying to replace a list item
    // with content that is not a valid list item itself.
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("input.md");
    input_file
        .write_str("# A List\n\n- Item One\n- Item Two\n")
        .unwrap();

    cmd()
        .arg("--file")
        .arg(input_file.path())
        .arg("replace")
        .arg("--select-type")
        .arg("li")
        .arg("--select-ordinal")
        .arg("1")
        .arg("--content")
        .arg("This is just a paragraph, not a list item.")
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Error: Invalid content for list item operation",
        ));
}

#[test]
fn test_li4_end_to_end_nested_insert() {
    // LI4: Use the CLI to insert a nested list into an existing list item.
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("input.md");
    input_file
        .write_str("# My Tasks\n- [x] Buy milk\n- [ ] Write the report\n- [ ] Call the client\n")
        .unwrap();
    let output_file = temp.child("output.md");

    cmd()
        .arg("--file")
        .arg(input_file.path())
        .arg("--output")
        .arg(output_file.path())
        .arg("insert")
        .arg("--select-type")
        .arg("li")
        .arg("--select-contains")
        .arg("Write the report")
        .arg("--position")
        .arg("append-child")
        .arg("--content")
        .arg("  - [ ] Write the first section") // Indentation is key here
        .assert()
        .success();

    let output_content = std::fs::read_to_string(output_file.path()).unwrap();
    insta::assert_snapshot!("li4_nested_list_insert", output_content);
}

#[test]
fn test_li4_end_to_end_nested_insert_alias() {
    // LI4-alias: Allow underscore form for insert position values.
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("input.md");
    input_file
        .write_str("# My Tasks\n- [x] Buy milk\n- [ ] Write the report\n- [ ] Call the client\n")
        .unwrap();
    let output_file = temp.child("output.md");

    cmd()
        .arg("--file")
        .arg(input_file.path())
        .arg("--output")
        .arg(output_file.path())
        .arg("insert")
        .arg("--select-type")
        .arg("li")
        .arg("--select-contains")
        .arg("Write the report")
        .arg("--position")
        .arg("append_child")
        .arg("--content")
        .arg("  - [ ] Write the first section")
        .assert()
        .success();

    let output_content = std::fs::read_to_string(output_file.path()).unwrap();
    assert!(output_content.contains("  - [ ] Write the first section"));
}

#[test]
fn test_i7_source_from_stdin() {
    // I7: Pipe a file into md-splice (no --file arg) and verify the output on STDOUT.
    let input_md = "# Source from STDIN\n\nThis is the original paragraph.\n";

    let mut cmd = cmd();
    cmd.arg("replace")
        .arg("--select-type")
        .arg("p")
        .arg("--content")
        .arg("This paragraph was replaced via STDIN.")
        .write_stdin(input_md)
        .assert()
        .success();

    let stdout = String::from_utf8(cmd.output().unwrap().stdout).unwrap();
    insta::assert_snapshot!("i7_source_from_stdin", stdout);
}

#[test]
fn test_i8_content_from_stdin() {
    // I8: Use --content-file - and pipe content into it.
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("input.md");
    input_file
        .write_str("# Title\n\nThis is the target.\n")
        .unwrap();
    let content_md = "This content comes from STDIN.";

    cmd()
        .arg("--file")
        .arg(input_file.path())
        .arg("replace")
        .arg("--select-type")
        .arg("p")
        .arg("--content-file")
        .arg("-")
        .write_stdin(content_md)
        .assert()
        .success();

    // The operation should modify the file in-place.
    let expected_content = "# Title\n\nThis content comes from STDIN.";
    input_file.assert(eq(expected_content));
}

#[test]
fn test_i9_error_on_ambiguous_stdin() {
    // I9: Test that running with no --file and with --content-file - produces an error.
    cmd()
        .arg("replace")
        .arg("--select-type")
        .arg("p")
        .arg("--content-file")
        .arg("-")
        .write_stdin("some content")
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Error: Cannot read both source document and splice content from stdin.",
        ));
}

#[test]
fn test_i10_source_from_stdin_output_to_file() {
    // I10: Pipe a file in, use --output, and verify the output file's content.
    let temp = assert_fs::TempDir::new().unwrap();
    let output_file = temp.child("output.md");
    let input_md = "# Source from STDIN\n\nThis will go to a file.\n";

    cmd()
        .arg("--output")
        .arg(output_file.path())
        .arg("replace")
        .arg("--select-type")
        .arg("p")
        .arg("--content")
        .arg("The output was redirected to a file.")
        .write_stdin(input_md)
        .assert()
        .success();

    let output_content = std::fs::read_to_string(output_file.path()).unwrap();
    insta::assert_snapshot!("i10_stdin_to_file", output_content);
}
