use assert_cmd::Command;
use predicates::prelude::*;

fn write_completion_config(temp: &tempfile::TempDir) -> std::path::PathBuf {
    let config_dir = temp.path().join("config");
    let harv_dir = config_dir.join("harv");
    std::fs::create_dir_all(&harv_dir).unwrap();
    std::fs::write(
        harv_dir.join("config.toml"),
        r#"access_token = "token"
account_id = "1"

[aliases.global-dev]
project_id = 1
task_id = 2

[aliases.shared]
project_id = 3
task_id = 4
"#,
    )
    .unwrap();
    config_dir
}

fn write_project_config(temp: &tempfile::TempDir) -> std::path::PathBuf {
    let project_root = temp.path().join("project");
    let nested_dir = project_root.join("nested");
    std::fs::create_dir_all(&nested_dir).unwrap();
    std::fs::write(
        project_root.join("harv.toml"),
        r#"[aliases.project-dev]
project_id = 5
task_id = 6

[aliases.shared]
project_id = 7
task_id = 8
"#,
    )
    .unwrap();
    nested_dir
}

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
fn test_completion_registration_uses_dynamic_protocol() {
    let mut cmd = Command::cargo_bin("harv").unwrap();
    cmd.args(["completion", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("HARV_COMPLETE=\"bash\""));
}

#[test]
fn test_start_completion_includes_project_aliases_from_ancestor() {
    let temp = tempfile::tempdir().unwrap();
    let config_dir = write_completion_config(&temp);
    let nested_dir = write_project_config(&temp);

    let mut cmd = Command::cargo_bin("harv").unwrap();
    cmd.args(["--", "harv", "start", ""])
        .current_dir(nested_dir)
        .env("HARV_CONFIG_DIR", config_dir)
        .env("HARV_COMPLETE", "bash")
        .env("_CLAP_COMPLETE_INDEX", "2")
        .env("_CLAP_COMPLETE_COMP_TYPE", "9")
        .env("_CLAP_COMPLETE_SPACE", "true")
        .env("_CLAP_IFS", "\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("global-dev"))
        .stdout(predicate::str::contains("project-dev"))
        .stdout(predicate::str::contains("shared").count(1));
}

#[test]
fn test_alias_delete_completion_excludes_project_aliases() {
    let temp = tempfile::tempdir().unwrap();
    let config_dir = write_completion_config(&temp);
    let nested_dir = write_project_config(&temp);

    let mut cmd = Command::cargo_bin("harv").unwrap();
    cmd.args(["--", "harv", "alias", "delete", ""])
        .current_dir(nested_dir)
        .env("HARV_CONFIG_DIR", config_dir)
        .env("HARV_COMPLETE", "bash")
        .env("_CLAP_COMPLETE_INDEX", "3")
        .env("_CLAP_COMPLETE_COMP_TYPE", "9")
        .env("_CLAP_COMPLETE_SPACE", "true")
        .env("_CLAP_IFS", "\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("global-dev"))
        .stdout(predicate::str::contains("project-dev").not());
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
