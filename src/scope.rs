use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

fn cwd_to_project_dir_name(cwd: &Path) -> String {
    let s = cwd.to_string_lossy();
    format!(
        "-{}",
        s.trim_start_matches('/').replace('/', "-").replace('.', "-")
    )
}

pub fn resolve_project_dirs(cwd: &Path, projects_dir: &Path, all: bool) -> Result<Vec<PathBuf>> {
    let mut dirs = Vec::new();
    let entries = fs::read_dir(projects_dir)?;

    if all {
        for entry in entries {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                dirs.push(entry.path());
            }
        }
        return Ok(dirs);
    }

    let project_prefix = cwd_to_project_dir_name(cwd);
    for entry in entries {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str == project_prefix || name_str.starts_with(&format!("{}-", project_prefix)) {
            dirs.push(entry.path());
        }
    }
    Ok(dirs)
}

pub fn list_session_files(project_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(project_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
            files.push(path);
        }
    }
    Ok(files)
}

pub fn default_projects_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".claude")
        .join("projects")
}
