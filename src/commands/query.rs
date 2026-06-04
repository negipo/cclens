use crate::db::Database;
use crate::indexer::update_index;
use crate::models::QueryResult;
use crate::scope::{default_projects_dir, resolve_project_dirs};
use anyhow::Result;

pub fn format_started_at(started_at: &Option<String>) -> String {
    started_at
        .as_deref()
        .map(|s| s.chars().take(16).collect())
        .unwrap_or_default()
}

pub fn project_label(result: &QueryResult) -> String {
    let raw = result
        .cwd
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or(&result.project_path);
    std::path::Path::new(raw)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(raw)
        .to_string()
}

pub fn format_snippet(result: &QueryResult) -> String {
    match result.matches.first() {
        Some(m) => m.snippet.replace('\n', " ").chars().take(40).collect(),
        None => String::new(),
    }
}

pub fn render_table(results: &[QueryResult]) -> String {
    let header = ["STARTED", "PROJECT", "SESSION", "SNIPPET"];
    let mut rows: Vec<[String; 4]> = vec![header.map(String::from)];
    for r in results {
        rows.push([
            format_started_at(&r.started_at),
            project_label(r),
            r.session_id.clone(),
            format_snippet(r),
        ]);
    }

    let mut widths = [0usize; 3];
    for row in &rows {
        for i in 0..3 {
            widths[i] = widths[i].max(row[i].chars().count());
        }
    }

    let mut out = String::new();
    for row in &rows {
        let mut line = String::new();
        for i in 0..3 {
            line.push_str(&row[i]);
            let pad = widths[i] - row[i].chars().count() + 2;
            line.push_str(&" ".repeat(pad));
        }
        line.push_str(&row[3]);
        out.push_str(line.trim_end());
        out.push('\n');
    }
    out
}

pub fn run(
    text: Option<String>,
    branch: Option<String>,
    after: Option<String>,
    before: Option<String>,
    limit: usize,
    json: bool,
) -> Result<()> {
    let db = Database::open()?;
    let projects_dir = default_projects_dir();
    let dirs = resolve_project_dirs(&projects_dir)?;

    update_index(&db, &dirs)?;

    let project_paths: Vec<String> = dirs
        .iter()
        .filter_map(|d| d.file_name().and_then(|n| n.to_str()).map(String::from))
        .collect();
    let project_refs: Vec<&str> = project_paths.iter().map(|s| s.as_str()).collect();

    let results = if let Some(ref text) = text {
        db.search_sessions(text, &project_refs, limit)?
    } else {
        db.list_sessions(
            &project_refs,
            branch.as_deref(),
            after.as_deref(),
            before.as_deref(),
            limit,
        )?
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&results)?);
    } else {
        print!("{}", render_table(&results));
    }
    Ok(())
}
