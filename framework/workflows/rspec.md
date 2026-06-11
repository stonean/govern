---
description: Run the RSpec suite for the Ruby codebase
---

# RSpec

Run the RSpec suite for the Ruby codebase.

## Instructions

1. Detect RSpec — look for a `spec/` directory and a `.rspec` file or `spec/spec_helper.rb`. If none is found, report `No RSpec suite found` and stop.
2. If the project has a `test` or `spec` task in `Rakefile`, `Makefile`, or `justfile`, mention it but prefer the direct `rspec` invocation for predictability.
3. Run `bundle exec rspec` from the repository root when a `Gemfile` is present, otherwise `rspec`. If neither is on PATH, report and stop.
4. If the user specified a path, example name, or tag filter, append it (e.g., `rspec spec/models/user_spec.rb:42` or `rspec -e "pattern"` or `rspec --tag focus`).
5. Display the results. For each failure, show the example description, the expectation failure or backtrace, and the rerun command RSpec prints.
6. Summarize: examples, failures, pending counts and total duration.
7. If any examples failed, treat the run as failed.

## What this workflow does NOT do

- Modify spec files or fixtures
- Update VCR cassettes or snapshot artifacts without confirmation
- Install RSpec or gems

## Common follow-ups

- Re-run with `--fail-fast` to stop at the first failure
- Re-run only failures with `--only-failures` (requires `example_status_persistence_file_path`)
- Add `--format documentation` for verbose per-example output
