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