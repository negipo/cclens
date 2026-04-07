use cclens::db::Database;
use cclens::models::{ParsedMessage, ParsedSession};

#[test]
fn test_insert_and_query_session() {
    let db = Database::in_memory().unwrap();

    let session = ParsedSession {
        session_id: "test-id".to_string(),
        project_path: "test-project".to_string(),
        cwd: Some("/test/cwd".to_string()),
        git_branch: Some("main".to_string()),
        entrypoint: Some("cli".to_string()),
        version: Some("2.1.81".to_string()),
        started_at: Some("2026-03-22T12:00:00Z".to_string()),
        ended_at: Some("2026-03-22T13:00:00Z".to_string()),
        messages: vec![
            ParsedMessage {
                role: "user".to_string(),
                content: "retryの仕組みについて教えて".to_string(),
                is_meta: false,
                timestamp: "2026-03-22T12:00:00Z".to_string(),
                uuid: "msg-1".to_string(),
            },
            ParsedMessage {
                role: "assistant".to_string(),
                content: "retryは指数バックオフで再試行する仕組みです".to_string(),
                is_meta: false,
                timestamp: "2026-03-22T12:00:05Z".to_string(),
                uuid: "msg-2".to_string(),
            },
        ],
    };

    db.upsert_session(&session, "/path/to/file.jsonl", 12345).unwrap();

    let results = db.search_sessions("retry", &["test-project"], 20).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].session_id, "test-id");
    assert_eq!(results[0].match_count, 2);
    assert_eq!(results[0].matches.len(), 2);
    assert_eq!(results[0].matches[0].role, "user");
    assert!(results[0].matches[0].snippet.contains("retry"));
}

#[test]
fn test_search_excludes_meta() {
    let db = Database::in_memory().unwrap();

    let session = ParsedSession {
        session_id: "test-id-2".to_string(),
        project_path: "test-project".to_string(),
        cwd: Some("/test".to_string()),
        git_branch: Some("main".to_string()),
        entrypoint: Some("cli".to_string()),
        version: Some("2.1.81".to_string()),
        started_at: Some("2026-03-22T12:00:00Z".to_string()),
        ended_at: Some("2026-03-22T13:00:00Z".to_string()),
        messages: vec![
            ParsedMessage {
                role: "user".to_string(),
                content: "skillキーワード".to_string(),
                is_meta: true,
                timestamp: "2026-03-22T12:00:00Z".to_string(),
                uuid: "msg-1".to_string(),
            },
        ],
    };

    db.upsert_session(&session, "/path/to/file2.jsonl", 12345).unwrap();

    let results = db.search_sessions("skillキーワード", &["test-project"], 20).unwrap();
    assert_eq!(results.len(), 0);
}
