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
