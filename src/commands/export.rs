use crate::db::Database;
use crate::indexer::update_index;
use crate::renderer::{render_line, RenderedMessage};
use crate::scope::{default_projects_dir, resolve_project_dirs};
use anyhow::{bail, Result};
use serde_json::Value;
use std::fmt::Write as _;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn run(session_id: &str) -> Result<()> {
    let db = Database::open()?;
    let projects_dir = default_projects_dir();
    let dirs = resolve_project_dirs(&projects_dir)?;
    update_index(&db, &dirs)?;

    let markdown = render_session_markdown(&db, session_id)?;
    print!("{}", markdown);
    Ok(())
}

pub fn render_session_markdown(db: &Database, session_id: &str) -> Result<String> {
    let session = db.get_session(session_id)?;
    if session.is_none() {
        bail!("Session not found: {}", session_id);
    }
    let session = session.unwrap();

    let source_file = db.get_source_file(session_id)?;
    if source_file.is_none() {
        bail!("Source file not found for session: {}", session_id);
    }
    let source_file = source_file.unwrap();

    let mut out = String::new();
    writeln!(out, "# Session {}", session.session_id)?;
    writeln!(out)?;
    if let Some(ref branch) = session.git_branch {
        writeln!(out, "- Branch: {}", branch)?;
    }
    if let Some(ref started) = session.started_at {
        writeln!(out, "- Started: {}", started)?;
    }
    if let Some(ref ended) = session.ended_at {
        writeln!(out, "- Ended: {}", ended)?;
    }
    writeln!(out)?;

    let file = File::open(&source_file)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let v: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        match render_line(&v) {
            Some(RenderedMessage::User(content)) => {
                writeln!(out, "---")?;
                writeln!(out)?;
                for l in content.lines() {
                    writeln!(out, "> {}", l)?;
                }
                writeln!(out)?;
            }
            Some(RenderedMessage::Assistant(content)) => {
                let mut lines = content.lines();
                if let Some(first) = lines.next() {
                    if needs_separate_marker(first) {
                        writeln!(out, "❋\n")?;
                        writeln!(out, "{}", first)?;
                    } else {
                        writeln!(out, "❋ {}", first)?;
                    }
                }
                for l in lines {
                    writeln!(out, "{}", l)?;
                }
                writeln!(out)?;
            }
            None => {}
        }
    }

    Ok(out)
}

fn needs_separate_marker(line: &str) -> bool {
    let md_prefixes = ["```", "#", "- ", "* ", "> ", "| "];
    if md_prefixes.iter().any(|p| line.starts_with(p)) {
        return true;
    }
    line.starts_with(|c: char| c.is_ascii_digit())
        && line.contains(". ")
}
