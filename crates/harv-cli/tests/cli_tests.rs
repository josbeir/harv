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
        .success()
        .stdout(predicate::str::contains("Config file:"))
        .stdout(predicate::str::contains("harv connect"));
}

#[test]
fn test_output_flag_json() {
    let temp = tempfile::tempdir().unwrap();
    let mut cmd = Command::cargo_bin("harv").unwrap();
    cmd.arg("config")
        .arg("--output")
        .arg("json")
        .env("XDG_CONFIG_HOME", temp.path())
        .assert()
        .success();
}

#[test]
fn test_output_flag_table() {
    let temp = tempfile::tempdir().unwrap();
    let mut cmd = Command::cargo_bin("harv").unwrap();
    cmd.arg("config")
        .arg("--output")
        .arg("table")
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
