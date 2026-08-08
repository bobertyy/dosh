# dosh

Rust workspace following hexagonal architecture (ports and adapters).

## Layout

- `crates/` — supporting libraries.
  - `dosh-domain` — business logic only: `model/` (entities, value objects), `port/` (traits the outside world implements), `use_case/` (application logic).
- `apps/` — runnable binaries.
  - `api` — wires adapters into the domain.

## Rules

- `dosh-domain` has **no** infrastructure dependencies. No `sqlx`, no HTTP clients, no runtimes, no serialization framework tied to a transport. If a dependency knows about a database, network, or framework, it does not belong here.
- Dependencies point inward: adapters and apps depend on `dosh-domain`; `dosh-domain` depends on nothing outside itself.
- Ports are defined in the domain as traits. Adapters that implement them live outside the domain crate.

## Adapters

- Adapters live in `apps/api/src/adapter/<technology>/`. `api` is a lib *and* a bin — `lib.rs` exists so integration tests can import adapters.
- Domain types never derive persistence or transport traits. Each adapter defines its own DTOs (`dto/`) and converts with `From<&DomainType>`.
- Stored representations are pinned to the schema: `AccountClassPgValue`'s strings must match the `class` CHECK constraint in the migration. Change both together.
- Adapters translate infrastructure errors into the port's error type (unique violation → `AlreadyExists`, everything else → `Internal`). No `sqlx::Error` escapes the adapter.
- Ports return `Pin<Box<dyn Future<Output = ...> + Send + 'a>>` by hand rather than using `async_trait`, which the domain cannot depend on.

## HTTP

- The axum adapter is `apps/api/src/adapter/http/`: `dto/` (wire types), `handler/` (one module per endpoint), `router.rs` (routes and state), `server.rs` (bind and serve).
- Use cases are the router's state, held as `Arc<UseCase>`. Handlers take `State<Arc<...>>` and nothing else from the app.
- **There is no shared error type.** Each endpoint declares its own error enum beside its handler — `CreateAccountApiError` in `handler/create_account.rs` — with a `#[from]` variant per failure it can hit, so the body of the handler is `?` throughout. Endpoints are free to answer with different shapes; nothing forces them into a common one.
- That enum implements `IntoResponse` directly, matching itself onto a status and a body. Every status code an endpoint can return is readable in one match, in the endpoint's own file. Adding an endpoint touches only its own module.
- Status codes for `POST /accounts`: `422` for a body that deserialised but the domain rejected, `409` for `AlreadyExists`, `500` with the cause hidden for anything internal. Extractor rejections keep axum's own status (`400` malformed JSON, `415` wrong content type), which is why handlers take `Result<Json<T>, JsonRejection>` instead of `Json<T>`.
- `dto/error.rs` holds `ErrorJson`, the `{"error": "..."}` body. It is a wire type an endpoint may choose, not a rule every endpoint obeys.
- A request DTO becomes a domain type through `TryFrom`, so the domain does the validating and the adapter only reports the outcome.

## Database

- Migrations: `apps/api/migrations/`, embedded with `sqlx::migrate!` and run at startup.
- **The build requires a live Postgres.** `sqlx::query!` checks SQL at compile time and there is no committed offline cache, so `cargo check`/`build`/`test` need `docker compose up -d postgres` and `DATABASE_URL` from `.env`.

## Testing

- Mapping and validation logic: unit tests in `#[cfg(test)] mod test` beside the code.
- Anything touching SQL: integration test in `apps/api/tests/`, one fresh testcontainers Postgres per test, pinned to the same image tag as `docker-compose.yml`.
- Endpoints: integration test in `apps/api/tests/`, driving the real router with `tower`'s `oneshot` and a stub port implementation. No database and no socket — these cover routing, extraction, status codes, and error bodies only.
