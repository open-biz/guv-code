# 🎨 GuvCode TUI Design Specification (v2): The Charm Evolution

This document defines the "God-Tier" evolution of the GuvCode interface. Version 2 is a technical and aesthetic translation of the **Charm (Bubble Tea, Lip Gloss, Gum)** ecosystem into Rust/Ratatui, specifically inspired by the implementation patterns in `ref/crush`.

---

## 🏛️ v2 Core Philosophy: The "Charm" Soul

### 1. Reactive Component Architecture (Inspiration: Bubble Tea)
Instead of a monolithic render loop, v2 adopts a component-based state machine.
- **Message Partials:** Every message is broken into atomic `MessageItems` (Text, ToolCall, AssistantInfo).
- **State-Driven Views:** Components (like the Command Palette) are distinct models with their own `Update` logic.
- **Async Monologue:** The "Thinking" state is a specialized component that tracks duration and streams thoughts before the final response (Reference: `ref/crush/internal/ui/chat/messages.go`).

### 2. The Command Palette (Inspiration: Gum / Gum Filter)
A dedicated, fuzzy-searchable interface for actions.
- **Fuzzy Logic:** Use the `fuzzy-matcher` crate to implement Charm-style filtering for subcommands and files.
- **Contextual Suggestions:** The palette defaults to the most logical next steps (e.g., `undo`, `retry`, `clear`) based on the `AgentState`.
- **Reference:** `ref/crush/internal/ui/list/filterable.go`.

### 3. High-Fidelity Multimodal Vision
- **Kitty Graphics Protocol:** First-class support for high-res images.
- **Block-Char Fallback:** Precise Quadrant/Block character rendering for legacy terminals.
- **Visual Context Pane:** A dedicated sidebar that renders the image context currently held in the LLM's prompt.
- **Reference:** `ref/crush/internal/ui/image/image.go`.

---

## 🛠️ Technical implementation: The "Lip Gloss" Engine

### 1. Spacing & Layout Rules
GuvCode v2 follows the **Charm Spacing Manifesto**:
- **Internal Padding:** All window panes MUST have 1 cell of horizontal and vertical padding.
- **Rounded Borders:** Use `BorderType::Rounded` globally.
- **Margin Buffers:** Use `Rect` manipulation to create "gutters" between panes.
- **Reference:** `ref/crush/internal/ui/styles/styles.go` (DefaultMargin = 2).

### 2. Semantic Color Palette
V2 uses a hex-based palette mapped to TUI roles:
- **Deep Charcoal (`#121212`):** Base background.
- **Charm Pink (`#FF5F87`):** Primary actions and Command Palette trigger.
- **Codebuff Cyan (`#00D7D7`):** Agent activity and streaming diffs.
- **Dolly Purple (`#D7A1FF`):** Assistant metadata and thinking states.

### 3. Iconography (Nerd Fonts)
| Role | Icon | Mechanism |
| :--- | :--- | :--- |
| **Success** | `✓` | Green bold |
| **Error** | `×` | Red bold |
| **Thinking** | `⋯` | Animated pulse |
| **Tool** | `●` | Semantic status strip |
| **Image** | `■` | Visual Context toggle |

---

## ⌨️ v2 Interaction Registry

| Key | Action | Context |
| :--- | :--- | :--- |
| `:` | **Command Palette** | Global (Fuzzy-filter subcommands) |
| `Ctrl+P` | **File Finder** | Input Mode (Fuzzy-filter workspace) |
| `@` | **Path Trigger** | Input Mode (Triggers inline file suggestions) |
| `Tab` | **Focus Cycle** | Navigation (Chat -> Sidebar -> Logs) |
| `j / k` | **Vim Scroll** | Active Pane (Half-page jumps via `Ctrl+D/U`) |
| `u` | **Instant Undo** | Global (Reverts last git auto-commit) |

---

## 📚 Contextual Reference Library (v2 Logic)

Feed these files to the agent to "copy the soul" of Charm:
- **Styling Manifesto:** `ref/crush/internal/ui/styles/styles.go` (The definition of Charm aesthetics).
- **Fuzzy Filtering:** `ref/crush/internal/ui/list/filterable.go` (The logic for the Command Palette).
- **Message Rendering:** `ref/crush/internal/ui/chat/messages.go` (Handling Partials and Thinking states).
- **Image Pipeline:** `ref/crush/internal/ui/image/image.go` (The gold standard for TUI multimodal).

---

## 🎯 The Final Goal
The TUI should not just be a "view"; it should be a **Dashboard of Action**. Every token generated should feel intentional, every file change should be visible as a diff, and the user should feel in total control of the "Guv'nor's" labor.
