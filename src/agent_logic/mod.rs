pub mod planner;
pub mod coder;
pub mod reviewer;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolStatus {
    Pending,
    Executing,
    Success,
    Error,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentPhase {
    Idle,
    Mapping,
    Planning,
    Coding,
    Reviewing,
    Complete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentMessage {
    // Planner messages
    PlanStarted,
    PlanUpdate(String),
    PlanCompleted(Vec<String>), // List of files to edit
    
    // Coder messages
    CoderStarted(String), // File being edited
    CoderUpdate(String),  // Token stream
    CoderCompleted(String, String), // File, Final Patch
    
    // Reviewer messages
    ReviewStarted(String),
    ReviewPassed(String),
    ReviewFailed(String, String), // File, Error
    
    // Tool execution messages
    ToolStarted { name: String, description: String },
    ToolOutput { name: String, line: String },
    ToolCompleted { name: String, status: ToolStatus },
    
    // Shell / PTY messages
    ShellRequested { command: String, destructive: bool },
    ShellApproved(String),
    ShellDenied(String),
    ShellOutput(String),
    ShellCompleted { exit_code: i32 },
    
    // Thinking / inner monologue
    Thinking(String),
    
    // Phase transitions
    PhaseChange(AgentPhase),
    
    // Image context
    ImageAttached { path: String, mime: String },
    
    // System messages
    Error(String),
    IndexingStarted,
    IndexingCompleted,

    // Auth completed — triggers config reload + model auto-select
    AuthCompleted(String), // provider name
}
