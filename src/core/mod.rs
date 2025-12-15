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