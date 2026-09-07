//! A tool for performing basic mathematical calculations.

use agent_core::tool::{Tool, ToolDefinition};
use serde_json::{Value, json};
use std::str::FromStr;

/// A tool that evaluates simple mathematical expressions (addition, subtraction, multiplication, division).
#[derive(Default)]
pub struct CalculatorTool {}

impl CalculatorTool {
    /// Create a new instance of CalculatorTool.
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait::async_trait]
impl Tool for CalculatorTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "calculator".to_string(),
            description: "Evaluates simple mathematical expressions. Supports addition (+), subtraction (-), multiplication (*), and division (/).".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "expression": {
                        "type": "string",
                        "description": "The expression to evaluate, e.g. '12 + 4.5'"
                    }
                },
                "required": ["expression"]
            }),
        }
    }

    async fn call(&self, arguments: Value) -> Result<Value, anyhow::Error> {
        let expression = arguments["expression"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'expression' argument"))?;

        // Simple single-operator evaluator
        let clean_expr: String = expression.chars().filter(|c| !c.is_whitespace()).collect();

        let result = if let Some(pos) = clean_expr.find('+') {
            let left = f64::from_str(&clean_expr[..pos])?;
            let right = f64::from_str(&clean_expr[pos + 1..])?;
            left + right
        } else if let Some(pos) = clean_expr.find('-') {
            let left = f64::from_str(&clean_expr[..pos])?;
            let right = f64::from_str(&clean_expr[pos + 1..])?;
            left - right
        } else if let Some(pos) = clean_expr.find('*') {
            let left = f64::from_str(&clean_expr[..pos])?;
            let right = f64::from_str(&clean_expr[pos + 1..])?;
            left * right
        } else if let Some(pos) = clean_expr.find('/') {
            let left = f64::from_str(&clean_expr[..pos])?;
            let right = f64::from_str(&clean_expr[pos + 1..])?;
            if right == 0.0 {
                return Err(anyhow::anyhow!("Division by zero"));
            }
            left / right
        } else {
            return Err(anyhow::anyhow!(
                "Unsupported expression or operator not found. Only single operations of +, -, *, / are supported."
            ));
        };

        Ok(json!({
            "expression": expression,
            "result": result
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_calculator_tool() {
        let tool = CalculatorTool::new();

        let args = json!({ "expression": "15 + 27" });
        let res = tool.call(args).await.unwrap();
        assert_eq!(res["result"].as_f64().unwrap(), 42.0);

        let args = json!({ "expression": "100 / 4" });
        let res = tool.call(args).await.unwrap();
        assert_eq!(res["result"].as_f64().unwrap(), 25.0);
    }
}
