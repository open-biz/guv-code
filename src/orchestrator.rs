use crate::agent_logic::{AgentMessage, AgentPhase, ToolStatus};
use crate::agent_logic::planner::PlannerAgent;
use crate::agent_logic::coder::CoderAgent;
use crate::agent_logic::reviewer::ReviewerAgent;
use crate::terminal::TerminalManager;
use crate::llm::{GeminiProvider, AnthropicProvider};
use crate::index::RepoIndex;
use tokio::sync::mpsc;
use anyhow::Result;
use std::path::PathBuf;

#[derive(Clone)]
pub struct Orchestrator {
    repo_path: PathBuf,
    gemini_key: String,
    anthropic_key: String,
}

impl Orchestrator {
    pub fn new(repo_path: PathBuf, gemini_key: String, anthropic_key: String) -> Self {
        Self { repo_path, gemini_key, anthropic_key }
    }

    pub async fn run(
        &self,
        query: String,
        ui_sender: mpsc::Sender<AgentMessage>,
        mut response_rx: mpsc::Receiver<AgentMessage>,
    ) -> Result<()> {
        let (agent_tx, mut agent_rx) = mpsc::channel(100);

        let scout_provider = GeminiProvider::new(self.gemini_key.clone());
        let coder_provider = AnthropicProvider::new(self.anthropic_key.clone());

        let repo_path = self.repo_path.clone();
        let query_clone = query.clone();
        let agent_tx_planner = agent_tx.clone();
        let scout_provider_clone = scout_provider.clone();
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

            let planner = PlannerAgent::new(&scout_provider_clone, agent_tx_planner.clone());
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
                            let coder_provider_clone = coder_provider.clone();
                            let agent_tx_coder = agent_tx.clone();
                            let ui_sender_c = ui_sender.clone();

                            tokio::spawn(async move {
                                let _ = ui_sender_c.send(AgentMessage::PhaseChange(AgentPhase::Coding)).await;
                                let _ = ui_sender_c.send(AgentMessage::ToolStarted {
                                    name: "coder".into(),
                                    description: format!("Editing {} file(s)", files.len()),
                                }).await;

                                let coder = CoderAgent::new(&coder_provider_clone, agent_tx_coder);
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
