use cclens::commands::query::{format_snippet, format_started_at, project_label, render_table};
use cclens::models::{MatchSnippet, QueryResult};

fn sample(cwd: Option<&str>, snippet: Option<&str>) -> QueryResult {
    QueryResult {
        session_id: "abcd1234".to_string(),
        project_path: "-Users-example-src-sample-repo".to_string(),
        cwd: cwd.map(String::from),
        git_branch: Some("main".to_string()),
        started_at: Some("2026-06-03T12:34:56Z".to_string()),
        ended_at: Some("2026-06-03T13:00:00Z".to_string()),
        match_count: 1,
        matches: snippet
            .map(|s| {
                vec![MatchSnippet {
                    role: "user".to_string(),
                    snippet: s.to_string(),
                    timestamp: "2026-06-03T12:34:56Z".to_string(),
                }]
            })
            .unwrap_or_default(),
        resume_command: "claude --resume abcd1234".to_string(),
    }
}

#[test]
fn test_format_started_at_truncates_to_minute() {
    let r = sample(Some("/Users/example/src/sample-repo"), None);
    assert_eq!(format_started_at(&r.started_at), "2026-06-03T12:34");
}

#[test]
fn test_format_started_at_none_is_empty() {
    let mut r = sample(None, None);
    r.started_at = None;
    assert_eq!(format_started_at(&r.started_at), "");
}

#[test]
fn test_project_label_uses_cwd_basename() {
    let r = sample(Some("/Users/example/src/sample-repo"), None);
    assert_eq!(project_label(&r), "sample-repo");
}

#[test]
fn test_project_label_falls_back_to_project_path() {
    let r = sample(None, None);
    assert_eq!(project_label(&r), "-Users-example-src-sample-repo");
}

#[test]
fn test_format_snippet_replaces_newlines_and_truncates() {
    let r = sample(Some("/Users/example/src/sample-repo"), Some("line one\nline two and a much longer tail that exceeds forty chars"));
    let s = format_snippet(&r);
    assert!(!s.contains('\n'));
    assert_eq!(s.chars().count(), 40);
    assert!(s.starts_with("line one line two"));
}

#[test]
fn test_format_snippet_empty_when_no_match() {
    let r = sample(Some("/Users/example/src/sample-repo"), None);
    assert_eq!(format_snippet(&r), "");
}

#[test]
fn test_render_table_has_header_and_rows() {
    let rows = vec![sample(Some("/Users/example/src/sample-repo"), Some("Notion連携の相談"))];
    let table = render_table(&rows);
    let lines: Vec<&str> = table.lines().collect();
    assert!(lines[0].contains("STARTED"));
    assert!(lines[0].contains("PROJECT"));
    assert!(lines[0].contains("SESSION"));
    assert!(lines[0].contains("SNIPPET"));
    assert!(lines[1].contains("2026-06-03T12:34"));
    assert!(lines[1].contains("sample-repo"));
    assert!(lines[1].contains("abcd1234"));
    assert!(lines[1].contains("Notion連携の相談"));
}
