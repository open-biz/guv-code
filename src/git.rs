use std::process::Command;
use anyhow::{Result, Context};
use std::path::Path;

pub struct GitManager;

impl GitManager {
    pub fn is_repo(path: &Path) -> bool {
        path.join(".git").exists()
    }

    pub fn auto_stage_all(path: &Path) -> Result<()> {
        Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(path)
            .status()
            .context("Failed to run git add")?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn commit(path: &Path, message: &str) -> Result<()> {
        Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(message)
            .current_dir(path)
            .status()
            .context("Failed to run git commit")?;
        Ok(())
    }

    pub fn undo(path: &Path) -> Result<()> {
        Command::new("git")
            .arg("reset")
            .arg("--hard")
            .arg("HEAD~1")
            .current_dir(path)
            .status()
            .context("Failed to run git reset")?;
        Ok(())
    }
}
