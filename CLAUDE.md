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

## Database

- Migrations: `apps/api/migrations/`, embedded with `sqlx::migrate!` and run at startup.
- **The build requires a live Postgres.** `sqlx::query!` checks SQL at compile time and there is no committed offline cache, so `cargo check`/`build`/`test` need `docker compose up -d postgres` and `DATABASE_URL` from `.env`.

## Testing

- Mapping and validation logic: unit tests in `#[cfg(test)] mod test` beside the code.
- Anything touching SQL: integration test in `apps/api/tests/`, one fresh testcontainers Postgres per test, pinned to the same image tag as `docker-compose.yml`.
