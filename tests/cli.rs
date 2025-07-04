use assert_cmd::Command;
use insta::assert_snapshot;

fn cmd() -> Command {
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
}

#[test]
fn test_i1_version_flag() {
    let output = cmd().arg("--version").output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_snapshot!("i1_version", stdout, {
        ".version" => "md-splice [VERSION]"
    });
}

#[test]
fn test_i1_help_flag() {
    let output = cmd().arg("--help").output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_snapshot!("i1_help", stdout, {
        ".version" => "md-splice [VERSION]"
    });
}

#[test]
fn test_i1_help_flag_insert() {
    let output = cmd().args(["insert", "--help"]).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_snapshot!("i1_help_insert", stdout, {
        ".version" => "md-splice [VERSION]"
    });
}

#[test]
fn test_i1_help_flag_replace() {
    let output = cmd().args(["replace", "--help"]).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_snapshot!("i1_help_replace", stdout, {
        ".version" => "md-splice [VERSION]"
    });
}
