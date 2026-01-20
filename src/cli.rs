use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "promoteme")]
#[command(about = "Generate brag documents from GitHub contributions", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate brag document from GitHub contributions
    Generate {
        #[arg(long)]
        start_date: Option<String>,

        #[arg(long)]
        end_date: Option<String>,

        #[arg(long)]
        org: Option<String>,

        #[arg(long)]
        repo: Option<String>,

        #[arg(short = 'l', long)]
        language: Option<String>,

        #[arg(short = 'm', long, default_value = "claude")]
        model: String,

        /// Directory with personal notes (.md/.txt) about non-code contributions
        #[arg(long)]
        notes: Option<String>,
    },
}
