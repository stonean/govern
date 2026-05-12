# gvrn

Deterministic runtime for the [govern](https://github.com/stonean/govern) spec-driven development framework.

`gvrn` is an **optional** native binary that accelerates slash commands by interpreting their parseable procedures directly — reading specs, walking tasks, checking dependencies, performing atomic checkbox updates, and handshaking pipeline gates in Rust instead of asking the LLM to do it in slow tokens. The LLM is invoked only at named extension points (`assessSpecQuality`, `writeCode`, `writeSpecBody`) where semantic judgment actually matters.

The govern framework's markdown-only path remains first-class: when `gvrn` is absent, the LLM walks the same procedure prose and produces the same results, just slower. See [§runtime-boundary](https://github.com/stonean/govern/blob/main/framework/constitution.md#runtime-boundary) in the constitution for the constraints this binary commits to.

## Install

Pre-built binaries are published to GitHub releases on every `gvrn-v*` tag. Download the asset for your target triple and verify the checksum:

```bash
# Example for aarch64-apple-darwin; substitute your target triple.
VERSION="0.1.0"
TARGET="aarch64-apple-darwin"
ARCHIVE="gvrn-${TARGET}.tar.gz"
BASE="https://github.com/stonean/govern/releases/download/gvrn-v${VERSION}"

tmp="$(mktemp -d)" && cd "${tmp}"
curl -LO "${BASE}/${ARCHIVE}"
curl -LO "${BASE}/${ARCHIVE}.sha256"
shasum -a 256 -c "${ARCHIVE}.sha256"
tar xzf "${ARCHIVE}"
sudo install -m 0755 gvrn /usr/local/bin/gvrn
gvrn --version
cd - >/dev/null && rm -rf "${tmp}"
```

Pre-built binaries cover `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, and `x86_64-pc-windows-msvc`.

Alternatively, build from source with `cargo install gvrn`.

## Usage

Two surfaces:

```bash
gvrn mcp                # start an MCP server exposing every primitive as a tool
gvrn exec <command>     # interpret a slash command procedure end-to-end via JSON-over-stdio
gvrn parse <file>       # parse a procedure file and print its AST (debug)
gvrn parse --check <f>  # exit 0 if parseable or on the legacy allowlist (CI lint)
gvrn <primitive> [...]  # invoke a single primitive standalone (e.g., gvrn read-spec --feature 001)
```

The runtime expects to be invoked from a govern-adopting repository — one with `framework/commands/*.md` (or `.claude/commands/gov/*.md` for installed projects) and feature directories under `specs/`.

## JSON-over-stdio protocol

`gvrn exec` and `gvrn <primitive>` emit one JSON envelope per line on stdout. The closed set of envelope types:

| Type | Direction | Payload |
| --- | --- | --- |
| `progress` | runtime → host | Step number and human-readable status text |
| `llm-request` | runtime → host | Extension point, request-id, payload to send to the LLM |
| `llm-response` | host → runtime | request-id correlating an open `llm-request` |
| `gate-confirm` | runtime → host | Named gate plus prompt to surface to the user |
| `gate-response` | host → runtime | request-id and the user's `confirmed: bool` decision |
| `complete` | runtime → host | Procedure finished cleanly; carries the runtime version |
| `error` | runtime → host | Procedure halted on operational failure; halts the walk |

See the [data model](https://github.com/stonean/govern/blob/main/specs/022-deterministic-runtime/data-model.md) for the full envelope shapes and primitive schemas.

## Stability

The CLI surface and the JSON-over-stdio protocol are the supported public surface. The Rust library exposed by this crate (`use gvrn::…`) ships only so the runtime's own integration tests can link against its internals; the module surface is **not stable** in 0.x and may change without a semver bump. Use the binary, not the library.

## Repository

Full source, specs, and the framework itself live at <https://github.com/stonean/govern>.

## License

MIT — see [LICENSE](LICENSE).
