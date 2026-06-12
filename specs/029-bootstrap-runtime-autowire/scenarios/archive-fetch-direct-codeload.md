---
section: "Follow-on scenarios"
---

# Archive-fetch-direct-codeload

## Context

029's State B wires gvrn and the next session is expected to run the deterministic primitive path. When the next session instead walks the markdown reference path (a host model that does not take the deterministic path, or any State-C run), the §Archive fetch step issues `curl -fsSL https://github.com/stonean/govern/archive/refs/heads/main.tar.gz`. That URL **302-redirects to `codeload.github.com`**. Surfaced 2026-06-11 during Antigravity testing: the archive `curl` prompted for permission even though the bootstrap permission seed pre-grants `curl` (`command(curl)` for antigravity, `Bash(curl *)` for Claude, the `^curl` regex matcher for Auggie). The grant matched the original host; the redirect landed the command on a new host (`codeload.github.com`) mid-flight, and the host re-prompted. The self-update `curl` against `raw.githubusercontent.com` (no redirect) was covered by the same seed and did **not** prompt — isolating the redirect as the cause.

## Behavior

The §Archive fetch step fetches the **direct `codeload.github.com` endpoint** — the redirect target — instead of the `github.com/.../archive/...` form:

```text
curl -fsSL https://codeload.github.com/stonean/govern/tar.gz/refs/heads/main \
  -o {tempdir}/main.tar.gz
```

The direct URL returns the archive with no redirect (HTTP 200, zero redirects), so the seed's pre-granted `curl` permission covers it and no prompt fires. The tarball is byte-equivalent: same `govern-main/` top-level directory, same `govern-main/framework/...` layout. Extraction and per-file resolution are unchanged.

## Edge Cases

- **Deterministic (State A) path.** Unaffected — the runtime's `fetch-archive` primitive uses its own HTTP client (which follows redirects without a permission prompt) and is covered by `mcp(gvrn/*)`. This scenario changes only the markdown-reference `curl`.
- **codeload outage / URL-format change.** Codeload is GitHub's archive backend; the `github.com/.../archive/...` form is a thin redirect onto it, so the direct form is no less stable. A failure still trips the existing fetch-or-extract abort with its error message.
- **sha256 / archive-url derived value.** The markdown path performs no sha256 verification of the archive (none today), so changing the URL has no checksum impact. The abstract `archive-url` walker-context value (§Instructions step 1) is unchanged in meaning.

## Open Questions

*None — all resolved.*

## Resolved Questions

- **Why not add the redirect-target host to the permission seed instead?** A host that prompts on cross-host redirect would need every possible redirect target pre-granted, and the seed cannot enumerate hosts a redirect might reach. Fetching the final URL directly removes the redirect entirely, which is both simpler and host-agnostic — it works the same on hosts that do and don't gate redirects.
