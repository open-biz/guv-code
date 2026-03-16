use crate::llm::{ModelProvider, Message};
use crate::agent_logic::AgentMessage;
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
            
            // Send thinking message before starting stream
            let _ = self.sender.send(AgentMessage::Thinking("Starting code generation stream...".into())).await;
            
            let mut stream = match self.provider.complete_stream(messages).await {
                Ok(s) => s,
                Err(e) => {
                    let _ = self.sender.send(AgentMessage::Error(format!("Stream init failed: {}", e))).await;
                    return Err(e);
                }
            };
            
            let _ = self.sender.send(AgentMessage::Thinking("Stream initialized, waiting for response...".into())).await;
            
            let mut full_patch = String::new();
            let mut chunk_count = 0;
            while let Some(chunk) = stream.recv().await {
                match chunk {
                    Ok(text) => {
                        chunk_count += 1;
                        if chunk_count == 1 {
                            let _ = self.sender.send(AgentMessage::Thinking(format!("Receiving chunks ({})", chunk_count))).await;
                        }
                        full_patch.push_str(&text);
                        let _ = self.sender.send(AgentMessage::CoderUpdate(text)).await;
                    }
                    Err(e) => {
                        let _ = self.sender.send(AgentMessage::Error(format!("Stream error after {} chunks: {}", chunk_count, e))).await;
                        break;
                    }
                }
            }

            let _ = self.sender.send(AgentMessage::Thinking(format!("Stream completed ({} chunks)", chunk_count))).await;
            let _ = self.sender.send(AgentMessage::CoderCompleted(file_path, full_patch)).await;
        }
        Ok(())
    }
}
