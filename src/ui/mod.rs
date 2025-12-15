use crate::adapters::BranchInfo;
use prettytable::{Table, Row, Cell, format};

pub fn show_branches(json: bool) {
    let branches = match crate::adapters::git_branches_detailed() {
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
