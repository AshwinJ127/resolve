use rfx::adapters::Repo;
use rfx::core::{self, FileChange};
use serde::Serialize;

#[derive(Serialize)]
struct RepoOverview {
    branches: Vec<UiBranchInfo>,
}

#[derive(Serialize)]
struct UiBranchInfo {
    name: String,
    current: bool,
    ahead: usize,
    behind: usize,
    author: String,
    date: String,
    message: String,
}

#[tauri::command]
fn create_commit(message: String) -> Result<String, String> {
    let repo = Repo::open()?;
    core::create_commit(&repo, &message).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_branch(name: String) -> Result<String, String> {
    let repo = Repo::open()?;
    core::create_branch(&repo, &name).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_pending_changes() -> Result<Vec<FileChange>, String> {
    let repo = Repo::open()?;
    core::get_changed_files(&repo).map_err(|e| e.to_string())
}

#[tauri::command]
fn commit_selection(message: String, files: Vec<String>) -> Result<String, String> {
    if files.is_empty() {
        return Err("No files selected.".to_string());
    }

    let repo = Repo::open()?;
    core::stage_files(&repo, &files).map_err(|e| e.to_string())?;
    core::create_commit(&repo, &message).map_err(|e| e.to_string())
}

#[tauri::command]
fn switch_branch(name: String) -> Result<String, String> {
    let repo = Repo::open()?;
    core::switch_branch(&repo, &name).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_repo_overview() -> Result<RepoOverview, String> {
    let repo = Repo::open()?;
    let all_branches = core::branches_detailed(&repo).map_err(|e| e.to_string())?;

    let status = core::get_status(&repo).map_err(|e| e.to_string())?;

    let ui_branches = all_branches.into_iter().map(|b| {
        let is_current = b.name == status.branch;

        let (ahead, behind) = if is_current {
            (status.ahead.unwrap_or(0), status.behind.unwrap_or(0))
        } else {
            (0, 0)
        };

        UiBranchInfo {
            name: b.name,
            current: is_current,
            ahead,
            behind,
            author: b.author,
            date: b.last_change,
            message: b.last_commit,
        }
    }).collect();

    Ok(RepoOverview { branches: ui_branches })
}

#[tauri::command]
fn get_git_status() -> String {
    let repo = match Repo::open() {
        Ok(repo) => repo,
        Err(e) => return format!("Error: {}", e),
    };
    match core::get_status(&repo) {
        Ok(status) => {
            format!(
                "Branch: {}\nSync: +{} / -{}\nPending Changes: {}",
                status.branch,
                status.ahead.unwrap_or(0),
                status.behind.unwrap_or(0),
                status.changes.len()
            )
        },
        Err(e) => format!("Error: {}", e),
    }
}

#[tauri::command]
fn smart_sync() -> Result<String, String> {
    let repo = Repo::open()?;
    let status = core::get_status(&repo).map_err(|e| e.to_string())?;
    let branch = status.branch;

    match core::pull_specific_branch(&repo, &format!("origin/{}", branch)) {
        Ok(_) => {},
        Err(e) => return Err(format!("Pull failed: {}", e)),
    }

    match core::push_branch(&repo, &branch) {
        Ok(out) => Ok(format!("Sync complete!\n{}", out)),
        Err(e) => Err(format!("Push failed: {}", e)),
    }
}

#[tauri::command]
fn switch_remote(remote: String) -> Result<String, String> {
    let repo = Repo::open()?;
    let branch = core::get_status(&repo).map_err(|e| e.to_string())?.branch;
    core::switch_remote(&repo, &branch, &remote).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_remotes() -> Result<Vec<core::RemoteInfo>, String> {
    let repo = Repo::open()?;
    core::remotes_detailed(&repo).map_err(|e| e.to_string())
}

#[tauri::command]
fn stash_changes() -> Result<String, String> {
    let repo = Repo::open()?;
    core::stash_changes(&repo).map_err(|e| e.to_string())
}

#[tauri::command]
fn pop_stash() -> Result<String, String> {
    let repo = Repo::open()?;
    core::pop_stash(&repo).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_conflict_files() -> Result<Vec<String>, String> {
    let repo = Repo::open()?;
    let files = repo.status_porcelain().map_err(|e| e.to_string())?;
    let conflict_statuses = ["UU", "AA", "DD", "AU", "UA", "DU", "UD"];
    let conflicts = files
        .into_iter()
        .filter(|(status, _)| conflict_statuses.contains(&status.as_str()))
        .map(|(_, path)| path)
        .collect();
    Ok(conflicts)
}

#[tauri::command]
fn abort_merge() -> Result<String, String> {
    let repo = Repo::open()?;
    repo.merge_abort().map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_git_status,
            smart_sync,
            get_repo_overview,
            create_commit,
            create_branch,
            get_pending_changes,
            commit_selection,
            switch_branch,
            switch_remote,
            get_remotes,
            stash_changes,
            pop_stash,
            get_conflict_files,
            abort_merge,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::process::Command;
    use tempfile::TempDir;

    fn temp_repo() -> TempDir {
        let dir = TempDir::new().unwrap();
        let run = |args: &[&str]| {
            let status = Command::new("git").current_dir(dir.path()).args(args).status().unwrap();
            assert!(status.success());
        };
        run(&["init", "-b", "main"]);
        run(&["config", "user.name", "Test User"]);
        run(&["config", "user.email", "test@example.com"]);
        std::fs::write(dir.path().join("README.md"), "init\n").unwrap();
        run(&["add", "README.md"]);
        run(&["commit", "-m", "Initial commit"]);
        dir
    }

    #[test]
    #[serial]
    fn get_conflict_files_returns_files_with_conflict_markers() {
        let dir = temp_repo();
        std::env::set_current_dir(dir.path()).unwrap();

        let run = |args: &[&str]| {
            Command::new("git").current_dir(dir.path()).args(args).status().unwrap()
        };

        // Create a conflicting change on a second branch.
        run(&["checkout", "-b", "feature"]);
        std::fs::write(dir.path().join("README.md"), "feature version\n").unwrap();
        run(&["commit", "-am", "Feature change"]);
        run(&["checkout", "main"]);
        std::fs::write(dir.path().join("README.md"), "main version\n").unwrap();
        run(&["commit", "-am", "Main change"]);
        // This merge will conflict — ignore its exit status.
        let _ = run(&["merge", "feature"]);

        let conflicts = get_conflict_files().expect("get_conflict_files failed");
        assert_eq!(conflicts, vec!["README.md".to_string()]);
    }

    #[test]
    #[serial]
    fn abort_merge_clears_conflict_state() {
        let dir = temp_repo();
        std::env::set_current_dir(dir.path()).unwrap();

        let run = |args: &[&str]| {
            Command::new("git").current_dir(dir.path()).args(args).status().unwrap()
        };

        run(&["checkout", "-b", "feature"]);
        std::fs::write(dir.path().join("README.md"), "feature version\n").unwrap();
        run(&["commit", "-am", "Feature change"]);
        run(&["checkout", "main"]);
        std::fs::write(dir.path().join("README.md"), "main version\n").unwrap();
        run(&["commit", "-am", "Main change"]);
        let _ = run(&["merge", "feature"]);

        abort_merge().expect("abort_merge failed");

        let conflicts = get_conflict_files().expect("get_conflict_files failed");
        assert!(conflicts.is_empty());
    }

    #[test]
    #[serial]
    fn get_repo_overview_attributes_ahead_behind_only_to_current_branch() {
        let dir = temp_repo();
        std::env::set_current_dir(dir.path()).unwrap();

        let run = |args: &[&str]| {
            Command::new("git").current_dir(dir.path()).args(args).status().unwrap()
        };
        run(&["checkout", "-b", "other"]);
        run(&["checkout", "main"]);

        let overview = get_repo_overview().expect("get_repo_overview failed");
        let main_branch = overview.branches.iter().find(|b| b.name == "main").unwrap();
        let other_branch = overview.branches.iter().find(|b| b.name == "other").unwrap();

        assert!(main_branch.current);
        assert!(!other_branch.current);
        assert_eq!(other_branch.ahead, 0);
        assert_eq!(other_branch.behind, 0);
    }
}
