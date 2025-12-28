use prettytable::{Table, Row, Cell, format};
use serde::Serialize;
use std::io;
use inquire::{Confirm, MultiSelect, Text, validator::Validation, Select};

use crate::core::{BranchInfo, CommitInfo, RemoteInfo, branches_detailed, 
    commits_detailed, remotes_detailed, create_commit, get_changed_files, 
    stage_all_files, stage_files,
    validate_new_branch_name, create_branch,
    get_status, pull_changes, get_remote_branches, pull_specific_branch,
};

/// Display branches in a table or JSON
pub fn show_branches(json: bool) {
    let branches = match crate::core::branches_detailed() {
        Ok(b) => b,
        Err(err) => {
            eprintln!("Error retrieving branches: {}", err);
            return;
        }
    };

    if json {
        print_branches_json(&branches);
        return;
    }

    let mut table = Table::new();

    // Compact format: header line only
    table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

    table.set_titles(Row::new(vec![
        Cell::new("Branch"),
        Cell::new("Author"),
        Cell::new("Created"),
        Cell::new("Last Change"),
        Cell::new("Last Commit"),
    ]));

    for b in branches {
        let branch = if b.name.len() > 10 { format!("{}…", &b.name[..9]) } else { b.name };
        let author = if b.author.len() > 15 { format!("{}…", &b.author[..14]) } else { b.author };
        let commit_msg = if b.last_commit.len() > 25 { format!("{}…", &b.last_commit[..24]) } else { b.last_commit };

        table.add_row(Row::new(vec![
            Cell::new(&branch),
            Cell::new(&author),
            Cell::new(&b.time_created),
            Cell::new(&b.last_change),
            Cell::new(&commit_msg),
        ]));
    }

    table.printstd();
}

/// Print branches as JSON
pub fn print_branches_json(branches: &[BranchInfo]) {
    match serde_json::to_string_pretty(branches) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("Failed to serialize branches: {}", e),
    }
}

/// Display remotes in a table or JSON
pub fn show_remotes(json: bool) {
    let remotes = match remotes_detailed() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error fetching remotes: {}", e);
            return;
        }
    };

    if json {
        match serde_json::to_string_pretty(&remotes) {
            Ok(j) => println!("{}", j),
            Err(e) => eprintln!("Failed to serialize remotes: {}", e),
        }
        return;
    }

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

    table.set_titles(Row::new(vec![
        Cell::new("Name"),
        Cell::new("Direction"),
        Cell::new("Host"),
        Cell::new("Owner"),
        Cell::new("Repo"),
    ]));

    for r in remotes {
        let owner = r.owner.unwrap_or_else(|| "-".into());
        let repo = r.repo.unwrap_or_else(|| "-".into());
        let host = r.host.unwrap_or_else(|| "-".into());

        table.add_row(Row::new(vec![
            Cell::new(&r.name),
            Cell::new(&r.direction),
            Cell::new(&host),
            Cell::new(&owner),
            Cell::new(&repo),
        ]));
    }

    table.printstd();
}


#[derive(Serialize)]
struct CommitDisplay {
    hash: String,
    author: String,
    date: String,
    message: String,
}

/// Display commits in a table or JSON
pub fn show_commits(branch: &str, count: usize, json: bool) {
    let commits = match commits_detailed(branch, count) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error fetching commits: {}", e);
            return;
        }
    };

    if json {
        match serde_json::to_string_pretty(&commits) {
            Ok(j) => println!("{}", j),
            Err(e) => eprintln!("Failed to serialize commits: {}", e),
        }
        return;
    }

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

    table.set_titles(Row::new(vec![
        Cell::new("Hash"),
        Cell::new("Author"),
        Cell::new("Date"),
        Cell::new("Message"),
    ]));

    for c in commits {
        let hash = if c.hash.len() > 7 { &c.hash[..7] } else { &c.hash };
        let author = if c.author.len() > 15 {
            format!("{}…", &c.author[..14])
        } else {
            c.author
        };
        let message = if c.message.len() > 30 {
            format!("{}…", &c.message[..29])
        } else {
            c.message
        };

        table.add_row(Row::new(vec![
            Cell::new(hash),
            Cell::new(&author),
            Cell::new(&c.date),
            Cell::new(&message),
        ]));
    }

    table.printstd();
}

