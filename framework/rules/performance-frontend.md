# Performance Rules — Frontend

Enforceable performance rules for browser-rendered UI. These rules apply to any project with a web frontend adopting `govern`. They derive from Google Core Web Vitals, browser-vendor performance guidance, and HTTP delivery best practices.

Rules use RFC 2119 language: **MUST** / **MUST NOT** are enforced by the validate command (errors); **SHOULD** / **SHOULD NOT** are flagged as warnings.

Rule IDs follow the format `FE-{CATEGORY}-{NNN}` and are permanent — once assigned, an ID is never renumbered, even if the rule is moved within the file or deprecated. Categories: `VITALS` (Core Web Vitals budgets), `BUNDLE` (JavaScript/CSS bundle size), `IMAGE` (image delivery), `LOAD` (resource loading and lazy-loading), `FONT` (web font discipline). See `specs/008-security-rules/data-model.md` for the full schema.

Projects without a frontend can pin this file in `.govern.toml` to skip it during `govern` updates.

## FE-VITALS — Core Web Vitals Budgets

### FE-VITALS-001

> The project MUST publish documented performance budgets for the three Core Web Vitals — Largest Contentful Paint (LCP), Interaction to Next Paint (INP), and Cumulative Layout Shift (CLS) — measured at the 75th percentile of real-user data. Default budgets at Google's "Good" threshold: LCP ≤ 2.5s, INP ≤ 200ms, CLS ≤ 0.1. Stricter project-specific budgets are encouraged; looser budgets MUST be justified in `specs/system.md`.

**Rationale:** Web Vitals correlate directly with user engagement, conversion, and search ranking — Google has documented bounce-rate increases at every measured regression past these thresholds. Publishing budgets makes performance a falsifiable commitment instead of an aspirational nice-to-have. The p75 measurement basis prevents tail regressions from hiding behind a healthy median.

**Verification:** Any spec or plan that introduces or modifies user-facing UI MUST commit to maintaining the documented Web Vitals budgets. Validate flags frontend-UI specs that omit the budget commitment, and flags specs that propose looser thresholds without a `specs/system.md` justification reference.

**Source:** web.dev Core Web Vitals, Chrome UX Report methodology

### FE-VITALS-002

> Web Vitals MUST be measured continuously on real users (RUM), not only via synthetic CI runs. The application MUST emit `web-vitals` (or equivalent) measurements to a backend the team monitors. Synthetic Lighthouse runs in CI are complementary — they catch regressions before deploy — but MUST NOT be the sole measurement source.

**Rationale:** Synthetic measurements run on a clean lab machine over a fast network — conditions almost no real user has. RUM captures the actual distribution: slow phones, congested networks, cold caches, third-party variability. Without RUM, you optimize for a benchmark and ship regressions to users.

**Verification:** Any spec or plan covering frontend instrumentation, observability, or CI performance MUST commit to (a) RUM emission of Web Vitals and (b) a monitored backend for them. Validate flags performance specs that describe only synthetic measurement without a RUM commitment.

**Source:** web.dev RUM guidance, `web-vitals` JavaScript library documentation

## FE-BUNDLE — Bundle Size

### FE-BUNDLE-001

> The project MUST publish a JavaScript bundle size budget for each route or entry point, measured as transferred (compressed) bytes. The build MUST fail when a bundle exceeds its budget. Default starting budgets: ≤ 170KB compressed JS for the initial route, ≤ 50KB compressed CSS. Project-specific budgets MUST be documented in `specs/system.md`.

**Rationale:** Bundle size is the single largest controllable input to LCP and INP on mid-tier mobile devices over typical mobile networks. Without an enforcing budget, bundles grow unbounded as dependencies are added — and the regression is invisible until users complain. CI-enforced budgets make the cost of every dependency addition visible at PR review.

**Verification:** Any spec or plan that introduces frontend code, dependencies, or build configuration MUST commit to per-entry-point bundle budgets and a CI gate that fails on exceedance. Validate flags build/CI specs that omit the budget gate, and flags dependency-adding specs that omit a bundle-impact estimate.

**Source:** web.dev Performance Budgets, addyosmani.com performance-budgets guide

### FE-BUNDLE-002

> Routes that are not part of the initial render MUST be code-split and loaded on demand. The initial JavaScript payload MUST NOT include code for routes the user has not yet navigated to. Framework-specific dynamic-import mechanisms (`React.lazy`, dynamic `import()`, route-level chunking) are the canonical implementation.

**Rationale:** A monolithic bundle forces every user to download every route's code on first load, including code for features they may never use. Route-level splitting is the highest-ROI performance optimization for SPA frameworks — typically halving initial bundle size with no functional cost.

**Verification:** Any spec or plan that introduces a multi-route SPA or that adds a new route MUST commit to per-route code-splitting. Validate flags SPA build specs that produce a single monolithic bundle and flags new-route specs that statically import the route module from the entry point.

**Source:** web.dev Code Splitting, framework documentation (React/Vue/Angular)

## FE-IMAGE — Image Delivery

### FE-IMAGE-001

> Raster images served as content imagery MUST use modern formats with fallback: AVIF or WebP with JPEG/PNG fallback via `<picture>` and `srcset`. Single-format `<img src="*.jpg">`/`<img src="*.png">` MUST NOT be used for non-decorative content imagery (hero images, product photos, article media).

