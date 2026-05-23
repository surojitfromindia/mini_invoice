# Repository Guidelines

## Project Structure & Module Organization
`src/main.rs` starts the Axum server on `127.0.0.1:8080`, and `src/app.rs` assembles routes, tracing, and shared state. Keep HTTP concerns in `src/api/`, business logic in `src/service/`, persistence in `src/db/` and `src/entity/`, reusable helpers in `src/utils/`, and typed failures in `src/errors/`. Runtime configuration lives in `src/config/`. Local API collections are stored in `bruno_endpoints/smart_audit/`. Infrastructure files such as [`docker-compose.yml`](/Users/surojit/Documents/rust/smart_audit/docker-compose.yml) and [`garage.toml`](/Users/surojit/Documents/rust/smart_audit/garage.toml) support local dependencies.

## Build, Test, and Development Commands
Use `cargo check` for a fast compile pass during development. Use `cargo run` to start the service locally; it expects environment variables such as `DATABASE_URL`, `LOGIN_SECRET_PEPPER`, `JWT_ACCESS_SECRET`, and `JWT_REFRESH_SECRET`. Use `cargo test` for unit and integration tests once added. Run `cargo fmt` before committing to keep formatting consistent. Start local services with `docker compose up -d` and inspect them with `docker compose logs -f`. The README also documents Garage bucket setup if object storage work is involved.

## Coding Style & Naming Conventions
Follow standard Rust formatting with `cargo fmt`; use 4-space indentation and keep modules focused by concern. Prefer `snake_case` for files, modules, functions, and fields, and `PascalCase` for structs and enums. Match the existing naming pattern for service and error files, for example `organization_service.rs` and `organization_service_errors.rs`. Keep route handlers thin and push validation, auth, and database orchestration into `service/`. Keep variable name meaning full and short.

## Testing Guidelines
There are currently no committed Rust tests under `src/`, so new work should add them alongside the feature. Prefer small unit tests near helper logic and integration-style tests for route behavior. Name tests for the behavior they verify, such as `creates_organization_with_primary_user`. Before opening a PR, run at least `cargo test` and manually exercise affected Bruno requests.

## Commit & Pull Request Guidelines
Recent commits use short, imperative summaries such as `create staff at time of organization creation.` Keep that style, but tighten grammar and scope when possible. Each PR should include a clear description, note any required env or schema changes, link the related issue if one exists, and include request/response examples or screenshots when API behavior changes.

## Database migration guidelines
Database migration are done automatically  by sea orm entity, no script need.

## Entity structure
If a entity span with multiple related entity, create separate folder for those.