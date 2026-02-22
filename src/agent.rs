use crate::llm::{ModelProvider, Message};
use crate::index::RepoIndex;
use anyhow::Result;
use std::fs;
use std::path::Path;

pub struct ScoutAgent<'a> {
    provider: &'a dyn ModelProvider,
}

impl<'a> ScoutAgent<'a> {
    pub fn new(provider: &'a dyn ModelProvider) -> Self {
        Self { provider }
    }

    pub async fn find_files(&self, index: &RepoIndex, query: &str) -> Result<Vec<String>> {
        let file_list = index.files.keys().cloned().collect::<Vec<String>>().join("\n");
        let prompt = format!(
            "Given the following list of files in a repository, identify which files are most likely relevant to this request: \"{}\"\n\nFiles:\n{}\n\nReturn only the list of file paths, one per line.",
            query, file_list
        );

        let messages = vec![Message { role: "user".to_string(), content: prompt }];
        let response = self.provider.chat(messages).await?;
        
        Ok(response.lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
    }
}

pub struct CoderAgent<'a> {
    provider: &'a dyn ModelProvider,
}

impl<'a> CoderAgent<'a> {
    pub fn new(provider: &'a dyn ModelProvider) -> Self {
        Self { provider }
    }

    pub async fn heal(&self, error: &str, files: Vec<(String, String)>) -> Result<String> {
        let mut context = String::new();
        for (path, content) in files {
            context.push_str(&format!("File: {}\n```\n{}\n```\n\n", path, content));
        }

        let system_prompt = "You are an expert software engineer. You have been provided with an error message from a failed build or test.
Your task is to FIX the error using SEARCH/REPLACE blocks.
Format your response exactly like this:
FILE: path/to/file
<<<<
existing code to search for
====
new code to replace it with
>>>>";

        let prompt = format!(
            "{}\n\nError Message:\n```\n{}\n```\n\nContext:\n{}",
            system_prompt, error, context
        );

        let messages = vec![Message { role: "user".to_string(), content: prompt }];
        self.provider.chat(messages).await
    }
}

pub struct Memory {
    path: std::path::PathBuf,
}

impl Memory {
    pub fn new(repo_path: &Path) -> Self {
        Self {
            path: repo_path.join(".guv/memory.md"),
        }
    }

    pub fn read(&self) -> Result<String> {
        if !self.path.exists() {
            return Ok("No project memory yet.".to_string());
        }
        fs::read_to_string(&self.path).map_err(|e| e.into())
    }

    pub fn write(&self, content: &str) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&self.path, content).map_err(|e| e.into())
    }
}
