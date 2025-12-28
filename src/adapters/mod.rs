use std::process::Command;

/// Run a git command and return the trimmed output
/// Use this for almost everything (getting branch names, hashes, etc.)
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

/// Run a git command and return the RAW output (preserving whitespace)
/// Use this ONLY when column alignment matters (like `git status`)
fn run_git_command_raw(args: &[&str]) -> Result<String, String> {
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

/// Get the status of the current Git repository (porcelain format)
pub fn git_status_porcelain() -> Result<Vec<(String, String)>, String> {
    // USE RAW COMMAND HERE
    let output = run_git_command_raw(&["status", "--porcelain=v1"])?;
    
    if output.is_empty() {
        return Ok(Vec::new());
    }

    let files = output
        .lines()
        .map(|line| {
            if line.len() < 4 {
                return ("?".to_string(), line.to_string());
            }
            let (status, path) = line.split_at(3);
            (status.trim().to_string(), path.trim().to_string())
        })
        .collect();

    Ok(files)
}

/// Check if local branch is ahead/behind remote
pub fn git_ahead_behind(branch: &str) -> Result<(usize, usize), String> {
    // "git rev-list --left-right --count HEAD...@{u}"
    let arg = format!("{}...@{{u}}", branch);
    let output = run_git_command(&["rev-list", "--left-right", "--count", &arg])?;

    let parts: Vec<&str> = output.split_whitespace().collect();
    if parts.len() >= 2 {
        let ahead = parts[0].parse().unwrap_or(0);
        let behind = parts[1].parse().unwrap_or(0);
        Ok((ahead, behind))
    } else {
        Ok((0, 0))
    }
}

/// Push the current branch to origin, establishing a tracking link
pub fn git_push_upstream(branch: &str) -> Result<String, String> {
    run_git_command(&["push", "-u", "origin", branch])
}

/// Create and switch to a new branch
pub fn git_create_branch(name: &str) -> Result<String, String> {
    run_git_command(&["checkout", "-b", name])
}

// --- Standard Wrappers (Use trimmed output) ---

pub fn git_branch() -> Result<String, String> {
    run_git_command(&["rev-parse", "--abbrev-ref", "HEAD"])
}

pub fn git_add(files: &[String]) -> Result<String, String> {
    let mut args = vec!["add"];
    args.extend(files.iter().map(|s| s.as_str()));
    run_git_command(&args)
}

pub fn git_add_all() -> Result<String, String> {
    run_git_command(&["add", "-A"])
}

pub fn git_commit(message: &str) -> Result<String, String> {
    run_git_command(&["commit", "-m", message])
}

pub fn git_list_branches() -> Result<Vec<String>, String> {
    let output = run_git_command(&["for-each-ref", "--format=%(refname:short)", "refs/heads/"])?;
    Ok(output.lines().map(|line| line.trim().to_string()).collect())
}

pub fn git_first_commit(branch: &str) -> Result<String, String> {
    let output = run_git_command(&["log", "--reverse", "--format=%an|%ad", "--date=short", branch])?;
    Ok(output.lines().next().unwrap_or("Unknown|Unknown").to_string())
}

pub fn git_last_commit(branch: &str) -> Result<String, String> {
    let output = run_git_command(&["log", "-1", "--format=%ad|%s", "--date=short", branch])?;
    Ok(output.lines().next().unwrap_or("Unknown|No commit").to_string())
}

pub fn git_list_remotes() -> Result<Vec<String>, String> {
    let output = run_git_command(&["remote", "-v"])?;
    Ok(output.lines().map(|line| line.trim().to_string()).collect())
}

pub fn git_list_commits(branch: &str, count: usize) -> Result<Vec<String>, String> {
    let count_arg = format!("-{}", count);
    let output = run_git_command(&["log", &count_arg, "--pretty=format:%h|%an|%ad|%s", "--date=short", branch])?;
    Ok(output.lines().map(|line| line.trim().to_string()).collect())
}