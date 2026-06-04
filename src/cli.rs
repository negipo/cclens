use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cclens", about = "Claude Code conversation history search")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Query {
        text: Option<String>,
        #[arg(long)]
        branch: Option<String>,
        #[arg(long)]
        after: Option<String>,
        #[arg(long)]
        before: Option<String>,
        #[arg(long, default_value = "20")]
        limit: usize,
        #[arg(long, short = 't')]
        table: bool,
    },
    Show {
        session_id: String,
    },
    Export {
        session_id: String,
    },
    Install,
    Reindex,
}
