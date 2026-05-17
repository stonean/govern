# Accessibility Rules — Frontend

Enforceable accessibility rules for browser-rendered UI. These rules apply to any project with a web frontend adopting `govern`. They derive from WCAG 2.2 AA, the de facto baseline for public-facing web applications in regulated jurisdictions (EU EAA, US Section 508, UK Equality Act, Ontario AODA).

Rules use RFC 2119 language: **MUST** / **MUST NOT** are enforced by the validate command (errors); **SHOULD** / **SHOULD NOT** are flagged as warnings.

Rule IDs follow the format `FE-{CATEGORY}-{NNN}` and are permanent — once assigned, an ID is never renumbered, even if the rule is moved within the file or deprecated. Categories: `SEMHTML` (semantic HTML), `KBD` (keyboard navigation and focus), `ARIA` (ARIA usage), `CONTRAST` (color and contrast), `A11YFORM` (accessible forms), `A11YMEDIA` (text alternatives and media). See `specs/008-security-rules/data-model.md` for the full schema.

Projects without a frontend can pin this file in `.govern.toml` to skip it during `govern` updates.

## FE-SEMHTML — Semantic HTML

### FE-SEMHTML-001

> Interactive UI elements MUST be implemented with the native HTML element that has the matching semantic role: `<button>` for buttons, `<a href>` for navigation, `<input>`/`<select>`/`<textarea>` for form fields, `<details>`/`<summary>` for disclosure widgets where suitable. Generic `<div>` and `<span>` elements styled as interactive (`onclick` on a `<div>`, "fake button" with `role="button"`) MUST NOT be used when a native element suffices.

**Rationale:** Native interactive elements arrive with keyboard focus behavior, accessible names from text content, screen-reader role announcements, browser-native state (`:disabled`, `:focus-visible`), and OS-level hooks (voice control, switch control, Windows High Contrast) — none of which a `div` provides. Re-implementing these in JavaScript is invariably incomplete and breaks assistive technology.

**Verification:** Any spec or plan that introduces interactive UI MUST name the native HTML element used per interaction. Validate flags affected-files snippets where `onClick` is bound to a `<div>` or `<span>` without a matching `role`, `tabIndex`, and explicit keyboard handling (`Enter`/`Space`).

**Source:** WCAG 2.2 §4.1.2 (Name, Role, Value), WAI-ARIA Authoring Practices "Rule of Least Power"

### FE-SEMHTML-002

> Document structure MUST use landmark elements: `<header>`, `<nav>`, `<main>` (exactly one per page), `<footer>`, and `<aside>` for their semantic purposes. Pages MUST have a single `<h1>` and a logically nested heading hierarchy with no skipped levels (no `<h1>` immediately followed by `<h3>`).

**Rationale:** Screen-reader users navigate by landmarks and heading levels. A page without `<main>` forces sequential reading from the top; a broken heading hierarchy makes the document outline noise. Landmark structure is the assistive-tech equivalent of a sitemap.

**Verification:** Any spec or plan that introduces a new page or top-level UI surface MUST commit to landmark structure and a heading-hierarchy plan. Validate flags page specs that omit `<main>` or that describe heading usage with skipped levels.

**Source:** WCAG 2.2 §1.3.1 (Info and Relationships), §2.4.1 (Bypass Blocks), §2.4.6 (Headings and Labels)

## FE-KBD — Keyboard Navigation and Focus

### FE-KBD-001

> All interactive UI MUST be operable by keyboard alone. Every focusable element MUST be reachable via `Tab` and operable via documented key bindings (`Enter`/`Space` for buttons, arrow keys for menus and tablists per WAI-ARIA Authoring Practices). Mouse-only or pointer-only interactions MUST NOT exist.

**Rationale:** Keyboard operability is the substrate every other assistive technology builds on — screen readers, switch control, voice control, and on-screen keyboards all surface keyboard events. A mouse-only interaction is invisible to all of them. Keyboard-only navigation is also the ergonomic baseline for power users and is mandatory for users with motor impairments.

**Verification:** Any spec or plan that introduces interactive UI MUST commit to keyboard operability and name the key bindings for any non-default interaction (menus, dialogs, custom widgets). Validate flags interactive-UI specs that describe drag-and-drop, hover-reveal, or pointer-only flows without a keyboard equivalent.

**Source:** WCAG 2.2 §2.1.1 (Keyboard), WAI-ARIA Authoring Practices

### FE-KBD-002

> Visible focus indication MUST be present on every interactive element. The default browser focus ring MUST NOT be removed (`outline: none`, `outline: 0`) without an equivalent or stronger replacement that meets WCAG 2.2 §2.4.13 (Focus Appearance) — minimum 2 CSS pixels thick on the perimeter of the focused element, with 3:1 contrast against adjacent colors.

