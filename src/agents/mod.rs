use serde::{Deserialize, Serialize};

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
    
    // System messages
    Error(String),
}
