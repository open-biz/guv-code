use crate::agent_logic::AgentMessage;
use crate::terminal::TerminalManager;
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

    pub async fn review(&self, repo_path: &Path, file_path: &str) -> Result<()> {
        let _ = self.sender.send(AgentMessage::ReviewStarted(file_path.to_string())).await;
        
        // Simulating a review by running a build check
        let check_res = TerminalManager::run_command(repo_path, "cargo", &["check"])?;
        
        if check_res.success {
            let _ = self.sender.send(AgentMessage::ReviewPassed(file_path.to_string())).await;
        } else {
            let _ = self.sender.send(AgentMessage::ReviewFailed(file_path.to_string(), check_res.stderr)).await;
        }
        
        Ok(())
    }
}
