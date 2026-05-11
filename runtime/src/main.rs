//! `govern` deterministic runtime CLI entrypoint.

use clap::{Parser, Subcommand};
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(
    name = "runtime",
    version,
    about = "Deterministic runtime for the govern pipeline."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Start the MCP server, exposing every primitive as a tool.
    Mcp,

    /// Execute a slash command end-to-end via the subprocess interpreter.
    Exec {
        /// Slash command name (e.g., "status", "validate").
        command: String,
        /// Arguments forwarded to the command.
        args: Vec<String>,
    },

    /// Parse a slash command file under the procedure conventions.
    Parse {
        /// Path to the markdown file. Required unless `--emit-schema` is set.
        file: Option<std::path::PathBuf>,
        /// Exit non-zero if the file is unparseable and not on the legacy
        /// allowlist.
        #[arg(long, conflicts_with = "emit_schema")]
        check: bool,
        /// Print the JSON Schema for the protocol envelope and exit. Debug
        /// surface used to inspect the wire contract.
        #[arg(long, conflicts_with_all = ["file", "check"])]
        emit_schema: bool,
    },
}

fn emit_protocol_schema() -> ExitCode {
    let schema = schemars::schema_for!(govern_runtime::schema::protocol::ProtocolMessage);
    match serde_json::to_string_pretty(&schema) {
        Ok(text) => {
            println!("{text}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("failed to serialize protocol schema: {err}");
            ExitCode::from(1)
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Mcp => {
            eprintln!("runtime mcp: not yet implemented");
            ExitCode::from(1)
        }
        Command::Exec { command, args: _ } => {
            eprintln!("runtime exec {command}: not yet implemented");
            ExitCode::from(1)
        }
        Command::Parse {
            file,
            check,
            emit_schema,
        } => {
            if emit_schema {
                return emit_protocol_schema();
            }
            if let Some(path) = file {
                eprintln!(
                    "runtime parse {} (check={check}): not yet implemented",
                    path.display()
                );
            } else {
                eprintln!(
                    "runtime parse: missing FILE argument (use --emit-schema for the debug surface)"
                );
            }
            ExitCode::from(1)
        }
    }
}
