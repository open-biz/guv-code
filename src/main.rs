mod config;
mod index;
mod llm;
mod git;
mod terminal;
mod agent_logic;
mod orchestrator;
mod ui;

use clap::{Parser, Subcommand};
use config::Config;
use git::GitManager;
use orchestrator::Orchestrator;
use ui::AgentStepper;
use ui::diff_viewer::DiffViewer;
use agent_logic::AgentMessage;
use std::env;
use owo_colors::OwoColorize;
use miette::{Result, miette};
use tokio::sync::mpsc;
use inquire::Text;

#[derive(Parser)]
#[command(name = "guv")]
#[command(about = "GUV-Code: Right away, Guv'nor.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
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
        /// Save to local .guvcode instead of global config
        #[arg(short, long)]
        local: bool,
    },
    /// Manage and view token budget
    Budget {
        /// Set a new budget limit in USD
        #[arg(short, long)]
        limit: Option<f64>,
        /// View current consumption and limit
        #[arg(short, long)]
        status: bool,
        /// Save to local .guvcode instead of global config
        #[arg(short, long)]
        local: bool,
    },
    /// Start an AI-powered chat session
    Chat {
        /// The message to send
        message: Option<String>,
    },
    /// Undo the last AI edit
    Undo,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut config = Config::load().map_err(|e| miette!("{}", e))?;

    match cli.command {
        Some(Commands::Auth { gemini, anthropic, local }) => {
            if let Some(ref key) = gemini {
                config.keys.gemini = Some(key.clone());
                println!("{} Gemini API key updated.", "✅".green());
            }
            if let Some(ref key) = anthropic {
                config.keys.anthropic = Some(key.clone());
                println!("{} Anthropic API key updated.", "✅".green());
            }
            
            if local {
                config.save_local().map_err(|e| miette!("{}", e))?;
            } else {
                config.save_global().map_err(|e| miette!("{}", e))?;
            }

            if gemini.is_none() && anthropic.is_none() {
                println!("{}:", "Current keys".bold().blue());
                println!("  Gemini:    {}", config.keys.gemini.as_ref().map(|_| "****").unwrap_or("Not set").yellow());
                println!("  Anthropic: {}", config.keys.anthropic.as_ref().map(|_| "****").unwrap_or("Not set").yellow());
            }
        }
        Some(Commands::Budget { limit, status, local }) => {
            if let Some(l) = limit {
                config.budget.limit = l;
                println!("{} Budget limit updated to ${:.2}", "✅".green(), l);
                if local {
                    config.save_local().map_err(|e| miette!("{}", e))?;
                } else {
                    config.save_global().map_err(|e| miette!("{}", e))?;
                }
            }
            if status || limit.is_none() {
                println!("{}:", "Budget Status".bold().blue());
                println!("  Limit:     ${:.2}", config.budget.limit);
                println!("  Consumed:  ${:.2}", config.budget.consumed);
                println!("  Remaining: ${:.2}", (config.budget.limit - config.budget.consumed).green());
            }
        }
        Some(Commands::Undo) => {
            let repo_path = env::current_dir().map_err(|e| miette!("{}", e))?;
            if GitManager::is_repo(&repo_path) {
                GitManager::auto_stage_all(&repo_path).map_err(|e| miette!("{}", e))?;
                GitManager::undo(&repo_path).map_err(|e| miette!("{}", e))?;
                println!("{} Undone last edit.", "✅".green());
            } else {
                println!("{} Not a git repository.", "❌".red());
            }
        }
        Some(Commands::Chat { message: _ }) | None => {
            println!("🎩 {}", "GUV-Code Initialized. Ready for orders, Guv'nor.".bold().cyan());
            
            let repo_path = env::current_dir().map_err(|e| miette!("{}", e))?;

            loop {
                let query = match Text::new("Guv'nor?").prompt() {
                    Ok(m) if !m.trim().is_empty() => m,
                    _ => break,
                };

                if query == "exit" || query == "quit" {
                    break;
                }

                // Check keys only when trying to run a task
                let gemini_key = config.keys.gemini.clone();
                let anthropic_key = config.keys.anthropic.clone();

                if gemini_key.is_none() || anthropic_key.is_none() {
                    println!("{} {}", "⚠".yellow(), "API keys are missing. Please run:".bold());
                    println!("  guv-code auth --gemini <key> --anthropic <key>");
                    println!("  (Add --local to save in this project only)");
                    continue;
                }

                let orchestrator = Orchestrator::new(
                    repo_path.clone(), 
                    gemini_key.unwrap(), 
                    anthropic_key.unwrap()
                );

                let (ui_tx, mut ui_rx) = mpsc::channel(100);
                let mut stepper = AgentStepper::new();
                
                let query_clone = query.clone();
                let orchestrator_clone = orchestrator.clone();
                let orch_handle = tokio::spawn(async move {
                    orchestrator_clone.run(query_clone, ui_tx).await
                });

                while let Some(msg) = ui_rx.recv().await {
                    stepper.handle_message(msg.clone());
                    
                    if let AgentMessage::CoderCompleted(file, patch) = msg {
                        let full_path = repo_path.join(&file);
                        let old_content = std::fs::read_to_string(&full_path).unwrap_or_default();
                        
                        DiffViewer::show(&file, &old_content, &patch);
                        
                        if Text::new("Apply this patch, Guv'nor? (y/n)").prompt().unwrap_or_default() == "y" {
                             if GitManager::is_repo(&repo_path) {
                                let _ = GitManager::auto_stage_all(&repo_path);
                             }
                             let _ = std::fs::write(full_path, patch);
                             if GitManager::is_repo(&repo_path) {
                                let _ = GitManager::commit(&repo_path, &format!("guv: {}", query));
                             }
                             println!("{} {}", "✅".green(), "Applied and committed.".dimmed());
                        }
                    }
                }

                let _ = orch_handle.await;
            }
        }
    }

    Ok(())
}
