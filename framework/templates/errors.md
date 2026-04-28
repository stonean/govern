# Errors

<!-- This is a living document describing your project's error handling conventions.
     Establish the core format and conventions early, then add error codes
     as modules are built. -->

## Error Response Format

<!-- The standard structure for error responses. Example:

All error responses use a consistent JSON envelope:

```json
{
  "error": {
    "code": "validation_failed",
    "message": "One or more fields are invalid.",
    "details": []
  }
}
```

The `details` array is optional and used for validation errors (see below).

-->

## Error Codes

<!-- Naming convention for error codes and a registry of codes in use. Example:

Error codes use `snake_case` and follow the pattern `{category}_{description}`.

| Code | Category | Meaning |
| --- | --- | --- |
| `auth_token_expired` | auth | Access token has expired |
| `validation_failed` | validation | Request body failed validation |
| `not_found` | resource | Requested resource does not exist |

-->

## Status Mapping

<!-- How error codes map to response status codes or equivalent. Example:

| Category | Status Code |
| --- | --- |
| `auth_*` | 401 |
| `forbidden_*` | 403 |
| `not_found` | 404 |
| `validation_*` | 422 |
| `rate_*` | 429 |
| `internal_*` | 500 |

-->

## Validation Errors

<!-- How per-field validation errors are structured. Example:

Validation errors populate the `details` array with one entry per invalid field:

```json
{
  "error": {
    "code": "validation_failed",
    "message": "One or more fields are invalid.",
    "details": [
      { "field": "email", "code": "required", "message": "Email is required." },
      { "field": "age", "code": "out_of_range", "message": "Age must be between 0 and 150." }
    ]
  }
}
```

-->

## Logging

<!-- How errors are logged and severity mapping. Example:

All errors are logged with structured fields including the error code,
request ID, and stack trace (for unexpected errors only).

| Severity | When |
| --- | --- |
| `warn` | Client errors (4xx) — expected, no action needed |
| `error` | Server errors (5xx) — unexpected, requires investigation |

-->

## Internal vs External Errors

<!-- Rules for what error information is exposed to callers vs kept internal. Example:

External responses include the error code and a safe, human-readable message.
Stack traces, internal identifiers, and infrastructure details are never
included in responses. They are logged server-side with the request ID
for correlation.

-->
