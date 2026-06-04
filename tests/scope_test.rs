use cclens::scope::resolve_project_dirs;

#[test]
fn test_resolve_project_dirs_lists_all() {
    let tmp = tempfile::tempdir().unwrap();
    let projects_dir = tmp.path().join("projects");
    std::fs::create_dir_all(&projects_dir).unwrap();

    for name in [
        "-Users-test-src-github-com-org-repo",
        "-Users-test-src-github-com-org-other",
        "-Users-test-src-github-com-org-third",
    ] {
        std::fs::create_dir_all(projects_dir.join(name)).unwrap();
    }
    std::fs::write(projects_dir.join("stray-file.txt"), "").unwrap();

    let dirs = resolve_project_dirs(&projects_dir).unwrap();
    assert_eq!(dirs.len(), 3);
}
