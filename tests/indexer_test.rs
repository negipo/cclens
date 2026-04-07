use cclens::db::Database;
use cclens::indexer::update_index;
use std::fs;

#[test]
fn test_incremental_index_skips_unchanged() {
    let tmp = tempfile::tempdir().unwrap();
    let project_dir = tmp.path().join("project");
    fs::create_dir_all(&project_dir).unwrap();

    let session_file = project_dir.join("abc-123.jsonl");
    fs::write(&session_file, r#"{"type":"user","message":{"role":"user","content":"hello"},"sessionId":"abc-123","timestamp":"2026-03-22T12:00:00Z","uuid":"msg-1","parentUuid":null,"cwd":"/test","gitBranch":"main","entrypoint":"cli","version":"2.1.81","isSidechain":false}"#).unwrap();

    let db = Database::in_memory().unwrap();

    let (indexed, skipped) = update_index(&db, &[project_dir.clone()]).unwrap();
    assert_eq!(indexed, 1);
    assert_eq!(skipped, 0);

    let (indexed, skipped) = update_index(&db, &[project_dir.clone()]).unwrap();
    assert_eq!(indexed, 0);
    assert_eq!(skipped, 1);
}
