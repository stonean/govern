//! `govern` deterministic runtime CLI entrypoint.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use std::io;

use govern_runtime::primitives;
use govern_runtime::schema::primitives::{
    CheckRuleIdsArgs, CheckStuckArgs, DeriveBoundaryArgs, GateConfirmArgs, LintMarkdownArgs,
    MarkCriterionArgs, MarkTaskArgs, ReadSpecArgs, ReadTasksArgs, ResolveAnchorArgs,
    RunGeneratorArgs, SetStatusArgs, TraverseDepsArgs, ValidateFrontmatterArgs,
};

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
        file: Option<PathBuf>,
        /// Exit non-zero if the file is unparseable and not on the legacy
        /// allowlist.
        #[arg(long, conflicts_with = "emit_schema")]
        check: bool,
        /// Print the JSON Schema for the protocol envelope and exit. Debug
        /// surface used to inspect the wire contract.
        #[arg(long, conflicts_with_all = ["file", "check"])]
        emit_schema: bool,
    },

    /// Parse spec frontmatter and body sections.
    ReadSpec(ReadSpecArgs),
    /// Parse `tasks.md` into a structured task list.
    ReadTasks(ReadTasksArgs),
    /// Validate frontmatter shape against the pipeline schema.
    ValidateFrontmatter(ValidateFrontmatterArgs),
    /// Verify `§anchor` references resolve to `<!-- §anchor -->` markers.
    ResolveAnchor(ResolveAnchorArgs),
    /// Traverse spec dependencies and check status compatibility.
    TraverseDeps(TraverseDepsArgs),
    /// Verify cited rule IDs exist in rule files and aren't deprecated.
    CheckRuleIds(CheckRuleIdsArgs),
    /// Count tasks.md commits since the spec entered `in-progress`.
    CheckStuck(CheckStuckArgs),
    /// Derive the runtime write boundary from git history.
    DeriveBoundary(DeriveBoundaryArgs),
    /// Flip a single subtask checkbox in `tasks.md` (atomic rewrite).
    MarkTask(MarkTaskArgs),
    /// Flip a single acceptance-criterion checkbox in `spec.md`.
    MarkCriterion(MarkCriterionArgs),
    /// Update the `status:` field in spec frontmatter, guarded by `from`.
    SetStatus(SetStatusArgs),
    /// Invoke a bash generator with `--dry-run`; non-zero exit is drift.
    RunGenerator(RunGeneratorArgs),
    /// Wrap `npx markdownlint-cli2` and surface violations.
    LintMarkdown(LintMarkdownArgs),
    /// Emit a `gate-confirm` envelope on stdout and block for a response.
    GateConfirm(GateConfirmArgs),
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

fn emit_result<T: serde::Serialize, E: std::fmt::Display>(
    result: std::result::Result<T, E>,
) -> ExitCode {
    match result {
        Ok(value) => match serde_json::to_string(&value) {
            Ok(text) => {
                println!("{text}");
                ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!("failed to serialize result: {err}");
                ExitCode::from(1)
            }
        },
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(1)
        }
    }
}

fn cwd() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let repo = cwd();
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
        Command::ReadSpec(args) => emit_result(primitives::read_spec::run(&args, &repo)),
        Command::ReadTasks(args) => emit_result(primitives::read_tasks::run(&args, &repo)),
        Command::ValidateFrontmatter(args) => {
            emit_result(primitives::validate_frontmatter::run(&args, &repo))
        }
        Command::ResolveAnchor(args) => emit_result(primitives::resolve_anchor::run(&args, &repo)),
        Command::TraverseDeps(args) => emit_result(primitives::traverse_deps::run(&args, &repo)),
        Command::CheckRuleIds(args) => emit_result(primitives::check_rule_ids::run(&args, &repo)),
        Command::CheckStuck(args) => emit_result(primitives::check_stuck::run(&args, &repo)),
        Command::DeriveBoundary(args) => {
            emit_result(primitives::derive_boundary::run(&args, &repo))
        }
        Command::MarkTask(args) => emit_result(primitives::mark_task::run(&args, &repo)),
        Command::MarkCriterion(args) => emit_result(primitives::mark_criterion::run(&args, &repo)),
        Command::SetStatus(args) => emit_result(primitives::set_status::run(&args, &repo)),
        Command::RunGenerator(args) => emit_result(primitives::run_generator::run(&args, &repo)),
        Command::LintMarkdown(args) => emit_result(primitives::lint_markdown::run(&args, &repo)),
        Command::GateConfirm(args) => {
            // The CLI binding is the subprocess-interpreter surface: emit the
            // gate-confirm envelope on stdout, then read one gate-response
            // line from stdin. The MCP surface routes prompts via the host
            // instead and is wired up in task 6.
            let request_id = primitives::gate_confirm::fresh_request_id();
            let stdin = io::stdin();
            let mut reader = stdin.lock();
            let result = {
                let stdout = io::stdout();
                let mut writer = stdout.lock();
                primitives::gate_confirm::run_blocking(&args, &request_id, &mut reader, &mut writer)
            };
            emit_result(result)
        }
    }
}
