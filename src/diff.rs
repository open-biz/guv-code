use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub struct DiffEngine;

impl DiffEngine {
    pub fn apply_edits(edits: &str, repo_path: &Path) -> Result<()> {
        let mut current_file: Option<String> = None;
        let mut old_content = String::new();
        let mut new_content = String::new();
        let mut state = "none";

        for line in edits.lines() {
            if line.starts_with("FILE: ") {
                if let Some(file) = current_file.take() {
                    Self::apply_to_file(&file, &old_content, &new_content, repo_path)?;
                }
                current_file = Some(line.strip_prefix("FILE: ").unwrap().trim().to_string());
                old_content.clear();
                new_content.clear();
                state = "none";
            } else if line == "<<<<" {
                state = "old";
            } else if line == "====" {
                state = "new";
            } else if line == ">>>>" {
                state = "none";
                if let Some(ref file) = current_file {
                    Self::apply_to_file(file, &old_content, &new_content, repo_path)?;
                }
                // Don't clear current_file, there might be more blocks for the same file
                old_content.clear();
                new_content.clear();
            } else {
                match state {
                    "old" => {
                        old_content.push_str(line);
                        old_content.push('\n');
                    }
                    "new" => {
                        new_content.push_str(line);
                        new_content.push('\n');
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn apply_to_file(file_path: &str, old: &str, new: &str, repo_path: &Path) -> Result<()> {
        let full_path = repo_path.join(file_path);
        let content = fs::read_to_string(&full_path).context("Failed to read file for diff application")?;
        
        // Very simple search and replace for now. 
        // Real Aider use more sophisticated diffing.
        let updated = content.replace(old.trim(), new.trim());
        
        if updated == content {
            anyhow::bail!("Could not find exact match for edit in {}", file_path);
        }

        fs::write(&full_path, updated).context("Failed to write updated file")
    }
}
