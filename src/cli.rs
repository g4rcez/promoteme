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

        /// Custom output directory (default: artifacts/{username}_{timestamp}Z). If provided, used as-is (no artifacts/ prefix).
        #[arg(long)]
        cwd: Option<String>,

        #[arg(long)]
        team: bool,

        /// Comma-separated GitHub usernames (required when --team is set)
        #[arg(long)]
        members: Option<String>,

        #[arg(long)]
        setup: bool,
    },
    /// Track and analyze job interviews
    Interview {
        #[command(subcommand)]
        command: InterviewCommands,
    },
}

#[derive(Subcommand)]
pub enum InterviewCommands {
    /// Initialize interview tracking for a company
    Init {
        company: String,
    },
    /// Create a new interview step
    New {
        step: u32,

        #[arg(long)]
        company: String,

        #[arg(long)]
        title: Option<String>,

        #[arg(long)]
        start_teleprompter: bool,
    },
    /// Generate an AI summary for a completed interview step
    Summarize {
        #[arg(long)]
        company: String,

        #[arg(long)]
        step: u32,

        #[arg(short = 'm', long, default_value = "claude")]
        model: String,

        #[arg(short = 'l', long)]
        language: Option<String>,
    },
    /// Analyze your interview progression over time
    Progression {
        #[arg(long)]
        company: Option<String>,

        #[arg(long)]
        start_date: Option<String>,

        #[arg(long)]
        end_date: Option<String>,

        #[arg(short = 'm', long, default_value = "claude")]
        model: String,

        #[arg(short = 'l', long)]
        language: Option<String>,
    },
}
