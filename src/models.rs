use serde::Serialize;

#[derive(Debug, Clone)]
pub struct ParsedSession {
    pub session_id: String,
    pub project_path: String,
    pub cwd: Option<String>,
    pub git_branch: Option<String>,
    pub entrypoint: Option<String>,
    pub version: Option<String>,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub messages: Vec<ParsedMessage>,
}

#[derive(Debug, Clone)]
pub struct ParsedMessage {
    pub role: String,
    pub content: String,
    pub is_meta: bool,
    pub timestamp: String,
    pub uuid: String,
}

#[derive(Debug, Serialize)]
pub struct MatchSnippet {
    pub role: String,
    pub snippet: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize)]
pub struct QueryResult {
    pub session_id: String,
    pub project_path: String,
    pub cwd: Option<String>,
    pub git_branch: Option<String>,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub match_count: i64,
    pub matches: Vec<MatchSnippet>,
    pub resume_command: String,
}

#[derive(Debug, Serialize)]
pub struct ShowResult {
    pub session_id: String,
    pub project_path: String,
    pub cwd: Option<String>,
    pub git_branch: Option<String>,
    pub entrypoint: Option<String>,
    pub version: Option<String>,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub user_message_count: i64,
    pub assistant_message_count: i64,
    pub resume_command: String,
}
