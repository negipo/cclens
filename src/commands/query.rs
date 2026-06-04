use crate::db::Database;
use crate::indexer::update_index;
use crate::scope::{default_projects_dir, resolve_project_dirs};
use anyhow::Result;

pub fn run(
    text: Option<String>,
    branch: Option<String>,
    after: Option<String>,
    before: Option<String>,
    limit: usize,
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

    println!("{}", serde_json::to_string_pretty(&results)?);
    Ok(())
}
