---
title: "004-tech-stack-selection — tasks"
---

# 004 — Tech Stack Selection Tasks

Tasks derived from the [plan](plan.md). Complete in order.

## 1. Update init.md inputs section

- [x] Replace input #4 ("Primary language(s)") with the tech stack questionnaire flow
- [x] Add project type question (backend, frontend, fullstack) as the new input #4
- [x] Add backend category questions (language, framework, database, messaging, test runner)
- [x] Add frontend category questions (language, framework, CSS/UI, test runner)
- [x] Document category visibility rules per project type
- [x] Document that each question includes "Other" and "Skip" options

Done when: `init.md`'s inputs section presents the tech-stack questionnaire (project-type question plus per-category backend/frontend questions, category visibility rules, and "Other"/"Skip" options) in place of the old "Primary language(s)" input.

## 2. Update init.md scaffolding steps

- [x] Modify step 3 (AGENTS.md) to populate Tech Stack table from selections
- [x] Define layer-to-role mapping for table rows
- [x] Handle fullstack dual-language rows (backend language, frontend language)
- [x] Handle "skip all" case — leave comment placeholder unchanged
- [x] Modify step 8 (.gitignore) to derive languages from tech stack selections instead of standalone input

Done when: `init.md`'s scaffolding steps populate the AGENTS.md Tech Stack table from the selections — with layer-to-role mapping, fullstack dual-language rows, and the skip-all placeholder preserved — and derive `.gitignore` languages from those selections.

## 3. Update AGENTS.md template

- [x] Adjust Tech Stack comment block so it can be cleanly replaced by init when selections are made

Done when: the AGENTS.md template's Tech Stack comment block is shaped so `/gov:init` can cleanly replace it when selections are made.

## 4. Update init.md display section

- [x] Remove step 3 from "Next steps" ("Fill in AGENTS.md — tech stack...") since Tech Stack is now populated
- [x] Adjust wording to reflect that only Code Style, Testing, and Gotchas need manual filling

Done when: `init.md`'s "Next steps" no longer lists filling in the Tech Stack table, and the wording reflects that only Code Style, Testing, and Gotchas need manual completion.

## 5. Validate

- [x] Run `npx markdownlint-cli2` on all modified files
- [x] Verify backwards compatibility: skip all categories → AGENTS.md unchanged

Done when: `npx markdownlint-cli2` passes on all modified files and skipping every category leaves the AGENTS.md Tech Stack block unchanged.

## 6. Framework-implies-language inference

- [x] `/gov:init` asks the framework before the language in each section and infers the language when the framework determines it (Rails → Ruby, Django → Python, Gin → Go, …)
- [x] The inferred language is still written to the AGENTS.md Tech Stack table, so `backend_language`-triggered workflows (RuboCop, RSpec) still match
- [x] The language question is still asked when the framework is skipped, answered "Other"/unrecognized, or is language-ambiguous (Node → TS/JS, JVM → Java/Kotlin)

Done when: `/gov:init` does not ask the language question when the chosen framework unambiguously implies it, per `scenarios/framework-implies-language.md`.
