use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(name = "guv")]
#[command(about = "GUV-Code: Right away, Guv'nor.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Configure API keys and authentication
    Auth {
        #[arg(short, long)]
        gemini: Option<String>,
        #[arg(short, long)]
        anthropic: Option<String>,
    },
    /// Manage and view token budget
    Budget {
        /// Set a new budget limit
        #[arg(short, long)]
        limit: Option<f64>,
    },
    /// Start an AI-powered chat session
    Chat {
        /// The message to send
        message: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Auth { gemini, anthropic } => {
            println!("Auth command: gemini={:?}, anthropic={:?}", gemini, anthropic);
        }
        Commands::Budget { limit } => {
            println!("Budget command: limit={:?}", limit);
        }
        Commands::Chat { message } => {
            println!("Chat command: message={:?}", message);
        }
    }

    Ok(())
}
