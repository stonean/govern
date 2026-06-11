---
spec: 004-tech-stack-selection
reviewed-at: 2026-06-11T01:48:34Z
reviewed-against: b9982910c3120ed67b63b90a7bb702a88de29403
diff-base: 45a8337d795f73e2ff95aaa8ac0d4d442bc60c3c
must-violations: 0
should-violations: 0
low-confidence: 0
captured-issues: 0
skipped-passes: []
---

# Review — 004-tech-stack-selection

## Summary

Reviewed the code added under the `framework-implies-language` scenario: the
backend/frontend questionnaire change in `.claude/commands/gov/init.md` (ask the
framework first; infer the language and skip its question — and its example
options — when the framework unambiguously determines it). **No MUST violations
— the spec remains validly `done`, with no SHOULD findings.** The change is a
prose command instruction; the loaded security/api/config rule files target
application-backend code and do not apply.

One boundary point worth recording rather than flagging: `init.md` was edited at
`.claude/commands/gov/init.md` directly, which the AGENTS.md §Boundaries rule
normally forbids for generated command files. `init.md` is the documented
exception — it is governance-specific and hand-maintained (`gen-claude-commands.sh`
explicitly never touches it, and there is no `framework/commands/init.md`
source), so the direct edit is correct, not a violation.

## MUST violations (blocking)

None.

## SHOULD violations (advisory)

None.

## Low-confidence findings

None.

## Waived findings

None.

## Captured issues (pending /gov:groom)

None.

## Skipped passes

None.

## Notes

- **Quality** — the framework→language mappings are accurate (Rails → Ruby,
  Django/FastAPI/Flask → Python, Gin/Echo → Go, Laravel → PHP, Phoenix → Elixir,
  ASP.NET → C#), and the fallback is safe: the language question is still asked
  when the framework is skipped, "Other"/unrecognized, or language-ambiguous
  (Node → TS/JS, JVM → Java/Kotlin). The inferred language is still written to
  the AGENTS.md Tech Stack table, so `backend_language`-triggered workflow
  registry entries (RuboCop, RSpec) continue to match — the inference suppresses
  the question, not the data.
- **Design principles** — the inference relies on the executing agent's
  judgment, not on author diligence, and degrades safely to asking; it does not
  trip the "never depend on human diligence" rule.
</content>
