use anyhow::Result;

pub fn run() -> Result<()> {
    let cache_dir = dirs::cache_dir()
        .ok_or_else(|| anyhow::anyhow!("Cache directory not found"))?
        .join("cclens");
    let db_path = cache_dir.join("index.db");

    if db_path.exists() {
        std::fs::remove_file(&db_path)?;
        eprintln!("Removed {}", db_path.display());
    }

    eprintln!("Rebuilding index...");
    let db = crate::db::Database::open()?;
    let projects_dir = crate::scope::default_projects_dir();
    let dirs = crate::scope::resolve_project_dirs(&projects_dir)?;
    crate::indexer::update_index(&db, &dirs)?;
    eprintln!("Done.");

    Ok(())
}
