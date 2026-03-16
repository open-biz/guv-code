use crate::agent_logic::AgentMessage;
use tokio::sync::mpsc;
use anyhow::Result;
use std::path::Path;

pub struct ReviewerAgent {
    sender: mpsc::Sender<AgentMessage>,
}

impl ReviewerAgent {
    pub fn new(sender: mpsc::Sender<AgentMessage>) -> Self {
        Self { sender }
    }

    pub async fn review(&self, _repo_path: &Path, file_path: &str) -> Result<()> {
        let _ = self.sender.send(AgentMessage::ReviewStarted(file_path.to_string())).await;

        // Request shell approval — orchestrator will execute after UI confirms
        let _ = self.sender.send(AgentMessage::ShellRequested {
            command: "cargo check".to_string(),
            destructive: true,
        }).await;

        Ok(())
    }
}
