use crate::db::Database;
use crate::parser::parse_session_file;
use crate::scope::list_session_files;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub fn update_index(db: &Database, project_dirs: &[PathBuf]) -> Result<(usize, usize)> {
    let mut indexed = 0;
    let mut skipped = 0;

    for dir in project_dirs {
        let project_path = dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let files = list_session_files(dir)?;

        for file in files {
            let file_str = file.to_string_lossy().to_string();
            let mtime = get_mtime(&file)?;

            if let Some(stored_mtime) = db.get_source_mtime(&file_str)? {
                if stored_mtime == mtime {
                    skipped += 1;
                    continue;
                }
            }

            match parse_session_file(&file, &project_path) {
                Ok(session) => {
                    if !session.messages.is_empty() {
                        db.upsert_session(&session, &file_str, mtime)?;
                    }
                    indexed += 1;
                }
                Err(_) => {
                    continue;
                }
            }
        }
    }

    Ok((indexed, skipped))
}

fn get_mtime(path: &Path) -> Result<i64> {
    let metadata = std::fs::metadata(path)?;
    let modified = metadata.modified()?;
    let duration = modified.duration_since(std::time::UNIX_EPOCH)?;
    Ok(duration.as_secs() as i64)
}
