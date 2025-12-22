use prettytable::{Table, Row, Cell, format};
use serde::Serialize;

use crate::core::{remotes_detailed, commits_detailed, CommitInfo, BranchInfo};

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


pub fn print_branches_json(branches: &[BranchInfo]) {
    match serde_json::to_string_pretty(branches) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("Failed to serialize branches: {}", e),
    }
}


pub fn show_remotes(json: bool) {
    match remotes_detailed() {
        Ok(remotes) => {
            if json {
                match serde_json::to_string_pretty(&remotes) {
                    Ok(j) => println!("{}", j),
                    Err(e) => eprintln!("Failed to serialize remotes: {}", e),
                }
            } else {
                for r in remotes {
                    println!("{}", r);
                }
            }
        }
        Err(e) => eprintln!("Error fetching remotes: {}", e),
    }
}

#[derive(Serialize)]
struct CommitDisplay {
    hash: String,
    author: String,
    date: String,
    message: String,
}

pub fn show_commits(branch: &str, count: usize, json: bool) {
    match commits_detailed(branch, count) {
        Ok(commits) => {
            if json {
                let display: Vec<CommitDisplay> = commits
                    .into_iter()
                    .map(|c| CommitDisplay {
                        hash: c.hash,
                        author: c.author,
                        date: c.date,
                        message: c.message,
                    })
                    .collect();
                match serde_json::to_string_pretty(&display) {
                    Ok(j) => println!("{}", j),
                    Err(e) => eprintln!("Failed to serialize commits: {}", e),
                }
            } else {
                let mut table = Table::new();
                table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
                table.set_titles(Row::new(vec![
                    Cell::new("Hash"),
                    Cell::new("Author"),
                    Cell::new("Date"),
                    Cell::new("Message"),
                ]));

                for c in commits {
                    table.add_row(Row::new(vec![
                        Cell::new(&c.hash),
                        Cell::new(&c.author),
                        Cell::new(&c.date),
                        Cell::new(&c.message),
                    ]));
                }

                table.printstd();
            }
        }
        Err(e) => eprintln!("Error fetching commits: {}", e),
    }
}
