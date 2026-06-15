//! CLI entrypoint for running the AI Agent interactively.

use agent_core::agent::Agent;
use agent_core::llm::MockLlmProvider;
use agent_core::memory::{FileMemory, Memory, SimpleMemory};
use agent_core::tool::ToolRegistry;
use agent_tools::calculator::CalculatorTool;
use agent_tools::time::TimeTool;
use clap::Parser;
use std::io::{self, Write};
use tracing_subscriber::EnvFilter;

/// CLI arguments for the AI Agent CLI.
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Interactive CLI to chat with the AI Agent",
    long_about = None
)]
struct Args {
    /// Path to a .env file to load (defaults to `.env` in the current directory).
    #[arg(long, default_value = ".env")]
    env_file: String,

    /// Override the log filter (e.g. "debug", "agent_core=trace"). Takes precedence
    /// over the RUST_LOG environment variable.
    #[arg(long)]
    log: Option<String>,

    /// Path to the JSON file used to persist conversation history across sessions.
    /// Defaults to `history.json` in the current directory.
    #[arg(long, default_value = "history.json")]
    history_file: String,

    /// Disable persistent history; use in-memory storage only for this session.
    #[arg(long, default_value_t = false)]
    no_persist: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Load environment variables from the specified .env file.
    // Errors are silently ignored (file may legitimately not exist in production).
    match dotenvy::from_filename(&args.env_file) {
        Ok(path) => eprintln!("Loaded env from: {}", path.display()),
        Err(dotenvy::Error::Io(_)) => {} // file not found — no-op
        Err(e) => eprintln!("Warning: failed to parse {}: {}", args.env_file, e),
    }

    // Resolve the log filter in priority order:
    //   1. --log flag
    //   2. RUST_LOG env var (set by .env or the shell)
    //   3. hard-coded fallback
    let env_filter = if let Some(log_arg) = args.log.as_deref() {
        EnvFilter::try_new(log_arg).unwrap_or_else(|_| EnvFilter::new("warn"))
    } else {
        EnvFilter::try_from_env("RUST_LOG")
            .unwrap_or_else(|_| EnvFilter::new("agent_core=info,warn"))
    };

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .with_writer(io::stderr)
        .init();

    println!("\x1b[1;36m==================================================\x1b[0m");
    println!("\x1b[1;36m           AI AGENT INTERACTIVE SHELL             \x1b[0m");
    println!("\x1b[1;36m==================================================\x1b[0m");
    println!("Type your message and press Enter.");
    println!("Available tools: \x1b[33mcalculator\x1b[0m, \x1b[33mget_time\x1b[0m.");
    println!("Special commands: \x1b[35mexit\x1b[0m, \x1b[35mclear\x1b[0m, \x1b[35menv\x1b[0m.");
    if args.no_persist {
        println!("Memory: \x1b[33min-memory only\x1b[0m (history will not be saved).");
    } else {
        println!(
            "Memory: persisting to \x1b[33m{}\x1b[0m.",
            args.history_file
        );
    }
    println!("--------------------------------------------------");

    // Set up tools
    let mut registry = ToolRegistry::new();
    registry.register(CalculatorTool::new());
    registry.register(TimeTool::new());

    struct TerminalApprovalHook;

    #[async_trait::async_trait]
    impl agent_core::agent::ToolExecutionHook for TerminalApprovalHook {
        async fn approve(&self, tool_name: &str, arguments: &serde_json::Value) -> bool {
            print!(
                "\n\x1b[1;33m⚠️  [Tool Authorization] Allow agent to run tool '{}' with arguments: {}? [y/N]: \x1b[0m",
                tool_name, arguments
            );
            let _ = io::stdout().flush();

            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_ok() {
                let trimmed = input.trim().to_lowercase();
                trimmed == "y" || trimmed == "yes"
            } else {
                false
            }
        }
    }

    // Set up agent — choose memory backend based on --no-persist
    let provider = MockLlmProvider::new();

    macro_rules! build_agent {
        ($mem:expr) => {
            Agent::new(provider, registry, $mem).with_execution_hook(TerminalApprovalHook)
        };
    }

