# Rust AI Agent Template

[![CI](https://github.com/ninoverse/claude-mit-rust-agent-template/actions/workflows/ci.yml/badge.svg)](https://github.com/ninoverse/claude-mit-rust-agent-template/actions/workflows/ci.yml)
[![Audit](https://github.com/ninoverse/claude-mit-rust-agent-template/actions/workflows/audit.yml/badge.svg)](https://github.com/ninoverse/claude-mit-rust-agent-template/actions/workflows/audit.yml)

Scaffolding for building and running custom agentic AI systems in Rust. This repository provides a type-safe, modular 4-crate architecture that runs out of the box with zero external API dependencies via a built-in mock LLM provider. The bundled tools (a single-operator calculator and a clock) are illustrative starting points — swap in real tools and a real `LlmProvider` to build something production-grade.

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

### 2. Run the merge gates

The `justfile` is the single source of truth for every command — CI and the
`.claude/` rules call these recipes rather than repeating cargo invocations.

```bash
cargo install --locked just
just setup      # cargo-nextest, cargo-watch, cargo-deny, cargo-audit
just ci         # fmt-check · lint · test · deny — all four, before every commit
```

Individually: `just fmt-check`, `just lint`, `just test`, `just deny`, plus
`just build`, `just check`, `just watch`, `just doc`, `just audit`,
`just release` and `just docker-build`.

CI runs the same four recipes as separate jobs, so a red check names the gate
that broke, plus an MSRV job and a coverage artifact. The gate definitions live
in [`ninoverse/.github`](https://github.com/ninoverse/.github); the workflows
here only set the triggers.

### 3. The MSRV lives in four places

`Cargo.toml`, `clippy.toml`, the `Dockerfile` base image, and the `msrv` input
in `.github/workflows/ci.yml`. Raise it in all four together — the MSRV job
compares the last against the first, so a partial bump fails rather than
drifting.

It is **1.86**, and that is measured rather than chosen: `clap` and
`idna_adapter` are edition 2024, which cargo 1.84 cannot parse at all, and the
`icu_*` chain reached through `jsonschema` requires 1.86.

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
> In CI it is an input: pass `allow-unauthenticated: false` in
> `.github/workflows/release.yml` and the reusable workflow deploys with
> `--no-allow-unauthenticated` instead.

Tagging by hand is rarely what you want. `bump-version.yml` reads the subject of
each commit merged to `main` and pushes the tag itself — `feat` minor,
`fix`/`perf`/`refactor`/`chore`/`docs` patch, `!` or `BREAKING CHANGE` major. So
the commit convention picks the version number, and a hand-pushed tag competes
with it.

---

## Rule files

| File | Purpose |
|------|---------|
| `.claude/git-flow.md` | The branch → commit → PR loop. One branch in flight, no stacked PRs |
| `.claude/branch-naming.md` | Branch prefix and format conventions |
| `.claude/commit-conventions.md` | Conventional Commits rules — and the version bump |
| `.claude/pr-guidelines.md` | PR title, description template, size guidance |
| `.claude/testing-requirements.md` | Test gates (fmt, clippy, nextest, deny) |
| `.claude/file-naming.md` | Workspace and per-crate layout |
| `.claude/code-review.md` | Review checklist (lint, error handling, unsafe, docs, deps) |
| `.claude/crate-workflow.md` | Step-by-step procedure to add a crate |
| `.claude/execution-order.md` | What order to build things in, and one PR per what |

`.claude/settings.json` carries the permission allowlist and two hooks: rustfmt
on save, and `cargo check` when a turn ends. `.claude/commands/` adds `/gates`
and `/new-crate`. [`CONTRIBUTING.md`](CONTRIBUTING.md) is the short version of
all of it.

### Not in this repository

`SECURITY.md`, `CODE_OF_CONDUCT.md` and `.github/ISSUE_TEMPLATE/` come from
[`ninoverse/.github`](https://github.com/ninoverse/.github), which GitHub serves
as the default to every repository in the organization that has none of its own.

### If you forked this

`.github/workflows/ci.yml` and `audit.yml` call reusable workflows from that
same repository. It is public and the calls are pinned to `@v1`, so they keep
working with no setup — but the job definitions are then maintained by someone
else. To own them, copy
[`rust-ci.yml`](https://github.com/ninoverse/.github/blob/main/.github/workflows/rust-ci.yml)
and
[`rust-audit.yml`](https://github.com/ninoverse/.github/blob/main/.github/workflows/rust-audit.yml)
into your own `.github/workflows/` and drop the `uses:` line. They call the same
`just` recipes either way.

`bump-version.yml` and `release.yml` will **not** work in a fork as-is: they need
organization-level GitHub App credentials a fork does not inherit, on top of the
Google Cloud setup above. `renovate.json` points at the same organization's
preset — replace it with your own policy. GitHub also serves organization health
files only within the owning organization, so a fork inherits none of them.
