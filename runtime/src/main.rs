//! `govern` deterministic runtime CLI entrypoint.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use std::io;

use gvrn::mcp::server::GovRuntimeServer;
use gvrn::primitives;
use gvrn::schema::primitives::{
    AppendTaskArgs, ApplyManifestArgs, CheckRuleIdsArgs, CheckStuckArgs, CreateScenarioArgs,
    DashboardArgs, DeriveBoundaryArgs, EnforceManifestArgs, ExtractArchiveArgs, FetchArchiveArgs,
    GateConfirmArgs, LintMarkdownArgs, MarkCriterionArgs, MarkTaskArgs, MergeClaudeMdArgs,
    MergeManagedBlockArgs, MergePermissionsArgs, ReadSpecArgs, ReadTasksArgs, ResolveAnchorArgs,
    RunGeneratorArgs, SetStatusArgs, SubstituteTemplatesArgs, TraverseDepsArgs,
    ValidateFrontmatterArgs, WriteSessionArgs,
};

#[derive(Parser, Debug)]
#[command(
    name = "gvrn",
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
    /// Download an archive plus its sha256 sidecar and verify the hash.
    FetchArchive(FetchArchiveArgs),
    /// Extract a local `.tar.gz` / `.zip` archive into a destination directory.
    ExtractArchive(ExtractArchiveArgs),
    /// Walk a source tree, apply `{key}` substitutions, and write to a destination.
    SubstituteTemplates(SubstituteTemplatesArgs),
    /// Strategy-aware bulk substitute + write driven by a manifest.
    ApplyManifest(ApplyManifestArgs),
    /// Remove files in a directory that are not in the expected manifest.
    EnforceManifest(EnforceManifestArgs),
    /// Idempotently merge a framework-managed block into the adopter's CLAUDE.md.
    MergeClaudeMd(MergeClaudeMdArgs),
    /// Idempotently merge a framework-managed block with configurable marker shape.
    MergeManagedBlock(MergeManagedBlockArgs),
    /// Idempotently merge a canonical permission allow/deny set into a JSON file with dedup.
    MergePermissions(MergePermissionsArgs),
    /// Write a new scenarios/{slug}.md file under a feature with frontmatter and body.
    CreateScenario(CreateScenarioArgs),
    /// Append a numbered task block to a feature's tasks.md (atomic rewrite).
    AppendTask(AppendTaskArgs),
    /// Emit a `gate-confirm` envelope on stdout and block for a response.
    GateConfirm(GateConfirmArgs),
    /// Single-call pipeline-state surface for `/gov:status`.
    Dashboard(DashboardArgs),
    /// Atomically rewrite `.claude/gov-session.json` with the session-target record.
    WriteSession(WriteSessionArgs),
}

fn emit_protocol_schema() -> ExitCode {
    let schema = schemars::schema_for!(gvrn::schema::protocol::ProtocolMessage);
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

fn run_parse(path: &std::path::Path, check_only: bool) -> ExitCode {
    use gvrn::parser;

    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(err) => {
            eprintln!("failed to read {}: {err}", path.display());
            return ExitCode::from(1);
        }
    };
    let command_name = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    match parser::parse(&source, &command_name) {
        Ok(procedure) => {
            if check_only {
                ExitCode::SUCCESS
            } else {
                match serde_json::to_string_pretty(&procedure) {
                    Ok(text) => {
                        println!("{text}");
                        ExitCode::SUCCESS
                    }
                    Err(err) => {
                        eprintln!("failed to serialize AST: {err}");
                        ExitCode::from(1)
                    }
                }
            }
        }
        Err(parser::ParseError::LegacyProse) => {
            if check_only {
                eprintln!("{}: legacy prose (allowed)", path.display());
                ExitCode::SUCCESS
            } else {
                eprintln!(
                    "{}: legacy prose — no parseable Instructions section",
                    path.display()
                );
                ExitCode::from(2)
            }
        }
        Err(err) => {
            eprintln!("{}: {err}", path.display());
            ExitCode::from(1)
        }
    }
}

