# Security Rules — Frontend

Enforceable security rules for browser-side code, UI rendering, and client-server interaction. These rules apply to all projects with a web frontend adopting governance.

Rules use RFC 2119 language: **MUST** and **MUST NOT** are enforced by the validate command (errors). **SHOULD** and **SHOULD NOT** are flagged as warnings.

Projects without a frontend can pin this file in `.governance.toml` to skip it during govern updates.

## FE-XSS — Cross-Site Scripting Prevention

### FE-XSS-001 — Output encoding

All untrusted data rendered in HTML MUST be encoded for the specific output context (HTML body, HTML attribute, JavaScript, CSS, URL). A single encoding method MUST NOT be used across all contexts.

- **Rationale:** Different parsing engines interpret characters differently. HTML encoding does not prevent XSS in JavaScript contexts.
- **Verification:** Code review confirms context-appropriate encoding at every output point.
- **Source:** OWASP XSS Prevention Cheat Sheet

### FE-XSS-002 — Framework auto-escaping

Applications MUST use framework-provided auto-escaping (React JSX, Angular templates, Vue templates) as the primary XSS defense. Explicit escape hatches (`dangerouslySetInnerHTML`, `bypassSecurityTrustHtml`, `v-html`) MUST be justified and reviewed.

- **Rationale:** Framework auto-escaping covers the common case with minimal developer effort. Escape hatches bypass protection entirely and require manual sanitization.
- **Verification:** Code review confirms escape hatch usage is justified and accompanied by sanitization.
- **Source:** OWASP XSS Prevention Cheat Sheet

### FE-XSS-003 — Safe DOM methods

Code MUST use safe DOM methods (`.textContent`, `.setAttribute()`, `.value`) over unsafe alternatives (`.innerHTML`, `eval()`, `document.write()`). When `.innerHTML` is unavoidable, content MUST be sanitized with a dedicated library (e.g., DOMPurify).

- **Rationale:** Unsafe DOM methods interpret input as executable code. Safe methods treat input as data.
- **Verification:** Code review confirms no unguarded use of unsafe DOM methods.
- **Source:** OWASP XSS Prevention Cheat Sheet

### FE-XSS-004 — No inline scripts or event handlers

Inline `<script>` blocks and inline event handlers (`onclick`, `onerror`, etc.) SHOULD NOT be used. JavaScript SHOULD be loaded from external files only.

- **Rationale:** Inline scripts cannot be distinguished from injected scripts without CSP nonces. Externalizing scripts enables strict CSP enforcement.
- **Verification:** Code review confirms no inline scripts or event handlers; CSP review confirms inline is restricted.
- **Source:** OWASP CSP Cheat Sheet

### FE-XSS-005 — HTML sanitization for user content

When users are allowed to author HTML (rich text editors, markdown preview), the rendered output MUST be sanitized using a dedicated sanitization library. Output MUST NOT be modified after sanitization.

- **Rationale:** Output encoding destroys intended formatting. Sanitization preserves safe markup while stripping dangerous elements. Post-sanitization modification can reintroduce vulnerabilities.
- **Verification:** Code review confirms sanitization library usage and no post-sanitization manipulation.
- **Source:** OWASP XSS Prevention Cheat Sheet

### FE-XSS-006 — URL validation

Untrusted URLs used in `href`, `src`, or redirect targets MUST be validated against an allowlist of protocols (`https:`, `mailto:`). The `javascript:` and `data:` protocols MUST be rejected.

- **Rationale:** `javascript:` and `data:` URLs execute arbitrary code when navigated to or loaded.
- **Verification:** Test confirms that `javascript:` and `data:` URLs are rejected in all user-controllable URL contexts.
- **Source:** OWASP XSS Prevention Cheat Sheet

## FE-CSRF — Cross-Site Request Forgery Prevention

### FE-CSRF-001 — Token-based protection

All state-changing requests MUST include a CSRF token validated by the server. Tokens MUST be unique per session, cryptographically random, and transmitted in the request body or a custom header — not in a cookie alone.

- **Rationale:** CSRF attacks forge requests from the victim's browser. Tokens prove the request originated from the application's own UI.
- **Verification:** Test confirms that state-changing requests without a valid CSRF token are rejected.
- **Source:** OWASP CSRF Prevention Cheat Sheet

### FE-CSRF-002 — SameSite cookie attribute

