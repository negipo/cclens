use anyhow::Result;
use std::fs;
use std::path::PathBuf;

const SKILLS: &[(&str, &str)] = &[
    ("searching-history", include_str!("../../skills/searching-history/SKILL.md")),
    ("exporting-history", include_str!("../../skills/exporting-history/SKILL.md")),
    ("resuming-from-history", include_str!("../../skills/resuming-from-history/SKILL.md")),
];

pub fn run() -> Result<()> {
    let skills_dir = skills_install_dir();

    for (name, content) in SKILLS {
        let dir = skills_dir.join(format!("cclens-{}", name));
        fs::create_dir_all(&dir)?;
        let path = dir.join("SKILL.md");
        fs::write(&path, content)?;
        eprintln!("Installed: {}", path.display());
    }

    eprintln!("Skills installed to {}", skills_dir.display());
    Ok(())
}

fn skills_install_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".claude")
        .join("skills")
}
