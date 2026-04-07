use crate::models::{ParsedMessage, ParsedSession};
use anyhow::{Context, Result};
use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub fn parse_session_file(path: &Path, project_path: &str) -> Result<ParsedSession> {
    let file = File::open(path).with_context(|| format!("Failed to open {}", path.display()))?;
    let reader = BufReader::new(file);

    let session_id = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let mut messages = Vec::new();
    let mut cwd: Option<String> = None;
    let mut git_branch: Option<String> = None;
    let mut entrypoint: Option<String> = None;
    let mut version: Option<String> = None;
    let mut started_at: Option<String> = None;
    let mut ended_at: Option<String> = None;

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let v: Value =
            serde_json::from_str(&line).with_context(|| "Failed to parse JSONL line")?;
        let msg_type = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
        if msg_type != "user" && msg_type != "assistant" {
            continue;
        }
        let timestamp = v
            .get("timestamp")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();
        if cwd.is_none() {
            cwd = v.get("cwd").and_then(|s| s.as_str()).map(String::from);
            git_branch = v
                .get("gitBranch")
                .and_then(|s| s.as_str())
                .map(String::from);
            entrypoint = v
                .get("entrypoint")
                .and_then(|s| s.as_str())
                .map(String::from);
            version = v
                .get("version")
                .and_then(|s| s.as_str())
                .map(String::from);
        }
        if started_at.is_none() {
            started_at = Some(timestamp.clone());
        }
        ended_at = Some(timestamp.clone());
        let uuid = v
            .get("uuid")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();
        let is_meta = v.get("isMeta").and_then(|b| b.as_bool()).unwrap_or(false);
        let content_val = v.get("message").and_then(|m| m.get("content"));
        if let Some(parsed) = extract_message(msg_type, content_val, is_meta, &timestamp, &uuid) {
            messages.push(parsed);
        }
    }

    Ok(ParsedSession {
        session_id,
        project_path: project_path.to_string(),
        cwd,
        git_branch,
        entrypoint,
        version,
        started_at,
        ended_at,
        messages,
    })
}

fn extract_message(
    msg_type: &str,
    content_val: Option<&Value>,
    is_meta: bool,
    timestamp: &str,
    uuid: &str,
) -> Option<ParsedMessage> {
    let content_val = content_val?;
    match msg_type {
        "user" => content_val.as_str().map(|s| ParsedMessage {
            role: "user".to_string(),
            content: s.to_string(),
            is_meta,
            timestamp: timestamp.to_string(),
            uuid: uuid.to_string(),
        }),
        "assistant" => {
            if let Some(arr) = content_val.as_array() {
                let text_parts: Vec<&str> = arr
                    .iter()
                    .filter(|block| {
                        block.get("type").and_then(|t| t.as_str()) == Some("text")
                    })
                    .filter_map(|block| block.get("text").and_then(|t| t.as_str()))
                    .collect();
                if text_parts.is_empty() {
                    None
                } else {
                    Some(ParsedMessage {
                        role: "assistant".to_string(),
                        content: text_parts.join("\n"),
                        is_meta: false,
                        timestamp: timestamp.to_string(),
                        uuid: uuid.to_string(),
                    })
                }
            } else {
                None
            }
        }
        _ => None,
    }
}
