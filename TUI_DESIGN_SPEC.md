# 🎨 GuvCode TUI Design Specification

This document serves as the high-signal context for upgrading the GuvCode TUI. The goal is to evolve our current **Ratatui** implementation into a "God-Tier" interface inspired by the aesthetic of **Charm (Lip Gloss/Bubbles)** and the functional transparency of **Codebuff** and **Amux**.

---

## 🏛️ Core Design Principles

### 1. The "Charm" Aesthetic (Layout & Style)
*   **Spacing as a Feature:** Do not crowd the edges. Use padding and margins in every block. A TUI feels premium when it has "room to breathe."
*   **Vibrant Color Palettes:** Move beyond standard 16-colors. Use hex-coded gradients and "Lip Gloss" style highlights (e.g., `#FF5F87` for hot pink, `#5FD7FF` for cyan).
*   **Rounded Primitives:** Where the terminal supports it, prefer rounded corners for borders (`Borders::ALL` with `BorderType::Rounded`).
*   **Ghost & Dimmed Text:** Use `Style::default().fg(Color::Indexed(240))` for secondary information or "ghost" hints.

### 2. The "Codebuff" Transparency (Activity & Steppers)
*   **The Agent Stepper:** Implement a vertical or horizontal "step" component that shows the agent's lifecycle in real-time:
    *   `[○] Mapping Codebase` -> `[●] Planning Edits` -> `[○] Applying Patches`.
*   **Inline Diff Streaming:** When the "Coder" agent is working, stream the diff directly into the UI using color-coded blocks (Green for additions, Red for deletions) instead of just showing a progress bar.
*   **Self-Healing Feedback:** If a build fails, show the error diagnostic in a "Crush-style" miette block within the TUI.

### 3. The "Amux" Layout (Structure)
*   **Status Bar (Persistent):** A bottom-aligned bar showing `[Session Budget: $1.24/5.00]`, `[Model: Gemini 2.5 Pro]`, and `[Mode: Autonomous]`.
*   **The Sidebar (Activity Log):** A scrollable side-pane that captures raw agent "thoughts" and logs without interrupting the main chat/code view.
*   **Main View:** Toggleable between a Markdown-rendered chat and an AST-aware code preview.

---

## 🛠️ Technical Implementation Strategy (Ratatui + Rust)

### 1. Components to Build/Refine
- `AgentStepper`: A custom widget that tracks `enum AgentState` with high-fidelity progress indicators.
- `DiffView`: A widget using the `similar` crate to render line-by-line colored diffs.
- `StatusBar`: A `Rect` spanning the bottom width with `Span`s for system metrics and budget tracking.
- `TerminalEmulator`: A sub-pane that renders the output of `guv run` commands with full ANSI support and interactive PTY focus.

### 2. Advanced Tool Calling UI (Inspiration: Gemini CLI)
- **Sticky Tool Headers:** When a tool (e.g., `read_file` or `shell_run`) generates long output, the tool name and status (`EXECUTING`, `SUCCESS`, `ERROR`) must remain pinned to the top of its viewport.
- **Interactive PTY Panes:** For shell tools, the TUI must allow the user to "focus" into the tool's sub-pane to provide stdin (e.g., responding to a `y/n` prompt from a build script).
- **Status Indicators:** Use a dedicated vertical strip (2-3 chars wide) to show the state of each tool call in the activity log.

### 3. Multimodal & Vision Spec (Inspiration: Crush + Gemini CLI)
- **Path Transformations:** User-typed image paths (e.g., `@path/to/img.png`) should be automatically collapsed into a `[Image filename.png]` tag. 
    - **Expansion:** When the cursor moves over the tag, it expands to show the full logical path.
- **Kitty Graphics Support:** Implement the **Kitty Graphics Protocol** for high-resolution image rendering in supported terminals.
- **High-Fidelity Fallback:** For non-Kitty terminals, implement a "Quadrant/Block" character renderer (using `unicode-blocks`) that preserves color and approximate shapes.
- **Dynamic Resizing:** Images must be scaled using Lanczos or similar filtering to fit precisely within their allotted TUI cells.

### 4. Interaction Model
- **Non-Blocking IO:** Ensure the TUI never hangs while the LLM is streaming. Use `tokio::mpsc` channels to push updates from the `Orchestrator` to the `App` state.
- **Smooth Spinners:** Use `indicatif`-style spinner frames within Ratatui `Paragraph` widgets for a high-performance feel.
- **Keyboard-First:** Vim-like keybindings (`j/k` for scrolling, `Enter` to accept a plan, `u` for undo).

---

## 📚 Reference Context (Internal `/ref`)

When building, refer to these specific files for implementation patterns:
- **`ref/amux/internal/ui`**: Look at how they handle Go-based layout primitives and PTY integration.
- **`ref/crush`**: Look at how they use `miette` and `owo-colors` for high-fidelity error reporting.
- **`ref/gemini-cli`**: Look at the `packages/cli` directory for React/Ink-style declarative UI mapping (which we should emulate in Rust logic).

## 🚀 Advanced Features & Integration

### 1. Fast-Resume (The Memory Layer)
*   **The Query Feedback:** When the Scout performs a search, the TUI should briefly display a "Memory Hit" indicator if the result came from the `fast-resume` semantic cache.
*   **Background Indexing:** Show a subtle "Indexing..." indicator in the Status Bar while Fast-Resume is mapping the codebase in the background.

### 2. Tool Calling (The Execution Loop)
*   **Tool Execution Log:** Implement a dedicated scrolling pane for "Active Agency." 
    *   `[SHELL] running 'cargo test'...`
    *   `[FILE] reading 'src/main.rs'...`
*   **Approval Gate:** When a tool requires permission (e.g., a destructive shell command), pop up a centered modal with a "Charm-style" [Yes/No] selection.

