# DTO-First Listing Flow

This document explains the reusable page-based `getAll()` flow currently used in `smart_audit`.

The current implementation keeps two things simple:

- API transport shapes live in `src/api/dto/`
- shared listing infrastructure only handles page-based pagination for now

The first reference implementation is branch listing.

## Goal

The listing flow is designed so that:

- handlers only deal with API DTOs
- services accept transport-agnostic input structs
- list responses use one shared page envelope
- each module defines its own typed filter, sort, include, and row DTO
- joins are relation-driven, not arbitrary client-built SQL
- pagination behavior stays consistent across modules

## High-Level Flow

1. A route handler receives query params into an API DTO from `src/api/dto/`.
2. The DTO converts into a service input struct owned by the feature service.
3. The service applies tenant scoping first from `ServiceContext`.
4. The service composes its own filters, joins, sort order, and selected columns.
5. The shared listing executor in `src/db/listing/` runs the page query.
6. The route returns `ApiResponse<PageListResult<T>>`.

## Module Boundaries

### API DTO layer

Files:

- `src/api/dto/mod.rs`
- `src/api/dto/common_dto.rs`
- feature DTO files such as `src/api/dto/branch_dto.rs`

This layer owns:

- request bodies
- query string DTOs
- shared response envelopes
- response row DTOs that are returned to the API caller

Examples:

- `CreateBranchRequestDto`
- `BranchListPageQueryDto`
- `PageListResult<T>`

### Service layer

Files:

- `src/service/branch_service.rs`
- similar feature services for future modules

This layer owns:

- service input structs
- business scoping
- query composition
- transaction handling
- calling shared listing helpers

Examples:

- `CreateBranchInput`
- `BranchListPageInput`
- `BranchSortField`
- `BranchInclude`

### Shared DB listing layer

File:

- `src/db/listing/mod.rs`

This layer owns:

- page pagination validation
- page execution through SeaORM paginator

It does not own:

- module filters
- which joins are allowed
- which columns are selected
- how each module maps rows

## Shared DTOs

The common transport DTOs live in `src/api/dto/common_dto.rs`.

### Page request

`PagePaginationQuery`

- `page`
- `per_page`

Defaults:

- `page = 1`
- `per_page = 20`

### Shared list response shapes

`PageListResult<T>`

- `rows`
- `meta`

`PageMeta`

- `page`
- `per_page`
- `total_rows`
- `total_pages`
- `has_next`
- `has_prev`

## Shared Listing Executor

The shared executor lives in `src/db/listing/mod.rs`.

Important functions:

- `validate_page_pagination(...)`
- `execute_page_query(...)`

`execute_page_query(...)`:

- paginates with SeaORM `PaginatorTrait`
- fetches rows for the requested page
- calculates `total_rows` and `total_pages`
- returns a `PageListResult<T>`

## Branch Reference Implementation

The first reusable list flow is implemented in:

- `src/api/routes/branch_routes.rs`
- `src/api/dto/branch_dto.rs`
- `src/service/branch_service.rs`

### Endpoints

Page pagination:

- `GET /api/v1/branch`

Create branch:

- `POST /api/v1/branch`

### Request flow

#### Create branch

1. `CreateBranchRequestDto` is deserialized in the route.
2. The DTO converts into `CreateBranchInput`.
3. `BranchService::create_branch(...)` performs business logic and persistence.
4. The route returns `ApiResponse<PublicIdResponse>`.

#### Branch page listing

1. `BranchListPageQueryDto` is deserialized from the query string.
2. The DTO converts into `BranchListPageInput`.
3. `BranchService::list_branches_page(...)` validates pagination.
4. The service applies organization scoping using `ctx.get_organization_id()`.
5. The service applies typed filters and sort rules.
6. The service selects the row DTO columns.
7. `execute_page_query(...)` returns `PageListResult<BranchListItemDto>`.
8. The route wraps the result in `ApiResponse`.

## Branch Query Contract

The branch module defines its own typed list contract.

### Filters

- `name`
- `is_primary`

### Sort fields

- `created_at`
- `name_primary`

### Sort direction

- `asc`
- `desc`

### Includes

- `organization`

### Row DTO

`BranchListItemDto` contains:

- `public_id`
- `name_primary`
- `name_secondary`
- `is_primary`
- optional `organization_name_primary`

## Relation-Driven Join

Branch listing joins organization data through SeaORM relations, not handwritten join graphs.

Relevant entity files:

- `src/entity/organization/branch_entity.rs`
- `src/entity/organization/organization_entity.rs`

Current relation setup:

- branch `belongs_to` organization
- organization `has_many` branches

This allows the branch query to safely use:

- `left_join(Organization::Entity)`

The service can then expose organization data only when the caller asks for the `organization` include.

## Why The Service Still Owns Query Composition

The shared listing module is intentionally small.

It does not try to become a runtime query DSL because that would make the system:

- harder to validate
- harder to secure
- harder to keep type-safe

Instead, each module keeps local control over:

- allowed filters
- allowed joins
- allowed sorts
- selected columns

This keeps the core reusable without losing module-level clarity.

## Example Requests

### Branch page listing

```http
GET /api/v1/branch?page=1&per_page=20&sort=created_at&direction=desc
```

### Branch page listing with filters and include

```http
GET /api/v1/branch?page=1&per_page=10&name=HQ&is_primary=true&include=organization
```

## How To Add The Next `getAll()` Module

For a new module, follow the same pattern:

1. Add request and response DTOs under `src/api/dto/`.
2. Add service input structs and typed enums in the feature service.
3. Add SeaORM relations if the list needs joins.
4. Build a scoped base query in the service.
5. Apply typed filters.
6. Apply deterministic sort rules.
7. Select a row DTO using `FromQueryResult` or `DerivePartialModel`.
8. Call `execute_page_query(...)`.
9. Return the shared list envelope from the route.

## Current Guardrails

The current implementation intentionally enforces these limits:

- only typed module filters are supported
- only typed module includes are supported
- only relation-backed joins are supported
- only page-based pagination is supported for now

These guardrails keep the API predictable and easier to evolve.

## Current Reference Files

Use these files as the copy pattern for future work:

- `src/api/dto/common_dto.rs`
- `src/api/dto/branch_dto.rs`
- `src/api/routes/branch_routes.rs`
- `src/db/listing/mod.rs`
- `src/service/branch_service.rs`
- `src/entity/organization/branch_entity.rs`
- `src/entity/organization/organization_entity.rs`
