# 004 — Tech Stack Selection Tasks

Tasks derived from the [plan](plan.md). Complete in order.

## 1. Update init.md inputs section

- [ ] Replace input #4 ("Primary language(s)") with the tech stack questionnaire flow
- [ ] Add project type question (backend, frontend, fullstack) as the new input #4
- [ ] Add backend category questions (language, framework, database, messaging, test runner)
- [ ] Add frontend category questions (language, framework, CSS/UI, test runner)
- [ ] Document category visibility rules per project type
- [ ] Document that each question includes "Other" and "Skip" options

## 2. Update init.md scaffolding steps

- [ ] Modify step 3 (AGENTS.md) to populate Tech Stack table from selections
- [ ] Define layer-to-role mapping for table rows
- [ ] Handle fullstack dual-language rows (backend language, frontend language)
- [ ] Handle "skip all" case — leave comment placeholder unchanged
- [ ] Modify step 8 (.gitignore) to derive languages from tech stack selections instead of standalone input

## 3. Update AGENTS.md template

- [ ] Adjust Tech Stack comment block so it can be cleanly replaced by init when selections are made

## 4. Update init.md display section

- [ ] Remove step 3 from "Next steps" ("Fill in AGENTS.md — tech stack...") since Tech Stack is now populated
- [ ] Adjust wording to reflect that only Code Style, Testing, and Gotchas need manual filling

## 5. Validate

- [ ] Run `markdownlint-cli2` on all modified files
- [ ] Verify backwards compatibility: skip all categories → AGENTS.md unchanged
