# Claude Code Rust Template

Claude Code configuration scaffolding for **Rust cargo-workspace** projects.
Fork or copy this repo to start a new Rust project that ships with the latest
stable toolchain, opinionated lint/format/test commands, and Claude Code rule
files already wired up.

## What's bundled

| File | Purpose |
|------|---------|
| `Cargo.toml` | Workspace root. `members = ["crates/*"]`, shared `[workspace.package]` and `[workspace.dependencies]`. |
| `rust-toolchain.toml` | Pins channel = `stable` so every contributor auto-pulls the latest stable Rust. |
| `rustfmt.toml` | Format config (edition 2021, 100-col, module-granular imports). |
| `clippy.toml` | MSRV pin for clippy lints. |
| `deny.toml` | `cargo-deny` config: allowed licenses, advisory denials, source restrictions. |
| `.gitignore` | Ignores `target/`. |
| `crates/` | Empty workspace member dir — add crates here via `cargo new --lib crates/<name>`. |
| `CLAUDE.md` | Top-level rules surfaced to Claude Code. |
| `.claude/*.md` | Per-task rule files (see table below). |

## Bootstrap a project from this template

```bash
# 1. Clone and rename
git clone https://github.com/ninoverse/claude_mit_rust_template my-project
cd my-project
rm -rf .git && git init

# 2. Update workspace identity in Cargo.toml
#    - [workspace.package].repository
#    - [workspace.package].license (if not MIT)

# 3. Install the auxiliary Rust tools (once per machine)
cargo install --locked cargo-nextest cargo-watch cargo-deny cargo-audit

# 4. Add your first crate
cargo new --lib crates/<your-crate>

# 5. Verify the toolchain and workspace
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace
```

## Daily commands

```bash
cargo build --workspace
cargo watch -x 'check --workspace'
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all
cargo nextest run --workspace
cargo deny check
```

## Rule files

| File | Purpose |
|------|---------|
| `.claude/branch-naming.md` | Branch prefix and format conventions |
| `.claude/commit-conventions.md` | Conventional Commits rules |
| `.claude/pr-guidelines.md` | PR title, description template, size guidance |
| `.claude/testing-requirements.md` | Test gates (fmt, clippy, nextest, deny) |
| `.claude/file-naming.md` | Workspace and per-crate layout |
| `.claude/code-review.md` | Review checklist (lint, error handling, unsafe, docs, deps) |
| `.claude/crate-workflow.md` | Step-by-step procedure to add a crate |
| `.claude/execution-order.md` | Branching strategy for crate groups |
