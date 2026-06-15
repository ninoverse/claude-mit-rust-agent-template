# Rust AI Agent Template

Scaffolding for building and running custom agentic AI systems in Rust. This repository provides a type-safe, modular 3-crate architecture that runs out of the box with zero external API dependencies via a built-in mock LLM provider. The bundled tools (a single-operator calculator and a clock) are illustrative starting points — swap in real tools and a real `LlmProvider` to build something production-grade.

## Workspace Layout

The project is structured as a cargo workspace with the following crates:

| Crate | Path | Purpose |
|-------|------|---------|
| `agent-core` | `crates/agent-core` | Foundation traits & struct orchestrators: LLM providers, Memory/History, Tools, and the Agent Execution Loop. |
| `agent-tools` | `crates/agent-tools` | Built-in out-of-the-box agent tools: mathematical calculations, system info/time, etc. |
| `agent-cli` | `crates/agent-cli` | Interactive CLI application providing a colored, user-friendly terminal interface to chat with the agent. |
| `agent-server` | `crates/agent-server` | HTTP server (`/chat` JSON API) wrapping the agent, deployable to Cloud Run via the repo-root `Dockerfile`. |

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

---

## Deploying to Cloud Run

The `agent-server` crate is packaged by the repo-root `Dockerfile` and deployed to
[Google Cloud Run](https://cloud.google.com/run) by `.github/workflows/release.yml`
on every `v*.*.*` tag push. Cloud Run injects `PORT`, which the server reads.

### 1. Prerequisites (one-time, local)

Install the [gcloud CLI](https://cloud.google.com/sdk/docs/install), then authenticate
and enable the required APIs:

```bash
gcloud auth login
gcloud config set project YOUR_PROJECT_ID

gcloud services enable \
  run.googleapis.com \
  cloudbuild.googleapis.com \
  artifactregistry.googleapis.com
```

### 2. Create a deploy service account

CI authenticates as a dedicated service account. Create it and grant the roles
needed to build (Cloud Build), store the image (Artifact Registry), and deploy
(Cloud Run):

```bash
PROJECT_ID=YOUR_PROJECT_ID
SA_NAME=github-deployer
SA_EMAIL="${SA_NAME}@${PROJECT_ID}.iam.gserviceaccount.com"

gcloud iam service-accounts create "$SA_NAME" \
  --display-name="GitHub Actions deployer"

for role in \
  roles/run.admin \
  roles/cloudbuild.builds.editor \
  roles/artifactregistry.writer \
  roles/iam.serviceAccountUser \
  roles/storage.admin
do
  gcloud projects add-iam-policy-binding "$PROJECT_ID" \
    --member="serviceAccount:${SA_EMAIL}" \
    --role="$role"
done
```

### 3. Generate the service account key

```bash
gcloud iam service-accounts keys create key.json \
  --iam-account="$SA_EMAIL"
```

This writes `key.json` — the JSON you'll store as the `GCP_SERVICE_ACCOUNT` secret.
It is gitignored, but **delete it after uploading** (`rm key.json`) so the key
doesn't linger on disk.

### 4. Configure the GitHub repository

Under **Settings → Secrets and variables → Actions**:

| Kind | Name | Value |
|------|------|-------|
| Secret | `GCP_SERVICE_ACCOUNT` | Full contents of `key.json` |
| Variable | `GCP_PROJECT` | Your GCP project ID |
| Variable | `GCP_REGION` | Deploy region, e.g. `europe-west1` |

With the [`gh`](https://cli.github.com/) CLI:

```bash
gh secret set GCP_SERVICE_ACCOUNT < key.json
gh variable set GCP_PROJECT --body "YOUR_PROJECT_ID"
gh variable set GCP_REGION  --body "europe-west1"
```

### 5. Deploy

Push a release tag (or let `bump-version.yml` create one on merge to `main`):

```bash
git tag v0.1.0
git push origin v0.1.0
```

The workflow builds the `Dockerfile` via Cloud Build and deploys the image. To
deploy manually without CI:

```bash
gcloud run deploy agent-server \
  --source . \
  --region "$GCP_REGION" \
  --allow-unauthenticated
```

> **Security note:** `--allow-unauthenticated` exposes the `/chat` endpoint publicly.
> Drop that flag (and call the service with an identity token) to keep it private.
