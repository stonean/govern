---
section: "Flow"
---

# Framework implies language

## Context

Spec 004's **Flow** asks **Backend language** and **Backend framework** as independent questions (and likewise for the frontend section). For a framework that fully determines its language — Rails is always Ruby — asking the language as a separate question is redundant and reads as a bug: the user has already told `/gov:init` the project is Rails, so being prompted "what language?" is noise.

## Behavior

- In each section (backend, then frontend), `/gov:init` asks the **framework** question before the **language** question.
- When the selected framework unambiguously determines its language — e.g. Rails → Ruby, Sinatra → Ruby, Django / FastAPI / Flask → Python, Gin / Echo → Go, Laravel → PHP, Phoenix → Elixir, ASP.NET → C# — `/gov:init` records that language automatically and presents **no** language question and **no** language example options.
- The inferred language is still written as a row in the AGENTS.md **Tech Stack** table. Workflow recommendation (spec 005) matches registry entries on `backend_language`, so omitting the row would silently drop the Ruby-triggered workflows (RuboCop, RSpec); the inference suppresses the *question*, not the *data*.
- The language question — with its example options — is shown only when the framework was skipped, answered "Other" with an unrecognized value, or is language-ambiguous: a Node framework that could be TypeScript or JavaScript, a JVM framework that could be Java or Kotlin, and the like.

## Edge Cases

- **Framework skipped** — there is nothing to infer from, so the language question is asked exactly as before.
- **"Other" framework** — infer the language when the typed name is recognizable (e.g. Hanami → Ruby); otherwise fall back to asking.
- **Fullstack** — the inference runs independently for the backend and frontend sections; a Rails + React project infers Ruby for the backend and still asks TypeScript-or-JavaScript for the frontend.
- **Frontend ambiguity** — most frontend frameworks (React, Vue, Svelte, Next.js) are language-ambiguous, so the frontend language question is usually still asked.

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

*None yet.*
