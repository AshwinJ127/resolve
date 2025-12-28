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
    
    Status,

    Undo,
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

    /// Create a new branch
    Branch,
}



fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Pull => ui::pull(),
        Commands::Push => ui::push(),
        Commands::Show { entity, json, branch, count } => match entity {
            ShowEntity::Branches => ui::show_branches(json),
            ShowEntity::Remotes => ui::show_remotes(json),
            ShowEntity::Commits => ui::show_commits(&branch, count, json),
        },
        Commands::New { entity } => match entity {
            NewEntity::Commit => ui::new_commit(),
            NewEntity::Branch => ui::new_branch(), 
        },
        Commands::Status => ui::show_status(),
        Commands::Undo => ui::undo(),
    }
}