Session cookies MUST set the `SameSite` attribute to `Lax` or `Strict`. `SameSite=None` MUST only be used when cross-site cookie transmission is explicitly required and justified.

- **Rationale:** SameSite prevents the browser from sending cookies with cross-site requests, blocking the most common CSRF vector.
- **Verification:** Cookie configuration review confirms SameSite is set.
- **Source:** OWASP CSRF Prevention Cheat Sheet

### FE-CSRF-003 — No state changes via GET

State-changing operations MUST NOT use GET requests. GET MUST be used only for data retrieval.

- **Rationale:** GET requests can be triggered by images, links, and prefetch — all vectors for CSRF. POST/PUT/DELETE require explicit form submission or JavaScript.
- **Verification:** Code review confirms no state changes on GET handlers.
- **Source:** OWASP CSRF Prevention Cheat Sheet

## FE-STORAGE — Secure Client-Side Storage

### FE-STORAGE-001 — No secrets in browser storage

Secrets, API tokens, session tokens, and credentials MUST NOT be stored in `localStorage` or `sessionStorage`.

- **Rationale:** Browser storage is accessible to any JavaScript running on the page. A single XSS vulnerability exposes all stored values.
- **Verification:** Code review confirms no sensitive data written to browser storage APIs.
- **Source:** OWASP Session Management Cheat Sheet

### FE-STORAGE-002 — Cookie security attributes

Session cookies MUST set the following attributes: `HttpOnly` (prevents JavaScript access), `Secure` (requires HTTPS), `SameSite` (restricts cross-site transmission). The `Domain` attribute SHOULD NOT be set (restricts to exact origin).

- **Rationale:** Each attribute closes a specific attack vector: HttpOnly blocks XSS-based theft, Secure blocks plaintext interception, SameSite blocks CSRF, and omitting Domain prevents subdomain attacks.
- **Verification:** Test confirms all attributes are present on session cookies.
- **Source:** OWASP Session Management Cheat Sheet

### FE-STORAGE-003 — No sensitive data in URLs

Sensitive data (tokens, credentials, PII) MUST NOT appear in URL query parameters, fragment identifiers, or path segments.

- **Rationale:** URLs appear in browser history, server logs, referrer headers, and proxy logs — all of which are accessible to unauthorized parties.
- **Verification:** Code review confirms sensitive data is transmitted in request bodies or headers only.
- **Source:** OWASP REST Security Cheat Sheet

## FE-CSP — Content Security Policy

### FE-CSP-001 — CSP header required

All responses serving HTML MUST include a `Content-Security-Policy` header. CSP MUST be delivered via HTTP header, not `<meta>` tags (which cannot enforce `frame-ancestors` or reporting).

- **Rationale:** CSP is the primary defense-in-depth against XSS. Meta tag CSP has feature gaps that leave attacks unblocked.
- **Verification:** Test confirms CSP header is present on all HTML responses.
- **Source:** OWASP CSP Cheat Sheet

### FE-CSP-002 — Strict CSP policy

CSP MUST use nonce-based or hash-based script restrictions. The policy MUST NOT include `unsafe-inline` or `unsafe-eval` for script sources. The policy MUST include `object-src 'none'` and `base-uri 'none'`.

- **Rationale:** `unsafe-inline` allows any injected script to execute, defeating CSP entirely. `unsafe-eval` allows string-to-code conversion. `object-src 'none'` blocks plugin-based attacks. `base-uri 'none'` prevents base tag injection.
- **Verification:** CSP header review confirms nonce/hash usage and absence of unsafe directives.
- **Source:** OWASP CSP Cheat Sheet

### FE-CSP-003 — Frame protection

CSP MUST include `frame-ancestors 'none'` (or `'self'` if same-origin framing is required). The `X-Frame-Options: DENY` header SHOULD also be set for legacy browser support.

- **Rationale:** Prevents clickjacking attacks where the application is embedded in a malicious page's iframe.
- **Verification:** Test confirms both headers are present.
- **Source:** OWASP HTTP Headers Cheat Sheet, OWASP CSP Cheat Sheet

### FE-CSP-004 — Form action restriction

CSP SHOULD include `form-action 'self'` to restrict form submission targets.

- **Rationale:** Prevents injected phishing forms from submitting credentials to attacker-controlled servers.
- **Verification:** CSP header review confirms form-action directive.
- **Source:** OWASP CSP Cheat Sheet

