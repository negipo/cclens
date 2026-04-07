use std::process::Command;
use std::sync::Once;

static BUILD: Once = Once::new();

fn cclens_bin() -> String {
    BUILD.call_once(|| {
        let status = Command::new("cargo")
            .args(["build"])
            .status()
            .unwrap();
        assert!(status.success());
    });

    let output = Command::new("cargo")
        .args(["metadata", "--format-version=1", "--no-deps"])
        .output()
        .unwrap();
    let meta: serde_json::Value =
        serde_json::from_slice(&output.stdout).unwrap();
    let target_dir = meta["target_directory"].as_str().unwrap();
    format!("{}/debug/cclens", target_dir)
}

#[test]
fn test_query_returns_json_array() {
    let bin = cclens_bin();
    let output = Command::new(&bin)
        .args(["query", "nonexistent-query-string-xyz"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(parsed.is_array());
}

#[test]
fn test_show_nonexistent_session() {
    let bin = cclens_bin();
    let output = Command::new(&bin)
        .args(["show", "nonexistent-session-id"])
        .output()
        .unwrap();

    assert!(!output.status.success());
}

#[test]
fn test_export_nonexistent_session() {
    let bin = cclens_bin();
    let output = Command::new(&bin)
        .args(["export", "nonexistent-session-id"])
        .output()
        .unwrap();

    assert!(!output.status.success());
}
