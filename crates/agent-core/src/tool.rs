//! Tool traits and definitions for agent actions.

use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Schema representing a tool parameter description (typically JSON Schema).
#[derive(Debug, Clone, Serialize)]
pub struct ToolDefinition {
    /// The name of the tool (must be unique, snake_case recommended).
    pub name: String,
    /// Detailed description of what the tool does and when to use it.
    pub description: String,
    /// JSON Schema of parameters required by the tool.
    pub parameters: Value,
}

/// Trait that any tool must implement to be usable by the Agent.
#[async_trait::async_trait]
pub trait Tool: Send + Sync {
    /// Returns the tool's schema definition.
    fn definition(&self) -> ToolDefinition;

    /// Executes the tool action with parsed JSON arguments.
    async fn call(&self, arguments: Value) -> Result<Value, anyhow::Error>;
}

/// Registry of tools available to the agent.
#[derive(Clone, Default)]
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Registers a tool in the registry.
    pub fn register<T>(&mut self, tool: T)
    where
        T: Tool + 'static,
    {
        let definition = tool.definition();
        self.tools.insert(definition.name, Arc::new(tool));
    }

    /// Registers an Arc-wrapped tool.
    pub fn register_arc(&mut self, tool: Arc<dyn Tool>) {
        let definition = tool.definition();
        self.tools.insert(definition.name.clone(), tool);
    }

    /// Looks up a tool by name.
    pub fn get(&self, name: &str) -> Option<&Arc<dyn Tool>> {
        self.tools.get(name)
    }

    /// List all tool definitions in the registry.
    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools.values().map(|t| t.definition()).collect()
    }
}