## FE-DEP — Dependency Management

### FE-DEP-001 — Vulnerability scanning

Frontend dependencies MUST be scanned for known vulnerabilities. Dependencies with known critical or high-severity vulnerabilities MUST be updated or replaced. Scanning SHOULD be automated in CI/CD.

- **Rationale:** Frontend dependencies are the most common vector for supply-chain attacks in web applications.
- **Verification:** CI/CD configuration review confirms scanning is configured; scan results show no unaddressed critical/high findings.
- **Source:** OWASP Dependency-Check

### FE-DEP-002 — Subresource integrity

Third-party scripts loaded from CDNs MUST include the `integrity` attribute with a valid hash. Scripts without integrity verification MUST NOT be loaded from external origins.

- **Rationale:** CDN compromise or DNS hijacking can replace legitimate scripts with malicious ones. SRI ensures the browser rejects tampered resources.
- **Verification:** Code review confirms `integrity` attributes on all externally hosted scripts.
- **Source:** OWASP HTTP Headers Cheat Sheet

### FE-DEP-003 — No dynamic third-party loading

The application MUST NOT dynamically load scripts from third-party origins at runtime without integrity verification. Ad-hoc script injection via `document.createElement('script')` with external sources MUST be avoided.

- **Rationale:** Dynamically loaded scripts bypass CSP in some configurations and cannot be verified with SRI if the content changes per request.
- **Verification:** Code review confirms no dynamic external script loading without integrity checks.
- **Source:** OWASP CSP Cheat Sheet

## FE-DATA — Sensitive Data Handling

### FE-DATA-001 — PII masking

Personally identifiable information displayed in the UI MUST be masked or partially redacted where the full value is not required for the user's task (e.g., show last four digits of phone number, masked email).

- **Rationale:** Shoulder surfing, screenshots, and screen sharing can expose PII displayed in full. Masking limits exposure.
- **Verification:** UI review confirms PII fields display masked values where appropriate.
- **Source:** OWASP Input Validation Cheat Sheet

### FE-DATA-002 — Autocomplete control

Forms collecting sensitive data (passwords, credit card numbers, security answers) MUST set `autocomplete="off"` or the appropriate autocomplete token to prevent browser storage of sensitive values.

- **Rationale:** Browser autocomplete stores form values locally. On shared or compromised machines, stored values are accessible to other users.
- **Verification:** Code review confirms autocomplete attributes on sensitive form fields.
- **Source:** OWASP Authentication Cheat Sheet

### FE-DATA-003 — Cache control for sensitive pages

Pages displaying sensitive data MUST set `Cache-Control: no-store` to prevent browser and proxy caching. Logout responses SHOULD include `Clear-Site-Data: "cache", "cookies", "storage"`.

- **Rationale:** Cached pages persist on disk and can be recovered after the session ends. Clear-Site-Data ensures cleanup on logout.
- **Verification:** Test confirms cache headers on sensitive pages and Clear-Site-Data on logout.
- **Source:** OWASP Session Management Cheat Sheet

## FE-AUTH — Authentication UX

### FE-AUTH-001 — Redirect validation

After authentication, redirect targets MUST be validated against an allowlist of application paths. Open redirects MUST NOT be allowed.

- **Rationale:** Open redirects enable phishing — attackers craft login URLs that redirect to malicious sites after authentication.
- **Verification:** Test confirms that redirect to an external domain after login is blocked.
- **Source:** OWASP Authentication Cheat Sheet

### FE-AUTH-002 — Session expiration UX

When a session expires, the application MUST redirect to the login page or display a clear re-authentication prompt. The application MUST NOT silently fail or display broken state.

- **Rationale:** Silent failures confuse users and may cause data loss. Clear prompts enable the user to re-authenticate and retry.
- **Verification:** Test confirms session expiration produces a clear authentication prompt.
- **Source:** OWASP Session Management Cheat Sheet

### FE-AUTH-003 — Logout completeness

The logout action MUST invalidate the session on the server, clear session cookies, and redirect to a public page. Client-side-only logout (clearing cookies without server invalidation) MUST NOT be used.

- **Rationale:** Client-only logout leaves the session valid on the server — anyone with the session ID can still use it.
- **Verification:** Test confirms server-side session invalidation on logout and that the old session ID is rejected afterward.
- **Source:** OWASP Session Management Cheat Sheet
