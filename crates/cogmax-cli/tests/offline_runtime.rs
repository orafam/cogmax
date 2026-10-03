use std::process::Command;

#[test]
fn inspect_reports_configured_local_data_directory_without_network() {
    let output = Command::new(env!("CARGO_BIN_EXE_cogmax"))
        .arg("inspect")
        .env("COGMAX_DATA_DIR", "/tmp/cogmax-offline-test")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("/tmp/cogmax-offline-test/cogmax.sqlite3"));
}
