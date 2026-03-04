use std::process::Command;
use anyhow::{Result, Context};
use std::path::Path;

pub struct CommandResult {
    pub success: bool,
    #[allow(dead_code)]
    pub stdout: String,
    pub stderr: String,
}

pub struct TerminalManager;

impl TerminalManager {
    pub fn run_command(path: &Path, cmd: &str, args: &[&str]) -> Result<CommandResult> {
        let output = Command::new(cmd)
            .args(args)
            .current_dir(path)
            .output()
            .context(format!("Failed to execute command: {} {:?}", cmd, args))?;

        Ok(CommandResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}
