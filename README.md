# 🎩 GUV-Code (General Unrestricted Vibe)

[![Written in Rust](https://img.shields.io/badge/Written_in-Rust-E34F26.svg?style=flat-square&logo=rust)](https://rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](LICENSE)
[![Version](https://img.shields.io/crates/v/guv-code?style=flat-square)](#)
[![Vibes](https://img.shields.io/badge/Vibes-Immaculate-FF1493.svg?style=flat-square)](#)

**"Right away, Guv'nor."**

GUV-Code is a blazingly fast, 100% Rust-native AI coding agent built for your terminal. It acts as your hyper-competent right-hand fixer, designed to parse massive codebases instantly, plan architectures, and surgically apply AST-aware code edits. 

No single-prompt guessing. No memory crashes. Just pure, multi-model vibecoding.

---

## 🛑 The Problem with Current Agents

Current terminal-based AI agents suffer from their underlying runtimes:
*   ❌ **The Node.js/V8 Bottleneck:** JavaScript-based agents struggle with massive enterprise codebases. When ingesting thousands of files, they hit V8's heap limits, resulting in garbage-collection thrashing and Out-Of-Memory (OOM) crashes.
*   ❌ **Python Dependency Hell:** Python-based agents force users to wrestle with `pipx`, `venv`, and conflicting system libraries just to install a CLI tool. They also suffer from the Global Interpreter Lock (GIL), making parallel AST parsing sluggish.
*   ❌ **Vendor Lock-in & "God Models":** Most tools force you to use one expensive model for everything—from basic file searching to deep reasoning—which burns through your wallet and API rate limits.

## ⚡ The Guv'nor's Solution

GUV-Code is distributed as a **single, statically linked Rust binary**. 
*   🚀 **Zero Bloat, Zero OOMs:** Installs in 1 second. Starts in `0.001s`. Uses `ripgrep`-style multithreaded directory walking to map 10,000+ files in milliseconds.
*   🧠 **Multi-Agent Routing:** GUV doesn't use one "God Prompt." It delegates. It uses high-context models (like Gemini) to map the codebase, and high-reasoning models (like Claude Opus) to write the actual code.
*   🌳 **Bulletproof AST Diffs:** GUV uses native `tree-sitter` bindings. The LLM dictates the logic, and Guv safely injects it into the Abstract Syntax Tree without breaking your indentation.
*   🛡️ **Git-Safe & Budgeted:** Auto-commits before every edit (`guv undo` to instantly revert hallucinations). Built-in token budgeting (`guv budget set $5`) ensures you never burn cash on a runaway loop.

---

## 🤖 Supported Models & Providers (BYOK)

GUV-Code strictly enforces **BYOK (Bring Your Own Key)**. You configure your providers, and GUV automatically routes tasks to the best-suited model.

| Provider | Recommended Model | Internal Agent Role | Why We Use It |
| :--- | :--- | :--- | :--- |
| **Google** | `gemini-2.5-pro` / `flash` | **The Scout** | Massive 2M+ token context window. Best for ingesting the entire codebase index and pinpointing relevant files instantly. |
| **Anthropic** | `claude-3-opus` / `3.7-sonnet` | **The Coder** | Industry-leading deep reasoning and precise AST-aware diff generation. |
| **Google**| `gemini-3.1-pro` | **The Architect** | High-level system design, multi-file execution planning, and complex refactoring logic. |
| **OpenAI** | `o3` / `gpt-4o` | **The Coder (Fallback)** | General purpose execution and syntax writing. |
| **Local** | `qwen-2.5-coder` (via Ollama) | **The Reviewer** | Fast, free, local syntax validation and post-edit linting. |

*(See [MODELS.md](./MODELS.md) for instructions on adding new providers and adjusting routing weights).*

---

## 📦 Quickstart

Stop wrestling with dependencies and start vibing.

```bash
# 1. Install via bash script
curl -sL https://guv.dev/install.sh | bash
# (Or via cargo: cargo install guv-code)

# 2. Hand Guv your keys (BYOK)
guv auth set --provider google "AIZA..."
guv auth set --provider anthropic "sk-ant..."

# 3. Put Guv to work
guv "Refactor the auth middleware in src/ to use JWTs instead of sessions."
```

## 🤝 Contributing
Want to help make the Guv'nor even better? Check out our [CONTRIBUTING.md](./CONTRIBUTING.md).
