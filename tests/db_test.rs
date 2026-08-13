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

#[test]
fn test_query_result_exposes_cwd() {
    let db = Database::in_memory().unwrap();

    let session = ParsedSession {
        session_id: "cwd-id".to_string(),
        project_path: "test-project".to_string(),
        cwd: Some("/Users/example/src/sample-repo".to_string()),
        git_branch: Some("main".to_string()),
        entrypoint: Some("cli".to_string()),
        version: Some("2.1.81".to_string()),
        started_at: Some("2026-03-22T12:00:00Z".to_string()),
        ended_at: Some("2026-03-22T13:00:00Z".to_string()),
        messages: vec![ParsedMessage {
            role: "user".to_string(),
            content: "Notionと連携したい".to_string(),
            is_meta: false,
            timestamp: "2026-03-22T12:00:00Z".to_string(),
            uuid: "msg-1".to_string(),
        }],
    };
    db.upsert_session(&session, "/path/to/file.jsonl", 12345).unwrap();

    let results = db.search_sessions("Notion", &["test-project"], 20).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].cwd.as_deref(), Some("/Users/example/src/sample-repo"));
}

#[test]
fn test_search_sessions_or_terms() {
    let db = Database::in_memory().unwrap();

    let make = |id: &str, content: &str| ParsedSession {
        session_id: id.to_string(),
        project_path: "test-project".to_string(),
        cwd: Some("/Users/example/src/sample-repo".to_string()),
        git_branch: Some("main".to_string()),
        entrypoint: Some("cli".to_string()),
        version: Some("2.1.81".to_string()),
        started_at: Some("2026-03-22T12:00:00Z".to_string()),
        ended_at: Some("2026-03-22T13:00:00Z".to_string()),
        messages: vec![ParsedMessage {
            role: "user".to_string(),
            content: content.to_string(),
            is_meta: false,
            timestamp: "2026-03-22T12:00:00Z".to_string(),
            uuid: format!("{}-msg", id),
        }],
    };

    db.upsert_session(&make("s-notion", "Notionの話"), "/a.jsonl", 1).unwrap();
    db.upsert_session(&make("s-slack", "Slackの話"), "/b.jsonl", 2).unwrap();
    db.upsert_session(&make("s-other", "無関係な話題"), "/c.jsonl", 3).unwrap();

    let results = db.search_sessions("Notion|Slack", &["test-project"], 20).unwrap();
    let ids: std::collections::HashSet<&str> =
        results.iter().map(|r| r.session_id.as_str()).collect();
    assert_eq!(results.len(), 2);
    assert!(ids.contains("s-notion"));
    assert!(ids.contains("s-slack"));
}

#[test]
fn test_list_sessions_previews_first_user_message() {
    let db = Database::in_memory().unwrap();

    let session = ParsedSession {
        session_id: "list-id".to_string(),
        project_path: "test-project".to_string(),
        cwd: Some("/Users/example/src/sample-repo".to_string()),
        git_branch: Some("main".to_string()),
        entrypoint: Some("cli".to_string()),
        version: Some("2.1.81".to_string()),
        started_at: Some("2026-03-22T12:00:00Z".to_string()),
        ended_at: Some("2026-03-22T13:00:00Z".to_string()),
        messages: vec![
            ParsedMessage {
                role: "user".to_string(),
                content: "最初のユーザ発言".to_string(),
                is_meta: false,
                timestamp: "2026-03-22T12:00:00Z".to_string(),
                uuid: "msg-1".to_string(),
            },
            ParsedMessage {
                role: "assistant".to_string(),
                content: "応答".to_string(),
                is_meta: false,
                timestamp: "2026-03-22T12:00:05Z".to_string(),
                uuid: "msg-2".to_string(),
            },
        ],
    };
    db.upsert_session(&session, "/path/to/file.jsonl", 12345).unwrap();

    let results = db
        .list_sessions(&["test-project"], None, None, None, 20, 240)
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].user_message_count, 1);
    assert_eq!(results[0].matches.len(), 1);
    assert_eq!(results[0].matches[0].role, "user");
    assert_eq!(results[0].matches[0].snippet, "最初のユーザ発言");
}

