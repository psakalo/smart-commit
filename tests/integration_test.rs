use assert_cmd::Command;
use predicates::prelude::*;

mod common;

#[test]
fn test_no_key_provided() {
    let mut cmd = Command::cargo_bin("smart-commit").unwrap();
    cmd.assert()
        .stderr(predicate::str::contains("--openai-api-key"));
}

#[test]
fn test_non_interactive() {
    let temp_dir = tempdir::TempDir::new("smart-commit").unwrap();
    let repo_path = common::setup_git_repo(&temp_dir).unwrap();

    let mut cmd = Command::cargo_bin("smart-commit").unwrap();

    cmd.arg("--non-interactive").arg("--path").arg(repo_path);

    println!(
        "Generated commit message: {:?}",
        String::from_utf8(cmd.unwrap().stdout).unwrap()
    );

    // Validate that the output is exactly one line commit message
    cmd.assert()
        .success()
        .stdout(predicate::str::is_match("^.*$").unwrap());
}
