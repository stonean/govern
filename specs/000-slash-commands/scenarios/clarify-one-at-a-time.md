# Clarify One at a Time

**spec-ref:** 000-slash-commands — Command Set / clarify

## Context

A spec in `draft` status has one or more open questions. The user runs the `clarify` command to resolve them.

## Behavior

- The `clarify` command processes open questions **one at a time**, not in batch.
- For each open question:
  1. Display the question with its full context.
  2. Propose a resolution or ask for the user's input.
  3. Wait for the user to review, discuss, refine, or approve the resolution.
  4. Only after the user confirms the resolution, move the question to Resolved Questions (or update it based on feedback).
- Do not present the next question until the current one is resolved.
- After all open questions are resolved, proceed to enumerate edge cases and verify acceptance criteria.

## Edge Cases

- If only one open question exists, the one-at-a-time flow still applies — present it, wait for confirmation, then proceed.
- If the user wants to skip a question and come back to it later, allow it — move to the next question and revisit skipped questions at the end.
- If resolving one question invalidates or changes another, note the impact when presenting the affected question.
