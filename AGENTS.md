Scaffolding for new Rust agent projects: a tool-calling agent core, an Axum server and a CLI, with the merge gates wired to CI and the agent rules already in place. Fork or copy it to start a project.

<!-- agentcfg:start -->
<!-- language/rust/tooling.md · v0.17.6 -->
# Build and test commands

**Toolchain:** Rust is pinned via `rust-toolchain.toml` (channel = `stable`). Every contributor automatically gets the latest stable toolchain on first `cargo` invocation. Required components: `rustfmt`, `clippy`.

Commands live in the `justfile`, which is the single source of truth — do not
copy the underlying cargo invocations into docs or CI, call the recipe.

```bash
just            # list every recipe
just ci         # all four merge gates, in order — run this before every commit
just build      # build all crates
just check      # type-check (faster than build)
just watch      # dev loop, re-checks on save
just fmt        # format in place
just fmt-check  # gate 1
just lint       # gate 2 — clippy, warnings as errors
just test       # gate 3 — nextest (falls back to cargo test) plus doc-tests
just deny       # gate 4 — licenses + advisories
just audit      # CVE check
just release    # optimized build
```

Install `just` and the auxiliary tools once per machine:

```bash
cargo install --locked just
just setup      # cargo-nextest, cargo-watch, cargo-deny, cargo-audit
```

Without `just`, `.cargo/config.toml` defines `cargo lint`, `cargo fmt-check` and
`cargo check-all`. There is no `cargo ci` — a cargo alias can only wrap one
subcommand, so the four gates have to be run in sequence.

## Architecture & Workspace Rules

**Layout:** Cargo workspace, edition `2024`. New code goes in a crate under `crates/<name>/`. The workspace root `Cargo.toml` declares `members = ["crates/*"]` and centralizes shared metadata under `[workspace.package]` and shared dependencies under `[workspace.dependencies]`.

**Crate inheritance:** Crate manifests inherit shared keys from the workspace using `<key>.workspace = true` (e.g. `edition.workspace = true`, `license.workspace = true`). Shared dependencies are referenced as `<crate> = { workspace = true }`.

**MSRV:** Declared in `[workspace.package].rust-version`, `clippy.toml`, and the `msrv` input in `.github/workflows/ci.yml`. Raising it means editing all of them together; the MSRV job compares the last against the first and fails a partial bump. Do not bump it incidentally.

<!-- core/behavior.md · v0.17.6 -->
# Behavioral guidelines

**Maintain the Build:** Never leave the codebase in a state where build, lint,
or tests fail. Run the relevant commands in *Build and test commands* to verify
your work before concluding a task.

**Tradeoff:** Bias toward caution over speed. For trivial tasks, use judgment.

## 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

- State your assumptions explicitly. If uncertain, stop and ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, propose it. Push back when warranted.

## 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked. No abstractions for single-use code.
- No "flexibility" or error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

## 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

- Don't "improve" adjacent code, comments, or formatting.
- Match existing style exactly.
- Remove imports/variables/functions that YOUR changes made unused. Don't remove pre-existing dead code unless asked.

## 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

- Transform tasks into verifiable goals (e.g., "Add validation" → "Write tests for invalid inputs, then make them pass").
- For multi-step tasks, state a brief plan and verify each step independently.

<!-- concerns/template/rules.md · v0.17.6 -->
# Template repository

This repository is a GitHub template: new projects start as a copy of it, and
every copy inherits everything here.

- Keep the example code minimal. It demonstrates the conventions and keeps the
  gates green on a fresh copy; it holds no real business logic.
- Where the template ships placeholder crates, they exist because the
  toolchain fails on an empty workspace and the `Dockerfile` needs a
  binary to build. Remove a placeholder only once a real crate covers its
  role, as a change of its own, and point the `Dockerfile` at the real binary in
  that change.
- A project created from this template removes `template` from `concerns` in its
  `.agentprofile.yml`.

<!-- agentcfg:index · v0.17.6 -->
# Extended rules

Read these when they apply; they are not loaded by default.

**By activity:**

- **Any change that ends in a PR:** [Git flow](.agents/git-flow.md) and [Releases and deploys](.agents/service-release.md)
- **Creating branches:** [Branch naming](.agents/branch-naming.md)
- **Reviewing PRs:** [Code review](.agents/code-review.md) and [Rust code review](.agents/rust-code-review.md)
- **Committing code:** [Commit message guidelines](.agents/commit-conventions.md)
- **Deciding what to build next / branching strategy:** [Execution order](.agents/execution-order.md)
- **Opening PRs:** [PR instructions](.agents/pr-guidelines.md)
- **Creating new files:** [Directories and file naming](.agents/rust-file-naming.md)
- **Checking your work:** [Merge gates](.agents/gates.md)
- **Adding or modifying a crate:** [Adding a crate](.agents/new-crate.md)
- **Testing/Verifying:** [Testing instructions](.agents/rust-testing.md)
<!-- agentcfg:end -->
