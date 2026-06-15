//! A tool for retrieving the current system time and date.

use agent_core::tool::{Tool, ToolDefinition};
use chrono::Local;
use serde_json::{json, Value};

/// A tool that returns the current local system date and time.
#[derive(Default)]
pub struct TimeTool {}

impl TimeTool {
    /// Create a new instance of TimeTool.
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait::async_trait]
impl Tool for TimeTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "get_time".to_string(),
            description: "Gets the current system time and date.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    async fn call(&self, _arguments: Value) -> Result<Value, anyhow::Error> {
        let now = Local::now();
        Ok(json!({
            "datetime": now.to_rfc3339(),
            "formatted": now.format("%A, %B %d, %Y - %H:%M:%S").to_string()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_time_tool() {
        let tool = TimeTool::new();
        let res = tool.call(json!({})).await.unwrap();
        assert!(res["datetime"].as_str().is_some());
        assert!(res["formatted"].as_str().is_some());
    }
}
