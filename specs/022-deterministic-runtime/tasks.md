# 022 — Deterministic Runtime Tasks

Tasks derived from the [plan](plan.md). Complete in order. Each task is small enough to complete and verify in a single session; later tasks depend on earlier ones.

## 65. Implement scenario: [mcp-arg-unknown-field-strictness](scenarios/mcp-arg-unknown-field-strictness.md)

- [x] Implement the behavior described in `scenarios/mcp-arg-unknown-field-strictness.md`

- **Done when**: an unknown field in an MCP tool call is rejected with a naming error via a derived per-primitive field allowlist; the exec path's superset-context binding is unaffected; a test covers a misspelled kebab arg on both surfaces; `cargo test` green.

## 66. Implement scenario: [fetch-archive-dns-rebinding](scenarios/fetch-archive-dns-rebinding.md)

- [x] Implement the behavior described in `scenarios/fetch-archive-dns-rebinding.md`

- **Done when**: `fetch-archive` connects only to the address `validate_fetch_url` screened (pinned `SocketAddr` or a connect-time re-check against the internal-address predicate), so a host that rebinds between validation and connect cannot reach an internal address; a connect-time internal address is a naming error, not a silent connect; a test covers the rebind case; `cargo test` green.

## 67. Implement scenario: [append-inbox-comment-aware-write](scenarios/append-inbox-comment-aware-write.md)

- [ ] Implement the behavior described in `scenarios/append-inbox-comment-aware-write.md`

- **Done when**: `append-inbox`'s write side is comment/fence-aware like its read side: a bullet appended to an `inbox.md` that ends inside an unclosed `<!--` comment (or fence) lands in a position `count_inbox_bullets` counts, never inside the comment; well-formed inboxes append unchanged; a test covers the unclosed-comment case; `cargo test` green.

## 68. Implement scenario: [write-review-known-field-quoting](scenarios/write-review-known-field-quoting.md)

- [ ] Implement the behavior described in `scenarios/write-review-known-field-quoting.md`

- **Done when**: `write-review` renders known waiver fields through the same `yaml_string` quoting as the extra fields, so a bare-numeric/bool/null-like known-field value is quoted and round-trips through `RawWaiver`; timestamp-shaped values (`waived-at`) produce no golden-fixture churn (verified before landing); a test covers a bool-like `reason`; `cargo test` green.
