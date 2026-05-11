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
        /// Path to the markdown file.
        file: std::path::PathBuf,
        /// Exit non-zero if the file is unparseable and not on the legacy allowlist.
        #[arg(long)]
        check: bool,
    },
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
        Command::Parse { file, check } => {
            eprintln!(
                "runtime parse {} (check={check}): not yet implemented",
                file.display()
            );
            ExitCode::from(1)
        }
    }
}
