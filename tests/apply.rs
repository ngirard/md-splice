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
