# 004 — Tech Stack Selection Tasks

Tasks derived from the [plan](plan.md). Complete in order.

## 1. Update init.md inputs section

- [x] Replace input #4 ("Primary language(s)") with the tech stack questionnaire flow
- [x] Add project type question (backend, frontend, fullstack) as the new input #4
- [x] Add backend category questions (language, framework, database, messaging, test runner)
- [x] Add frontend category questions (language, framework, CSS/UI, test runner)
- [x] Document category visibility rules per project type
- [x] Document that each question includes "Other" and "Skip" options

## 2. Update init.md scaffolding steps

- [x] Modify step 3 (AGENTS.md) to populate Tech Stack table from selections
- [x] Define layer-to-role mapping for table rows
- [x] Handle fullstack dual-language rows (backend language, frontend language)
- [x] Handle "skip all" case — leave comment placeholder unchanged
- [x] Modify step 8 (.gitignore) to derive languages from tech stack selections instead of standalone input

## 3. Update AGENTS.md template

- [x] Adjust Tech Stack comment block so it can be cleanly replaced by init when selections are made

## 4. Update init.md display section

- [x] Remove step 3 from "Next steps" ("Fill in AGENTS.md — tech stack...") since Tech Stack is now populated
- [x] Adjust wording to reflect that only Code Style, Testing, and Gotchas need manual filling

## 5. Validate

- [x] Run `npx markdownlint-cli2` on all modified files
- [x] Verify backwards compatibility: skip all categories → AGENTS.md unchanged
