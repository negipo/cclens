mod cli;

use clap::Parser;
use cli::{Cli, Commands};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Query {
            text,
            branch,
            after,
            before,
            limit,
            json,
        } => {
            cclens::commands::query::run(text, branch, after, before, limit, json)?;
        }
        Commands::List {
            branch,
            after,
            before,
            limit,
            json,
        } => {
            cclens::commands::list::run(branch, after, before, limit, json)?;
        }
        Commands::Show { session_id } => {
            cclens::commands::show::run(&session_id)?;
        }
        Commands::Export { session_id } => {
            cclens::commands::export::run(&session_id)?;
        }
        Commands::Install => {
            cclens::commands::install::run()?;
        }
        Commands::Reindex => {
            cclens::commands::reindex::run()?;
        }
        Commands::Browse => {
            cclens::commands::browse::run()?;
        }
    }

    Ok(())
}
