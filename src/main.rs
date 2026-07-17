use rfx::{adapters, core};
use clap::{Parser, Subcommand, ValueEnum};
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

    /// Switch to a different branch or remote
    Switch {
        #[command(subcommand)]
        entity: SwitchEntity,
    },
}

#[derive(Subcommand)]
enum SwitchEntity {
    /// Switch to a different branch
    Branch,
    /// Switch to a different remote
    Remote,
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
    let repo = match adapters::Repo::open() {
        Ok(repo) => repo,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    match cli.command {
        Commands::Pull => ui::pull(&repo),
        Commands::Push => ui::push(&repo),
        Commands::Show { entity, json, branch, count } => match entity {
            ShowEntity::Branches => ui::show_branches(&repo, json),
            ShowEntity::Remotes => ui::show_remotes(&repo, json),
            ShowEntity::Commits => ui::show_commits(&repo, &branch, count, json),
        },
        Commands::New { entity } => match entity {
            NewEntity::Commit => ui::new_commit(&repo),
            NewEntity::Branch => ui::new_branch(&repo),
        },
        Commands::Status => ui::show_status(&repo),
        Commands::Undo => ui::undo(&repo),
        Commands::Switch { entity } => match entity {
            SwitchEntity::Branch => ui::switch_branch(&repo),
            SwitchEntity::Remote => ui::switch_remote(&repo),
        },
    }
}
