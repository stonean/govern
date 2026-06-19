# CLAUDE.md

@import framework/constitution.md
@import AGENTS.md

## Auto-Memory Routing

> Agent-specific routing for the constitution's *shared knowledge stays in git* principle ([§drift-prevention](framework/constitution.md#drift-prevention)).

Before saving an auto-memory entry, ask: **would this learning help any other contributor to this project?**

- **Yes** → it belongs in a git-tracked artifact, never local auto-memory. Local memory lives under the user's home directory, invisible to everyone else and absent from clones — parking contributor-beneficial guidance there defeats the purpose of a committed governance framework. Route a project learning to `AGENTS.md` (matching section: Gotchas, Workflow, Boundaries, Code Style, Testing, Design Principles); route a framework rule, schema, or behavior to its canonical artifact under `framework/` (see constitution §drift-prevention for the canonical-source map). Skip the memory entry.
- **No** → auto-memory is correct. Reserve it for facts that are purely personal to this user and carry no value to other contributors: cross-project user facts (role, persistent style preferences) and external reference pointers (Linear/Slack/dashboard bookmarks).
