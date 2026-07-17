mod common;

use common::temp_repo;
use rfx::core;

#[test]
fn get_status_reports_clean_working_directory() {
    let (_guard, repo) = temp_repo();

    let status = core::get_status(&repo).expect("get_status failed");

    assert_eq!(status.branch, "main");
    assert!(status.changes.is_empty());
}

#[test]
fn get_status_reports_dirty_working_directory() {
    let (guard, repo) = temp_repo();
    std::fs::write(guard.path().join("new_file.txt"), "hello\n").unwrap();

    let status = core::get_status(&repo).expect("get_status failed");

    assert_eq!(status.changes.len(), 1);
    assert_eq!(status.changes[0].path, "new_file.txt");
    assert_eq!(status.changes[0].status, "??");
}

#[test]
fn branches_detailed_lists_all_local_branches_with_metadata() {
    let (_guard, repo) = temp_repo();
    core::create_branch(&repo, "feature-x").expect("create_branch failed");
    repo.switch_branch("main").expect("switch back to main failed");

    let branches = core::branches_detailed(&repo).expect("branches_detailed failed");
    let names: Vec<&str> = branches.iter().map(|b| b.name.as_str()).collect();

    assert!(names.contains(&"main"));
    assert!(names.contains(&"feature-x"));
    for b in &branches {
        assert_eq!(b.author, "Test User");
        assert_eq!(b.last_commit, "Initial commit");
    }
}

#[test]
fn create_commit_stages_and_commits_end_to_end() {
    let (guard, repo) = temp_repo();
    std::fs::write(guard.path().join("a.txt"), "content\n").unwrap();
    core::stage_all_files(&repo).expect("stage_all_files failed");

    let result = core::create_commit(&repo, "Add a.txt");
    assert!(result.is_ok());

    let commits = core::commits_detailed(&repo, "main", 5).expect("commits_detailed failed");
    assert_eq!(commits[0].message, "Add a.txt");
}

#[test]
fn create_commit_rejects_empty_message() {
    let (_guard, repo) = temp_repo();
    let result = core::create_commit(&repo, "   ");
    assert_eq!(result, Err("Commit message cannot be empty.".to_string()));
}

#[test]
fn create_commit_rejects_too_short_message() {
    let (_guard, repo) = temp_repo();
    let result = core::create_commit(&repo, "ab");
    assert_eq!(result, Err("Commit message is too short.".to_string()));
}

#[test]
fn validate_new_branch_name_rejects_empty() {
    let (_guard, repo) = temp_repo();
    let result = core::validate_new_branch_name(&repo, "");
    assert_eq!(result, Err("Branch name cannot be empty.".to_string()));
}

#[test]
fn validate_new_branch_name_rejects_whitespace() {
    let (_guard, repo) = temp_repo();
    let result = core::validate_new_branch_name(&repo, "has space");
    assert_eq!(result, Err("Branch names cannot contain spaces.".to_string()));
}

#[test]
fn validate_new_branch_name_rejects_existing_branch() {
    let (_guard, repo) = temp_repo();
    let result = core::validate_new_branch_name(&repo, "main");
    assert_eq!(result, Err("A branch named 'main' already exists.".to_string()));
}

#[test]
fn validate_new_branch_name_accepts_new_unique_name() {
    let (_guard, repo) = temp_repo();
    let result = core::validate_new_branch_name(&repo, "feature-y");
    assert!(result.is_ok());
}

#[test]
fn create_branch_then_switch_branch_round_trips() {
    let (_guard, repo) = temp_repo();
    core::create_branch(&repo, "feature-z").expect("create_branch failed");
    // create_branch does `checkout -b`, so we should already be on it.
    assert_eq!(repo.branch().unwrap(), "feature-z");

    core::switch_branch(&repo, "main").expect("switch_branch failed");
    assert_eq!(repo.branch().unwrap(), "main");
}

#[test]
fn stash_changes_then_pop_stash_restores_dirty_file() {
    let (guard, repo) = temp_repo();
    std::fs::write(guard.path().join("README.md"), "wip changes\n").unwrap();

    core::stash_changes(&repo).expect("stash_changes failed");
    let status_after_stash = core::get_status(&repo).expect("get_status failed");
    assert!(status_after_stash.changes.is_empty());

    core::pop_stash(&repo).expect("pop_stash failed");
    let status_after_pop = core::get_status(&repo).expect("get_status failed");
    assert_eq!(status_after_pop.changes.len(), 1);
    assert_eq!(status_after_pop.changes[0].path, "README.md");
}

#[test]
fn undo_last_commit_removes_commit_but_keeps_changes() {
    let (guard, repo) = temp_repo();
    std::fs::write(guard.path().join("b.txt"), "content\n").unwrap();
    core::stage_all_files(&repo).expect("stage_all_files failed");
    core::create_commit(&repo, "Add b.txt").expect("create_commit failed");

    core::undo_last_commit(&repo).expect("undo_last_commit failed");

    let commits = core::commits_detailed(&repo, "main", 5).expect("commits_detailed failed");
    assert_eq!(commits[0].message, "Initial commit");

    let status = core::get_status(&repo).expect("get_status failed");
    assert_eq!(status.changes.len(), 1);
    assert_eq!(status.changes[0].path, "b.txt");
}

#[test]
fn remotes_detailed_parses_added_remote() {
    let (guard, repo) = temp_repo();
    let fake_remote_path = guard.path().join("does-not-need-to-exist");
    std::process::Command::new("git")
        .current_dir(guard.path())
        .args(&["remote", "add", "origin", &format!("https://github.com/AshwinJ127/resolve.git")])
        .status()
        .expect("failed to add remote");
    let _ = fake_remote_path; // path unused beyond documenting intent

    let remotes = core::remotes_detailed(&repo).expect("remotes_detailed failed");
    assert_eq!(remotes.len(), 2); // git remote -v returns both (fetch) and (push)
    assert_eq!(remotes[0].name, "origin");
    assert_eq!(remotes[0].host, Some("github.com".to_string()));
    assert_eq!(remotes[0].owner, Some("AshwinJ127".to_string()));
    assert_eq!(remotes[0].repo, Some("resolve".to_string()));
}
