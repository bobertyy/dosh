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

## Comments

- Code is expected to document itself. A name, a signature, or a type that needs a comment to be understood is a name, signature, or type to change first.
- Doc comments are welcome on the things a reader meets from outside — a type, a public function, an endpoint — and say what it is in a line or two. They are not the place to justify the design, walk through the body, or restate what the signature already says.
- Ordinary `//` comments are a last resort, for the rare fact the code cannot carry: a constraint imposed from elsewhere, a deliberate choice whose alternative looks equally correct. Everything else should be deleted rather than written.
- Never narrate. A comment that paraphrases the line below it, labels a branch of a match, or explains why an error maps to the status it obviously maps to is noise.
- Tests are code too: a test name states the behaviour, so a comment above it is redundant, and a helper needs a doc comment only when what it sets up is not visible in its body.

## Adapters

- Adapters live in `apps/api/src/adapter/<technology>/`. `api` is a lib *and* a bin — `lib.rs` exists so integration tests can import adapters.
- Domain types never derive persistence or transport traits. Each adapter defines its own DTOs (`dto/`) and converts with `From<&DomainType>`.
- Stored representations are pinned to the schema: the strings an adapter writes must match the CHECK constraint that admits them. Change both together.
- A domain enum earns a representation type in an adapter only when something holds one — a field of a wire DTO, with a serialization framework deriving on it. Where a value is converted and immediately consumed, as it is on the way to and from a SQL parameter, a pair of functions over the domain type does the job: one mapping a domain value to its stored string, one parsing a string back into a domain value or failing. Don't mirror a domain enum into an adapter enum that nothing stores.
- Adapters translate infrastructure errors into the port's error type — a unique violation becomes the port's already-exists variant, everything else its internal one. No `sqlx::Error` escapes the adapter.
- Ports return `Pin<Box<dyn Future<Output = ...> + Send + 'a>>` by hand rather than using `async_trait`, which the domain cannot depend on.

## HTTP

- The axum adapter is `apps/api/src/adapter/http/`: `dto/` (wire types), `handler/` (one module per endpoint), `router.rs` (routes and state), `server.rs` (bind and serve).
- Use cases are the router's state, held as `Arc<UseCase>`. One state struct in `router.rs` collects them and hand-written `FromRef` impls hand each one out, so a handler still takes `State<Arc<TheOneUseCaseItNeeds>>` and sees nothing of the others. A new endpoint adds a field, a `FromRef`, and nothing to the existing handlers.
- **There is no shared error type.** Each endpoint declares its own error enum beside its handler, with a `#[from]` variant per failure it can hit, so the body of the handler is `?` throughout. Endpoints are free to answer with different shapes; nothing forces them into a common one.
- That enum implements `IntoResponse` directly, matching itself onto a status and a body. Every status code an endpoint can return is readable in one match, in the endpoint's own file. Adding an endpoint touches only its own module.
- Input that deserialised but the domain rejected is `422`; anything internal is `500` with the cause hidden. Beyond that an endpoint picks whatever status its own failures deserve.
- Extractor rejections keep axum's own status rather than being remapped, which is why handlers take `Result<Json<T>, JsonRejection>` / `Result<Query<T>, QueryRejection>` instead of the bare extractor.
- `dto/` holds the `{"error": "..."}` body as a type of its own. It is a wire type an endpoint may choose, not a rule every endpoint obeys.
- A request DTO becomes a domain type through `TryFrom`, so the domain does the validating and the adapter only reports the outcome.
- Collections are paged with a keyset cursor, never an offset. The use case reads one row more than the page holds to learn whether a next page exists; the extra row never leaves the domain. Clients are told to treat the cursor as opaque.
- What a cursor means is the paged item's business, settled by one domain trait: an item designates the key its collection is ordered by and turns a cursor back into one, and a blanket impl gives every implementor its item-to-cursor conversion — a new paged collection implements the trait and writes no conversions. A cursor that names no key is the use case's to reject, so ports take the key, never the cursor.
- `dto/` holds a pagination type built from any page. A list endpoint's own DTO carries its items and defers the paging to it.
- Wire names are the adapter's to choose and need not match the domain's — a query parameter may take the name its clients expect even where the domain has another word for the same thing. Operator-style filters are bracketed — `<field>[starts_with]` — so more operators can join them without new top-level parameters.

## Database

- Migrations: `apps/api/migrations/`, embedded with `sqlx::migrate!` and run at startup.
- Favour `sqlx::query_as!` over `sqlx::query!` where it fits — mapping straight into an adapter DTO beats hand-assembling one from an anonymous record. It is a preference, never a rule: if a query does not map cleanly onto a single struct, use whichever macro reads better rather than contorting the code to fit.
- **The build requires a live Postgres.** `sqlx::query!` checks SQL at compile time and there is no committed offline cache, so `cargo check`/`build`/`test` need `docker compose up -d postgres` and `DATABASE_URL` from `.env`.

## Testing

- Mapping and validation logic: unit tests in `#[cfg(test)] mod test` beside the code.
- **A use case is unit tested only for discrete logic** — a rule it applies on its own, with no port in the way. The moment a test needs a port it belongs in `apps/api/tests/`, driving the use case over a real adapter.
- **No stub, mock, or spy implements a port.** A stub can only be asked whether it was called the way the use case happens to call it today, so a test written against one asserts the implementation rather than the behaviour, and rewriting the use case without changing what a caller sees breaks it. Assert on what comes back — the page, the cursor, the error — and let a real adapter answer whether the interaction was right.
- Use case integration tests are one file per use case, named for it: `apps/api/tests/<use_case>_use_case.rs`. Shared setup lives in `apps/api/tests/common/mod.rs`.
- Anything touching SQL: integration test in `apps/api/tests/`, one fresh testcontainers Postgres per test, pinned to the same image tag as `docker-compose.yml`.
- Repository tests cover what the adapter does with a query; use case tests cover what a caller gets. Both hit Postgres, and the overlap is the point — neither is standing in for the other.
- `dosh-domain` has no dev-dependencies. Nothing in it is async-tested, so it needs no runtime, in tests or out.
- **The HTTP adapter gets no integration tests.** Nothing in `apps/api/tests/` drives the router. Wire types, request-to-domain conversions, and each endpoint's error-enum-to-response mapping are covered by unit tests beside the code — an error enum's `IntoResponse` is exercised directly, asserting on the status and the body it produces.
