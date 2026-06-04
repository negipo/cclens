use anyhow::{Context, Result};
use rusqlite::{params, Connection};

use crate::models::{MatchSnippet, ParsedSession, QueryResult, ShowResult};

const SCHEMA_VERSION: i64 = 2;

fn truncate_around_match(content: &str, max_len: usize) -> String {
    let trimmed = content.trim();
    if trimmed.chars().count() <= max_len {
        return trimmed.to_string();
    }
    let chars: Vec<char> = trimmed.chars().collect();
    let end = max_len.min(chars.len());
    let mut s: String = chars[..end].iter().collect();
    s.push_str("...");
    s
}

fn split_search_terms(text: &str) -> Vec<String> {
    text.split('|')
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .map(String::from)
        .collect()
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open() -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .context("キャッシュディレクトリが見つかりません")?
            .join("cclens");
        std::fs::create_dir_all(&cache_dir)?;
        let db_path = cache_dir.join("index.db");
        let conn = Connection::open(db_path)?;
        let db = Self { conn };
        db.create_tables()?;
        let needs_rebuild = match db.get_meta("schema_version") {
            Ok(Some(v)) => v.parse::<i64>().unwrap_or(0) != SCHEMA_VERSION,
            _ => true,
        };
        if needs_rebuild {
            db.conn.execute_batch("DROP TABLE IF EXISTS sessions; DROP TABLE IF EXISTS messages;")?;
            db.create_tables()?;
            db.set_meta("schema_version", &SCHEMA_VERSION.to_string())?;
        }
        Ok(db)
    }

    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.create_tables()?;
        Ok(db)
    }

    fn get_meta(&self, key: &str) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare("SELECT value FROM meta WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    fn set_meta(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO meta (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    fn create_tables(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS sessions (
                session_id TEXT PRIMARY KEY,
                project_path TEXT NOT NULL,
                cwd TEXT,
                git_branch TEXT,
                entrypoint TEXT,
                version TEXT,
                started_at TEXT,
                ended_at TEXT,
                user_message_count INTEGER DEFAULT 0,
                assistant_message_count INTEGER DEFAULT 0,
                source_file TEXT,
                source_mtime INTEGER
            );
            CREATE TABLE IF NOT EXISTS messages (
                rowid INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                is_meta INTEGER NOT NULL DEFAULT 0,
                timestamp TEXT,
                uuid TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_messages_session ON messages(session_id);
            CREATE INDEX IF NOT EXISTS idx_sessions_project ON sessions(project_path);",
        )?;
        Ok(())
    }

    pub fn upsert_session(
        &self,
        session: &ParsedSession,
        source_file: &str,
        source_mtime: i64,
    ) -> Result<()> {
        self.conn.execute(
            "DELETE FROM messages WHERE session_id = ?1",
            params![session.session_id],
        )?;
        self.conn.execute(
            "DELETE FROM sessions WHERE session_id = ?1",
            params![session.session_id],
        )?;

        let user_count = session
            .messages
            .iter()
            .filter(|m| m.role == "user" && !m.is_meta)
            .count() as i64;
        let assistant_count = session
            .messages
            .iter()
            .filter(|m| m.role == "assistant" && !m.is_meta)
            .count() as i64;

        self.conn.execute(
            "INSERT INTO sessions (session_id, project_path, cwd, git_branch, entrypoint, version, started_at, ended_at, user_message_count, assistant_message_count, source_file, source_mtime)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                session.session_id,
                session.project_path,
                session.cwd,
                session.git_branch,
                session.entrypoint,
                session.version,
                session.started_at,
                session.ended_at,
                user_count,
                assistant_count,
                source_file,
                source_mtime,
            ],
        )?;

        for msg in &session.messages {
            self.conn.execute(
                "INSERT INTO messages (session_id, role, content, is_meta, timestamp, uuid)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    session.session_id,
                    msg.role,
                    msg.content,
                    msg.is_meta,
                    msg.timestamp,
                    msg.uuid,
                ],
            )?;
        }

        Ok(())
    }

    pub fn get_source_file(&self, session_id: &str) -> Result<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT source_file FROM sessions WHERE session_id = ?1")?;
        let mut rows = stmt.query(params![session_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_source_mtime(&self, source_file: &str) -> Result<Option<i64>> {
        let mut stmt = self
            .conn
            .prepare("SELECT source_mtime FROM sessions WHERE source_file = ?1 LIMIT 1")?;
        let mut rows = stmt.query(params![source_file])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    pub fn search_sessions(
        &self,
        text: &str,
        project_paths: &[&str],
        limit: usize,
    ) -> Result<Vec<QueryResult>> {
        if project_paths.is_empty() {
            return Ok(Vec::new());
        }

        let terms = split_search_terms(text);
        if terms.is_empty() {
            return Ok(Vec::new());
        }
        let patterns: Vec<String> = terms.iter().map(|t| format!("%{}%", t)).collect();

        let like_clause = (0..patterns.len())
            .map(|i| format!("m.content LIKE ?{}", i + 1))
            .collect::<Vec<_>>()
            .join(" OR ");
        let proj_start = patterns.len() + 1;
        let in_clause = (0..project_paths.len())
            .map(|i| format!("?{}", proj_start + i))
            .collect::<Vec<_>>()
            .join(", ");

        let sql = format!(
            "SELECT s.session_id, s.project_path, s.cwd, s.git_branch, s.started_at, s.ended_at, COUNT(*) as match_count
             FROM sessions s
             JOIN messages m ON s.session_id = m.session_id
             WHERE ({})
               AND m.is_meta = 0
               AND s.project_path IN ({})
             GROUP BY s.session_id, s.project_path, s.cwd, s.git_branch, s.started_at, s.ended_at
             ORDER BY s.started_at DESC
             LIMIT {}",
            like_clause, in_clause, limit
        );

        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        for p in &patterns {
            param_values.push(Box::new(p.clone()));
        }
        for path in project_paths {
            param_values.push(Box::new(path.to_string()));
        }
        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();

        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = stmt.query(param_refs.as_slice())?;

        let mut results = Vec::new();
        while let Some(row) = rows.next()? {
            let session_id: String = row.get(0)?;
            let matches = self.get_match_snippets(&session_id, &patterns, 5)?;
            results.push(QueryResult {
                resume_command: format!("claude --resume {}", session_id),
                session_id,
                project_path: row.get(1)?,
                cwd: row.get(2)?,
                git_branch: row.get(3)?,
                started_at: row.get(4)?,
                ended_at: row.get(5)?,
                match_count: row.get(6)?,
                matches,
            });
        }

        Ok(results)
    }

    fn get_match_snippets(
        &self,
        session_id: &str,
        patterns: &[String],
        max_snippets: usize,
    ) -> Result<Vec<MatchSnippet>> {
        let like_clause = (0..patterns.len())
            .map(|i| format!("content LIKE ?{}", i + 2))
            .collect::<Vec<_>>()
            .join(" OR ");
        let limit_idx = patterns.len() + 2;
        let sql = format!(
            "SELECT role, content, timestamp FROM messages
             WHERE session_id = ?1 AND ({}) AND is_meta = 0
             ORDER BY timestamp LIMIT ?{}",
            like_clause, limit_idx
        );

        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        param_values.push(Box::new(session_id.to_string()));
        for p in patterns {
            param_values.push(Box::new(p.clone()));
        }
        param_values.push(Box::new(max_snippets as i64));
        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();

        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = stmt.query(param_refs.as_slice())?;

        let mut snippets = Vec::new();
        while let Some(row) = rows.next()? {
            let content: String = row.get(1)?;
            let snippet = truncate_around_match(&content, 80);
            snippets.push(MatchSnippet {
                role: row.get(0)?,
                snippet,
                timestamp: row.get(2)?,
            });
        }
        Ok(snippets)
    }

    pub fn list_sessions(
        &self,
        project_paths: &[&str],
        branch: Option<&str>,
        after: Option<&str>,
        before: Option<&str>,
        limit: usize,
    ) -> Result<Vec<QueryResult>> {
        if project_paths.is_empty() {
            return Ok(Vec::new());
        }

        let mut conditions = Vec::new();
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        let mut param_idx = 1;

        let placeholders: Vec<String> = (0..project_paths.len())
            .map(|i| format!("?{}", param_idx + i))
            .collect();
        conditions.push(format!("project_path IN ({})", placeholders.join(", ")));
        for path in project_paths {
            param_values.push(Box::new(path.to_string()));
        }
        param_idx += project_paths.len();

        if let Some(b) = branch {
            conditions.push(format!("git_branch = ?{}", param_idx));
            param_values.push(Box::new(b.to_string()));
            param_idx += 1;
        }

        if let Some(a) = after {
            conditions.push(format!("started_at >= ?{}", param_idx));
            param_values.push(Box::new(a.to_string()));
            param_idx += 1;
        }

        if let Some(b) = before {
            conditions.push(format!("started_at <= ?{}", param_idx));
            param_values.push(Box::new(b.to_string()));
        }

        let where_clause = format!("WHERE {}", conditions.join(" AND "));

        let sql = format!(
            "SELECT session_id, project_path, cwd, git_branch, started_at, ended_at, 0 as match_count
             FROM sessions
             {}
             ORDER BY started_at DESC
             LIMIT {}",
            where_clause, limit
        );

        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = stmt.query(param_refs.as_slice())?;

        let mut results = Vec::new();
        while let Some(row) = rows.next()? {
            let session_id: String = row.get(0)?;
            results.push(QueryResult {
                resume_command: format!("claude --resume {}", session_id),
                session_id,
                project_path: row.get(1)?,
                cwd: row.get(2)?,
                git_branch: row.get(3)?,
                started_at: row.get(4)?,
                ended_at: row.get(5)?,
                match_count: row.get(6)?,
                matches: Vec::new(),
            });
        }

        Ok(results)
    }

    pub fn get_session(&self, session_id: &str) -> Result<Option<ShowResult>> {
        let mut stmt = self.conn.prepare(
            "SELECT session_id, project_path, cwd, git_branch, entrypoint, version, started_at, ended_at, user_message_count, assistant_message_count
             FROM sessions WHERE session_id = ?1",
        )?;
        let mut rows = stmt.query(params![session_id])?;

        if let Some(row) = rows.next()? {
            let sid: String = row.get(0)?;
            Ok(Some(ShowResult {
                resume_command: format!("claude --resume {}", sid),
                session_id: sid,
                project_path: row.get(1)?,
                cwd: row.get(2)?,
                git_branch: row.get(3)?,
                entrypoint: row.get(4)?,
                version: row.get(5)?,
                started_at: row.get(6)?,
                ended_at: row.get(7)?,
                user_message_count: row.get(8)?,
                assistant_message_count: row.get(9)?,
            }))
        } else {
            Ok(None)
        }
    }

}
