use rfx::adapters::Repo;
use std::process::Command;
use tempfile::TempDir;

/// Creates a fresh git repo in a temp dir with one initial commit on `main`,
/// and returns (guard, repo). Keep the guard alive for the test's duration —
/// dropping it deletes the directory.
pub fn temp_repo() -> (TempDir, Repo) {
    let dir = TempDir::new().expect("failed to create temp dir");
    let path = dir.path();

    run(path, &["init", "-b", "main"]);
    run(path, &["config", "user.name", "Test User"]);
    run(path, &["config", "user.email", "test@example.com"]);

    std::fs::write(path.join("README.md"), "init\n").expect("failed to write README");
    run(path, &["add", "README.md"]);
    run(path, &["commit", "-m", "Initial commit"]);

    let repo = Repo::open_at(path);
    (dir, repo)
}

/// Creates a bare repo (usable as a fake "origin") in its own temp dir.
pub fn bare_repo() -> TempDir {
    let dir = TempDir::new().expect("failed to create temp dir");
    run(dir.path(), &["init", "--bare", "-b", "main"]);
    dir
}

fn run(dir: &std::path::Path, args: &[&str]) {
    let status = Command::new("git")
        .current_dir(dir)
        .args(args)
        .status()
        .expect("failed to run git");
    assert!(status.success(), "git {:?} failed in {:?}", args, dir);
}
