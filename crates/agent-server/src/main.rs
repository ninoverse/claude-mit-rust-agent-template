//! HTTP server exposing the AI Agent over a small JSON API.
//!
//! Designed to run on Firebase App Hosting (or any container platform): it binds
//! to the port given by the `PORT` environment variable, defaulting to `8080`.

use agent_core::agent::Agent;
use agent_core::llm::MockLlmProvider;
use agent_core::memory::SimpleMemory;
use agent_core::tool::ToolRegistry;
use agent_tools::calculator::CalculatorTool;
use agent_tools::time::TimeTool;
use axum::{routing::get, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use tracing_subscriber::EnvFilter;

/// Request body for `POST /chat`.
#[derive(Debug, Deserialize)]
struct ChatRequest {
    message: String,
}

/// Response body for `POST /chat`.
#[derive(Debug, Serialize)]
struct ChatResponse {
    response: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("agent_server=info,agent_core=info,warn")),
        )
        .with_target(false)
        .init();

    let app = Router::new()
        .route("/", get(health))
        .route("/chat", post(chat));

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let addr = format!("0.0.0.0:{port}");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!(%addr, "agent-server listening");
    axum::serve(listener, app).await?;
    Ok(())
}

/// Liveness/readiness probe.
async fn health() -> &'static str {
    "ok"
}

/// Run a single stateless prompt through the agent and return the final reply.
///
/// Each request gets a fresh [`SimpleMemory`], so conversations are not retained
/// across calls. Swap in a shared, keyed memory backend to support sessions.
async fn chat(Json(req): Json<ChatRequest>) -> Json<ChatResponse> {
    let mut registry = ToolRegistry::new();
    registry.register(CalculatorTool::new());
    registry.register(TimeTool::new());

    let mut agent = Agent::new(MockLlmProvider::new(), registry, SimpleMemory::new());

    let response = match agent.prompt(&req.message).await {
        Ok(text) => text,
        Err(e) => {
            tracing::warn!(error = %e, "agent prompt failed");
            format!("error: {e}")
        }
    };

    Json(ChatResponse { response })
}
