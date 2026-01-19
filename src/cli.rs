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
        /// Start date (YYYY-MM-DD). Default: 6 months ago
        #[arg(long)]
        start_date: Option<String>,

        /// End date (YYYY-MM-DD). Default: today
        #[arg(long)]
        end_date: Option<String>,

        /// Filter by organization(s), comma-separated
        #[arg(long)]
        org: Option<String>,

        /// Filter by repository(s), comma-separated (format: owner/repo)
        #[arg(long)]
        repo: Option<String>,

        /// Output language for the Brag Document (e.g., 'English', 'Portuguese')
        #[arg(short = 'l', long)]
        language: Option<String>,

        /// Specify the AI model to use (default: 'gemini')
        #[arg(short = 'm', long, default_value = "gemini")]
        model: String,

        /// Directory with personal notes (.md/.txt) about non-code contributions
        #[arg(long)]
        notes: Option<String>,
    },
}
