use rfx::core;
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
}

#[tauri::command]
fn get_repo_overview() -> Result<RepoOverview, String> {
    let all_branches = core::branches_detailed().map_err(|e| e.to_string())?;
    
    let status = core::get_status().map_err(|e| e.to_string())?;

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
        }
    }).collect();

    Ok(RepoOverview { branches: ui_branches })
}

#[tauri::command]
fn get_git_status() -> String {
    match core::get_status() {
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
    let status = core::get_status().map_err(|e| e.to_string())?;
    let branch = status.branch;

    match core::pull_specific_branch(&format!("origin/{}", branch)) {
        Ok(_) => {},
        Err(e) => return Err(format!("Pull failed: {}", e)),
    }

    match core::push_branch(&branch) {
        Ok(out) => Ok(format!("Sync complete!\n{}", out)),
        Err(e) => Err(format!("Push failed: {}", e)),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_git_status, 
            smart_sync,
            get_repo_overview
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}