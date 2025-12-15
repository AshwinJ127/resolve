use clap::{Parser, Subcommand};

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

        #[arg(long)]
        json: bool,
    },

}

#[derive(clap::ValueEnum, Clone)]
enum ShowEntity {
    Branches,
    Remotes,
    Commits,
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
        Commands::Show { entity, json } => match entity {
            ShowEntity::Branches => ui::show_branches(json),
            ShowEntity::Remotes => println!("Show remotes not implemented yet"),
            ShowEntity::Commits => println!("Show commits not implemented yet"),
        },
    }
}
