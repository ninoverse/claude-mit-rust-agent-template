//! # Agent Tools
//!
//! A collection of default tools that can be registered with the agent-core registry.
//!
//! Available tools:
//! - **Calculator**: [`calculator::CalculatorTool`] for simple arithmetic.
//! - **Time**: [`time::TimeTool`] to check current system time.

pub mod calculator;
pub mod time;
