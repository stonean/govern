# /anvil/smoke

A minimal procedure exercising the parameterized command-resolution
path for the `opencode` layout. The fixture omits `framework/commands/`
entirely so the only way for `gvrn exec smoke` to find this file is via
the installed candidate, which for OpenCode expands to the *singular*
`{cli-config-dir}/command/{project}/<name>.md` —
`.opencode/command/anvil/smoke.md` after reading `.govern.toml`'s
`[host]` block.

## Instructions

1. Invoke `read-spec` against the targeted feature.
