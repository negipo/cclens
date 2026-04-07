use cclens::parser::parse_session_file;
use std::path::Path;

#[test]
fn test_parse_session_filters_correctly() {
    let fixture = Path::new("tests/fixtures/sample_session.jsonl");
    let session = parse_session_file(fixture, "test-project").unwrap();

    assert_eq!(session.session_id, "sample_session");
    assert_eq!(session.cwd.as_deref(), Some("/Users/test/project"));
    assert_eq!(session.git_branch.as_deref(), Some("main"));

    assert_eq!(session.messages.len(), 3);

    let user_msg = &session.messages[0];
    assert_eq!(user_msg.role, "user");
    assert_eq!(user_msg.content, "retryの仕組みについて教えて");
    assert!(!user_msg.is_meta);

    let meta_msg = &session.messages[1];
    assert_eq!(meta_msg.role, "user");
    assert!(meta_msg.is_meta);

    let assistant_msg = &session.messages[2];
    assert_eq!(assistant_msg.role, "assistant");
    assert_eq!(assistant_msg.content, "retryは指数バックオフで再試行する仕組みです。");
    assert!(!assistant_msg.is_meta);
}