    // We need a single concrete type in the loop, so we use Box<dyn Memory>.
    // FileMemory::load surfaces errors early so the user gets a clear message.
    enum AnyMemory {
        Persistent(FileMemory),
        Ephemeral(SimpleMemory),
    }

    impl Memory for AnyMemory {
        fn add_message(&mut self, m: agent_core::llm::ChatMessage) {
            match self {
                AnyMemory::Persistent(inner) => inner.add_message(m),
                AnyMemory::Ephemeral(inner) => inner.add_message(m),
            }
        }
        fn messages(&self) -> &[agent_core::llm::ChatMessage] {
            match self {
                AnyMemory::Persistent(inner) => inner.messages(),
                AnyMemory::Ephemeral(inner) => inner.messages(),
            }
        }
        fn clear(&mut self) {
            match self {
                AnyMemory::Persistent(inner) => inner.clear(),
                AnyMemory::Ephemeral(inner) => inner.clear(),
            }
        }
    }

    let memory: AnyMemory = if args.no_persist {
        AnyMemory::Ephemeral(SimpleMemory::new())
    } else {
        match FileMemory::load(&args.history_file) {
            Ok(fm) => {
                let count = fm.messages().len();
                if count > 0 {
                    println!(
                        "\x1b[1;32m✓ Restored {} message(s) from {}\x1b[0m",
                        count, args.history_file
                    );
                }
                AnyMemory::Persistent(fm)
            }
            Err(e) => {
                eprintln!(
                    "\x1b[1;31mWarning: could not load history file '{}': {}. Starting fresh.\x1b[0m",
                    args.history_file, e
                );
                AnyMemory::Ephemeral(SimpleMemory::new())
            }
        }
    };

    let mut agent = build_agent!(memory);

    let mut input = String::new();
    let stdin = io::stdin();

    loop {
        print!("\x1b[1;32mYou > \x1b[0m");
        io::stdout().flush()?;

        input.clear();
        let bytes_read = stdin.read_line(&mut input)?;
        if bytes_read == 0 {
            break; // EOF
        }

        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed == "exit" || trimmed == "quit" {
            println!("\x1b[1;34mGoodbye!\x1b[0m");
            break;
        }

        if trimmed == "clear" {
            agent.memory_mut().clear();
            println!("\x1b[1;33mConversation history cleared.\x1b[0m");
            continue;
        }

        // Print relevant environment configuration at runtime
        if trimmed == "env" {
            let rust_log = std::env::var("RUST_LOG").unwrap_or_else(|_| "(not set)".to_string());
            let gemini_key = std::env::var("GEMINI_API_KEY")
                .map(|v| {
                    if v.is_empty() {
                        "(empty)".to_string()
                    } else {
                        "(set)".to_string()
                    }
                })
                .unwrap_or_else(|_| "(not set)".to_string());
            let openai_key = std::env::var("OPENAI_API_KEY")
                .map(|v| {
                    if v.is_empty() {
                        "(empty)".to_string()
                    } else {
                        "(set)".to_string()
                    }
                })
                .unwrap_or_else(|_| "(not set)".to_string());
            let anthropic_key = std::env::var("ANTHROPIC_API_KEY")
                .map(|v| {
                    if v.is_empty() {
                        "(empty)".to_string()
                    } else {
                        "(set)".to_string()
                    }
                })
                .unwrap_or_else(|_| "(not set)".to_string());
            let model = std::env::var("AGENT_MODEL").unwrap_or_else(|_| "(not set)".to_string());
            println!("\x1b[1;36mEnvironment Configuration:\x1b[0m");
            println!("  RUST_LOG          = {}", rust_log);
            println!("  GEMINI_API_KEY    = {}", gemini_key);
            println!("  OPENAI_API_KEY    = {}", openai_key);
            println!("  ANTHROPIC_API_KEY = {}", anthropic_key);
            println!("  AGENT_MODEL       = {}", model);
            println!("  env file          = {}", args.env_file);
            continue;
        }

        // Run the agent loop
        match agent.prompt(trimmed).await {
            Ok(response) => {
                println!("\x1b[1;34mAgent > \x1b[0m{}", response);
            }
            Err(e) => {
                eprintln!("\x1b[1;31mError: \x1b[0m{}", e);
            }
        }
        println!();
    }

    Ok(())
}