### 3. Multimodal Support (Vision)
*   **Image Context:** If a user provides an image (`--image`), display a small high-res (Kitty) or high-fidelity block-char preview of the image in the sidebar. 
*   **Visual Reasoner:** Add a specialized "Vision" state to the `AgentStepper` to show when the model is specifically analyzing visual assets.

## 📚 Contextual Reference Library

When implementing these features, feed the following files into the agent's context to ensure alignment with industry-leading patterns:

### 1. TUI Layout & Aesthetics
- **`ref/amux/internal/ui/dashboard/dashboard_render.go`**: High-signal patterns for multi-pane layouts and toolbar rendering.
- **`ref/amux/internal/ui/dashboard/model_update.go`**: Logic for handling complex keyboard/mouse interactions in a TUI.
- **`ref/gemini-cli/packages/cli/src/ui/components/AppContainer.tsx`**: State management for a massive multi-agent TUI.

### 2. Tool Calling & Sticky UI
- **`ref/gemini-cli/packages/cli/src/ui/components/messages/ToolMessage.tsx`**: Implementation of sticky headers and per-tool status indicators.
- **`ref/gemini-cli/packages/cli/src/ui/components/messages/ToolResultDisplay.tsx`**: Handling of large tool outputs and terminal focus.
- **`ref/aider/aider/coders/editblock_func_coder.py`**: Clean definitions for function/tool schemas and parameters.

### 3. Multimodal & Path Logic
- **`ref/crush/internal/ui/image/image.go`**: The gold standard for Kitty Graphics Protocol and Block-char fallback in Go.
- **`ref/gemini-cli/packages/cli/src/ui/components/shared/text-buffer.ts`**: Regex-based path transformations and interactive image-tag expansion.
- **`ref/gemini-cli/packages/core/src/utils/fileUtils.ts`**: Logic for detecting and reading image/PDF mime-types.

### 4. PTY & Terminal Emulation
- **`ref/amux/internal/pty/pty.go`**: Low-level PTY allocation and signal handling.
- **`ref/gemini-cli/packages/cli/src/ui/components/shared/Terminal.tsx`**: Rendering ANSI streams into a TUI component.

## 🗺️ Implementation Roadmap & Agent Prompts

Use these targeted prompts to guide the coding agent through the build process in logical phases.

### Phase 1: The "Charm" Aesthetic Overhaul
**Goal:** Achieve the polished, rounded, padded "Charm" look using Ratatui.

> **Prompt:**
> "I want to overhaul the GUV-Code TUI to feel like a **Charm (Lip Gloss/Bubbles)** application.
> 
> **Step 1:** Study `ref/crush/internal/ui/styles` and `internal/ui/common`. Notice the heavy use of **padding**, **rounded borders**, and **high-contrast hex colors** (#FF5F87, #00D7D7).
> **Step 2:** Refactor `src/ui/app.rs` to move away from a 'cramped' layout:
> - Implement **'Lip Gloss' style blocks**: Every window (Chat, Sidebar, Logs) must have internal padding and a `BorderType::Rounded` style.
> - **Color Palette:** Use a base theme of Deep Charcoal (`#121212`) with primary accents in 'Charm Pink' and 'Codebuff Cyan'.
> - **Visual Hierarchy:** Use dimmed, italicized text for metadata and bold, bright spans for primary actions (inspired by `ref/crush`).
> 
> The goal is a TUI that feels like a modern CLI app, not a 1990s terminal."

### Phase 2: The "Codebuff" Transparency & Agency
**Goal:** Implement the high-transparency agent tracking that makes Codebuff feel "alive."

> **Prompt:**
> "Now, let's implement the **Codebuff-style agency loop**. We need total transparency in what the agents are thinking and doing.
> 
> **Step 1:** Study `ref/gemini-cli/packages/cli/src/ui/components/messages/SubagentProgressDisplay.tsx`. This is the 'Codebuff' model of showing sub-task progress.
> **Step 2:** Build the **Agent Stepper Widget**:
> - It should show a vertical checklist of the agent's current 'Chain of Thought'.
> - Use animated spinners (indicatif style) for the active step and checkmarks for completed ones.
> **Step 3:** Implement the **'Thinking' Sidebar**:
> - Dedicate the sidebar to a live stream of raw LLM 'thoughts' (inner monologue) separate from the main chat. 
> **Step 4:** **Inline Diff Streaming**: When the Coder agent is writing, don't just show a progress bar. Stream the colored diff (inspired by Codebuff’s UI) directly into the main view so I can see the code being 'typed'."

### Phase 3: The "Agency & Tools" Loop
**Goal:** Make tool execution as interactive and safe as the industry leaders.

> **Prompt:**
> "Final Step: Integrate the **Tool Calling & Interaction Loop**.
> 
> **Step 1:** Study `ref/gemini-cli/packages/cli/src/ui/components/messages/ToolMessage.tsx`.
> **Step 2:** Implement **Sticky Tool Headers**. If a tool produces 50 lines of output, its name/status (`[SHELL] RUNNING...`) must stay pinned to the top of its box.
> **Step 3:** Implement **Keyboard-First Interaction**:
> - If a tool (like `cargo test`) fails, the TUI should highlight the error and offer a one-key `[r]` to Retry or `[h]` for the agent to 'Heal' (fix) it automatically.
> **Step 4:** Build the **Approval Gate Modal**: A centered, high-contrast modal that pops up for destructive actions, using the rounded Charm style we built in Phase 1."

---

## 🎯 The Final Goal
The TUI should not just be a "view"; it should be a **Dashboard of Action**. Every token generated should feel intentional, every file change should be visible as a diff, and the user should feel in total control of the "Guv'nor's" labor.

