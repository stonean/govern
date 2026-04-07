# Target Argument Parsing

**spec-ref:** 000-slash-commands — Command Set / target

## Context

The `target` command accepts an optional argument to set the working feature. The argument field in the command template uses a placeholder (e.g., `000`) that is replaced with the user's input. When the user passes a feature number that happens to match the placeholder value, the command must treat it as a valid feature number — not as an empty or missing argument.

## Behavior

- The `target` command determines whether an argument was provided by checking if the argument field was populated by the user, not by comparing its value against known placeholders.
- `target 000` resolves to feature `000-slash-commands` (or whichever feature has the `000` prefix).
- `target` with no argument displays the current session target.
- Any string in the argument position is treated as a feature identifier — feature numbers, partial names, and full directory names are all valid inputs regardless of their value.

## Edge Cases

- Feature number `000` must not be treated as falsy, empty, or as a no-argument invocation.
- If the argument is whitespace-only, treat it as no argument (display current target).
