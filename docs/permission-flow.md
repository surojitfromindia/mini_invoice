# Permission Flow

This document explains how staff permissions work in `smart_audit` from definition, to storage, to request-time enforcement.

## Goal

The permission system is designed so that:

- all valid permission codes come from one Rust source of truth
- roles store permission codes as data, not as hard-coded branching logic
- authenticated requests load the caller's role permissions once
- routes and services can require one permission, all permissions, or any permission
- the code remains manageable even if the system grows to hundreds of permissions

## High-Level Flow

1. Permissions are defined in `src/auth/permission.rs` as the `Permission` enum.
2. Staff roles accept permission code strings through APIs such as role creation.
3. Incoming permission code strings are normalized and validated before being stored.
4. Role permissions are persisted in the `staff_role.permissions` column as a comma-separated string.
5. During authentication, the user's active staff membership and role are resolved.
6. The stored permission string is deserialized into a list of codes.
7. That list is also converted into a `HashSet<String>` for fast repeated lookups.
8. Protected routes use `AuthorizedContext` and declare the permission rule they need.
9. `AuthorizationService` checks the loaded permission set and returns `Unauthorized` or `Forbidden` when access is missing.

## 1. Permission Definition

File: `src/auth/permission.rs`

`Permission` is the central catalog for valid staff permissions.

Each enum variant maps to a stable string code through:

- `code()`: converts enum to stored/runtime string code
- `from_code()`: validates an incoming string code
- `all_codes()`: returns every defined permission code

This keeps permission validation, storage, bootstrap defaults, and authorization checks aligned.

## 2. Permission Input and Storage

File: `src/service/staff_role_service.rs`

When a staff role is created, the API payload sends:

```json
{
  "name_primary": "Manager",
  "permission_codes": [
    "branch.create",
    "staff.invite"
  ]
}
```

Before saving:

- `normalize_permission_codes()` trims whitespace
- empty values are ignored
- unknown values are rejected with `InvalidPermission`
- duplicates are removed

After validation, `serialize_permission_codes()` stores the result in the role row as a comma-separated string.

Current storage example:

```text
branch.create,staff.invite,staff.invitation.resend
```

This means the database stays data-driven. A role can hold any combination of permissions without schema changes.

## 3. Default Roles

File: `src/service/staff_role_service.rs`

When an organization is created, the system bootstraps default roles:

- `Owner`: gets `Permission::all_codes()`
- `Admin`: gets a selected elevated subset
- `Manager`: gets a smaller operational subset
- `Staff`: starts with no special permissions

This is important because the authorization layer does not care about role names. It only cares about which permission codes are attached to the role.

## 4. Loading Permissions During Authentication

Files:

- `src/api/context.rs`
- `src/resolver/auth_resolver.rs`
- `src/service/service_context.rs`

When a protected request arrives:

1. `AuthenticatedContext` reads the bearer token.
2. The JWT is verified.
3. The actor is resolved from the token subject.
4. If the token includes an organization context, the active staff membership is resolved.
5. The staff member's role is loaded.
6. `role.permissions` is deserialized into `Vec<String>`.
7. The same codes are converted into `HashSet<String>` using `build_permission_code_set()`.
8. Both are attached to `OrganizationStaffAccess` inside `ServiceContext`.

The important runtime object is:

- `permission_codes: Vec<String>` for raw permission data
- `permission_code_set: HashSet<String>` for fast authorization checks

The set exists so permission checks stay cheap even when roles eventually carry many permissions.

## 5. Route-Level Authorization

Files:

- `src/api/authorized_context.rs`
- `src/auth/authorization_service.rs`

`AuthorizedContext` only guarantees that the request is authenticated.

The handler then decides what authorization rule it needs:

```rust
let ctx = authorized_ctx.require_permission(Permission::StaffRoleCreate)?;
```

```rust
let ctx = authorized_ctx.require_all([
    Permission::BranchCreate,
    Permission::StaffInvite,
])?;
```

```rust
let ctx = authorized_ctx.require_any([
    Permission::StaffInvite,
    Permission::StaffRoleCreate,
])?;
```

This is the key scalability improvement.

Old pattern:

- one Rust marker type per permission

New pattern:

- one authenticated extractor
- permission rules declared directly where they are needed
- support for single, all-of, and any-of checks

That means adding a new permission does not require creating a new extractor type.

## 6. AuthorizationService Behavior

File: `src/auth/authorization_service.rs`

`AuthorizationService` performs the actual permission check against `ctx.get_staff_access()`.

It exposes:

- `require_permission()`
- `require_all_permissions()`
- `require_any_permission()`

Behavior:

- if the request has no organization staff access, return `AppError::Unauthorized`
- if the required permission rule is satisfied, return `Ok(())`
- otherwise return `AppError::Forbidden`

For failed `all` checks, the service reports which permissions are missing.

For failed `any` checks, the service reports the list of acceptable permissions.

## 7. Route Examples

Files:

- `src/api/routes/branch_routes.rs`
- `src/api/routes/staff_routes.rs`
- `src/api/routes/staff_role_routes.rs`

Examples in the current codebase:

- branch creation uses `require_all([Permission::BranchCreate])`
- staff role creation uses `require_permission(Permission::StaffRoleCreate)`
- staff invitation creation shows the `require_any([...])` pattern

Even if a route currently uses one permission, it can be upgraded to a multi-permission rule without changing the extractor design.

## 8. Error Outcomes

File: `src/errors/app_error.rs`

The main authorization-related failures are:

- `Unauthorized`
  - token is missing or invalid
  - user is not an authenticated organization staff member for the requested organization context

- `Forbidden`
  - the user is authenticated, but their role does not contain the required permission rule

The forbidden error carries a human-readable permission description such as:

```text
branch.create
```

or:

```text
all of [branch.create, staff.invite]
```

## 9. Why This Scales Better

This design scales because:

- new permissions only need to be added to the `Permission` enum mappings
- roles remain data-driven, so combinations do not require new code paths
- one route can require many permissions without creating many Rust types
- `HashSet` lookups keep authorization efficient as permission counts grow
- permission logic stays centralized in `permission.rs` and `authorization_service.rs`

## 10. How To Add a New Permission

When adding a new permission:

1. Add a new variant to `Permission` in `src/auth/permission.rs`.
2. Map it in `code()`.
3. Map it in `from_code()`.
4. Include it in `all_codes()` if it should be part of the full owner permission set.
5. Add it to any default roles that should receive it in `src/service/staff_role_service.rs`.
6. Use it in routes or services with `require_permission`, `require_all`, or `require_any`.
7. Add or update tests for normalization and authorization behavior if needed.

You do not need to:

- create a new extractor type
- add a new request context type
- change the database schema

## 11. Future Improvement Ideas

The current flow is solid for growth, but there are a few natural future upgrades:

- move from comma-separated permission storage to a join table if querying permissions directly in SQL becomes important
- group permissions into higher-level policies if many routes start sharing the same bundles
- add dedicated authorization tests around route handlers
- introduce helper constants for common permission groups used by multiple routes

## Reference Files

- `src/auth/permission.rs`
- `src/auth/authorization_service.rs`
- `src/api/authorized_context.rs`
- `src/api/context.rs`
- `src/resolver/auth_resolver.rs`
- `src/service/service_context.rs`
- `src/service/staff_role_service.rs`
- `src/api/routes/branch_routes.rs`
- `src/api/routes/staff_routes.rs`
- `src/api/routes/staff_role_routes.rs`
