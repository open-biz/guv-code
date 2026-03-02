use similar::{ChangeTag, TextDiff};
use owo_colors::OwoColorize;

pub struct DiffViewer;

impl DiffViewer {
    pub fn show(file_path: &str, old: &str, new: &str) {
        println!("\n╭── {} (AST Patch Ready) ──────────────╮", file_path.bold().blue());
        
        let diff = TextDiff::from_lines(old, new);
        
        for (i, change) in diff.iter_all_changes().enumerate() {
            let line_num = (i + 1).to_string();
            let sign = match change.tag() {
                ChangeTag::Delete => "-".red().to_string(),
                ChangeTag::Insert => "+".green().to_string(),
                ChangeTag::Equal => " ".to_string(),
            };
            
            let content = match change.tag() {
                ChangeTag::Delete => change.value().red().to_string(),
                ChangeTag::Insert => change.value().green().to_string(),
                ChangeTag::Equal => change.value().to_string(),
            };
            
            print!("│ {:>3} │ {} {}", line_num.dimmed(), sign, content);
        }
        
        println!("╰──────────────────────────────────────────────────╯");
    }
}
