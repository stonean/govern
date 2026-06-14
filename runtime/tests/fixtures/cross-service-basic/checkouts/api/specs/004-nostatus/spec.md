---
status: archived
dependencies: []
---

# 004 — Out-of-set status

Fixture stand-in whose `status` value (`archived`) is outside the allowed
lifecycle set (`draft` / `clarified` / `planned` / `in-progress` / `done`).
The target file exists and its frontmatter parses, but the status cannot be
read as a lifecycle value, so the consumer's reference resolves to
`status-unreadable` — surfaced, never silent; the defect is upstream's.