fn run_exec(command: &str, args: &[String], repo: &std::path::Path) -> ExitCode {
    use gvrn::interpreter::{WalkOutcome, Walker};
    use gvrn::parser;
    use serde_json::{Map, Value};

    let candidates = [
        repo.join("framework/commands")
            .join(format!("{command}.md")),
        repo.join(".claude/commands/gov")
            .join(format!("{command}.md")),
        // Bootstrap procedures (`/govern` and its successors) live outside
        // the project-installable command namespace because they're invoked
        // before any framework files exist in the adopter's project. See
        // spec 022 scenario `govern-bootstrap`.
        repo.join("framework/bootstrap")
            .join(format!("{command}.md")),
    ];
    let Some(path) = candidates.iter().find(|p| p.exists()) else {
        eprintln!(
            "runtime exec: command file not found (tried {})",
            candidates
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
        return ExitCode::from(1);
    };

    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(err) => {
            eprintln!("failed to read {}: {err}", path.display());
            return ExitCode::from(1);
        }
    };
    let procedure = match parser::parse(&source, command) {
        Ok(p) => p,
        Err(err) => {
            eprintln!("{}: {err}", path.display());
            return ExitCode::from(2);
        }
    };

    // Seed the walker context: session file (when present) overlaid with
    // CLI `key=value` arg overrides.
    let mut context = Map::new();
    let session_path = repo.join(".claude/gov-session.json");
    if let Ok(text) = std::fs::read_to_string(&session_path)
        && let Ok(Value::Object(map)) = serde_json::from_str::<Value>(&text)
    {
        context.extend(map);
    }
    for arg in args {
        if let Some((key, value)) = arg.split_once('=') {
            context.insert(key.to_string(), Value::String(value.to_string()));
        }
    }

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = stdin.lock();
    let mut writer = stdout.lock();
    let mut walker = Walker::new(
        &procedure,
        repo.to_path_buf(),
        context,
        &mut reader,
        &mut writer,
    );
    match walker.run() {
        Ok(WalkOutcome::Complete) => ExitCode::SUCCESS,
        Ok(WalkOutcome::Errored { .. }) => ExitCode::from(1),
        Err(err) => {
            eprintln!("runtime exec: I/O error: {err}");
            ExitCode::from(74) // EX_IOERR
        }
    }
}

fn run_mcp_server(repo: PathBuf) -> ExitCode {
    use rmcp::ServiceExt;
    use rmcp::transport::stdio;

    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(err) => {
            eprintln!("failed to start tokio runtime: {err}");
            return ExitCode::from(1);
        }
    };

    runtime.block_on(async move {
        let server = GovRuntimeServer::new(repo);
        let service = match server.serve(stdio()).await {
            Ok(svc) => svc,
            Err(err) => {
                eprintln!("failed to start mcp server: {err}");
                return ExitCode::from(1);
            }
        };
        if let Err(err) = service.waiting().await {
            eprintln!("mcp server terminated with error: {err}");
            return ExitCode::from(1);
        }
        ExitCode::SUCCESS
    })
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let repo = cwd();
    match cli.command {
        Command::Mcp => run_mcp_server(repo),
        Command::Exec { command, args } => run_exec(&command, &args, &repo),
        Command::Parse {
            file,
            check,
            emit_schema,
        } => {
            if emit_schema {
                return emit_protocol_schema();
            }
            let Some(path) = file else {
                eprintln!(
                    "runtime parse: missing FILE argument (use --emit-schema for the debug surface)"
                );
                return ExitCode::from(1);
            };
            run_parse(&path, check)
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
        Command::FetchArchive(args) => emit_result(primitives::fetch_archive::run(&args, &repo)),
        Command::ExtractArchive(args) => {
            emit_result(primitives::extract_archive::run(&args, &repo))
        }
        Command::SubstituteTemplates(args) => {
            emit_result(primitives::substitute_templates::run(&args, &repo))
        }
        Command::ApplyManifest(args) => emit_result(primitives::apply_manifest::run(&args, &repo)),
        Command::EnforceManifest(args) => {
            emit_result(primitives::enforce_manifest::run(&args, &repo))
        }
        Command::MergeClaudeMd(args) => emit_result(primitives::merge_claude_md::run(&args, &repo)),
        Command::MergeManagedBlock(args) => {
            emit_result(primitives::merge_managed_block::run(&args, &repo))
        }
        Command::MergePermissions(args) => {
            emit_result(primitives::merge_permissions::run(&args, &repo))
        }
        Command::CreateScenario(args) => {
            emit_result(primitives::create_scenario::run(&args, &repo))
        }
        Command::AppendTask(args) => emit_result(primitives::append_task::run(&args, &repo)),
        Command::Dashboard(args) => emit_result(primitives::dashboard::run(&args, &repo)),
        Command::WriteSession(args) => emit_result(primitives::write_session::run(&args, &repo)),
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
