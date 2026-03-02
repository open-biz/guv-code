pub mod diff_viewer;

use crate::agent_logic::AgentMessage;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::collections::HashMap;
use owo_colors::OwoColorize;

pub struct AgentStepper {
    multi: MultiProgress,
    bars: HashMap<String, ProgressBar>,
}

impl AgentStepper {
    pub fn new() -> Self {
        Self {
            multi: MultiProgress::new(),
            bars: HashMap::new(),
        }
    }

    pub fn handle_message(&mut self, msg: AgentMessage) {
        match msg {
            AgentMessage::PlanStarted => {
                let pb = self.multi.add(ProgressBar::new_spinner());
                pb.set_style(self.spinner_style());
                pb.set_message("Planner (Gemini) analyzing repository...");
                self.bars.insert("planner".to_string(), pb);
            }
            AgentMessage::PlanCompleted(files) => {
                if let Some(pb) = self.bars.get("planner") {
                    pb.finish_with_message(format!("✔ Planner: Identified {} files.", files.len()).green().to_string());
                }
            }
            AgentMessage::CoderStarted(file) => {
                let pb = self.multi.add(ProgressBar::new_spinner());
                pb.set_style(self.spinner_style());
                pb.set_message(format!("Coder (Opus) writing diffs for {}...", file));
                self.bars.insert(format!("coder_{}", file), pb);
            }
            AgentMessage::CoderCompleted(file, _) => {
                if let Some(pb) = self.bars.get(&format!("coder_{}", file)) {
                    pb.finish_with_message(format!("✔ Coder: Patch ready for {}.", file).green().to_string());
                }
            }
            AgentMessage::ReviewStarted(file) => {
                let pb = self.multi.add(ProgressBar::new_spinner());
                pb.set_style(self.spinner_style());
                pb.set_message(format!("Reviewer (Local) checking {}...", file));
                self.bars.insert(format!("review_{}", file), pb);
            }
            AgentMessage::ReviewPassed(file) => {
                if let Some(pb) = self.bars.get(&format!("review_{}", file)) {
                    pb.finish_with_message(format!("✔ Reviewer: {} is clean.", file).green().to_string());
                }
            }
            _ => {}
        }
    }

    fn spinner_style(&self) -> ProgressStyle {
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.blue} {msg}").unwrap()
    }
}