#[test]
fn test_list_sessions_honors_snippet_limit() {
    let db = Database::in_memory().unwrap();
    let long_content = "あ".repeat(200);
    let session = ParsedSession {
        session_id: "limit-id".to_string(),
        project_path: "test-project".to_string(),
        cwd: Some("/Users/example/src/sample-repo".to_string()),
        git_branch: Some("main".to_string()),
        entrypoint: Some("cli".to_string()),
        version: Some("2.1.81".to_string()),
        started_at: Some("2026-03-22T12:00:00Z".to_string()),
        ended_at: Some("2026-03-22T13:00:00Z".to_string()),
        messages: vec![ParsedMessage {
            role: "user".to_string(),
            content: long_content.clone(),
            is_meta: false,
            timestamp: "2026-03-22T12:00:00Z".to_string(),
            uuid: "msg-1".to_string(),
        }],
    };
    db.upsert_session(&session, "/path/to/file.jsonl", 12345).unwrap();

    let short = db.list_sessions(&["test-project"], None, None, None, 20, 80).unwrap();
    assert_eq!(short[0].matches[0].snippet.chars().count(), 83);

    let wide = db.list_sessions(&["test-project"], None, None, None, 20, 240).unwrap();
    assert_eq!(wide[0].matches[0].snippet, long_content);
}

#[test]
fn test_search_sessions_empty_terms() {
    let db = Database::in_memory().unwrap();
    let results = db.search_sessions("|", &["test-project"], 20).unwrap();
    assert_eq!(results.len(), 0);
}

#[test]
fn test_browse_search_and_terms_across_fields() {
    let db = Database::in_memory().unwrap();

    let make = |id: &str, project: &str, cwd: &str, branch: &str, content: &str, started: &str| {
        ParsedSession {
            session_id: id.to_string(),
            project_path: project.to_string(),
            cwd: Some(cwd.to_string()),
            git_branch: Some(branch.to_string()),
            entrypoint: Some("cli".to_string()),
            version: Some("2.1.81".to_string()),
            started_at: Some(started.to_string()),
            ended_at: Some(started.to_string()),
            messages: vec![ParsedMessage {
                role: "user".to_string(),
                content: content.to_string(),
                is_meta: false,
                timestamp: started.to_string(),
                uuid: format!("{}-msg", id),
            }],
        }
    };

    let alpha = make(
        "id-alpha",
        "-Users-x-proj-alpha",
        "/Users/x/proj/alpha",
        "po/foo",
        "add retry logic to the client",
        "2026-03-22T12:00:00Z",
    );
    let beta = make(
        "id-beta",
        "-Users-x-proj-beta",
        "/Users/x/proj/beta",
        "main",
        "unrelated discussion about caching",
        "2026-03-23T12:00:00Z",
    );
    db.upsert_session(&alpha, "/path/alpha.jsonl", 1).unwrap();
    db.upsert_session(&beta, "/path/beta.jsonl", 2).unwrap();

    let projects = ["-Users-x-proj-alpha", "-Users-x-proj-beta"];

    // プロジェクト名 alpha かつ 本文 retry → alphaのみ
    let r = db.browse_search("alpha retry", &projects, 120, 240).unwrap();
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].session_id, "id-alpha");
    assert_eq!(r[0].user_message_count, 1);
    // matchesはlist同様の最初のuserメッセージ抜粋
    assert_eq!(r[0].matches.len(), 1);
    assert!(r[0].matches[0].snippet.contains("retry"));

    // ブランチ名マッチ
    let r = db.browse_search("po/foo", &projects, 120, 240).unwrap();
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].session_id, "id-alpha");

    // どのセッションも両語を満たさない → 0件
    let r = db.browse_search("alpha beta", &projects, 120, 240).unwrap();
    assert_eq!(r.len(), 0);

    // 空入力 → 0件
    assert_eq!(db.browse_search("   ", &projects, 120, 240).unwrap().len(), 0);
}

#[test]
fn test_browse_search_orders_by_recency() {
    let db = Database::in_memory().unwrap();
    let make = |id: &str, started: &str| ParsedSession {
        session_id: id.to_string(),
        project_path: "test-project".to_string(),
        cwd: Some("/Users/x/proj/sample".to_string()),
        git_branch: Some("main".to_string()),
        entrypoint: Some("cli".to_string()),
        version: Some("2.1.81".to_string()),
        started_at: Some(started.to_string()),
        ended_at: Some(started.to_string()),
        messages: vec![ParsedMessage {
            role: "user".to_string(),
            content: "shared keyword here".to_string(),
            is_meta: false,
            timestamp: started.to_string(),
            uuid: format!("{}-msg", id),
        }],
    };
    db.upsert_session(&make("old", "2026-03-01T00:00:00Z"), "/p/old.jsonl", 1).unwrap();
    db.upsert_session(&make("new", "2026-03-10T00:00:00Z"), "/p/new.jsonl", 2).unwrap();

    let r = db.browse_search("keyword", &["test-project"], 120, 240).unwrap();
    assert_eq!(r.len(), 2);
    assert_eq!(r[0].session_id, "new");
    assert_eq!(r[1].session_id, "old");
}
