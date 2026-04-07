use regex::Regex;
use serde_json::Value;

pub enum RenderedMessage {
    User(String),
    Assistant(String),
}

pub fn render_line(v: &Value) -> Option<RenderedMessage> {
    let msg_type = v.get("type").and_then(|t| t.as_str())?;
    let is_meta = v.get("isMeta").and_then(|b| b.as_bool()).unwrap_or(false);
    if is_meta {
        return None;
    }
    let content_val = v.get("message").and_then(|m| m.get("content"))?;

    match msg_type {
        "user" => render_user(content_val),
        "assistant" => render_assistant(content_val),
        _ => None,
    }
}

fn render_user(content_val: &Value) -> Option<RenderedMessage> {
    let raw = if let Some(s) = content_val.as_str() {
        s.to_string()
    } else if let Some(arr) = content_val.as_array() {
        render_user_array(arr)
    } else {
        return None;
    };

    let cleaned = clean_user_content(&raw);
    let trimmed = cleaned.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(RenderedMessage::User(trimmed.to_string()))
}

fn render_user_array(arr: &[Value]) -> String {
    let launch_re = Regex::new(r"Launching skill: (.+)").unwrap();
    let mut parts = Vec::new();
    for block in arr {
        let btype = block.get("type").and_then(|t| t.as_str()).unwrap_or("");
        match btype {
            "tool_result" => {
                if let Some(text) = block.get("content").and_then(|c| c.as_str()) {
                    if let Some(caps) = launch_re.captures(text) {
                        parts.push(format!("⚙ {}", &caps[1]));
                    }
                }
            }
            "text" => {
                if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                    parts.push(text.to_string());
                }
            }
            _ => {}
        }
    }
    parts.join("\n")
}

fn render_assistant(content_val: &Value) -> Option<RenderedMessage> {
    let arr = content_val.as_array()?;
    let text_parts: Vec<&str> = arr
        .iter()
        .filter(|block| block.get("type").and_then(|t| t.as_str()) == Some("text"))
        .filter_map(|block| block.get("text").and_then(|t| t.as_str()))
        .collect();
    if text_parts.is_empty() {
        return None;
    }
    let joined = text_parts.join("\n");
    let trimmed = joined.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(RenderedMessage::Assistant(trimmed.to_string()))
}

const NOISE_TAGS: &[&str] = &[
    "command-message",
    "local-command-caveat",
    "system-reminder",
    "user-prompt-submit-hook",
];

fn extract_tag(input: &str, tag: &str) -> Option<String> {
    let pattern = format!(r"(?s)<{0}>(.*?)</{0}>", regex::escape(tag));
    Regex::new(&pattern)
        .ok()
        .and_then(|re| re.captures(input).map(|c| c[1].trim().to_string()))
}

fn strip_tag(input: &str, tag: &str) -> String {
    let pattern = format!(r"(?s)<{0}>.*?</{0}>\s*", regex::escape(tag));
    match Regex::new(&pattern) {
        Ok(re) => re.replace_all(input, "").to_string(),
        Err(_) => input.to_string(),
    }
}

fn clean_user_content(input: &str) -> String {
    let command_name = extract_tag(input, "command-name");
    let command_args = extract_tag(input, "command-args");

    let mut result = input.to_string();
    for tag in &["command-name", "command-message", "command-args"] {
        result = strip_tag(&result, tag);
    }
    for tag in NOISE_TAGS {
        result = strip_tag(&result, tag);
    }

    let launch_re = Regex::new(r"Launching skill: (.+)").unwrap();
    let mut skill_lines = Vec::new();
    let mut other_lines = Vec::new();

    if let Some(name) = command_name {
        match command_args {
            Some(args) if !args.is_empty() => skill_lines.push(format!("{} {}", name, args)),
            _ => skill_lines.push(name),
        }
    }

    for line in result.trim().lines() {
        if let Some(caps) = launch_re.captures(line) {
            skill_lines.push(format!("⚙ {}", &caps[1]));
        } else if !line.trim().is_empty() {
            other_lines.push(line.to_string());
        }
    }

    let mut parts = Vec::new();
    parts.extend(skill_lines);
    parts.extend(other_lines);
    parts.join("\n")
}
