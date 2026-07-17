mod common;

use common::{bare_repo, temp_repo};
use rfx::core;
use std::process::Command;

fn add_origin(repo_path: &std::path::Path, origin_path: &std::path::Path) {
    let status = Command::new("git")
        .current_dir(repo_path)
        .args(&["remote", "add", "origin", origin_path.to_str().unwrap()])
        .status()
        .expect("failed to add origin");
    assert!(status.success());
}

#[test]
fn push_branch_establishes_upstream_tracking() {
    let (guard, repo) = temp_repo();
    let origin = bare_repo();
    add_origin(guard.path(), origin.path());

    let result = core::push_branch(&repo, "main");
    assert!(result.is_ok(), "push failed: {:?}", result);

    let (ahead, behind) = repo.ahead_behind("main").expect("ahead_behind failed");
    assert_eq!((ahead, behind), (0, 0));
}

#[test]
fn ahead_behind_reports_ahead_after_local_commit_not_yet_pushed() {
    let (guard, repo) = temp_repo();
    let origin = bare_repo();
    add_origin(guard.path(), origin.path());
    core::push_branch(&repo, "main").expect("initial push failed");

    std::fs::write(guard.path().join("c.txt"), "content\n").unwrap();
    core::stage_all_files(&repo).expect("stage_all_files failed");
    core::create_commit(&repo, "Add c.txt").expect("create_commit failed");

    let (ahead, behind) = repo.ahead_behind("main").expect("ahead_behind failed");
    assert_eq!((ahead, behind), (1, 0));
}

#[test]
fn pull_specific_branch_brings_in_remote_commits() {
    let (guard_a, repo_a) = temp_repo();
    let origin = bare_repo();
    add_origin(guard_a.path(), origin.path());
    core::push_branch(&repo_a, "main").expect("push from repo_a failed");

    // A second clone of the same origin, simulating a teammate's machine.
    let guard_b = tempfile::TempDir::new().unwrap();
    let clone_status = Command::new("git")
        .args(&["clone", origin.path().to_str().unwrap(), guard_b.path().to_str().unwrap()])
        .status()
        .expect("clone failed");
    assert!(clone_status.success());
    Command::new("git")
        .current_dir(guard_b.path())
        .args(&["config", "user.name", "Test User"])
        .status()
        .unwrap();
    Command::new("git")
        .current_dir(guard_b.path())
        .args(&["config", "user.email", "test@example.com"])
        .status()
        .unwrap();
    let repo_b = rfx::adapters::Repo::open_at(guard_b.path());

    // repo_a makes a new commit and pushes it.
    std::fs::write(guard_a.path().join("d.txt"), "content\n").unwrap();
    core::stage_all_files(&repo_a).expect("stage_all_files failed");
    core::create_commit(&repo_a, "Add d.txt").expect("create_commit failed");
    core::push_branch(&repo_a, "main").expect("second push failed");

    // repo_b pulls it.
    let pull_result = core::pull_specific_branch(&repo_b, "origin/main");
    assert!(pull_result.is_ok(), "pull failed: {:?}", pull_result);

    let commits = core::commits_detailed(&repo_b, "main", 5).expect("commits_detailed failed");
    assert_eq!(commits[0].message, "Add d.txt");
}
