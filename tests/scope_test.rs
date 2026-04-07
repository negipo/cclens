use cclens::scope::resolve_project_dirs;
use std::path::Path;

#[test]
fn test_resolve_project_dirs_finds_worktrees() {
    let tmp = tempfile::tempdir().unwrap();
    let projects_dir = tmp.path().join("projects");
    std::fs::create_dir_all(&projects_dir).unwrap();

    let main_dir = projects_dir.join("-Users-test-src-github-com-org-repo");
    std::fs::create_dir_all(&main_dir).unwrap();
    std::fs::write(main_dir.join("session-1.jsonl"), "").unwrap();

    let wt_dir = projects_dir.join("-Users-test-src-github-com-org-repo-worktree-branch");
    std::fs::create_dir_all(&wt_dir).unwrap();
    std::fs::write(wt_dir.join("session-2.jsonl"), "").unwrap();

    let other_dir = projects_dir.join("-Users-test-src-github-com-org-other");
    std::fs::create_dir_all(&other_dir).unwrap();
    std::fs::write(other_dir.join("session-3.jsonl"), "").unwrap();

    let cwd = Path::new("/Users/test/src/github.com/org/repo");
    let dirs = resolve_project_dirs(cwd, &projects_dir, false).unwrap();
    assert_eq!(dirs.len(), 2);

    let all_dirs = resolve_project_dirs(cwd, &projects_dir, true).unwrap();
    assert_eq!(all_dirs.len(), 3);
}
