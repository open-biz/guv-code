mod config;

use clap::{Parser, Subcommand};
use anyhow::Result;
use config::Config;

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
        /// Set Gemini API key
        #[arg(short, long)]
        gemini: Option<String>,
        /// Set Anthropic API key
        #[arg(short, long)]
        anthropic: Option<String>,
    },
    /// Manage and view token budget
    Budget {
        /// Set a new budget limit in USD
        #[arg(short, long)]
        limit: Option<f64>,
        /// View current consumption and limit
        #[arg(short, long)]
        status: bool,
    },
    /// Start an AI-powered chat session
    Chat {
        /// The message to send
        message: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut config = Config::load()?;

    match cli.command {
        Commands::Auth { gemini, anthropic } => {
            if let Some(key) = gemini {
                config.keys.gemini = Some(key);
                println!("Gemini API key updated.");
            }
            if let Some(key) = anthropic {
                config.keys.anthropic = Some(key);
                println!("Anthropic API key updated.");
            }
            config.save()?;
            if gemini.is_none() && anthropic.is_none() {
                println!("Current keys: Gemini: {:?}, Anthropic: {:?}", 
                    config.keys.gemini.as_ref().map(|_| "****"),
                    config.keys.anthropic.as_ref().map(|_| "****")
                );
            }
        }
        Commands::Budget { limit, status } => {
            if let Some(l) = limit {
                config.budget.limit = l;
                println!("Budget limit updated to ${:.2}", l);
                config.save()?;
            }
            if status || limit.is_none() {
                println!("Budget Status:");
                println!("  Limit:     ${:.2}", config.budget.limit);
                println!("  Consumed:  ${:.2}", config.budget.consumed);
                println!("  Remaining: ${:.2}", config.budget.limit - config.budget.consumed);
            }
        }
        Commands::Chat { message } => {
            println!("Chat command: message={:?}", message);
        }
    }

    Ok(())
}
