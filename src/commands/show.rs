use crate::db::Database;
use crate::indexer::update_index;
use crate::scope::{default_projects_dir, resolve_project_dirs};
use anyhow::{bail, Result};

pub fn run(session_id: &str) -> Result<()> {
    let db = Database::open()?;
    let cwd = std::env::current_dir()?;
    let projects_dir = default_projects_dir();
    let dirs = resolve_project_dirs(&cwd, &projects_dir, true)?;
    update_index(&db, &dirs)?;

    match db.get_session(session_id)? {
        Some(session) => {
            println!("{}", serde_json::to_string_pretty(&session)?);
        }
        None => {
            bail!("Session not found: {}", session_id);
        }
    }
    Ok(())
}
