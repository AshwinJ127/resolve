use crate::adapters::BranchInfo;
use prettytable::{Table, Row, Cell, format};

pub fn show_branches() {
    match crate::adapters::git_branches_detailed() {
        Ok(branches) => {
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
                let branch = if b.name.len() > 10 { format!("{}…", &b.name[..9]) } else { b.name.clone() };
                let author = if b.author.len() > 15 { format!("{}…", &b.author[..14]) } else { b.author.clone() };
                let commit_msg = if b.last_commit.len() > 25 { format!("{}…", &b.last_commit[..24]) } else { b.last_commit.clone() };

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
        Err(err) => eprintln!("Error retrieving branches: {}", err),
    }
}
