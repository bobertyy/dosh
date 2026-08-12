mod common;

use std::assert_matches;

use api::adapter::postgres::account_repository::PostgresAccountRepository;
use common::{TestDb, migrated_database};
use dosh_domain::{
    model::{
        account::{Account, AccountClass},
        account_code::{AccountCode, AccountCodePrefix},
        account_filter::AccountFilter,
    },
    port::account_repository::{AccountRepository, CreateAccountError},
};
use sqlx::PgPool;

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
    use api::adapter::postgres::db;

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

mod look_up_codes {
    use super::*;

    async fn repository_holding(codes: &[&str]) -> (TestDb, PostgresAccountRepository) {
        let cases: Vec<(&str, AccountClass)> = codes
            .iter()
            .map(|code| (*code, AccountClass::Asset))
            .collect();

        seeded_repository(&cases).await
    }

    fn parse(codes: &[&str]) -> Vec<AccountCode> {
        codes
            .iter()
            .map(|code| AccountCode::parse(*code).unwrap())
            .collect()
    }

    #[tokio::test]
    async fn finds_the_codes_it_holds() {
        let (_db, repository) = repository_holding(&["100", "200"]).await;

        let lookup = repository
            .look_up_codes(&parse(&["200", "100"]))
            .await
            .unwrap();

        assert_eq!(lookup.found(), parse(&["100", "200"]));
        assert!(lookup.missing().is_empty());
    }

    #[tokio::test]
    async fn misses_the_codes_it_does_not_hold() {
        let (_db, repository) = repository_holding(&["100"]).await;

        let lookup = repository
            .look_up_codes(&parse(&["100", "999"]))
            .await
            .unwrap();

        assert_eq!(lookup.found(), parse(&["100"]));
        assert_eq!(lookup.missing(), parse(&["999"]));
    }

    #[tokio::test]
    async fn misses_every_code_when_it_holds_none_of_them() {
        let (_db, repository) = repository_holding(&["100"]).await;

        let lookup = repository
            .look_up_codes(&parse(&["999", "998"]))
            .await
            .unwrap();

        assert!(lookup.found().is_empty());
        assert_eq!(lookup.missing(), parse(&["998", "999"]));
    }

    #[tokio::test]
    async fn answers_for_no_codes_at_all() {
        let (_db, repository) = repository_holding(&["100"]).await;

        let lookup = repository.look_up_codes(&[]).await.unwrap();

        assert!(lookup.found().is_empty());
        assert!(lookup.missing().is_empty());
    }

    #[tokio::test]
    async fn names_a_code_once_however_often_it_is_asked_after() {
        let (_db, repository) = repository_holding(&["100"]).await;

        let lookup = repository
            .look_up_codes(&parse(&["100", "100", "999", "999"]))
            .await
            .unwrap();

        assert_eq!(lookup.found(), parse(&["100"]));
        assert_eq!(lookup.missing(), parse(&["999"]));
    }
}

async fn seeded_repository(cases: &[(&str, AccountClass)]) -> (TestDb, PostgresAccountRepository) {
    let db = migrated_database().await;
    let repository = PostgresAccountRepository::new(db.pool.clone());

    for (code, class) in cases {
        let account = Account::new(AccountCode::parse(*code).unwrap(), *class);
        repository.create(&account).await.unwrap();
    }

    (db, repository)
}

mod list {
    use super::*;

    fn codes(accounts: &[Account]) -> Vec<String> {
        accounts
            .iter()
            .map(|account| account.code().to_string())
            .collect()
    }

    #[tokio::test]
    async fn returns_accounts_ordered_by_code() {
        let (_db, repository) = seeded_repository(&[
            ("200", AccountClass::Revenue),
            ("100", AccountClass::Asset),
            ("1000", AccountClass::Asset),
            ("110", AccountClass::Asset),
        ])
        .await;

        let accounts = repository
            .list(&AccountFilter::default(), None, 10)
            .await
            .unwrap();

        // Codes are text, so they order as text: 1000 sits inside the 100s.
        assert_eq!(codes(&accounts), vec!["100", "1000", "110", "200"]);
    }

