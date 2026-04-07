# 001 — System Spec Templates Tasks

Tasks derived from the [plan](plan.md). Complete in order.

## 1. Create templates/system.md

- [x] Create the template with sections: Configuration, Application Lifecycle, Request Lifecycle, Shared Infrastructure, Module Pattern
- [x] Add HTML comments with commented-out examples for each section
- [x] Include a multi-tenancy section as optional (commented guidance on when to include)
- [x] Ensure examples are technology-agnostic

Done when: `templates/system.md` exists with all sections, examples are commented out, no technology-specific references, passes markdownlint.

## 2. Create templates/errors.md

- [x] Create the template with sections: Error Response Format, Error Codes, Status Mapping, Validation Errors, Logging, Internal vs External
- [x] Add HTML comments with commented-out examples for each section
- [x] Include example error response structure (using generic JSON, not framework-specific)
- [x] Include example error code naming convention

Done when: `templates/errors.md` exists with all sections, examples are commented out, no technology-specific references, passes markdownlint.

## 3. Create templates/events.md

- [x] Create the template with sections: Event Catalog, Envelope Format, Naming Convention
- [x] Add a commented-out example event entry showing the expected catalog format
- [x] Include a comment suggesting projects consider specifying retry policy and dead-letter handling as dedicated feature specs
- [x] Include publisher/subscriber documentation pattern in the example

Done when: `templates/events.md` exists with catalog structure, example entry, retry/dead-letter suggestion, passes markdownlint.

## 4. Final review and lint

- [x] Run `npx markdownlint-cli2` on all three new templates
- [x] Verify consistency with existing template style (compare against `templates/spec.md` and `templates/plan.md`)
- [x] Verify no technology-specific language, framework, or library references
- [x] Update spec status to `planned`

Done when: all three templates pass lint, match existing style, and are technology-agnostic.
