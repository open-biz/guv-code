# 🤝 Contributing to GUV-Code

First off, thanks for taking the time to contribute! GUV-Code is built by and for developers who want a frictionless, uncompromising AI coding agent. 

## 🏗️ Architecture & Stack

Everything in GUV-Code is written in memory-safe, blazingly fast Rust.
* **CLI Engine:** `clap`
* **UI/Terminal Panes:** `ratatui` (for multiplexed agent views) and `indicatif` (for spinners)
* **Search / Context:** `ignore` (for ripgrep speed) + custom `fast-resume` bincode caching
* **AST Diffs:** `tree-sitter` bindings
* **Network / APIs:** `reqwest` + `tokio` (for async SSE streaming)

## 💻 Local Setup

1. **Clone the repo:**
   ```bash
   git clone https://github.com/yourusername/guv-code.git
   cd guv-code
   ```
2. **Build the project:**
   ```bash
   cargo build --release
   ```
3. **Run tests:**
   ```bash
   cargo test
   ```

## ✅ Development Guidelines

* **Keep it Fast:** GUV-Code's main selling point against JS/Python agents is pure speed. Avoid heavy allocations in the `fast-resume` indexing path. Use `rayon` for parallel directory walking.
* **Commit Logical Steps:** Keep your commits atomic. If you are adding a new model provider, don't bundle it with a UI tweak in `ratatui`. 
* **Respect the Vibe:** The CLI should feel like a helpful right-hand fixer ("The Guv'nor"). Keep error messages beautiful (using `miette`) and helpful. Don't dump raw stack traces to the user.

## 🚀 Submitting a Pull Request

1. Fork the repository and create your branch from `main`.
2. Ensure `cargo fmt` and `cargo clippy` pass without warnings.
3. Write tests for your specific AST or Routing logic.
4. Open a PR with a clear description of the problem solved and the models/providers tested.
