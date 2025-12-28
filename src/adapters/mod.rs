use std::process::Command;

/// Run a git command and return Result<String, String>
fn run_git_command(args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute git: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
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

/// Commit changes with a message
pub fn git_commit(message: &str) -> Result<String, String> {
    run_git_command(&["commit", "-m", message])
}

/// Create and switch to a new branch
pub fn git_create_branch(name: &str) -> Result<String, String> {
    run_git_command(&["checkout", "-b", name])
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

/// List branches
pub fn git_list_branches() -> Result<Vec<String>, String> {
    let output = run_git_command(&["for-each-ref", "--format=%(refname:short)", "refs/heads/"])?;
    let branches = output
        .lines()
        .map(|line| line.trim().to_string())
        .collect();
    Ok(branches)
}

/// Get the first commit info for a branch
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

/// Get the last commit info for a branch
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
    let output = run_git_command(&["remote", "-v"])?;
    let remotes = output
        .lines()
        .map(|line| line.trim().to_string())
        .collect();
    Ok(remotes)
}


// List commits for a branch
pub fn git_list_commits(branch: &str, count: usize) -> Result<Vec<String>, String> {
    let count_arg = format!("-{}", count);
    let output = run_git_command(&[
        "log",
        &count_arg,
        "--pretty=format:%h|%an|%ad|%s",
        "--date=short",
        branch,
    ])?;
    let commits = output
        .lines()
        .map(|line| line.trim().to_string())
        .collect();
    Ok(commits)
}

/// Get a list of changed files with their status code
/// Returns a tuple of (Status Code, File Path)
pub fn git_status_porcelain() -> Result<Vec<(String, String)>, String> {
    let output = run_git_command(&["status", "--porcelain=v1"])?;
    
    if output.is_empty() {
        return Ok(Vec::new());
    }

    let files = output
        .lines()
        .map(|line| {
            // --porcelain output is fixed: first 2 chars are status, then space, then path
            // Example: " M src/main.rs" or "?? newfile.txt"
            let (status, path) = line.split_at(3);
            (status.trim().to_string(), path.trim().to_string())
        })
        .collect();

    Ok(files)
}

/// Stage specific files
pub fn git_add(files: &[String]) -> Result<String, String> {
    let mut args = vec!["add"];
    args.extend(files.iter().map(|s| s.as_str()));
    run_git_command(&args)
}

/// Stage all files
pub fn git_add_all() -> Result<String, String> {
    run_git_command(&["add", "-A"])
}


