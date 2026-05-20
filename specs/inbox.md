# Inbox

- `mark-task` primitive returns `task '19' not found` when the heading contains backticks — observed 2026-05-19 during `/gov:implement` on spec 023 task #19 (`### 19. Dedup ` + backtick + `/configure` + backtick + ` permission entries via new gvrn primitive`). `read-tasks` parses the same heading correctly and returns the task with `"number":"19"`; the inconsistency is between the two primitives' heading parsers. Worked around by editing the checkbox directly. Worth filing as a follow-up scenario on spec 022 alongside the `runtime-primitive-structural-bugs` family.