**Rationale:** Keyboard users navigate by following the focus ring. Removing it without replacement leaves them blind — they cannot tell which element they are about to activate. This is one of the most common accessibility failures and one of the easiest to detect mechanically.

**Verification:** Any spec or plan that introduces styling for interactive elements MUST commit to visible focus indication meeting §2.4.13. Validate flags CSS in affected files that contains `outline: none` or `outline: 0` on focusable elements without an accompanying `:focus`/`:focus-visible` replacement.

**Source:** WCAG 2.2 §2.4.7 (Focus Visible), §2.4.13 (Focus Appearance, new in 2.2)

### FE-KBD-003

> Modal dialogs and disclosure UI that take over the page MUST trap focus while open (Tab and Shift+Tab cycle within the dialog) and MUST return focus to the triggering element on close. Pressing `Escape` MUST close the dialog.

**Rationale:** Without a focus trap, Tab takes the user out of the dialog and into the page underneath — interacting with elements they cannot see. Without focus restoration, the user loses their place when the dialog closes. Both behaviors are disorienting; together they make modal UI effectively unusable with a keyboard or screen reader.

**Verification:** Any spec or plan that introduces modal dialogs, drawers, or full-screen overlays MUST commit to focus-trap, focus-restoration, and Escape-key dismissal. Validate flags modal-UI specs that omit any of the three.

**Source:** WAI-ARIA Authoring Practices: Dialog (Modal) Pattern, WCAG 2.2 §2.1.2 (No Keyboard Trap)

## FE-ARIA — ARIA Usage

### FE-ARIA-001

> ARIA roles, states, and properties MUST be used to fill gaps in native HTML semantics — not to override them. The first rule of ARIA ("if you can use a native element with the semantics and behavior you need, do so") MUST take precedence: `role="button"` on a `<div>` is forbidden when a `<button>` would serve. ARIA MUST be added only when no native element exists for the pattern (tabs, comboboxes, treeviews, custom listboxes, etc.).

**Rationale:** Wrong ARIA is worse than no ARIA — it overrides correct native semantics with a broken substitute. The vast majority of ARIA bugs are unnecessary ARIA: `aria-label` on text already-labeled, `role="button"` on `<a>`, `aria-hidden` on focusable elements. The "fill gaps, don't override" discipline eliminates the bulk of these failures by construction.

**Verification:** Any spec or plan that introduces custom widgets MUST justify each ARIA role/state/property against the WAI-ARIA Authoring Practices pattern it implements. Validate flags affected-files snippets that add ARIA to native interactive elements (`role` on `<button>`/`<a>`/inputs) or that apply `aria-hidden="true"` to focusable elements.

**Source:** W3C WAI-ARIA 1.2 §2.1 (First Rule of ARIA Use), W3C ARIA in HTML

### FE-ARIA-002

> Every interactive element MUST have an accessible name. Where visible text serves as the name (button labels, link text), no `aria-label` is required. Where the element has no visible text (icon-only buttons, image links, decorative-text-only links), `aria-label` or `aria-labelledby` MUST provide a meaningful name describing the action or destination. Generic names (`"button"`, `"link"`, `"click here"`, `"read more"` without context) MUST NOT be used.

**Rationale:** Screen readers announce elements by their accessible name. An icon-only button with no name announces as `"button"` — useless. A meaningful name (`"close dialog"`, `"add to cart"`, `"read more about pricing"`) makes the UI navigable by ear and also feeds rotor-based navigation (lists of links/buttons sorted alphabetically).

**Verification:** Any spec or plan that introduces icon-only buttons, image links, or non-text interactive elements MUST commit to an accessible-name strategy per element. Validate flags affected-files snippets with `<button>` containing only an `<svg>` or icon character without `aria-label`, and flags accessible-name values from a generic-name denylist.

**Source:** WCAG 2.2 §4.1.2 (Name, Role, Value), §2.4.4 (Link Purpose in Context)

## FE-CONTRAST — Color and Contrast

### FE-CONTRAST-001

> Text MUST meet the WCAG 2.2 AA contrast ratio against its background: 4.5:1 for normal text, 3:1 for large text (≥18pt or ≥14pt bold) and for UI-component graphical elements (form-field borders, focus rings, status indicators, icons that convey information). Color combinations that fail these ratios MUST NOT be used for text or for functional UI elements.

**Rationale:** Low-contrast text is unreadable for users with low vision, color-deficient vision, in bright sunlight, or on aging displays. The 4.5:1 / 3:1 thresholds are not aesthetic guidelines — they are the empirical line below which legibility falls off for a significant fraction of users.

**Verification:** Any spec or plan that introduces a color palette, theme, or text-on-color UI element MUST commit to AA contrast across all foreground/background pairs in use. Validate runs against the project's documented palette and flags pair combinations below threshold; flags affected-files snippets that hard-code colors without referencing the palette.

**Source:** WCAG 2.2 §1.4.3 (Contrast Minimum), §1.4.11 (Non-text Contrast)

