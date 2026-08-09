use api::adapter::postgres::db;
use sqlx::PgPool;
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{ContainerAsync, ImageExt, runners::AsyncRunner},
};

/// The tag `docker-compose.yml` runs. Tests meet the same Postgres as the app.
const POSTGRES_TAG: &str = "18-alpine";

pub struct TestDb {
    _container: ContainerAsync<Postgres>,
    pub pool: PgPool,
}

pub async fn start_database() -> TestDb {
    let container = Postgres::default()
        .with_tag(POSTGRES_TAG)
        .start()
        .await
        .expect("failed to start postgres container");

    let host = container.get_host().await.expect("failed to get host");
    let port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("failed to get port");

    let pool = db::connect(&format!(
        "postgres://postgres:postgres@{host}:{port}/postgres"
    ))
    .await
    .expect("failed to connect to postgres");

    TestDb {
        _container: container,
        pool,
    }
}

pub async fn migrated_database() -> TestDb {
    let db = start_database().await;
    db::migrate(&db.pool).await.expect("failed to migrate");
    db
}
