use crate::adapters;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct BranchInfo {
    pub name: String,
    pub author: String,
    pub time_created: String,
    pub last_change: String,
    pub last_commit: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct CommitInfo {
    pub hash: String,
    pub author: String,
    pub date: String,
    pub message: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct RemoteInfo {
    pub name: String,
    pub url: String,
    pub direction: String,

    // Derived (best-effort)
    pub host: Option<String>,
    pub owner: Option<String>,
    pub repo: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct RemoteBranchInfo {
    pub full_name: String, // e.g. origin/main
    pub short_name: String, // e.g. main
    pub author: String,
    pub date: String,
}

/// Represents a file change in a commit
#[derive(Clone, Debug, Serialize)]
pub struct FileChange {
    pub status: String, // e.g., "M", "??", "D"
    pub path: String,
}

/// Summary of the repository status
#[derive(Serialize)]
pub struct StatusSummary {
    pub branch: String,
    pub ahead: Option<usize>,
    pub behind: Option<usize>,
    pub changes: Vec<FileChange>,
}

/// Get the current status summary
pub fn get_status() -> Result<StatusSummary, String> {
    let branch = adapters::git_branch()?;
    let changes = get_changed_files()?;
    
    let (ahead, behind) = match adapters::git_ahead_behind(&branch) {
        Ok((a, b)) => (Some(a), Some(b)),
        Err(_) => (None, None),
    };

    Ok(StatusSummary {
        branch,
        ahead,
        behind,
        changes,
    })
}

/// List branches with detailed info
pub fn branches_detailed() -> Result<Vec<BranchInfo>, String> {
    let branch_names = adapters::git_list_branches()?;

    let branches: Vec<BranchInfo> = branch_names
        .into_iter()
        .map(|branch| {
            // First commit (creator info)
            let first_commit = adapters::git_first_commit(&branch)
                .unwrap_or_else(|_| "Unknown|Unknown".to_string());
            let mut parts = first_commit.split('|');
            let author = parts.next().unwrap_or("Unknown").to_string();
            let time_created = parts.next().unwrap_or("Unknown").to_string();

            // Last commit info
            let last_commit = adapters::git_last_commit(&branch)
                .unwrap_or_else(|_| "Unknown|No commit".to_string());
            let mut last_parts = last_commit.split('|');
            let last_change = last_parts.next().unwrap_or("Unknown").to_string();
            let last_commit_msg = last_parts.next().unwrap_or("No commit").to_string();

            BranchInfo {
                name: branch,
                author,
                time_created,
                last_change,
                last_commit: last_commit_msg,
            }
        })
        .collect();

    Ok(branches)
}


fn parse_remote_url(url: &str) -> (Option<String>, Option<String>, Option<String>) {
    // HTTPS: https://github.com/owner/repo.git
    if let Some(stripped) = url.strip_prefix("https://") {
        let parts: Vec<&str> = stripped.split('/').collect();
        if parts.len() >= 3 {
            return (
                Some(parts[0].to_string()),
                Some(parts[1].to_string()),
                Some(parts[2].trim_end_matches(".git").to_string()),
            );
        }
    }

    // SSH: git@github.com:owner/repo.git
    if let Some(stripped) = url.strip_prefix("git@") {
        let parts: Vec<&str> = stripped.split(':').collect();
        if parts.len() == 2 {
            let host = parts[0].to_string();
            let path: Vec<&str> = parts[1].split('/').collect();
            if path.len() == 2 {
                return (
                    Some(host),
                    Some(path[0].to_string()),
                    Some(path[1].trim_end_matches(".git").to_string()),
                );
            }
        }
    }

    (None, None, None)
}

/// List remotes with detailed info
pub fn remotes_detailed() -> Result<Vec<RemoteInfo>, String> {
    let raw_remotes = adapters::git_list_remotes()?;

    let remotes = raw_remotes
        .into_iter()
        .filter_map(|line| {
            // Example line:
            // origin https://github.com/user/repo.git (fetch)
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 3 {
                return None;
            }

            let name = parts[0].to_string();
            let url = parts[1].to_string();
            let direction = parts[2]
                .trim_start_matches('(')
                .trim_end_matches(')')
                .to_string();

            let (host, owner, repo) = parse_remote_url(&url);

            Some(RemoteInfo {
                name,
                url,
                direction,
                host,
                owner,
                repo,
            })
        })
        .collect();

    Ok(remotes)
}

/// List commits with detailed info
pub fn commits_detailed(branch: &str, count: usize) -> Result<Vec<CommitInfo>, String> {
    let raw_commits = crate::adapters::git_list_commits(branch, count)?;
    let commits: Vec<CommitInfo> = raw_commits.into_iter().map(|line| {
        let parts: Vec<&str> = line.split('|').collect();
        CommitInfo {
            hash: parts.get(0).unwrap_or(&"").to_string(),
            author: parts.get(1).unwrap_or(&"").to_string(),
            date: parts.get(2).unwrap_or(&"").to_string(),
            message: parts.get(3).unwrap_or(&"").to_string(),
        }
    }).collect();
    Ok(commits)
}

// Get list of changed files
pub fn get_changed_files() -> Result<Vec<FileChange>, String> {
    let raw_files = adapters::git_status_porcelain()?;
    
    let changes = raw_files
        .into_iter()
        .map(|(status, path)| FileChange { status, path })
        .collect();

    Ok(changes)
}

// Stage specific files
pub fn stage_files(files: &[String]) -> Result<String, String> {
    if files.is_empty() {
        return Ok("No files to stage".to_string());
    }
    adapters::git_add(files)
}

// Stage all files
pub fn stage_all_files() -> Result<String, String> {
    adapters::git_add_all()
}

// Create commit with message
pub fn create_commit(message: &str) -> Result<String, String> {
    let msg = message.trim();

    if msg.is_empty() {
        return Err("Commit message cannot be empty.".to_string());
    }
    
    // We can keep the length check here as a business rule
    if msg.len() < 3 {
        return Err("Commit message is too short.".to_string());
    }

    adapters::git_commit(msg)
}

/// Check if a branch name is valid and available
pub fn validate_new_branch_name(name: &str) -> Result<(), String> {
    let name = name.trim();

    // 1. Basic Syntax Rules
    if name.is_empty() {
        return Err("Branch name cannot be empty.".to_string());
    }
    if name.contains(char::is_whitespace) {
        return Err("Branch names cannot contain spaces.".to_string());
    }

    // 2. Check Existence
    let existing_branches = adapters::git_list_branches()?;
    if existing_branches.iter().any(|b| b == name) {
        return Err(format!("A branch named '{}' already exists.", name));
    }

    Ok(())
}

/// Create and switch to a new branch
pub fn create_branch(name: &str) -> Result<String, String> {
    adapters::git_create_branch(name)
}

/// Pull changes safely
/*
pub fn pull_changes() -> Result<String, String> {
    // 1. Safety Check: Ensure working directory is clean
    let changes = get_changed_files()?;
    if !changes.is_empty() {
        return Err("You have uncommitted changes. Please commit or stash them before pulling.".to_string());
    }

    // 2. Execute Pull
    adapters::git_pull()
}
*/

/// Get list of remote branches with details
pub fn get_remote_branches() -> Result<Vec<RemoteBranchInfo>, String> {
    // 1. Fetch first
    let _ = adapters::git_fetch(); 

    // 2. Get list
    let raw = adapters::git_list_remote_branches()?;
    
    let branches = raw.into_iter().filter_map(|line| {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() < 3 { return None; }
        
        let full_name = parts[0].to_string();
        
        // --- FILTERS ---
        if full_name.ends_with("/HEAD") || full_name == "HEAD" {
            return None;
        }
        
        if !full_name.contains('/') {
            return None;
        }

        let short_name = full_name.splitn(2, '/').nth(1).unwrap_or(&full_name).to_string();

        Some(RemoteBranchInfo {
            full_name,
            short_name,
            author: parts[1].to_string(),
            date: parts[2].to_string(),
        })
    }).collect();

    Ok(branches)
}

/// Execute the pull for a specific branch
pub fn pull_specific_branch(branch_full_name: &str) -> Result<String, String> {
    adapters::git_pull_branch(branch_full_name)
}

/// Push changes to the remote
/*
pub fn push_changes() -> Result<String, String> {
    let branch = adapters::git_branch()?;
    adapters::git_push_upstream(&branch)
}
*/
pub fn push_branch(branch_name: &str) -> Result<String, String> {
    adapters::git_push_upstream(branch_name)
}

/// Undo the last commit
pub fn undo_last_commit() -> Result<String, String> {
    // We strictly undo 1 commit
    adapters::git_reset_soft(1)
}