<p align="center">
  <img src="GuvCode.png" alt="GuvCode Hero" width="800px">
</p>

<h1 align="center">GuvCode</h1>

<p align="center">
  <strong>"Right away, Guv'nor."</strong><br>
  <em>A blazingly fast, 100% Rust-native AI coding agent built for your terminal.</em>
</p>

<p align="center">
  <a href="https://rust-lang.org"><img src="https://img.shields.io/badge/Written_in-Rust-E34F26.svg?style=flat-square&logo=rust" alt="Written in Rust"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square" alt="License: MIT"></a>
  <a href="#"><img src="https://img.shields.io/crates/v/guv-code?style=flat-square" alt="Version"></a>
  <a href="#"><img src="https://img.shields.io/badge/Vibes-Immaculate-FF1493.svg?style=flat-square" alt="Vibes"></a>
</p>

---

GuvCode acts as your hyper-competent right-hand fixer, designed to parse massive codebases instantly, plan architectures, and surgically apply AST-aware code edits. No single-prompt guessing. No memory crashes. Just pure, multi-model vibecoding.

## 📑 Table of Contents

- [Why GuvCode?](#-why-guvcode)
- [Features](#-features)
- [Getting Started](#-getting-started)
- [Usage](#-usage)
- [Supported Models & Providers](#-supported-models--providers)
- [Web Dashboard](#-web-dashboard)
- [Project Structure](#-project-structure)
- [Contributing](#-contributing)
- [License](#-license)

---

## 🤔 Why GuvCode?

Current terminal-based AI agents suffer from their underlying runtimes. We built GuvCode to solve three major pain points:

| Constraint | The Problem (Other Agents) | The GuvCode Solution | Impact |
| :--- | :--- | :--- | :--- |
| **Memory** | Node.js/V8 heap limits cause Out-Of-Memory (OOM) crashes on large codebases. | **Rust-native memory management.** No garbage collection overhead. | Ingests massive enterprise codebases without crashing. |
| **Concurrency** | Python's GIL blocks true parallelism, making AST parsing and file searching sluggish. | **Multithreaded execution.** Uses `ripgrep`-style directory walking. | Maps 10,000+ files in milliseconds. Starts in `0.001s`. |
| **Distribution** | Python agents force users into dependency hell (`pipx`, `venv`, conflicting system libraries). | **Statically compiled.** Distributed as a single, self-contained executable. | Zero dependencies. Installs and runs instantly. |
| **Cost** | Vendor lock-in forces using expensive "God Models" for simple search tasks, burning cash. | **Multi-Agent Routing.** Low-cost high-context models search; high-reasoning models code. | Maximizes API budget efficiency and prevents rate limits. |

---

## ✨ Features

GuvCode is distributed as a **single, statically linked Rust binary**.

| Category | Feature | Mechanism | Benefit |
| :--- | :--- | :--- | :--- |
| **Performance** | 🚀 **Blazing Fast Execution** | Pure Rust, multi-threaded codebase architecture. | Zero bloat, zero OOMs, and instant startup times. |
| **Intelligence** | 🧠 **Multi-Agent Routing** | Delegates tasks to Scout, Coder, Architect, and Reviewer models. | Cost-effective, specialized, and highly accurate code generation. |
| **Integrity** | 🌳 **Bulletproof AST Diffs** | Native `tree-sitter` bindings parse code syntax structurally. | LLM logic is surgically injected without breaking formatting or indentation. |
| **Safety** | 🛡️ **Git-Safe & Budgeted** | Auto-commits before edits and strictly enforces hard token limits. | Instant `guv undo` rollback and zero risk of runaway loop billing. |
| **Privacy** | 🔒 **BYOK (Bring Your Own Key)** | Local configuration via TOML. No external proxy servers or middle-men. | You completely own your data, your API keys, and your usage budget. |

---

## 🚀 Getting Started

Stop wrestling with dependencies and start vibing.

### Prerequisites

- A terminal. That's it. (If building from source, you'll need the [Rust toolchain](https://rustup.rs/)).

### Installation

**Option 1: Quick Install (Recommended)**
```bash
curl -sL https://guv.dev/install.sh | bash
```

**Option 2: Install via Cargo**
```bash
cargo install guv-code
```

### Configuration

Hand Guv your keys (BYOK). Your keys are stored locally and securely.

```bash
guv auth set --provider google "AIZA..."
guv auth set --provider anthropic "sk-ant..."
```

---

## 💻 Usage

### The Basics

Tell GuvCode what you need to be done. It will automatically scout the codebase, formulate a plan, and execute the changes.

```bash
guv "Refactor the auth middleware in src/ to use JWTs instead of sessions."
```

### Advanced Usage

<details>
  <summary>Click to see more commands</summary>

  **Set a session budget:**
  ```bash
  guv budget set $5
  ```

  **Undo the last AI edit:**
  ```bash
  guv undo
  ```

  **Start an interactive chat session:**
  ```bash
  guv chat
  ```
</details>

---

## 🤖 Supported Models & Providers

GuvCode strictly enforces **BYOK (Bring Your Own Key)**. You configure your providers, and GUV automatically routes tasks to the best-suited model.

| Provider | Recommended Model | Internal Agent Role | Why We Use It |
| :--- | :--- | :--- | :--- |
| **Google** | `gemini-2.5-pro` / `flash` | **The Scout** | Massive 2M+ token context window. Best for ingesting the entire codebase index and pinpointing relevant files instantly. |
| **Anthropic** | `claude-3-opus` / `3.7-sonnet` | **The Coder** | Industry-leading deep reasoning and precise AST-aware diff generation. |
| **Google** | `gemini-3.1-pro` | **The Architect** | High-level system design, multi-file execution planning, and complex refactoring logic. |
| **OpenAI** | `o3` / `gpt-4o` | **The Coder (Fallback)** | General purpose execution and syntax writing. |
| **Local** | `qwen-2.5-coder` (via Ollama) | **The Reviewer** | Fast, free, local syntax validation and post-edit linting. |

*(See [MODELS.md](./MODELS.md) for instructions on adding new providers and adjusting routing weights).*

---

## 🌐 Web Dashboard

GuvCode includes a web dashboard for managing your account, API keys, and usage quotas. It is **not** a code editor — the CLI is the coding interface.

The dashboard is a **Next.js 16** app located in the [`web/`](./web) directory.

**Running locally:**

```bash
cd web
bun install
bun run dev
# → http://localhost:3000
```

See the [Web Dashboard README](./web/README.md) for more details.

---

## 🏗️ Project Structure

```text
guv-code/
├── src/              # Rust CLI agent source
│   ├── main.rs       # CLI entrypoint (clap)
│   ├── llm.rs        # LLM provider routing & streaming
│   ├── orchestrator.rs # Multi-agent task orchestration
│   ├── index.rs      # Fast-resume codebase indexer
│   ├── config.rs     # TOML config & API key management
│   ├── git.rs        # Git auto-commit & undo
│   ├── agent_logic/  # Scout, Architect, Coder, Reviewer logic
│   └── ui/           # TUI with ratatui
├── web/              # Next.js web dashboard
├── Cargo.toml        # Rust dependencies
├── MODELS.md         # Model routing docs
└── CONTRIBUTING.md   # Contributor guide
```

---

## 🤝 Contributing

We welcome contributions! Want to help make the Guv'nor even better? Check out our [Contribution Guidelines](./CONTRIBUTING.md) to get started.

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
