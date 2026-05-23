# Error Code Guide

This file defines how to choose error codes for a module in this repository.

## Format

Error code format is:

`AAA.BBB.CCCC`

Meaning:

- `AAA`: module family
- `BBB`: sub-group inside the module
- `CCCC`: concrete error number

Examples already in the codebase:

- `001.000.0001`: JWT error
- `100.000.0001`: user error
- `101.000.0001`: user credential error
- `102.000.0002`: organization error
- `103.000.0003`: staff invitation error

## Current Module Family Map

Reserve one `AAA` block per primary module.

- `000`: app-level shared errors
- `001`: auth token and JWT
- `100`: user
- `101`: user credential
- `102`: organization and branch
- `103`: staff, staff invitation, staff role

When adding a new primary domain module, allocate the next free family block.

Suggested next blocks:

- `104`: audit
- `105`: report
- `106`: file or attachment
- `107`: client app

## How To Decide A Code

Follow this order:

1. Choose the module family `AAA`.
2. Use `000` in `BBB` unless you really need a separate subgroup.
3. Pick the next free `CCCC` inside that module.
4. Never reuse an old code for a different meaning.
5. Keep one code bound to one exact business failure.

For most new service errors, use:

`AAA.000.0001`
`AAA.000.0002`
`AAA.000.0003`

Only introduce a new `BBB` subgroup when the module becomes large enough that one flat list stops being readable.

## When To Use A New BBB Group

Use a new `BBB` group only for stable sub-domains.

Good examples:

- `103.001.xxxx`: staff role errors
- `103.002.xxxx`: staff invitation errors
- `102.001.xxxx`: branch-specific errors

Do not create a new `BBB` just because one file has a few extra enum variants.

## Recommended Meaning By HTTP Type

The numeric code is not the HTTP status, but the error meaning should stay consistent.

- `NotFound`: missing record or missing scoped resource
- `Conflict`: invalid state, duplicate, expired, already used, invalid transition
- `Unauthorized`: invalid token or missing login
- `Forbidden`: authenticated but missing permission
- `InternalServerError`: unexpected server-side failure only

Important rule:

Do not use `InternalServerError` for business cases that the client can understand and fix.

## Naming Rule Inside A Module

Keep codes ordered by first appearance.

Example for a new `AuditServiceError`:

- `104.000.0001`: audit already exists
- `104.000.0002`: audit not found
- `104.000.0003`: audit is locked

If later audit comments become large enough to deserve their own subgroup:

- `104.001.0001`: audit comment not found
- `104.001.0002`: audit comment already resolved

## Reserved Shared Codes

These should stay shared and should not be reused by module errors:

- `000.000.0001`: invalid credentials
- `000.000.0002`: unauthorized
- `000.000.0003`: forbidden

## Database Codes

Database adapter level errors are separate from business module errors.

Current examples:

- `100.001.001`: duplicate record from database unique constraint
- `100.000.000`: generic database runtime failure
- `100.000.500`: uncategorized database error

Rule:

- Prefer a business-module code when the failure is expected in domain logic.
- Fall back to database codes only when the failure comes directly from the persistence layer and has not been translated into domain meaning.

## Decision Checklist

Before adding a new code, check:

1. Is this a shared app error or a module error?
2. Which `AAA` module family owns this failure?
3. Do I really need a new `BBB` subgroup?
4. Is this code already used anywhere else?
5. Does the HTTP status match the actual business meaning?

## Recommended Workflow

When creating a new module:

1. Reserve one `AAA` family in this file.
2. Start with `BBB = 000`.
3. Add codes in order in that module error enum.
4. Keep the mapping readable in one place.
5. If the module splits into stable sub-domains later, then introduce `BBB` subgroups.

## Example Template

```rust
ErrorMeta::new(
    "104.000.0001",
    "Audit already exists",
    HttpErrorCode::Conflict,
)
```

## Practical Rule

If unsure, do this:

- new module -> assign next free `AAA`
- first errors -> keep `BBB = 000`
- first error -> start from `0001`
- never encode HTTP status into the number
- never change meaning of an existing code after release
