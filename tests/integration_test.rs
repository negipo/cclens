use std::process::Command;
use std::sync::Once;

static BUILD: Once = Once::new();

fn cclens_bin() -> String {
    BUILD.call_once(|| {
        let status = Command::new("cargo").args(["build"]).status().unwrap();
        assert!(status.success());
    });

    let output = Command::new("cargo")
        .args(["metadata", "--format-version=1", "--no-deps"])
        .output()
        .unwrap();
    let meta: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let target_dir = meta["target_directory"].as_str().unwrap();
    format!("{}/debug/cclens", target_dir)
}

#[test]
fn test_query_json_flag_returns_json_array() {
    let bin = cclens_bin();
    let output = Command::new(&bin)
        .args(["query", "--json", "nonexistent-query-string-xyz"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(parsed.is_array());
}

#[test]
fn test_query_default_outputs_table_header() {
    let bin = cclens_bin();
    let output = Command::new(&bin)
        .args(["query", "nonexistent-query-string-xyz"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("STARTED"));
    assert!(stdout.contains("PROJECT"));
    assert!(stdout.contains("SESSION"));
    assert!(stdout.contains("SNIPPET"));
}

#[test]
fn test_list_json_flag_returns_json_array() {
    let bin = cclens_bin();
    let output = Command::new(&bin)
        .args(["list", "--json", "--branch", "nonexistent-branch-xyz"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(parsed.is_array());
}

#[test]
fn test_list_default_outputs_table_header() {
    let bin = cclens_bin();
    let output = Command::new(&bin)
        .args(["list", "--branch", "nonexistent-branch-xyz"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("STARTED"));
    assert!(stdout.contains("PROJECT"));
    assert!(stdout.contains("SESSION"));
    assert!(stdout.contains("SNIPPET"));
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

#[test]
fn test_render_session_markdown_returns_export_text() {
    use cclens::commands::export::render_session_markdown;
    use cclens::db::Database;
    use cclens::models::{ParsedMessage, ParsedSession};
    use std::io::Write;

    let dir = tempfile::tempdir().unwrap();
    let jsonl_path = dir.path().join("conv.jsonl");
    let mut f = std::fs::File::create(&jsonl_path).unwrap();
    writeln!(
        f,
        r#"{{"type":"user","message":{{"content":"Hello there"}}}}"#
    )
    .unwrap();
    writeln!(
        f,
        r#"{{"type":"assistant","message":{{"content":[{{"type":"text","text":"Hi, how can I help?"}}]}}}}"#
    )
    .unwrap();

    let db = Database::in_memory().unwrap();
    let session = ParsedSession {
        session_id: "render-id".to_string(),
        project_path: "test-project".to_string(),
        cwd: Some("/Users/x/proj/sample".to_string()),
        git_branch: Some("main".to_string()),
        entrypoint: Some("cli".to_string()),
        version: Some("2.1.81".to_string()),
        started_at: Some("2026-03-22T12:00:00Z".to_string()),
        ended_at: Some("2026-03-22T13:00:00Z".to_string()),
        messages: vec![ParsedMessage {
            role: "user".to_string(),
            content: "Hello there".to_string(),
            is_meta: false,
            timestamp: "2026-03-22T12:00:00Z".to_string(),
            uuid: "m1".to_string(),
        }],
    };
    db.upsert_session(&session, jsonl_path.to_str().unwrap(), 1)
        .unwrap();

    let md = render_session_markdown(&db, "render-id").unwrap();
    assert!(md.starts_with("# Session render-id"));
    assert!(md.contains("> Hello there"));
    assert!(md.contains("❋ Hi, how can I help?"));
}
