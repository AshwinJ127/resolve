use rfx::core; 

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_git_status])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}