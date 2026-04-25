# 012 — Multi-Agent Govern Data Model

The agent registry is a structured, in-file data model carried by `govern/govern.md`. It is a table of supported agents that the unified govern command iterates over during scaffolding. There is no database, no language-level type — the "data" is markdown rows the prompt reads at run time.

## Agent Registry

Each row describes one supported agent. Five fields per row.

| Field | Type | Required | Description |
| --- | --- | --- | --- |
| `key` | string | yes | Lowercase identifier. Stable across releases. Used in `--agents={key}` flags, the `commands/setup/{key}.md` setup source path, and in user-facing prompt text. Must be unique within the registry. |
| `name` | string | yes | Human-readable label shown in `AskUserQuestion` prompts and the post-scaffolding summary. Example: "Claude Code", "Auggie". |
| `config_dir` | string | yes | Project-relative directory where the agent stores its config and commands. Example: `.claude`, `.augment`. Used to detect existing adoptions and to compute every per-agent destination path. |
| `settings_template` | object (JSON) | yes | Bootstrap entries written into `{config_dir}/settings.local.json` if missing. Native to the agent's settings format. Contains only the curl/ls entries needed for `/govern` itself — not the full permission set. |
| `rules_file_note` | string | yes | Short note about the agent's relationship to `CLAUDE.md`. Surfaced in the post-scaffolding summary. Example: "Claude Code reads `CLAUDE.md` natively." |

### Derived values

These are computed from the registry by convention; they are not stored as fields.

| Derived value | Formula |
| --- | --- |
| Setup source | `commands/setup/{key}.md` |
| Session JSON path | `{config_dir}/{project}-session.json` |
| Project commands directory | `{config_dir}/commands/{project}/` |
| Govern install path | `{config_dir}/commands/govern.md` |

### Initial population

| `key` | `name` | `config_dir` | `settings_template` (summary) | `rules_file_note` |
| --- | --- | --- | --- | --- |
| `claude` | Claude Code | `.claude` | `permissions.allow: ["Bash(curl *)", "Bash(ls *)"]` | Claude Code reads `CLAUDE.md` natively. |
| `auggie` | Auggie | `.augment` | `toolPermissions: [launch-process curl, launch-process ls]` | Auggie reads `CLAUDE.md` natively — no second rules file is needed. |

## Adding a new agent

A new agent is a one-row addition to the registry plus two satellite files:

1. Append a row to the registry with the five required fields.
2. Add `commands/setup/{key}.md` with the agent's full permission set (in its native settings format).
3. Add a curl snippet for the new agent to the README's "Adopting in an Existing Project" section.

No other changes are required — the rest of the govern logic references registry values, not agent names.

## Invariants

- `key` values are unique across the registry.
- `key` is a valid filename component — lowercase letters, digits, hyphens only — because it is interpolated into a path (`commands/setup/{key}.md`).
- `config_dir` is a project-relative path, no trailing slash, no leading `./`.
- `settings_template` is valid JSON and matches the agent's native settings format. Merging the template into an existing settings file must preserve all entries the user or `/{project}:setup` has previously written.
- The set of rows is intrinsic to `govern/govern.md` — no runtime registration, no external file. Adding an agent is a code change to govern, reviewed in PR.
