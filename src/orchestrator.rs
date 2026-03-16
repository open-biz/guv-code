use crate::agent_logic::{AgentMessage, AgentPhase, ToolStatus};
use crate::agent_logic::planner::PlannerAgent;
use crate::agent_logic::coder::CoderAgent;
use crate::agent_logic::reviewer::ReviewerAgent;
use crate::terminal::TerminalManager;
use crate::llm::{GeminiProvider, AnthropicProvider, OpenRouterProvider, ModelProvider};
use crate::config::{Config, Provider};
use crate::index::RepoIndex;
use tokio::sync::mpsc;
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;

/// Create the appropriate ModelProvider based on the current Config.
pub fn create_provider(config: &Config) -> Result<Arc<dyn ModelProvider>> {
    let api_key = config.active_api_key()
        .ok_or_else(|| anyhow::anyhow!(
            "No API key for {} — use /auth to sign in or set a key",
            config.model.provider
        ))?
        .to_string();

    let model_id = &config.model.model_id;

    Ok(match config.model.provider {
        Provider::Google => {
            let mut provider = GeminiProvider::new(api_key).with_model(model_id);
            // Detect OAuth tokens (start with "ya29.") and use Bearer auth
            if let Some(key) = config.active_api_key() {
                if key.starts_with("ya29.") {
                    provider = provider.with_bearer_auth();
                }
            }
            Arc::new(provider)
        }
        Provider::Anthropic => Arc::new(
            AnthropicProvider::new(api_key).with_model(model_id),
        ),
        Provider::OpenRouter => Arc::new(
            OpenRouterProvider::new(api_key).with_model(model_id),
        ),
    })
}

#[derive(Clone)]
pub struct Orchestrator {
    repo_path: PathBuf,
    provider: Arc<dyn ModelProvider>,
}

impl Orchestrator {
    pub fn new(repo_path: PathBuf, provider: Arc<dyn ModelProvider>) -> Self {
        Self { repo_path, provider }
    }

    pub fn from_config(repo_path: PathBuf, config: &Config) -> Result<Self> {
        let provider = create_provider(config)?;
        Ok(Self { repo_path, provider })
    }