/// Create a new commit with user-provided message
pub fn new_commit() {
    // 1. Get current status via Core
    let changes = match get_changed_files() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to check status: {}", e);
            return;
        }
    };

    if changes.is_empty() {
        println!("Working directory is clean. Nothing to commit.");
        return;
    }

    // 2. Display changes
    println!("\nChanged files:");
    for file in &changes {
        let label = match file.status.as_str() {
            "??" => "[New]",
            "M" => "[Mod]",
            "D" => "[Del]",
            _ => "[...]",
        };
        println!("  {} {}", label, file.path);
    }
    println!();

    // 3. Ask: Commit everything?
    let commit_all = Confirm::new("Do you want to commit all changes?")
        .with_default(true)
        .prompt();

    match commit_all {
        Ok(true) => {
            if let Err(e) = stage_all_files() {
                eprintln!("Error staging files: {}", e);
                return;
            }
        }
        Ok(false) => {
            // 4. Interactive Selection
            let file_options: Vec<String> = changes
                .iter()
                .map(|f| f.path.clone())
                .collect();

            let selected_files = MultiSelect::new("Select files to include (Space to toggle):", file_options)
                .with_page_size(10)
                .prompt();

            match selected_files {
                Ok(files) if files.is_empty() => {
                    println!("No files selected. Aborting commit.");
                    return;
                }
                Ok(files) => {
                    if let Err(e) = stage_files(&files) {
                        eprintln!("Error staging files: {}", e);
                        return;
                    }
                }
                Err(_) => {
                    println!("Selection cancelled.");
                    return;
                }
            }
        }
        Err(_) => return,
    }

    // 5. Prompt for Message
    let message_prompt = Text::new("Commit message:")
        .with_validator(|input: &str| {
            if input.trim().len() < 3 {
                Ok(Validation::Invalid("Message is too short.".into()))
            } else {
                Ok(Validation::Valid)
            }
        })
        .prompt();

    match message_prompt {
        Ok(msg) => {
            match create_commit(msg.trim()) {
                Ok(out) => {
                    println!("\nSuccess! Commit created.");
                    // Only show the summary line from git output
                    if let Some(line) = out.lines().next() {
                         println!("{}", line);
                    }
                }
                Err(e) => eprintln!("\nError committing: {}", e),
            }
        }
        Err(_) => println!("Commit cancelled."),
    }
}

pub fn new_branch() {
    // 1. Prompt for Name
    let name_prompt = Text::new("Name for new branch:")
        .with_validator(|input: &str| {
            match validate_new_branch_name(input) {
                Ok(_) => Ok(Validation::Valid),
                Err(msg) => Ok(Validation::Invalid(msg.into())),
            }
        })
        .prompt();

    let name = match name_prompt {
        Ok(n) => n.trim().to_string(),
        Err(_) => { println!("Cancelled."); return; }
    };

    // 2. Check for Uncommitted Changes (The "Error" Prevention)
    let changes = match get_changed_files() {
        Ok(c) => c,
        Err(_) => Vec::new(), 
    };

    if !changes.is_empty() {
        println!("\nWarning: You have uncommitted changes.");
        println!("   If you create a new branch now, these changes will move with you.");
        
        let count = changes.len();
        if count <= 5 {
            for file in changes {
                println!("   - {}", file.path);
            }
        } else {
            println!("   - {} files changed...", count);
        }
        println!();

        let confirm = Confirm::new("Do you want to proceed and carry these changes over?")
            .with_default(false)
            .prompt();

        match confirm {
            Ok(true) => (),
            _ => {
                println!("Cancelled. Please commit or stash your changes first.");
                return;
            }
        }
    }

    // 3. Execute
    match create_branch(&name) {
        Ok(_) => {
            println!("\nSuccess! New branch '{}' created.", name);
            println!("   You have been switched to this branch automatically.");
        },
        Err(e) => eprintln!("\nError creating branch: {}", e),
    }
}

