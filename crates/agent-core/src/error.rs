//! Error types for the agent-core crate.

use thiserror::Error;

/// Core error type for the agent-core crate.
#[derive(Debug, Error)]
pub enum AgentError {
    /// Errors originating from the LLM provider.
    #[error("LLM provider error: {0}")]
    Llm(String),

    /// Errors originating from a tool execution.
    #[error("Tool error ({tool_name}): {source}")]
    Tool {
        /// Name of the tool that failed.
        tool_name: String,
        /// Underlying error cause.
        source: anyhow::Error,
    },

    /// Error when a tool is requested by the LLM but not registered in the agent.
    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    /// Serialization/deserialization errors.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Any other custom/internal errors.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
