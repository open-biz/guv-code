use crate::llm::{ModelProvider, Message};
use crate::agents::AgentMessage;
use tokio::sync::mpsc;
use anyhow::Result;
use std::fs;
use std::path::Path;

pub struct CoderAgent<'a> {
    provider: &'a dyn ModelProvider,
    sender: mpsc::Sender<AgentMessage>,
}

impl<'a> CoderAgent<'a> {
    pub fn new(provider: &'a dyn ModelProvider, sender: mpsc::Sender<AgentMessage>) -> Self {
        Self { provider, sender }
    }

    pub async fn code(&self, repo_path: &Path, query: &str, files: Vec<String>) -> Result<()> {
        for file_path in files {
            let _ = self.sender.send(AgentMessage::CoderStarted(file_path.clone())).await;
            
            let full_path = repo_path.join(&file_path);
            let content = fs::read_to_string(&full_path).unwrap_or_default();
            
            let system_prompt = "You are an expert software engineer. Generate surgical SEARCH/REPLACE blocks.";
            let prompt = format!(
                "{}\n\nRequest: {}\n\nFile: {}\n```\n{}\n```",
                system_prompt, query, file_path, content
            );

            let messages = vec![Message { role: "user".to_string(), content: prompt }];
            let mut stream = self.provider.complete_stream(messages).await?;
            
            let mut full_patch = String::new();
            while let Some(chunk) = stream.recv().await {
                if let Ok(text) = chunk {
                    full_patch.push_str(&text);
                    let _ = self.sender.send(AgentMessage::CoderUpdate(text)).await;
                }
            }

            let _ = self.sender.send(AgentMessage::CoderCompleted(file_path, full_patch)).await;
        }
        Ok(())
    }
}
