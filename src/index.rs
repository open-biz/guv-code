use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::{Context, Result};
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use xxhash_rust::xxh3::xxh3_64;
use rayon::prelude::*;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct RepoIndex {
    pub files: HashMap<String, u64>, // path -> hash
}

impl RepoIndex {
    pub fn load_or_create(repo_path: &Path) -> Result<Self> {
        let cache_path = repo_path.join(".guv/cache/index.bin");
        if cache_path.exists() {
            let content = fs::read(&cache_path).context("Failed to read index cache")?;
            bincode::deserialize(&content).context("Failed to deserialize index cache")
        } else {
            Ok(RepoIndex::default())
        }
    }

    pub fn save(&self, repo_path: &Path) -> Result<()> {
        let cache_dir = repo_path.join(".guv/cache");
        fs::create_dir_all(&cache_dir).context("Failed to create cache directory")?;
        let cache_path = cache_dir.join("index.bin");
        let content = bincode::serialize(self).context("Failed to serialize index")?;
        fs::write(cache_path, content).context("Failed to write index cache")
    }

    pub fn update(&mut self, repo_path: &Path) -> Result<Vec<String>> {
        let mut changed_files = Vec::new();

        let walker = WalkBuilder::new(repo_path)
            .hidden(false)
            .build();

        let entries: Vec<_> = walker.filter_map(|e| e.ok()).collect();

        let new_files: HashMap<String, u64> = entries.par_iter().filter_map(|entry| {
            if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                let path = entry.path();
                
                if path.components().any(|c| c.as_os_str() == ".git" || c.as_os_str() == ".guv") {
                    return None;
                }

                let relative_path = path.strip_prefix(repo_path).ok()?
                    .to_str()?
                    .to_string();

                let content = fs::read(path).ok()?;
                let hash = xxh3_64(&content);

                Some((relative_path, hash))
            } else {
                None
            }
        }).collect();

        for (path, hash) in &new_files {
            if self.files.get(path) != Some(hash) {
                changed_files.push(path.clone());
            }
        }

        self.files = new_files;
        Ok(changed_files)
    }
}
