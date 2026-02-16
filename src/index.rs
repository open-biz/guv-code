use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use hex;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct RepoIndex {
    pub files: HashMap<String, String>, // path -> hash
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
        let mut new_files = HashMap::new();

        let walker = WalkBuilder::new(repo_path)
            .hidden(false) // we want to respect gitignore but maybe see hidden files?
            .build();

        for result in walker {
            let entry = match result {
                Ok(e) => e,
                Err(_) => continue,
            };

            if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                let path = entry.path();
                
                // Skip .git and .guv directories
                if path.components().any(|c| c.as_os_str() == ".git" || c.as_os_str() == ".guv") {
                    continue;
                }

                let relative_path = path.strip_prefix(repo_path)?
                    .to_str()
                    .context("Non-UTF8 path")?
                    .to_string();

                let hash = self.hash_file(path)?;

                if let Some(old_hash) = self.files.get(&relative_path) {
                    if old_hash != &hash {
                        changed_files.push(relative_path.clone());
                    }
                } else {
                    changed_files.push(relative_path.clone());
                }

                new_files.insert(relative_path, hash);
            }
        }

        self.files = new_files;
        Ok(changed_files)
    }

    fn hash_file(&self, path: &Path) -> Result<String> {
        let mut file = fs::File::open(path)?;
        let mut hasher = Sha256::new();
        std::io::copy(&mut file, &mut hasher)?;
        Ok(hex::encode(hasher.finalize()))
    }
}
