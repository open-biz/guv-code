use crate::agents::AgentMessage;
use crate::agents::planner::PlannerAgent;
use crate::agents::coder::CoderAgent;
use crate::agents::reviewer::ReviewerAgent;
use crate::llm::{ModelProvider, GeminiProvider, AnthropicProvider};
use crate::index::RepoIndex;
use tokio::sync::mpsc;
use anyhow::Result;
use std::path::PathBuf;

pub struct Orchestrator {
    repo_path: PathBuf,
    gemini_key: String,
    anthropic_key: String,
}

impl Orchestrator {
    pub fn new(repo_path: PathBuf, gemini_key: String, anthropic_key: String) -> Self {
        Self { repo_path, gemini_key, anthropic_key }
    }

    pub async fn run(&self, query: String, ui_sender: mpsc::Sender<AgentMessage>) -> Result<()> {
        let (agent_tx, mut agent_rx) = mpsc::channel(100);
        
        let scout_provider = GeminiProvider::new(self.gemini_key.clone());
        let coder_provider = AnthropicProvider::new(self.anthropic_key.clone());
        
        let planner = PlannerAgent::new(&scout_provider, agent_tx.clone());
        let coder = CoderAgent::new(&coder_provider, agent_tx.clone());
        let reviewer = ReviewerAgent::new(agent_tx.clone());
        
        let repo_path = self.repo_path.clone();
        let query_clone = query.clone();

        // Start Orchestration Task
        tokio::spawn(async move {
            let mut index = RepoIndex::load_or_create(&repo_path).unwrap();
            let _ = index.update(&repo_path);
            
            // 1. Planning
            if let Err(e) = planner.plan(&index, &query_clone).await {
                let _ = agent_tx.send(AgentMessage::Error(e.to_string())).await;
                return;
            }
        });

        // Loop to handle logic between agents
        while let Some(msg) = agent_rx.recv().await {
            // Forward everything to UI
            let _ = ui_sender.send(msg.clone()).await;

            match msg {
                AgentMessage::PlanCompleted(files) => {
                    let repo_path = self.repo_path.clone();
                    let query_clone = query.clone();
                    let coder_provider = AnthropicProvider::new(self.anthropic_key.clone());
                    let agent_tx = agent_tx.clone();
                    
                    tokio::spawn(async move {
                        let coder = CoderAgent::new(&coder_provider, agent_tx);
                        let _ = coder.code(&repo_path, &query_clone, files).await;
                    });
                }
                AgentMessage::CoderCompleted(file, _patch) => {
                    let repo_path = self.repo_path.clone();
                    let agent_tx = agent_tx.clone();
                    
                    tokio::spawn(async move {
                        let reviewer = ReviewerAgent::new(agent_tx);
                        let _ = reviewer.review(&repo_path, &file).await;
                    });
                }
                _ => {}
            }
        }

        Ok(())
    }
}
