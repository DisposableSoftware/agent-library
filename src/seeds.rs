use crate::document::Document;
use crate::error::AppError;
use crate::store::DocumentStore;

pub fn seed_documents(store: &DocumentStore) -> Result<(), AppError> {
    for d in default_documents() {
        store.save_document(&d)?;
    }
    Ok(())
}

fn default_documents() -> Vec<Document> {
    vec![
        Document {
            name: "REST API Design".to_owned(),
            enabled: true,
            date_added: "2026-05-12".to_owned(),
            task_type: "Architecture".to_owned(),
            file_path: Default::default(),
            content: REST_API_DESIGN.to_owned(),
        },
        Document {
            name: "Git Commit Style".to_owned(),
            enabled: true,
            date_added: "2026-05-12".to_owned(),
            task_type: "Style Guide".to_owned(),
            file_path: Default::default(),
            content: GIT_COMMIT_STYLE.to_owned(),
        },
        Document {
            name: "Code Documentation".to_owned(),
            enabled: true,
            date_added: "2026-05-12".to_owned(),
            task_type: "Documentation".to_owned(),
            file_path: Default::default(),
            content: CODE_DOCUMENTATION.to_owned(),
        },
    ]
}

const REST_API_DESIGN: &str = r#"# REST API Design

## Guiding Principles

Design APIs around resources, not actions. Each endpoint should represent a noun
(e.g. `/users`, `/orders`), and the HTTP verb expresses the operation.

## URL Structure

- Use plural nouns: `/users`, not `/user`
- Nest sub-resources shallowly: `/users/{id}/orders`, not deeper than two levels
- Keep paths lowercase with hyphens for multi-word segments: `/payment-methods`
- Never include the API version in the path for internal services; use headers instead

## HTTP Verbs

| Verb   | Idempotent | Safe | Use for                             |
|--------|-----------|------|-------------------------------------|
| GET    | Yes       | Yes  | Fetch resource(s)                   |
| POST   | No        | No   | Create a new resource               |
| PUT    | Yes       | No   | Replace a resource entirely         |
| PATCH  | No        | No   | Partial update                      |
| DELETE | Yes       | No   | Remove a resource                   |

## Status Codes

- `200 OK` — successful GET, PUT, PATCH
- `201 Created` — successful POST; include `Location` header pointing to new resource
- `204 No Content` — successful DELETE or action with no body
- `400 Bad Request` — client sent invalid input; include error details in body
- `401 Unauthorized` — authentication required
- `403 Forbidden` — authenticated but not authorized
- `404 Not Found` — resource does not exist
- `409 Conflict` — state conflict (e.g. duplicate creation)
- `422 Unprocessable Entity` — syntactically valid but semantically wrong
- `500 Internal Server Error` — never expose stack traces

## Request and Response Shape

Use JSON. Always set `Content-Type: application/json`.

Error responses should follow a consistent envelope:
```json
{
  "error": {
    "code": "VALIDATION_FAILED",
    "message": "Human-readable summary",
    "details": [{ "field": "email", "issue": "must be a valid email address" }]
  }
}
```

## Pagination

Prefer cursor-based pagination for large or frequently-updated collections:
```json
{
  "data": [...],
  "next_cursor": "opaque-token",
  "has_more": true
}
```

Offset pagination is acceptable for small, stable datasets.

## Versioning

Version via the `Accept` header: `Accept: application/vnd.myapi.v2+json`.
Maintain at least one previous major version for 6 months after a breaking change.
"#;

const GIT_COMMIT_STYLE: &str = r#"# Git Commit Style — Conventional Commits

Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification.

## Format

```
<type>(<scope>): <subject>

[optional body]

[optional footers]
```

## Types

| Type     | When to use                                              |
|----------|----------------------------------------------------------|
| `feat`   | A new feature visible to users or API consumers         |
| `fix`    | A bug fix                                               |
| `docs`   | Documentation changes only                              |
| `style`  | Formatting, whitespace — no logic change                |
| `refactor` | Code restructuring — no feature change or bug fix     |
| `test`   | Adding or correcting tests                              |
| `chore`  | Build process, dependency updates, tooling              |
| `perf`   | Performance improvement                                 |
| `ci`     | CI/CD pipeline changes                                  |

## Subject Line Rules

- Use the imperative mood: "add feature", not "added feature" or "adds feature"
- Do not capitalise the first letter
- No period at the end
- Keep under 72 characters
- Be specific: "fix null deref in user login flow" not "fix bug"

## Body

- Separate from subject with a blank line
- Wrap at 72 characters
- Explain *what* and *why*, not *how* (the diff shows how)

## Footers

- `BREAKING CHANGE: <description>` — triggers a major version bump
- `Fixes #123` or `Closes #123` — links to issue tracker

## Examples

```
feat(auth): add OAuth 2.0 PKCE flow for mobile clients

Replaces the implicit flow which is deprecated in OAuth 2.1.
Existing sessions are unaffected; new logins use PKCE automatically.

Closes #482
```

```
fix(api): return 404 instead of 500 when order not found

The order lookup was throwing an unhandled exception when the record
was missing. Now returns a structured 404 with a human-readable message.
```
"#;

const CODE_DOCUMENTATION: &str = r#"# Code Documentation

## Philosophy

Code should be self-documenting through clear naming and structure.
Comments explain *why*, not *what*. If you need a comment to explain what
a piece of code does, consider renaming or restructuring it instead.

## When to Write a Comment

Write a comment when:
- There is a non-obvious constraint or invariant (e.g. "must be called after init()")
- The code works around a bug in an external library (cite the issue)
- A performance optimisation makes the code less readable — explain the trade-off
- Business logic encodes a rule that isn't obvious from the domain model

Do **not** write comments that:
- Restate what the code already says (`i++ // increment i`)
- Reference the task or ticket that created the code (goes in the commit message)
- Describe what a previous version did ("previously this used X")

## Function/Method Documentation

For public API surfaces, write a one-sentence doc comment that states the
contract (what it returns, what it requires). Include:

- Any preconditions or invariants the caller must satisfy
- Side effects (I/O, mutations of external state)
- Error cases if non-obvious

Example (Rust):
```rust
/// Returns the decoded payload if the token is valid and unexpired.
/// Returns `Err(AuthError::Expired)` if the token has passed its `exp` claim.
pub fn verify_token(token: &str, secret: &[u8]) -> Result<Claims, AuthError>
```

## Module/File Documentation

Each file should start with a one-line description of its purpose.
Avoid restating the directory structure ("this file contains helpers for...").

## README Documentation

Every project and every public library should have a README covering:

1. What problem it solves (one paragraph)
2. Quickstart / minimal usage example
3. Configuration options
4. How to run tests
5. How to contribute

Keep the README current. A wrong README is worse than none.
"#;
