use std::process::Command;

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

/// Get all local branches with last commit info
pub fn git_branches_detailed() -> Result<Vec<BranchInfo>, String> {
    let branches_output = run_git_command(&["for-each-ref", "--format=%(refname:short)", "refs/heads/"])?;
    
    let branches: Vec<BranchInfo> = branches_output
        .lines()
        .map(|branch_name| {
            // first commit (creator)
            let creator = run_git_command(&[
                "log",
                "--reverse",
                "--format=%an|%ad",
                "--date=short",
                branch_name,
            ])
            .ok()
            .and_then(|out| out.lines().next().map(|line| line.to_string()))
            .unwrap_or_else(|| "Unknown|Unknown".to_string());

            let mut parts = creator.split('|');
            let author = parts.next().unwrap_or("Unknown").to_string();
            let time_created = parts.next().unwrap_or("Unknown").to_string();

            // last commit
            let last_commit_output = run_git_command(&[
                "log",
                "-1",
                "--format=%ad|%s",
                "--date=short",
                branch_name,
            ])
            .unwrap_or_else(|_| "Unknown|No commit".to_string());

            let mut last_parts = last_commit_output.split('|');
            let last_change = last_parts.next().unwrap_or("Unknown").to_string();
            let last_commit = last_parts.next().unwrap_or("No commit").to_string();

            BranchInfo {
                name: branch_name.to_string(),
                author,
                time_created,
                last_change,
                last_commit,
            }
        })
        .collect();

    Ok(branches)
}

/// Struct for branch info
pub struct BranchInfo {
    pub name: String,
    pub author: String,
    pub time_created: String,
    pub last_change: String,
    pub last_commit: String,
}

