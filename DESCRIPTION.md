# 🎩 GUV-Code (General Unrestricted Vibe)

**"Right away, Guv'nor."**

GUV-Code is your blazingly fast, hyper-competent AI right-hand man. Built in zero-overhead Rust, GUV replaces sluggish, memory-hogging JS/Python agents (like Claude Code and Aider) with a single, lightning-fast binary. 

You give the orders. Guv handles the syntax, the AST-aware diffs, and the thousands of files in your repo—in milliseconds.

## 🛠 Why hire Guv?

*   🎩 **At your service, instantly:** No Node.js bloat. No Python `pipx` nightmares. Guv is a single Rust binary that starts in `0.001s`.
*   🧠 **Never loses the plot (Zero OOMs):** While JS agents crash on massive enterprise codebases, Guv uses `ripgrep`-style directory walking to build repository maps instantly. 
*   🌳 **Surgical precision:** Guv uses `tree-sitter` to smartly inject code right into the Abstract Syntax Tree. No more hallucinated indentation errors.
*   🔓 **Loyal only to you (BYOK):** Seamlessly hot-swap between Claude 3.7, GPT-4o, or local Ollama models. No vendor lock-in.

## 📦 Quickstart

Hire Guv for your terminal:

```bash
curl -sL https://guv.dev/install.sh | bash

# Set your API key
guv auth --anthropic "YOUR_KEY"

# Put Guv to work
guv "Swap out all the REST endpoints in src/api for GraphQL, would you?"
