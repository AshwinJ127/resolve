use std::process::Command;
use serde::Serialize;

/// Run a git command and return Result<String, String>
fn run_git_command(args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute git: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// Get the status of the current Git repository
pub fn git_status() -> Result<String, String> {
    run_git_command(&["status"])
}

/// Get the current branch name
pub fn git_branch() -> Result<String, String> {
    run_git_command(&["rev-parse", "--abbrev-ref", "HEAD"])
}

/// Pull changes from the remote for the current branch
pub fn git_pull() -> Result<String, String> {
    run_git_command(&["pull"])
}

/// Push changes to the remote for the current branch
pub fn git_push() -> Result<String, String> {
    run_git_command(&["push"])
}

/// Stash uncommitted changes
pub fn git_stash() -> Result<String, String> {
    run_git_command(&["stash", "push", "-u"])
}

/// Apply the latest stash
pub fn git_stash_apply() -> Result<String, String> {
    run_git_command(&["stash", "pop"])
}

/// Show a summary of changes
pub fn git_diff() -> Result<String, String> {
    run_git_command(&["diff", "--stat"])
}

/// Fetch updates from remote without merging
pub fn git_fetch() -> Result<String, String> {
    run_git_command(&["fetch"])
}

pub fn git_list_branches() -> Result<Vec<String>, String> {
    let output = run_git_command(&["for-each-ref", "--format=%(refname:short)", "refs/heads/"])?;
    let branches = output
        .lines()
        .map(|line| line.trim().to_string())
        .collect();
    Ok(branches)
}

pub fn git_first_commit(branch: &str) -> Result<String, String> {
    let output = run_git_command(&[
        "log",
        "--reverse",
        "--format=%an|%ad",
        "--date=short",
        branch,
    ])?;
    Ok(output.lines().next().unwrap_or("Unknown|Unknown").to_string())
}

pub fn git_last_commit(branch: &str) -> Result<String, String> {
    let output = run_git_command(&[
        "log",
        "-1",
        "--format=%ad|%s",
        "--date=short",
        branch,
    ])?;
    Ok(output.lines().next().unwrap_or("Unknown|No commit").to_string())
}

// List remotes
pub fn git_list_remotes() -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .args(["remote", "-v"])
        .output()
        .map_err(|e| format!("Failed to execute git: {}", e))?;

    if output.status.success() {
        let remotes = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|line| line.to_string())
            .collect();
        Ok(remotes)
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

// List commits for a branch
pub fn git_list_commits(branch: &str, count: usize) -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .args(["log", &format!("-{}", count), "--pretty=format:%h|%an|%ad|%s", "--date=short", branch])
        .output()
        .map_err(|e| format!("Failed to execute git: {}", e))?;

    if output.status.success() {
        let commits = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|line| line.to_string())
            .collect();
        Ok(commits)
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

