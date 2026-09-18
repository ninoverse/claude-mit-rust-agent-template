<!-- agentcfg:start -->
<!-- agentcfg:import · v0.17.6 -->
@AGENTS.md

<!-- language/rust/automation.md · v0.17.6 -->
# Automation

`.claude/settings.json` allowlists the commands in *Build and test commands* so
they do not prompt, runs `rustfmt` on every `.rs` file you edit, and warns if the
workspace stops compiling when a turn ends. Formatting is therefore already
handled — do not run `cargo fmt` after each edit.
<!-- agentcfg:end -->
