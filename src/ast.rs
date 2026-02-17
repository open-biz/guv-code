use tree_sitter::{Parser, Language};
use anyhow::{Context, Result};

pub struct AstParser {
    parser: Parser,
}

impl AstParser {
    pub fn new(lang: Language) -> Result<Self> {
        let mut parser = Parser::new();
        parser.set_language(&lang).context("Failed to set language")?;
        Ok(Self { parser })
    }

    pub fn parse(&mut self, source_code: &str) -> Result<Vec<String>> {
        let tree = self.parser.parse(source_code, None).context("Failed to parse code")?;
        let root_node = tree.root_node();
        
        let mut chunks = Vec::new();
        let mut cursor = root_node.walk();

        if cursor.goto_first_child() {
            loop {
                let node = cursor.node();
                // Simple chunking: top-level nodes (functions, structs, etc.)
                let chunk = &source_code[node.start_byte()..node.end_byte()];
                chunks.push(chunk.to_string());

                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        } else {
            // If no children, just return the whole thing
            chunks.push(source_code.to_string());
        }

        Ok(chunks)
    }
}
