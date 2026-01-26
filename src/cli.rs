use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "promoteme")]
#[command(about = "Generate brag documents from GitHub or Azure DevOps contributions", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum, Debug)]
pub enum Source {
    Github,
    Azuredevops,
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Source::Github => write!(f, "github"),
            Source::Azuredevops => write!(f, "azuredevops"),
        }
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate brag document from GitHub or Azure DevOps contributions
    Generate {
        #[arg(long, value_enum, default_value = "github")]
        source: Source,

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

        #[arg(long)]
        notes: Option<String>,
    },
}