    pub async fn run(
        &self,
        query: String,
        ui_sender: mpsc::Sender<AgentMessage>,
        mut response_rx: mpsc::Receiver<AgentMessage>,
    ) -> Result<()> {
        let (agent_tx, mut agent_rx) = mpsc::channel(100);

        let provider = self.provider.clone();

        let repo_path = self.repo_path.clone();
        let query_clone = query.clone();
        let agent_tx_planner = agent_tx.clone();
        let provider_planner = provider.clone();
        let ui_sender_idx = ui_sender.clone();

        // Start Orchestration Task
        tokio::spawn(async move {
            // ── Indexing Phase ───────────────────────────────────────
            let _ = ui_sender_idx.send(AgentMessage::IndexingStarted).await;
            let _ = ui_sender_idx.send(AgentMessage::PhaseChange(AgentPhase::Mapping)).await;

            let mut index = RepoIndex::load_or_create(&repo_path).unwrap();
            let cache_existed = repo_path.join(".guv/cache/index.bin").exists();
            let _ = index.update(&repo_path);
            let _ = index.save(&repo_path);

            let _ = ui_sender_idx.send(AgentMessage::IndexingCompleted).await;

            if cache_existed {
                let _ = ui_sender_idx.send(AgentMessage::Thinking(
                    format!("Memory Hit: index cache loaded ({} files)", index.files.len()),
                )).await;
            } else {
                let _ = ui_sender_idx.send(AgentMessage::Thinking(
                    format!("Fresh index: {} files scanned", index.files.len()),
                )).await;
            }

            // ── Planning Phase ──────────────────────────────────────
            let _ = ui_sender_idx.send(AgentMessage::PhaseChange(AgentPhase::Planning)).await;
            let _ = ui_sender_idx.send(AgentMessage::ToolStarted {
                name: "planner".into(),
                description: "Analyzing codebase to identify target files".into(),
            }).await;

            let planner = PlannerAgent::new(provider_planner.as_ref(), agent_tx_planner.clone());
            if let Err(e) = planner.plan(&index, &query_clone).await {
                let _ = ui_sender_idx.send(AgentMessage::ToolCompleted {
                    name: "planner".into(),
                    status: ToolStatus::Error,
                }).await;
                let _ = agent_tx_planner.send(AgentMessage::Error(e.to_string())).await;
                return;
            }

            let _ = ui_sender_idx.send(AgentMessage::ToolCompleted {
                name: "planner".into(),
                status: ToolStatus::Success,
            }).await;
        });

        // Pending shell approval state: (command, file_being_reviewed)
        let mut pending_shell: Option<(String, String)> = None;

        loop {
            tokio::select! {
                // Messages from agents
                agent_msg = agent_rx.recv() => {
                    let msg = match agent_msg {
                        Some(m) => m,
                        None => break,
                    };

                    // Forward everything to UI
                    let _ = ui_sender.send(msg.clone()).await;

                    match msg {
                        AgentMessage::PlanCompleted(files) => {
                            let repo_path = self.repo_path.clone();
                            let query_clone = query.clone();
                            let provider_coder = provider.clone();
                            let agent_tx_coder = agent_tx.clone();
                            let ui_sender_c = ui_sender.clone();

                            tokio::spawn(async move {
                                let _ = ui_sender_c.send(AgentMessage::PhaseChange(AgentPhase::Coding)).await;
                                let _ = ui_sender_c.send(AgentMessage::ToolStarted {
                                    name: "coder".into(),
                                    description: format!("Editing {} file(s)", files.len()),
                                }).await;

                                let coder = CoderAgent::new(provider_coder.as_ref(), agent_tx_coder);
                                let _ = coder.code(&repo_path, &query_clone, files).await;

                                let _ = ui_sender_c.send(AgentMessage::ToolCompleted {
                                    name: "coder".into(),
                                    status: ToolStatus::Success,
                                }).await;
                            });
                        }
                        AgentMessage::CoderCompleted(file, _patch) => {
                            let repo_path = self.repo_path.clone();
                            let agent_tx_reviewer = agent_tx.clone();
                            let ui_sender_r = ui_sender.clone();
                            let file_clone = file.clone();

                            tokio::spawn(async move {
                                let _ = ui_sender_r.send(AgentMessage::PhaseChange(AgentPhase::Reviewing)).await;
                                let _ = ui_sender_r.send(AgentMessage::ToolStarted {
                                    name: "reviewer".into(),
                                    description: format!("Verifying {}", file_clone),
                                }).await;

                                let reviewer = ReviewerAgent::new(agent_tx_reviewer);
                                let _ = reviewer.review(&repo_path, &file_clone).await;
                            });
                        }
                        AgentMessage::ShellRequested { ref command, .. } => {
                            // Stash the pending shell info so we know what to execute
                            // once the UI responds with Approved/Denied.
                            // Try to figure out which file is being reviewed from agent_logs
                            let review_file = pending_shell
                                .as_ref()
                                .map(|(_, f)| f.clone())
                                .unwrap_or_default();
                            pending_shell = Some((command.clone(), review_file));
                        }
                        AgentMessage::ReviewStarted(ref file) => {
                            pending_shell = Some(("".into(), file.clone()));
                        }
                        _ => {}
                    }
                }

                // Responses from UI (approval gate)
                response_msg = response_rx.recv() => {
                    let msg = match response_msg {
                        Some(m) => m,
                        None => break,
                    };

                    match msg {
                        AgentMessage::ShellApproved(cmd) => {
                            let _ = ui_sender.send(AgentMessage::ShellApproved(cmd.clone())).await;

                            let file = pending_shell.take().map(|(_, f)| f).unwrap_or_default();
                            let repo_path = self.repo_path.clone();
                            let ui_s = ui_sender.clone();
                            let agent_tx_done = agent_tx.clone();

                            // Execute the approved command and stream output
                            tokio::spawn(async move {
                                let _ = ui_s.send(AgentMessage::ToolStarted {
                                    name: format!("shell: {}", cmd),
                                    description: "Running approved command".into(),
                                }).await;

                                match TerminalManager::run_command(&repo_path, "cargo", &["check"]) {
                                    Ok(result) => {
                                        // Stream stdout lines
                                        for line in result.stdout.lines() {
                                            let _ = ui_s.send(AgentMessage::ToolOutput {
                                                name: format!("shell: {}", cmd),
                                                line: line.to_string(),
                                            }).await;
                                        }
                                        // Stream stderr lines
                                        for line in result.stderr.lines() {
                                            let _ = ui_s.send(AgentMessage::ToolOutput {
                                                name: format!("shell: {}", cmd),
                                                line: line.to_string(),
                                            }).await;
                                        }

                                        let exit_code = if result.success { 0 } else { 1 };
                                        let _ = ui_s.send(AgentMessage::ShellCompleted { exit_code }).await;

                                        let _ = ui_s.send(AgentMessage::ToolCompleted {
                                            name: format!("shell: {}", cmd),
                                            status: if result.success { ToolStatus::Success } else { ToolStatus::Error },
                                        }).await;

                                        if result.success {
                                            let _ = agent_tx_done.send(AgentMessage::ReviewPassed(file)).await;
                                        } else {
                                            let _ = agent_tx_done.send(AgentMessage::ReviewFailed(file, result.stderr)).await;
                                        }
                                    }
                                    Err(e) => {
                                        let _ = ui_s.send(AgentMessage::ShellCompleted { exit_code: -1 }).await;
                                        let _ = agent_tx_done.send(AgentMessage::ReviewFailed(file, e.to_string())).await;
                                    }
                                }
                            });
                        }
                        AgentMessage::ShellDenied(cmd) => {
                            let _ = ui_sender.send(AgentMessage::ShellDenied(cmd)).await;

                            let file = pending_shell.take().map(|(_, f)| f).unwrap_or_default();
                            let _ = ui_sender.send(AgentMessage::ReviewFailed(
                                file,
                                "Shell command denied by user".into(),
                            )).await;

                            let _ = ui_sender.send(AgentMessage::ToolCompleted {
                                name: "reviewer".into(),
                                status: ToolStatus::Cancelled,
                            }).await;
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }
}
