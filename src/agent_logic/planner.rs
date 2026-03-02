use crate::llm::{ModelProvider, Message};
use crate::index::RepoIndex;
use crate::agent_logic::AgentMessage;
use tokio::sync::mpsc;
use anyhow::Result;

pub struct PlannerAgent<'a> {
    provider: &'a dyn ModelProvider,
    sender: mpsc::Sender<AgentMessage>,
}

impl<'a> PlannerAgent<'a> {
    pub fn new(provider: &'a dyn ModelProvider, sender: mpsc::Sender<AgentMessage>) -> Self {
        Self { provider, sender }
    }

    pub async fn plan(&self, index: &RepoIndex, query: &str) -> Result<()> {
        let _ = self.sender.send(AgentMessage::PlanStarted).await;
        
        let file_list = index.files.keys().cloned().collect::<Vec<String>>().join("\n");
        let prompt = format!(
            "Analyze the repository and the following request: \"{}\"\n\nFiles available:\n{}\n\nIdentify the files that need modification. Return a JSON array of file paths.",
            query, file_list
        );

        let messages = vec![Message { role: "user".to_string(), content: prompt }];
        
        // In a real implementation, we would parse JSON from the response.
        // For now, let's simulate the scouting logic.
        let response = self.provider.chat(messages).await?;
        
        // Basic extraction for prototype
        let files: Vec<String> = response.lines()
            .filter(|l| l.contains("/") || l.contains(".rs") || l.contains(".ts"))
            .map(|s| s.trim_matches(|c| c == '"' || c == ',' || c == '[' || c == ']').trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let _ = self.sender.send(AgentMessage::PlanCompleted(files)).await;
        Ok(())
    }
}
