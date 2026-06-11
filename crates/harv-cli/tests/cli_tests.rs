use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_help() {
    let mut cmd = Command::cargo_bin("harv").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Harvest time tracking"))
        .stdout(predicate::str::contains("connect"))
        .stdout(predicate::str::contains("config"))
        .stdout(predicate::str::contains("track"))
        .stdout(predicate::str::contains("start"))
        .stdout(predicate::str::contains("stop"))
        .stdout(predicate::str::contains("status"));
}

#[test]
fn test_version() {
    let mut cmd = Command::cargo_bin("harv").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("harv"));
}

#[test]
fn test_config_no_config_file() {
    let temp = tempfile::tempdir().unwrap();
    let mut cmd = Command::cargo_bin("harv").unwrap();
    cmd.arg("config")
        .env("XDG_CONFIG_HOME", temp.path())
        .assert()
        .code(1)
        .stderr(predicate::str::contains("harv connect"))
        .stderr(predicate::str::contains("harv connect"));
}

#[test]
fn test_output_flag_json() {
    let temp = tempfile::tempdir().unwrap();
    let mut cmd = Command::cargo_bin("harv").unwrap();
    cmd.arg("--output")
        .arg("json")
        .arg("completion")
        .arg("bash")
        .env("XDG_CONFIG_HOME", temp.path())
        .assert()
        .success();
}

#[test]
fn test_output_flag_table() {
    let temp = tempfile::tempdir().unwrap();
    let mut cmd = Command::cargo_bin("harv").unwrap();
    cmd.arg("--output")
        .arg("table")
        .arg("completion")
        .arg("bash")
        .env("XDG_CONFIG_HOME", temp.path())
        .assert()
        .success();
}

#[test]
fn test_connect_help() {
    let mut cmd = Command::cargo_bin("harv").unwrap();
    cmd.arg("connect")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Authenticate"));
}

#[test]
fn test_track_help() {
    let mut cmd = Command::cargo_bin("harv").unwrap();
    cmd.arg("track")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("project-id"))
        .stdout(predicate::str::contains("editor"));
}

#[test]
fn test_alias_help() {
    let mut cmd = Command::cargo_bin("harv").unwrap();
    cmd.arg("alias")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("delete"));
}

// --- Auth guard tests ---

#[test]
fn test_requires_auth_no_config() {
    let temp = tempfile::tempdir().unwrap();
    let mut cmd = Command::cargo_bin("harv").unwrap();
    cmd.arg("status")
        .env("XDG_CONFIG_HOME", temp.path())
        .assert()
        .code(1)
        .stderr(predicate::str::contains("harv connect"))
        .stderr(predicate::str::contains("harv connect"));
}

#[test]
fn test_connect_allowed_without_config() {
    let temp = tempfile::tempdir().unwrap();
    let mut cmd = Command::cargo_bin("harv").unwrap();
    cmd.arg("connect")
        .arg("--help")
        .env("XDG_CONFIG_HOME", temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Authenticate"));
}

#[test]
fn test_completion_allowed_without_config() {
    let temp = tempfile::tempdir().unwrap();
    let mut cmd = Command::cargo_bin("harv").unwrap();
    cmd.arg("completion")
        .arg("bash")
        .env("XDG_CONFIG_HOME", temp.path())
        .assert()
        .success();
}

#[test]
fn test_edit_help() {
    let mut cmd = Command::cargo_bin("harv").unwrap();
    cmd.arg("edit")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Edit an existing time entry"))
        .stdout(predicate::str::contains("[ENTRY_ID]"))
        .stdout(predicate::str::contains("--project-id"))
        .stdout(predicate::str::contains("--task-id"))
        .stdout(predicate::str::contains("--hours"))
        .stdout(predicate::str::contains("--notes"))
        .stdout(predicate::str::contains("--date"))
        .stdout(predicate::str::contains("--editor"))
        .stdout(predicate::str::contains("--overwrite"))
        .stdout(predicate::str::contains("--refresh"));
}

#[test]
fn test_edit_requires_auth() {
    let temp = tempfile::tempdir().unwrap();
    let mut cmd = Command::cargo_bin("harv").unwrap();
    cmd.arg("edit")
        .env("XDG_CONFIG_HOME", temp.path())
        .assert()
        .code(1)
        .stderr(predicate::str::contains("harv connect"))
        .stderr(predicate::str::contains("harv connect"));
}
