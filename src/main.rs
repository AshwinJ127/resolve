use clap::{Parser, Subcommand, ValueEnum};

mod adapters;
mod core;
mod ui;

/// Resolve CLI - rfx
#[derive(Parser)]
#[command(name = "rfx")]
#[command(about = "A beginner-friendly tool to fix Git workflow issues", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Pull changes safely
    Pull,
    /// Push changes safely
    Push,
    /// Attempt to fix issues automatically
    Fix,

    Show {
        #[arg(value_enum)]
        entity: ShowEntity,

        /// Output as JSON instead of table
        #[arg(long)]
        json: bool,

        /// Branch name for commits (optional)
        #[arg(long, default_value = "main")]
        branch: String,

        /// Number of commits to show
        #[arg(long, default_value_t = 10)]
        count: usize,
    },

    /// Create something new (commit, branch, etc.)
    New {
        #[command(subcommand)]
        entity: NewEntity,
    },    

}

#[derive(ValueEnum, Clone)]
enum ShowEntity {
    Branches,
    Remotes,
    Commits,
}

#[derive(Subcommand)]
enum NewEntity {
    /// Create a new commit
    Commit,

}



fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Pull => {
            println!("rfx pull: running Smart Pull...");
            match adapters::git_branch() {
                Ok(branch) => println!("Current branch:\n{}", branch),
                Err(err) => eprintln!("Error: {}", err),
            }
        }
        Commands::Push => {

        }
        Commands::Fix => {
            
        }
        Commands::Show { entity, json, branch, count } => match entity {
            ShowEntity::Branches => ui::show_branches(json),
            ShowEntity::Remotes => ui::show_remotes(json),
            ShowEntity::Commits => ui::show_commits(&branch, count, json),
        },
        Commands::New { entity } => match entity {
            NewEntity::Commit => ui::new_commit(),
            //NewEntity::Branch => ui::new_branch(), // stub for now
        },
    }
}