**Rationale:** AVIF and WebP deliver 25–50% smaller files than JPEG/PNG at equivalent visual quality. Image bytes typically dominate a page's transferred size, so modern formats are the largest single payload reduction available. The `<picture>` + `srcset` fallback is universally supported and adds no runtime cost.

**Verification:** Any spec or plan that introduces image-bearing UI MUST commit to modern-format delivery with fallback. Validate flags affected-files snippets with `<img src="*.jpg">` or `<img src="*.png">` for content imagery without an accompanying `<picture>` element or modern-format pipeline.

**Source:** web.dev Modern Image Formats, MDN `<picture>` documentation

### FE-IMAGE-002

> Images MUST declare their intrinsic dimensions via `width` and `height` attributes (or via CSS `aspect-ratio`) so the browser can reserve layout space before the image loads. Images that load without declared dimensions cause layout shift (CLS regression).

**Rationale:** A late-arriving image with undeclared dimensions pushes content down after the user has started reading or interacting — measurable as CLS, perceived as jarring. Declared dimensions let the browser reserve the space and load the image without disturbing surrounding content.

**Verification:** Any spec or plan that introduces image-bearing UI MUST commit to declared intrinsic dimensions per image. Validate flags affected-files snippets with `<img>` elements lacking `width` and `height` attributes (or a corresponding `aspect-ratio` rule).

**Source:** web.dev Optimize Cumulative Layout Shift, MDN `<img>` attribute documentation

## FE-LOAD — Resource Loading

### FE-LOAD-001

> Non-critical resources (below-the-fold images, off-route JavaScript, third-party widgets, embedded media) MUST be lazy-loaded. Images SHOULD use `loading="lazy"`; iframes SHOULD use `loading="lazy"`; off-route JavaScript MUST be code-split per `FE-BUNDLE-002`.

**Rationale:** Eagerly loading every resource on initial page load competes for bandwidth and CPU with the critical-path content the user is waiting for. Lazy-loading defers non-critical work until it is actually needed, freeing the main thread and the network for what matters first.

**Verification:** Any spec or plan that introduces below-the-fold UI, third-party embeds, or non-critical media MUST commit to lazy-loading. Validate flags affected-files snippets that statically include heavy third-party widgets in the initial render, and flags long-form-page specs that omit lazy-loading commitments for below-fold imagery.

**Source:** web.dev Native Lazy-Loading, MDN `loading` attribute documentation

### FE-LOAD-002

> The critical rendering path MUST be defined and documented per route — the minimum set of CSS and JavaScript required to render above-the-fold content. Critical CSS SHOULD be inlined in the document `<head>`; non-critical CSS SHOULD be loaded asynchronously (e.g., via `<link rel="preload" as="style" onload="this.rel='stylesheet'">`). Render-blocking JavaScript MUST be minimized; `<script>` tags MUST use `defer` or `async` unless inline execution is required for correctness (with the reason documented).

**Rationale:** Render-blocking resources delay First Contentful Paint by their full round-trip time. Inlined critical CSS lets the browser render immediately on receipt of the HTML; deferred JavaScript lets parsing continue without waiting for downloads. The difference is measured in seconds on slow networks.

**Verification:** Any spec or plan that introduces a new route or that modifies the document `<head>` MUST commit to critical-path discipline and to `defer`/`async` on non-essential scripts. Validate flags affected-files snippets with `<script>` tags lacking `defer` or `async` (other than documented inline-required scripts) and flags HTML specs that propose synchronous external stylesheets without an inline critical-CSS strategy.

**Source:** web.dev Critical Rendering Path, MDN script async/defer documentation

## FE-FONT — Web Font Discipline

### FE-FONT-001

> Web fonts MUST be loaded with a documented fallback strategy. `@font-face` declarations MUST use `font-display: swap` (or `optional` for non-critical typography) so text remains visible during font load. The default `font-display: auto` / `block` behaviors that produce Flash of Invisible Text (FOIT) MUST NOT be relied on.

**Rationale:** A blank page while a font downloads is worse than a momentarily mis-styled page — users read text content; they tolerate a font swap. `font-display: swap` shows fallback text immediately and swaps to the web font when it arrives. The FOIT default makes content invisible until the font lands, which on slow connections can be seconds.

**Verification:** Any spec or plan that introduces web fonts MUST commit to `font-display: swap` (or `optional`) on every `@font-face` declaration. Validate flags CSS in affected files containing `@font-face` rules without a `font-display` declaration or with `font-display: auto`/`block`.

**Source:** web.dev Font Display, MDN `@font-face` documentation

### FE-FONT-002

> Self-hosted web fonts MUST be served as WOFF2 (broadest support, smallest size); legacy formats (TTF, OTF, EOT, WOFF) MUST NOT be served as the primary source. The font payload SHOULD be subset to the Unicode ranges actually used by the application.

**Rationale:** WOFF2 compresses approximately 30% smaller than WOFF and is supported by every browser the project plausibly targets. Subsetting (e.g., dropping CJK glyphs from a Latin-only application) routinely halves the font payload again. Together, the two reduce font bytes by roughly 65% versus unsubsetted TTF.

**Verification:** Any spec or plan that introduces self-hosted fonts MUST commit to WOFF2 delivery and to a subsetting strategy. Validate flags affected-files snippets with `@font-face` rules that load `.ttf`/`.otf`/`.eot` URLs as the primary source.

**Source:** web.dev Optimize WebFont Loading, Google Fonts documentation
