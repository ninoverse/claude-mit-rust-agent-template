//! # Agent Core
//!
//! A modular and extensible scaffolding for building AI Agents in Rust.
//!
//! This crate provides the foundational building blocks:
//! - **LLM Abstraction**: Define and interface with different Large Language Models via [`llm::LlmProvider`].
//! - **Tool Registry**: Define and register custom executable functions via [`tool::Tool`] and [`tool::ToolRegistry`].
//! - **Memory Management**: Track chat history in memory ([`memory::SimpleMemory`]) or on disk ([`memory::FileMemory`]).
//! - **Execution Loop**: Orchestrate the agent reasoning and actions using [`agent::Agent`].

pub mod agent;
pub mod error;
pub mod llm;
pub mod memory;
pub mod tool;

#[cfg(test)]
mod tests {
    use super::*;
    use agent::Agent;
    use error::AgentError;
    use llm::MockLlmProvider;
    use memory::SimpleMemory;
    use serde_json::{Value, json};
    use tool::{Tool, ToolDefinition, ToolRegistry};

    struct TestTool;

    #[async_trait::async_trait]
    impl Tool for TestTool {
        fn definition(&self) -> ToolDefinition {
            ToolDefinition {
                name: "calculator".to_string(),
                description: "Basic calculator tool".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "expression": { "type": "string" }
                    },
                    "required": ["expression"]
                }),
            }
        }

        async fn call(&self, arguments: Value) -> Result<Value, anyhow::Error> {
            let expr = arguments["expression"].as_str().unwrap_or("");
            if expr.contains("2 + 2") {
                Ok(json!({ "result": 4 }))
            } else {
                Ok(json!({ "result": 0 }))
            }
        }
    }

    #[tokio::test]
    async fn test_agent_with_mock_llm_and_tool() -> Result<(), AgentError> {
        let mut registry = ToolRegistry::new();
        registry.register(TestTool);

        let provider = MockLlmProvider::new();
        let memory = SimpleMemory::new();
        let mut agent = Agent::new(provider, registry, memory);

        let response = agent.prompt("what is 2 + 2").await?;
        assert!(response.contains("Result: {\"result\":4}"));
        Ok(())
    }

    struct RejectingHook;

    #[async_trait::async_trait]
    impl agent::ToolExecutionHook for RejectingHook {
        async fn approve(&self, _tool_name: &str, _arguments: &Value) -> bool {
            false
        }
    }

    #[tokio::test]
    async fn test_agent_schema_validation_failure() -> Result<(), AgentError> {
        let mut registry = ToolRegistry::new();
        registry.register(TestTool);

        struct InvalidArgsProvider;
        #[async_trait::async_trait]
        impl llm::LlmProvider for InvalidArgsProvider {
            async fn generate(
                &self,
                messages: &[llm::ChatMessage],
                _tools: &[ToolDefinition],
            ) -> Result<llm::LlmResponse, AgentError> {
                if messages.len() == 1 {
                    Ok(llm::LlmResponse::ToolCalls(vec![llm::ToolCall {
                        id: "call_calc_invalid".to_string(),
                        name: "calculator".to_string(),
                        arguments: json!({ "wrong_param": 123 }).to_string(),
                    }]))
                } else {
                    let last = messages.last().unwrap();
                    let content = last.content.as_ref().unwrap();
                    assert!(content.contains("validation failed") || content.contains("required"));
                    Ok(llm::LlmResponse::Text(
                        "Handled validation error".to_string(),
                    ))
                }
            }
        }

        let memory = SimpleMemory::new();
        let mut agent = Agent::new(InvalidArgsProvider, registry, memory);
        let res = agent.prompt("trigger invalid").await?;
        assert_eq!(res, "Handled validation error");
        Ok(())
    }

    #[tokio::test]
    async fn test_agent_hook_rejection() -> Result<(), AgentError> {
        let mut registry = ToolRegistry::new();
        registry.register(TestTool);

        let provider = MockLlmProvider::new();
        let memory = SimpleMemory::new();
        let mut agent = Agent::new(provider, registry, memory).with_execution_hook(RejectingHook);

        let response = agent.prompt("what is 2 + 2").await?;
        assert!(response.contains("rejected by user"));
        Ok(())
    }
}
