use crate::commands::query::{open_indexed_db, print_results};
use anyhow::Result;

pub fn run(
    branch: Option<String>,
    after: Option<String>,
    before: Option<String>,
    limit: Option<usize>,
    json: bool,
) -> Result<()> {
    let (db, project_paths) = open_indexed_db()?;
    let project_refs: Vec<&str> = project_paths.iter().map(|s| s.as_str()).collect();

    let results = db.list_sessions(
        &project_refs,
        branch.as_deref(),
        after.as_deref(),
        before.as_deref(),
        limit.unwrap_or(30),
        80,
    )?;

    print_results(&results, json)
}
