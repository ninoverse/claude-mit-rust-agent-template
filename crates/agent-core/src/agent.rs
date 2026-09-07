//! Agent orchestrator that runs the message-completion-execution loop.

use crate::error::AgentError;
use crate::llm::{ChatMessage, LlmProvider, LlmResponse};
use crate::memory::Memory;
use crate::tool::ToolRegistry;
use std::fmt;
use std::sync::Arc;

/// Hook invoked before executing a tool to request user or system confirmation.
#[async_trait::async_trait]
pub trait ToolExecutionHook: Send + Sync {
    /// Request approval to run the tool. Return `true` if approved, `false` if rejected.
    async fn approve(&self, tool_name: &str, arguments: &serde_json::Value) -> bool;
}

/// Core AI Agent orchestrator.
pub struct Agent<P, M>
where
    P: LlmProvider,
    M: Memory,
{
    provider: P,
    registry: ToolRegistry,
    memory: M,
    max_steps: usize,
    execution_hook: Option<Arc<dyn ToolExecutionHook>>,
}

// Written out rather than derived: a derive would add `P: Debug, M: Debug`
// bounds that no provider or memory implementation is obliged to satisfy, and
// `dyn ToolExecutionHook` could not satisfy one at all. The remaining fields are
// what is actually useful to see, and `finish_non_exhaustive` says so rather
// than pretending this is the whole struct.
impl<P, M> fmt::Debug for Agent<P, M>
where
    P: LlmProvider,
    M: Memory,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Agent")
            .field("registry", &self.registry)
            .field("max_steps", &self.max_steps)
            .field("execution_hook", &self.execution_hook.is_some())
            .finish_non_exhaustive()
    }
}

impl<P, M> Agent<P, M>
where
    P: LlmProvider,
    M: Memory,
{
    /// Create a new Agent instance.
    pub fn new(provider: P, registry: ToolRegistry, memory: M) -> Self {
        Self {
            provider,
            registry,
            memory,
            max_steps: 10,
            execution_hook: None,
        }
    }

    /// Set the maximum number of loop steps for a single run to prevent infinite loops.
    pub fn with_max_steps(mut self, max_steps: usize) -> Self {
        self.max_steps = max_steps;
        self
    }

    /// Configure an execution approval hook for tool execution.
    pub fn with_execution_hook<H>(mut self, hook: H) -> Self
    where
        H: ToolExecutionHook + 'static,
    {
        self.execution_hook = Some(Arc::new(hook));
        self
    }

    /// Access the agent's memory registry.
    pub fn memory(&self) -> &M {
        &self.memory
    }

    /// Mutable access to the agent's memory.
    pub fn memory_mut(&mut self) -> &mut M {
        &mut self.memory
    }

    /// Access the agent's tool registry.
    pub fn registry(&self) -> &ToolRegistry {
        &self.registry
    }

    /// Process a new user message, executing any requested tool calls in a loop
    /// until a final text response is produced.
    pub async fn prompt(&mut self, input: &str) -> Result<String, AgentError> {
        tracing::info!(user_input = %input, "Received user prompt");
        self.memory.add_message(ChatMessage::user(input));

        for step in 0..self.max_steps {
            tracing::debug!(step = step, "Starting agent loop step");

            let tools = self.registry.definitions();
            let response = self
                .provider
                .generate(self.memory.messages(), &tools)
                .await?;

            match response {
                LlmResponse::Text(text) => {
                    tracing::info!(response = %text, "Received final text response from LLM");
                    self.memory
                        .add_message(ChatMessage::assistant(text.clone()));
                    return Ok(text);
                }
                LlmResponse::ToolCalls(calls) => {
                    tracing::info!(calls_count = calls.len(), "LLM requested tool execution");

                    // First record the assistant message containing the tool calls
                    self.memory
                        .add_message(ChatMessage::assistant_tool_calls(calls.clone()));

                    // Execute each tool requested
                    for call in calls {
                        tracing::info!(
                            tool_name = %call.name,
                            tool_id = %call.id,
                            arguments = %call.arguments,
                            "Executing tool"
                        );

                        let result_content = match self.registry.get(&call.name) {
                            Some(tool) => {
                                // Parse arguments
                                match serde_json::from_str::<serde_json::Value>(&call.arguments) {
                                    Ok(args) => {
                                        // 1. JSON Schema Validation (Point 5)
                                        let schema_value = tool.definition().parameters;
                                        let validation_error =
                                            match jsonschema::validator_for(&schema_value) {
                                                Ok(validator) => {
                                                    if let Err(err) = validator.validate(&args) {
                                                        Some(format!(
                                                            "Path '{}': {}",
                                                            err.instance_path, err
                                                        ))
                                                    } else {
                                                        None
                                                    }
                                                }
                                                Err(err) => Some(format!(
                                                    "Invalid tool parameter schema: {}",
                                                    err
                                                )),
                                            };

                                        if let Some(err_msg) = validation_error {
                                            tracing::warn!(
                                                tool_name = %call.name,
                                                error = %err_msg,
                                                "Tool arguments failed JSON Schema validation"
                                            );
                                            serde_json::json!({
                                                "error": format!("Argument validation failed: {}", err_msg)
                                            })
                                        } else {
                                            // 2. Safety Execution Hook Approval (Point 4)
                                            let approved = if let Some(ref hook) =
                                                self.execution_hook
                                            {
                                                tracing::debug!(tool_name = %call.name, "Requesting execution hook approval");
                                                hook.approve(&call.name, &args).await
                                            } else {
                                                true
                                            };

                                            if !approved {
                                                tracing::warn!(tool_name = %call.name, "Tool execution rejected by hook");
                                                serde_json::json!({
                                                    "error": "Tool execution rejected by user"
                                                })
                                            } else {
                                                // Execute the tool
                                                match tool.call(args).await {
                                                    Ok(val) => {
                                                        tracing::debug!(
                                                            tool_name = %call.name,
                                                            result = %val,
                                                            "Tool execution succeeded"
                                                        );
                                                        val
                                                    }
                                                    Err(err) => {
                                                        tracing::warn!(
                                                            tool_name = %call.name,
                                                            error = %err,
                                                            "Tool execution failed"
                                                        );
                                                        serde_json::json!({
                                                            "error": err.to_string()
                                                        })
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        tracing::error!(
                                            tool_name = %call.name,
                                            error = %err,
                                            "Failed to parse tool arguments"
                                        );
                                        serde_json::json!({
                                            "error": format!("Invalid JSON arguments: {}", err)
                                        })
                                    }
                                }
                            }
                            None => {
                                tracing::warn!(tool_name = %call.name, "Requested tool not found");
                                serde_json::json!({
                                    "error": format!("Tool '{}' not found", call.name)
                                })
                            }
                        };

                        // Feed the tool result back into history
                        let result_str = serde_json::to_string(&result_content)?;
                        self.memory
                            .add_message(ChatMessage::tool_response(call.id, result_str));
                    }
                }
            }
        }

        Err(AgentError::Llm(format!(
            "Exceeded max execution steps ({}) without completing the task",
            self.max_steps
        )))
    }
}
