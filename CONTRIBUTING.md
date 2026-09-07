# Contributing

The rules that govern this repository live in [`CLAUDE.md`](CLAUDE.md) and
[`.claude/`](.claude/). They are written for Claude Code but they are not
agent-specific — they are the conventions, and they apply to humans identically.
This file is the short version and points at the authoritative one for each
topic.

## Setup

```bash
cargo install --locked just
just setup      # cargo-nextest, cargo-watch, cargo-deny, cargo-audit
just ci         # confirm a clean checkout passes
```

The toolchain installs itself: [`rust-toolchain.toml`](rust-toolchain.toml) pins
the channel, so the first `cargo` command pulls the right compiler.

To run the agent rather than just build it, copy `.env.example` to `.env` and
fill in a provider key. `.env` is gitignored; never commit one.

## The loop

One branch, one commit, one PR, merged before the next begins. No stacked PRs.
Full rules in [`.claude/git-flow.md`](.claude/git-flow.md).

```bash
git switch main && git pull --ff-only
git switch -c <type>/<short-description>     # .claude/branch-naming.md
# ... change ...
just ci                                      # must pass before you push
git commit                                   # .claude/commit-conventions.md
git push -u origin <branch>
```

Then open a PR using the template. If Claude Code prepared the branch, it stops
before opening the PR by design — that step is yours.

Because a branch is only pushed once the gates already pass, there is no
work-in-progress state to represent. Draft PRs are not used.

## The four gates

```bash
just ci
```

`fmt-check` · `lint` · `test` · `deny`. All four, zero warnings, before you push.
CI runs the same recipes, one job per gate, plus an MSRV job.
See [`.claude/testing-requirements.md`](.claude/testing-requirements.md).

## Adding a crate

Follow the nine steps in
[`.claude/crate-workflow.md`](.claude/crate-workflow.md).

Three things that are easy to miss and that the gates will catch:

- `[lints] workspace = true` in the crate manifest. Without it the crate opts out
  of the workspace lints entirely, and `just lint` becomes plain clippy.
- `#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic_in_result_fn)]`
  inside the test module. Those are workspace lints and fire in test targets too;
  the third is needed as soon as a test returns `Result` and asserts.
- A `Debug` impl on every public type. `missing_debug_implementations` is on, and
  a type holding a `dyn` trait object needs a written-out impl rather than a
  derive — see `ToolRegistry` and `Agent` for the two shapes that come up.

## The MSRV lives in four places

`Cargo.toml`, `clippy.toml`, the `Dockerfile` base image, and the `msrv` input in
`.github/workflows/ci.yml`. Raising it means editing all four together. CI
compares the last against the first on every run, so a partial bump fails rather
than drifting.

It is currently **1.86**, and that is not arbitrary: `clap` and `idna_adapter`
are edition 2024, which cargo 1.84 cannot parse at all, and the `icu_*` chain
reached through `jsonschema` requires 1.86.

## Releases

Merging to `main` bumps `[workspace.package].version` from the commit subject and
pushes a matching tag — `feat` minor, `fix`/`perf`/`refactor`/`chore`/`docs`
patch, `!` or `BREAKING CHANGE` major. Anything else bumps nothing. So the commit
convention is not only documentation: it picks the version number.

The tag then triggers the Cloud Run deploy, which needs `vars.GCP_PROJECT`,
`vars.GCP_REGION` and `secrets.GCP_SERVICE_ACCOUNT`. Neither workflow is defined
here; both call
[`ninoverse/.github`](https://github.com/ninoverse/.github).

## Dependency updates

Renovate opens them. It runs **centrally**, from
[`ninoverse/.github`](https://github.com/ninoverse/.github), so there is no
workflow and no token in this repository. `renovate.json` here is one line
extending the shared preset; deleting it would opt this repository out.

Review the changelog rather than rubber-stamping, and check that the MSRV job
still passes: a dependency raising *its* MSRV is the usual reason that job goes
red, and this workspace is already held at 1.86 by one. Majors wait for approval
on the Dependency Dashboard issue; everything non-breaking arrives as one grouped
PR on Monday. Security fixes ignore the schedule entirely.

The shared preset is configured **not** to touch `dtolnay/rust-toolchain`. The
MSRV job pins it deliberately — bumping it would leave the job green while it
quietly stopped testing anything.

Two things in [`deny.toml`](deny.toml) are worth knowing before you change them:

- `Unicode-3.0` and `MIT-0` are allowed. The whole `icu_*` chain relicensed to
  the first; `borrow-or-share` uses the second, which is MIT minus attribution.
- `allow-wildcard-paths` plus `publish = false` are what let the crates depend on
  each other by path. cargo-deny reads a path dependency with no version as a
  wildcard and only exempts one on a private crate. Publishing a crate from here
  means giving its path dependencies explicit versions.

Anything that should change for *every* project — the schedule, the grouping,
the major-approval gate — belongs in the org preset, not here.

## If you forked this

Four things point at `ninoverse` and will not work as-is:

- `renovate.json` extends `github>ninoverse/.github`. Replace it with your own
  policy, or point it at your own preset.
- `.github/workflows/ci.yml` and `audit.yml` call reusable workflows from that
  same repository. They are public and pinned to `@v1`, so they keep working —
  see the README for how to vendor them instead.
- `.github/workflows/bump-version.yml` and `release.yml` do the same, and also
  need organization-level GitHub App and Google Cloud credentials that a fork
  does not inherit.
- `.github/CODEOWNERS` names `@nicolapasqua99`.

`SECURITY.md`, `CODE_OF_CONDUCT.md` and the issue forms are **not** in this
repository; they come from the organization defaults, which a fork does not
inherit. Add your own.