### FE-CONTRAST-002

> Information MUST NOT be conveyed by color alone. Status, validation state, required-field marking, and selection state MUST be reinforced by a non-color cue: icon, text label, underline, weight, pattern, or position.

**Rationale:** Color-only signals are invisible to color-blind users (~8% of men, ~0.5% of women), to grayscale displays, to screen readers, and to printed copies. A red border with no error icon and no error text is silent to a significant fraction of users — they see the form reject their submission and do not know why.

**Verification:** Any spec or plan that introduces state indicators (errors, warnings, success, selection, required) MUST commit to a non-color cue per state. Validate flags spec descriptions that describe state via color only ("the field turns red"), and flags affected-files snippets that style validation state without an accompanying icon or text node.

**Source:** WCAG 2.2 §1.4.1 (Use of Color)

## FE-A11YFORM — Accessible Forms

### FE-A11YFORM-001

> Every form input MUST have a programmatically associated label. The label MUST be one of: a `<label>` element with `for` matching the input's `id`, an enclosing `<label>` wrapping the input, or an `aria-labelledby` reference to a visible text element. Placeholder text MUST NOT substitute for a label.

**Rationale:** Screen readers announce labels when focus enters an input — without a programmatic association, the input is just "edit text." Placeholders disappear on focus and on autofill, leaving the user with no field identity at the moment they need it most. The label↔input pairing is also what makes click-on-label work for checkboxes and radio buttons — the difference between a small click target and a usable one for users with motor impairments.

**Verification:** Any spec or plan that introduces a form MUST commit to programmatic labels per input. Validate flags affected-files snippets with `<input>`, `<select>`, or `<textarea>` that carry a `placeholder` but no associated `<label>`, `aria-label`, or `aria-labelledby`.

**Source:** WCAG 2.2 §1.3.1 (Info and Relationships), §3.3.2 (Labels or Instructions), §4.1.2 (Name, Role, Value)

### FE-A11YFORM-002

> Form validation errors MUST be associated with their inputs via `aria-describedby` pointing at the error message element, AND inputs in an error state MUST set `aria-invalid="true"`. Error messages MUST be visible (not screen-reader-only) and MUST identify the field by name in their text.

**Rationale:** Screen-reader users move from input to input via Tab; the `aria-describedby` reference is what causes the error message to be announced on focus. Without `aria-invalid`, browsers and assistive tech cannot offer error-summary navigation. Visible, named errors help every user — not just screen-reader users — because they survive losing focus on the offending field.

**Verification:** Any spec or plan that introduces form validation UI MUST commit to `aria-describedby` + `aria-invalid` per error-bearing input. Validate flags form-validation specs that omit either ARIA attribute, that hide errors visually with `display: none` / `visibility: hidden` while revealing them only to screen readers, or that surface errors without naming the offending field.

**Source:** WCAG 2.2 §3.3.1 (Error Identification), §3.3.3 (Error Suggestion), §4.1.3 (Status Messages)

## FE-A11YMEDIA — Text Alternatives and Media

### FE-A11YMEDIA-001

> Every `<img>` element MUST have an `alt` attribute. Informational images MUST have descriptive alt text; purely decorative images MUST have `alt=""` (empty string) so screen readers skip them. SVG icons used as interactive triggers MUST carry an accessible name per `FE-ARIA-002`, either via the enclosing element's `aria-label` or via `<title>` inside the SVG.

**Rationale:** A missing `alt` attribute causes screen readers to announce the image's file path — useless and noisy. An empty `alt` correctly skips decorative images. The distinction between informational and decorative is a content-author judgement the `alt` attribute exists to capture — omitting it forces the screen reader to guess, badly.

**Verification:** Any spec or plan that introduces image rendering MUST commit to an alt-text policy. Validate flags affected-files snippets with `<img>` elements lacking an `alt` attribute, and flags SVG-as-button patterns without an accessible name.

**Source:** WCAG 2.2 §1.1.1 (Non-text Content)

### FE-A11YMEDIA-002

> Video content MUST provide synchronized captions; audio-only content MUST provide a transcript. Captions and transcripts for content authored by the application MUST be human-edited (not solely auto-generated).

**Rationale:** Without captions, video is inaccessible to deaf and hard-of-hearing users and to anyone in a sound-off context (open offices, public transit, libraries). Auto-generated captions are useful as a starting point but produce error rates that make critical content (proper names, numbers, technical terminology) unreliable — and unreliable captions are sometimes worse than none, because users trust them and act on the errors.

**Verification:** Any spec or plan that introduces video or audio media MUST commit to captions/transcripts and to the editorial pipeline that produces them. Validate flags media-introducing specs that omit caption/transcript commitments or that rely solely on auto-generation for application-authored content.

**Source:** WCAG 2.2 §1.2.1 (Audio-only and Video-only, Prerecorded), §1.2.2 (Captions, Prerecorded)