    #[tokio::test]
    async fn returns_the_account_as_it_was_stored() {
        let db = migrated_database().await;
        let repository = PostgresAccountRepository::new(db.pool.clone());

        let stored = Account::new_with_description(
            AccountCode::parse("200").unwrap(),
            AccountClass::Revenue,
            "Sales revenue".to_string(),
        )
        .unwrap();
        repository.create(&stored).await.unwrap();

        let accounts = repository
            .list(&AccountFilter::default(), None, 10)
            .await
            .unwrap();

        let account = accounts.first().unwrap();
        assert_eq!(account.code(), &AccountCode::parse("200").unwrap());
        assert_eq!(account.class(), &AccountClass::Revenue);
        assert_eq!(account.description(), &Some("Sales revenue".to_string()));
    }

    #[tokio::test]
    async fn returns_no_more_than_the_limit() {
        let (_db, repository) = seeded_repository(&[
            ("100", AccountClass::Asset),
            ("110", AccountClass::Asset),
            ("200", AccountClass::Revenue),
        ])
        .await;

        let accounts = repository
            .list(&AccountFilter::default(), None, 2)
            .await
            .unwrap();

        assert_eq!(codes(&accounts), vec!["100", "110"]);
    }

    #[tokio::test]
    async fn starts_after_the_given_code() {
        let (_db, repository) = seeded_repository(&[
            ("100", AccountClass::Asset),
            ("110", AccountClass::Asset),
            ("200", AccountClass::Revenue),
        ])
        .await;

        let after = AccountCode::parse("110").unwrap();

        let accounts = repository
            .list(&AccountFilter::default(), Some(&after), 10)
            .await
            .unwrap();

        assert_eq!(codes(&accounts), vec!["200"]);
    }

    #[tokio::test]
    async fn filters_by_class() {
        let (_db, repository) = seeded_repository(&[
            ("100", AccountClass::Asset),
            ("200", AccountClass::Revenue),
            ("300", AccountClass::Expense),
        ])
        .await;

        let filter = AccountFilter::new(Some(AccountClass::Revenue), None);

        let accounts = repository.list(&filter, None, 10).await.unwrap();

        assert_eq!(codes(&accounts), vec!["200"]);
    }

    #[tokio::test]
    async fn filters_by_code_prefix() {
        let (_db, repository) = seeded_repository(&[
            ("100", AccountClass::Asset),
            ("200", AccountClass::Revenue),
            ("210", AccountClass::Revenue),
            ("2", AccountClass::Revenue),
        ])
        .await;

        let filter = AccountFilter::new(None, Some(AccountCodePrefix::parse("21").unwrap()));

        let accounts = repository.list(&filter, None, 10).await.unwrap();

        assert_eq!(codes(&accounts), vec!["210"]);
    }

    #[tokio::test]
    async fn applies_every_filter_at_once() {
        let (_db, repository) = seeded_repository(&[
            ("200", AccountClass::Revenue),
            ("210", AccountClass::Revenue),
            ("220", AccountClass::Revenue),
            ("230", AccountClass::Asset),
            ("300", AccountClass::Revenue),
        ])
        .await;

        let filter = AccountFilter::new(
            Some(AccountClass::Revenue),
            Some(AccountCodePrefix::parse("2").unwrap()),
        );
        let after = AccountCode::parse("200").unwrap();

        let accounts = repository.list(&filter, Some(&after), 10).await.unwrap();

        assert_eq!(codes(&accounts), vec!["210", "220"]);
    }

    #[tokio::test]
    async fn returns_nothing_when_no_account_matches() {
        let (_db, repository) = seeded_repository(&[("100", AccountClass::Asset)]).await;

        let filter = AccountFilter::new(Some(AccountClass::Revenue), None);

        let accounts = repository.list(&filter, None, 10).await.unwrap();

        assert!(accounts.is_empty());
    }
}
