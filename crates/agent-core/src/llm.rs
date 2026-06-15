//! LLM abstractions, message structures, and mock implementation.

use crate::error::AgentError;
use crate::tool::ToolDefinition;
use serde::{Deserialize, Serialize};

/// Role of a message in a conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// System instruction that guides agent behavior.
    System,
    /// Human user message.
    User,
    /// AI agent assistant message.
    Assistant,
    /// Result payload from executing a tool.
    Tool,
}

/// A single call to a tool requested by the LLM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolCall {
    /// Unique identifier for this invocation instance.
    pub id: String,
    /// Name of the tool to invoke.
    pub name: String,
    /// Arguments as a JSON string.
    pub arguments: String,
}

/// A message in the chat history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Role of the speaker.
    pub role: MessageRole,
    /// Text content of the message. Optional if the message only contains tool calls.
    pub content: Option<String>,
    /// Tool calls requested by this message (Assistant role only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    /// ID of the tool call this message is responding to (Tool role only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl ChatMessage {
    /// Helper to create a system message.
    pub fn system<S: Into<String>>(content: S) -> Self {
        Self {
            role: MessageRole::System,
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// Helper to create a user message.
    pub fn user<S: Into<String>>(content: S) -> Self {
        Self {
            role: MessageRole::User,
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// Helper to create an assistant message with text content.
    pub fn assistant<S: Into<String>>(content: S) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// Helper to create an assistant message with tool calls.
    pub fn assistant_tool_calls(tool_calls: Vec<ToolCall>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: None,
            tool_calls: Some(tool_calls),
            tool_call_id: None,
        }
    }

    /// Helper to create a tool response message.
    pub fn tool_response<S: Into<String>>(tool_call_id: S, content: S) -> Self {
        Self {
            role: MessageRole::Tool,
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: Some(tool_call_id.into()),
        }
    }
}

/// Possible responses from the LLM provider.
#[derive(Debug, Clone)]
pub enum LlmResponse {
    /// Plain text reply.
    Text(String),
    /// A set of tool calls the agent must execute.
    ToolCalls(Vec<ToolCall>),
}

/// Abstract representation of an LLM provider.
#[async_trait::async_trait]
pub trait LlmProvider: Send + Sync {
    /// Generate a response based on the conversation history and available tools.
    async fn generate(
        &self,
        messages: &[ChatMessage],
        tools: &[ToolDefinition],
    ) -> Result<LlmResponse, AgentError>;
}

/// A mock LLM provider for testing and demonstration purposes.
/// It responds to simple keywords to trigger tool calls.
#[derive(Debug, Clone, Default)]
pub struct MockLlmProvider;

impl MockLlmProvider {
    /// Create a new mock provider.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl LlmProvider for MockLlmProvider {
    async fn generate(
        &self,
        messages: &[ChatMessage],
        tools: &[ToolDefinition],
    ) -> Result<LlmResponse, AgentError> {
        // Find the last user message or tool result to decide response logic.
        if let Some(last_msg) = messages.last() {
            match last_msg.role {
                MessageRole::User => {
                    let text = last_msg.content.as_deref().unwrap_or("");

                    // Trigger calculator tool if expression looks like math
                    if (text.contains('+')
                        || text.contains('-')
                        || text.contains('*')
                        || text.contains('/'))
                        && tools.iter().any(|t| t.name == "calculator")
                    {
                        return Ok(LlmResponse::ToolCalls(vec![ToolCall {
                            id: "call_calc_1".to_string(),
                            name: "calculator".to_string(),
                            arguments: serde_json::json!({ "expression": text.trim() }).to_string(),
                        }]));
                    }

                    // Trigger system info/time tool if asked about time
                    if text.to_lowercase().contains("time")
                        && tools.iter().any(|t| t.name == "get_time")
                    {
                        return Ok(LlmResponse::ToolCalls(vec![ToolCall {
                            id: "call_time_1".to_string(),
                            name: "get_time".to_string(),
                            arguments: "{}".to_string(),
                        }]));
                    }

                    // Default text response
                    Ok(LlmResponse::Text(format!(
                        "Hello! I am a mock agent. You said: \"{}\". No matching tools were triggered.",
                        text
                    )))
                }
                MessageRole::Tool => {
                    // LLM processes tool output and produces final answer
                    let tool_id = last_msg.tool_call_id.as_deref().unwrap_or("unknown");
                    let result = last_msg.content.as_deref().unwrap_or("null");
                    Ok(LlmResponse::Text(format!(
                        "Tool execution ({}) finished successfully. Result: {}. Based on this, your query has been resolved.",
                        tool_id, result
                    )))
                }
                _ => Ok(LlmResponse::Text("I'm waiting for your input.".to_string())),
            }
        } else {
            Ok(LlmResponse::Text("Conversation is empty.".to_string()))
        }
    }
}