pub fn show_status() {
    let status = match get_status() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error getting status: {}", e);
            return;
        }
    };

    println!("\nBranch: {}", status.branch);

    // 1. Sync Status Logic
    match (status.ahead, status.behind) {
        (Some(0), Some(0)) => println!("Status: Up to date with remote"),
        (Some(a), Some(b)) => {
            if a > 0 { println!("Status: {} commit(s) ahead (Needs Push)", a); }
            if b > 0 { println!("Status: {} commit(s) behind (Needs Pull)", b); }
        }
        (None, None) => {
            println!("Status: Not published (Local only)");
        }
        _ => {}, 
    }
    println!();

    // 2. File Status
    if status.changes.is_empty() {
        println!("Working directory is clean.");
    } else {
        println!("Unsaved Changes:");
        for file in status.changes {
            let label = match file.status.as_str() {
                "??" => "[New]",
                "M" | "M " => "[Mod]",
                "D" | "D " => "[Del]",
                _ => "[...]",
            };
            println!("  {} {}", label, file.path);
        }
        println!("\nTip: Use 'rfx new commit' to save these.");
    }
    println!();
}

pub fn pull() {
    // --- STEP 1: SAFETY CHECK (The "Action Prompt") ---
    loop {
        let changes = match get_changed_files() {
            Ok(c) => c,
            Err(_) => Vec::new(),
        };

        if changes.is_empty() {
            break;
        }

        println!("\nYou have uncommitted changes:");
        for file in changes.iter().take(5) {
            println!("   - {}", file.path);
        }
        if changes.len() > 5 { println!("   ...and {} more.", changes.len() - 5); }
        println!();

        let options = vec!["Commit changes now", "Cancel"];
        let choice = Select::new("What would you like to do?", options).prompt();

        match choice {
            Ok("Commit changes now") => {
                new_commit(); 
            }
            _ => {
                println!("Pull cancelled.");
                return;
            }
        }
    }

    // --- STEP 2: BRANCH SELECTION ---
    println!("\nFetching latest updates from remote...");
    
    let branches = match get_remote_branches() {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error fetching branches: {}", e);
            return;
        }
    };

    if branches.is_empty() {
        println!("No remote branches found. (Are you connected to the internet?)");
        return;
    }

    let options: Vec<String> = branches.iter().map(|b| {
        format!("{: <15} | {: <15} | {}", b.short_name, b.author, b.date)
    }).collect();

    let selection = Select::new("Select branch to pull from:", options)
        .with_page_size(10)
        .prompt();

    let selected_branch = match selection {
        Ok(s) => {
            let index = branches.iter().position(|b| {
                 let fmt = format!("{: <15} | {: <15} | {}", b.short_name, b.author, b.date);
                 fmt == s
            }).unwrap();
            &branches[index]
        }
        Err(_) => {
            println!("Cancelled.");
            return;
        }
    };

    // --- STEP 3: EXECUTE ---
    println!("\n⬇ Pulling from '{}'...", selected_branch.full_name);

    match pull_specific_branch(&selected_branch.full_name) {
        Ok(out) => {
            if out.contains("Already up to date") {
                 println!("Already up to date.");
            } else {
                 println!("Success! Updates received.");
                 println!("{}", out);
            }
        }
        Err(e) => {
            if e.to_lowercase().contains("conflict") {
                eprintln!("\nMerge Conflict Detected:");
                eprintln!("   We downloaded the code, but couldn't combine it automatically.");
                eprintln!("   Please open the conflicting files and resolve the issues.");
            } else {
                eprintln!("\nError pulling:");
                eprintln!("{}", e);
            }
        }
    }
}