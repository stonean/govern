# 008 — Security Rules Research

## Sources

The security rules are derived from the following authoritative sources. Each rule traces back to one or more of these references.

### OWASP Cheat Sheet Series

Primary source for actionable security rules. Each cheat sheet is maintained by the OWASP community and represents industry consensus.

| Cheat Sheet | URL | Categories Informed |
| --- | --- | --- |
| Input Validation | cheatsheetseries.owasp.org/cheatsheets/Input_Validation_Cheat_Sheet | BE-INPUT |
| Authentication | cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet | BE-AUTH |
| Session Management | cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet | BE-AUTH, FE-STORAGE |
| Authorization | cheatsheetseries.owasp.org/cheatsheets/Authorization_Cheat_Sheet | BE-AUTHZ |
| Password Storage | cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet | BE-DATA |
| Cryptographic Storage | cheatsheetseries.owasp.org/cheatsheets/Cryptographic_Storage_Cheat_Sheet | BE-DATA |
| REST Security | cheatsheetseries.owasp.org/cheatsheets/REST_Security_Cheat_Sheet | BE-API, BE-INFRA |
| SQL Injection Prevention | cheatsheetseries.owasp.org/cheatsheets/SQL_Injection_Prevention_Cheat_Sheet | BE-INPUT |
| Logging | cheatsheetseries.owasp.org/cheatsheets/Logging_Cheat_Sheet | BE-LOG |
| Secrets Management | cheatsheetseries.owasp.org/cheatsheets/Secrets_Management_Cheat_Sheet | BE-DATA |
| Error Handling | cheatsheetseries.owasp.org/cheatsheets/Error_Handling_Cheat_Sheet | BE-ERROR |
| File Upload | cheatsheetseries.owasp.org/cheatsheets/File_Upload_Cheat_Sheet | BE-INPUT |
| HTTP Headers | cheatsheetseries.owasp.org/cheatsheets/HTTP_Headers_Cheat_Sheet | BE-API, FE-CSP |
| XSS Prevention | cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet | FE-XSS |
| CSRF Prevention | cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet | FE-CSRF |
| CSP | cheatsheetseries.owasp.org/cheatsheets/Content_Security_Policy_Cheat_Sheet | FE-CSP |

### Standards and Specifications

| Standard | Relevance |
| --- | --- |
| OWASP Top 10 (2021) | Threat categorization and priority ordering |
| OWASP API Security Top 10 (2023) | API-specific threat categorization |
| RFC 2119 | Requirement level keywords (MUST, SHOULD, etc.) |
| RFC 7807 | Problem Details for HTTP APIs (error response format) |
| NIST SP 800-63B | Digital Identity Guidelines — password and authenticator requirements |
| CIS Benchmarks | Infrastructure hardening (database, message broker, container) |

## Notes

- Rules are generalized from the sources above — they do not prescribe specific libraries, languages, or frameworks
- The OWASP Cheat Sheet Series provided the most directly actionable content; most rules map to specific cheat sheet recommendations
- CIS Benchmarks inform infrastructure-level rules (management port exposure, database access) but are not reproduced directly — the rules reference the principles rather than specific benchmark items
