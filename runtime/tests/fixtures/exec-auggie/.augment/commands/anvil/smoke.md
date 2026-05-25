# /anvil:smoke

A minimal procedure exercising the parameterized command-resolution
path. The fixture omits `framework/commands/` entirely so the only way
for `gvrn exec smoke` to find this file is via the second candidate
(`{cli-config-dir}/commands/{project}/<name>.md`), which expands to
`.augment/commands/anvil/smoke.md` after reading `.govern.toml`'s
`[host]` block.

## Instructions

1. Invoke `read-spec` against the targeted feature.
