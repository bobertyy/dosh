use std::assert_matches;

use api::adapter::postgres::{account_repository::PostgresAccountRepository, db};
use dosh_domain::{
    model::{
        account::{Account, AccountClass},
        account_code::AccountCode,
    },
    port::account_repository::{AccountRepository, CreateAccountError},
};
use sqlx::PgPool;
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{ContainerAsync, ImageExt, runners::AsyncRunner},
};

const POSTGRES_TAG: &str = "18-alpine";

struct TestDb {
    _container: ContainerAsync<Postgres>,
    pool: PgPool,
}

async fn start_database() -> TestDb {
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

async fn migrated_database() -> TestDb {
    let db = start_database().await;
    db::migrate(&db.pool).await.expect("failed to migrate");
    db
}

struct StoredAccount {
    class: String,
    description: Option<String>,
}

async fn fetch_account(pool: &PgPool, code: &str) -> Option<StoredAccount> {
    sqlx::query_as!(
        StoredAccount,
        "SELECT class, description FROM accounts WHERE code = $1",
        code
    )
    .fetch_optional(pool)
    .await
    .expect("failed to query accounts")
}

mod migrate {
    use super::*;

    #[tokio::test]
    async fn creates_the_accounts_table() {
        let db = migrated_database().await;

        assert!(fetch_account(&db.pool, "100").await.is_none());
    }

    #[tokio::test]
    async fn is_idempotent_across_restarts() {
        let db = migrated_database().await;

        db::migrate(&db.pool)
            .await
            .expect("re-running migrations should succeed");
    }
}

mod create {
    use super::*;

    #[tokio::test]
    async fn persists_account_with_description() {
        let db = migrated_database().await;
        let repository = PostgresAccountRepository::new(db.pool.clone());

        let account = Account::new_with_description(
            AccountCode::parse("200").unwrap(),
            AccountClass::Revenue,
            "Sales revenue".to_string(),
        )
        .unwrap();

        repository.create(&account).await.unwrap();

        let stored = fetch_account(&db.pool, "200").await.unwrap();
        assert_eq!(stored.class, "revenue");
        assert_eq!(stored.description, Some("Sales revenue".to_string()));
    }

    #[tokio::test]
    async fn persists_account_without_description() {
        let db = migrated_database().await;
        let repository = PostgresAccountRepository::new(db.pool.clone());

        let account = Account::new(AccountCode::parse("200").unwrap(), AccountClass::Revenue);

        repository.create(&account).await.unwrap();

        let stored = fetch_account(&db.pool, "200").await.unwrap();
        assert_eq!(stored.class, "revenue");
        assert_eq!(stored.description, None);
    }

    #[tokio::test]
    async fn persists_every_account_class() {
        let db = migrated_database().await;
        let repository = PostgresAccountRepository::new(db.pool.clone());

        let cases = [
            ("100", AccountClass::Asset, "asset"),
            ("200", AccountClass::Equity, "equity"),
            ("300", AccountClass::Expense, "expense"),
            ("400", AccountClass::Liability, "liability"),
            ("500", AccountClass::Revenue, "revenue"),
        ];

        for (code, class, expected) in cases {
            let account = Account::new(AccountCode::parse(code).unwrap(), class);

            repository.create(&account).await.unwrap();

            let stored = fetch_account(&db.pool, code).await.unwrap();
            assert_eq!(stored.class, expected);
        }
    }

    #[tokio::test]
    async fn returns_already_exists_when_code_is_taken() {
        let db = migrated_database().await;
        let repository = PostgresAccountRepository::new(db.pool.clone());

        let existing = Account::new(AccountCode::parse("200").unwrap(), AccountClass::Revenue);
        repository.create(&existing).await.unwrap();

        let duplicate = Account::new_with_description(
            AccountCode::parse("200").unwrap(),
            AccountClass::Asset,
            "A different account, same code".to_string(),
        )
        .unwrap();

        let error = repository.create(&duplicate).await.unwrap_err();

        assert_matches!(
            error,
            CreateAccountError::AlreadyExists(code) if code == AccountCode::parse("200").unwrap()
        );

        let stored = fetch_account(&db.pool, "200").await.unwrap();
        assert_eq!(stored.class, "revenue");
        assert_eq!(stored.description, None);
    }
}
