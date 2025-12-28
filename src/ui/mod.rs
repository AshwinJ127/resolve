use prettytable::{Table, Row, Cell, format};
use serde::Serialize;

use crate::core::{remotes_detailed, commits_detailed, BranchInfo};

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

