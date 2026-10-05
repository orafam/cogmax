use std::process::Command;

#[test]
fn inspect_reports_configured_local_data_directory_without_network() {
    let output = Command::new(env!("CARGO_BIN_EXE_cogmax"))
        .arg("inspect")
        .env("COGMAX_DATA_DIR", "/tmp/cogmax-offline-test")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("/tmp/cogmax-offline-test/cogmax.sqlite3")
    );
}

#[test]
fn onboarding_summary_is_read_only_and_apply_is_idempotent() {
    let data_dir = tempfile::tempdir().unwrap();
    let summary = Command::new(env!("CARGO_BIN_EXE_cogmax"))
        .args(["onboard", "--summary"])
        .env("COGMAX_DATA_DIR", data_dir.path())
        .output()
        .unwrap();
    assert!(summary.status.success());
    assert!(String::from_utf8_lossy(&summary.stdout).contains("\"initialized\":false"));
    assert!(!data_dir.path().join("onboarding.json").exists());
    let summary_json: serde_json::Value = serde_json::from_slice(&summary.stdout).unwrap();
    let digest = summary_json["confirmation_digest"].as_str().unwrap();

    let first = Command::new(env!("CARGO_BIN_EXE_cogmax"))
        .args(["onboard", "--apply", "--confirm", digest])
        .env("COGMAX_DATA_DIR", data_dir.path())
        .output()
        .unwrap();
    assert!(first.status.success());
    assert!(String::from_utf8_lossy(&first.stdout).contains("\"initialized\":true"));

    let second = Command::new(env!("CARGO_BIN_EXE_cogmax"))
        .args(["onboard", "--apply", "--confirm", digest])
        .env("COGMAX_DATA_DIR", data_dir.path())
        .output()
        .unwrap();
    assert!(second.status.success());
    assert!(String::from_utf8_lossy(&second.stdout).contains("\"already_initialized\":true"));
}

#[test]
fn onboarding_rejects_stale_confirmation() {
    let data_dir = tempfile::tempdir().unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_cogmax"))
        .args(["onboard", "--apply", "--confirm", "stale"])
        .env("COGMAX_DATA_DIR", data_dir.path())
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(!data_dir.path().join("onboarding.json").exists());
}
