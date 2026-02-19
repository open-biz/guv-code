# 🧠 GUV-Code Model Routing & Architecture

GUV-Code operates on a **Multi-Agent Routing System**. Because LLM APIs have vastly different strengths, context limits, and pricing structures, GUV splits your instructions into specialized tasks and sends them to the most efficient model.

## 🔀 How the Routing Works

When you run `guv "Refactor my database layer"`, here is what happens under the hood:

1. **The Scout (Context Ingestion)**
   * **Default Model:** `gemini-2.5-flash` or `gemini-2.5-pro`
   * **The Job:** GUV compiles a `fast-resume` index of your local repository and fires it to Gemini. Gemini's massive 2M+ context window easily swallows the entire tree and returns *only* the specific files that need editing.
2. **The Architect (Planning)**
   * **Default Model:** `gemini-3.1-pro` or `claude-3.7-sonnet`
   * **The Job:** Takes the user's prompt and the files identified by The Scout, and writes a strict, step-by-step execution plan.
3. **The Coder (Execution)**
   * **Default Model:** `claude-3-opus` or `gemini-3.1-pro`
   * **The Job:** Executes the plan, generating highly precise, AST-aware diff blocks.
4. **The Reviewer (Validation)**
   * **Default Model:** Local `Ollama` models or `gpt-4o-mini`
   * **The Job:** A fast, cheap (or free) pass to verify syntax and ensure indentation wasn't hallucinated before the AST engine injects the code.

## 🛠️ Adding New Models (For Contributors)

The AI landscape moves fast. GUV-Code is designed to support new models the day they drop. To add a new model:

1. Open `src/llm/models.rs`.
2. Add the new model identifier to the `ModelIdentifier` enum.
3. Define its context window limits, cost-per-1k-tokens (for the budgeting engine), and its preferred Agent Role (Scout, Architect, Coder, Reviewer).
4. Implement the `ModelProvider` trait if it uses a completely novel API structure (otherwise, map it to the existing OpenAI-compatible or Anthropic REST traits).
