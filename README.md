# Rust AI Agent Template

Scaffolding for building and running custom agentic AI systems in Rust. This repository provides a type-safe, modular 3-crate architecture that runs out of the box with zero external API dependencies via a built-in mock LLM provider. The bundled tools (a single-operator calculator and a clock) are illustrative starting points — swap in real tools and a real `LlmProvider` to build something production-grade.

## Workspace Layout

The project is structured as a cargo workspace with the following crates:

| Crate | Path | Purpose |
|-------|------|---------|
| `agent-core` | `crates/agent-core` | Foundation traits & struct orchestrators: LLM providers, Memory/History, Tools, and the Agent Execution Loop. |
| `agent-tools` | `crates/agent-tools` | Built-in out-of-the-box agent tools: mathematical calculations, system info/time, etc. |
| `agent-cli` | `crates/agent-cli` | Interactive CLI application providing a colored, user-friendly terminal interface to chat with the agent. |

## Quick Start

### 1. Build and Run the Interactive CLI

Run the interactive shell:
```bash
cargo run -p agent-cli
```

To see trace logs of agent thinking, planning, and tool execution, raise the log level:
```bash
cargo run -p agent-cli -- --log debug
```

### 2. Run Workspace Tests

Verify the workspace builds and all tests pass:
```bash
cargo test --workspace
```

### 3. Run Linter and Formatting

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

---

## Adding Custom Tools

To define a new tool, implement the `Tool` trait from `agent-core::tool` in the `agent-tools` crate (or a custom crate):

```rust
use agent_core::tool::{Tool, ToolDefinition};
use serde_json::{json, Value};

#[derive(Default)]
pub struct GreetingTool {}

#[async_trait::async_trait]
impl Tool for GreetingTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "greet".to_string(),
            description: "Greets the user by name.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "The user's name" }
                },
                "required": ["name"]
            }),
        }
    }

    async fn call(&self, arguments: Value) -> Result<Value, anyhow::Error> {
        let name = arguments["name"].as_str().unwrap_or("Friend");
        Ok(json!({ "greeting": format!("Hello, {}!", name) }))
    }
}
```

Then register it in your application registry before starting the agent:

```rust
let mut registry = ToolRegistry::new();
registry.register(GreetingTool::new());
```

---

## Implementing a Real LLM Provider

To connect the agent to a real LLM provider (like Gemini, OpenAI, or Anthropic), implement the `LlmProvider` trait in `agent-core::llm`:

```rust
use agent_core::llm::{LlmProvider, ChatMessage, LlmResponse, ToolCall};
use agent_core::tool::ToolDefinition;
use agent_core::error::AgentError;

pub struct CustomLlmClient {
    api_key: String,
}

#[async_trait::async_trait]
impl LlmProvider for CustomLlmClient {
    async fn generate(
        &self,
        messages: &[ChatMessage],
        tools: &[ToolDefinition],
    ) -> Result<LlmResponse, AgentError> {
        // 1. Send HTTP request to provider API (e.g. using reqwest)
        // 2. Parse response and detect if it wants to call a tool or reply with text
        // 3. Return LlmResponse::Text or LlmResponse::ToolCalls
        todo!()
    }
}
```
