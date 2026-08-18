mod common;

use std::assert_matches;

use api::adapter::postgres::{
    account_repository::PostgresAccountRepository,
    journal_entry_repository::PostgresJournalEntryRepository,
};
use chrono::NaiveDate;
use common::{TestDb, migrated_database};
use dosh_domain::{
    model::{
        account::{Account, AccountClass, AssetClass, ExpenseClass},
        account_code::AccountCode,
        amount::Amount,
        journal_date::JournalDate,
        journal_entry::{JournalEntry, JournalEntryId},
        journal_line_item::{EntryType, JournalLineItem},
    },
    port::{
        account_repository::AccountRepository,
        journal_entry_repository::{CreateJournalEntryError, JournalEntryRepository},
    },
};
use sqlx::PgPool;
use uuid::Uuid;

struct StoredEntry {
    date: NaiveDate,
    description: String,
}

struct StoredLineItem {
    account_code: String,
    amount: i64,
    entry_type: String,
    description: Option<String>,
}

async fn fetch_entry(pool: &PgPool, id: &JournalEntryId) -> Option<StoredEntry> {
    sqlx::query_as!(
        StoredEntry,
        "SELECT date, description FROM journal_entries WHERE id = $1",
        id.uuid()
    )
    .fetch_optional(pool)
    .await
    .expect("failed to query journal entries")
}

async fn fetch_line_items(pool: &PgPool, id: &JournalEntryId) -> Vec<StoredLineItem> {
    sqlx::query_as!(
        StoredLineItem,
        r#"
        SELECT account_code, amount, entry_type, description
        FROM journal_line_items
        WHERE journal_entry_id = $1
        ORDER BY account_code
        "#,
        id.uuid()
    )
    .fetch_all(pool)
    .await
    .expect("failed to query journal line items")
}

async fn count_line_items(pool: &PgPool) -> i64 {
    sqlx::query_scalar!("SELECT COUNT(*) FROM journal_line_items")
        .fetch_one(pool)
        .await
        .expect("failed to count journal line items")
        .unwrap_or_default()
}

async fn ledger() -> (TestDb, PostgresJournalEntryRepository) {
    let db = migrated_database().await;
    let accounts = PostgresAccountRepository::new(db.pool.clone());

    for (code, class) in [
        ("100", AccountClass::Asset(AssetClass::Bank)),
        ("300", AccountClass::Expense(ExpenseClass::General)),
    ] {
        accounts
            .create(&Account::new(AccountCode::parse(code).unwrap(), class))
            .await
            .unwrap();
    }

    let repository = PostgresJournalEntryRepository::new(db.pool.clone());

    (db, repository)
}

fn line(code: &str, minor_units: i64, entry_type: EntryType) -> JournalLineItem {
    JournalLineItem::new(
        AccountCode::parse(code).unwrap(),
        Amount::parse(minor_units).unwrap(),
        entry_type,
    )
}

fn entry_of(line_items: Vec<JournalLineItem>) -> JournalEntry {
    JournalEntry::new(
        JournalDate::parse("2026-08-09").unwrap(),
        "Coffee for the office".to_string(),
        line_items,
    )
    .unwrap()
}

fn coffee() -> JournalEntry {
    entry_of(vec![
        line("100", 1250, EntryType::Credit),
        line("300", 1250, EntryType::Debit),
    ])
}

mod migrate {
    use super::*;
    use api::adapter::postgres::db;

    #[tokio::test]
    async fn creates_the_journal_tables() {
        let db = migrated_database().await;

        assert!(
            fetch_entry(&db.pool, &JournalEntryId::generate())
                .await
                .is_none()
        );
        assert_eq!(count_line_items(&db.pool).await, 0);
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
    async fn persists_the_entry() {
        let (db, repository) = ledger().await;
        let entry = coffee();

        repository.create(&entry).await.unwrap();

        let stored = fetch_entry(&db.pool, entry.id()).await.unwrap();
        assert_eq!(stored.date, NaiveDate::from_ymd_opt(2026, 8, 9).unwrap());
        assert_eq!(stored.description, "Coffee for the office");
    }

    #[tokio::test]
    async fn persists_every_line_item_under_the_entry() {
        let (db, repository) = ledger().await;
        let entry = coffee();

        repository.create(&entry).await.unwrap();

        let stored = fetch_line_items(&db.pool, entry.id()).await;
        assert_eq!(stored.len(), 2);

        assert_eq!(stored[0].account_code, "100");
        assert_eq!(stored[0].amount, 1250);
        assert_eq!(stored[0].entry_type, "credit");
        assert_eq!(stored[0].description, None);

        assert_eq!(stored[1].account_code, "300");
        assert_eq!(stored[1].amount, 1250);
        assert_eq!(stored[1].entry_type, "debit");
    }

    #[tokio::test]
    async fn persists_the_identity_each_line_item_was_given() {
        let (db, repository) = ledger().await;
        let entry = coffee();

        repository.create(&entry).await.unwrap();

        let stored: Vec<Uuid> = sqlx::query_scalar!(
            "SELECT id FROM journal_line_items WHERE journal_entry_id = $1 ORDER BY account_code",
            entry.id().uuid()
        )
        .fetch_all(&db.pool)
        .await
        .unwrap();

        let posted: Vec<Uuid> = entry
            .line_items()
            .iter()
            .map(|item| item.id().uuid())
            .collect();

        assert_eq!(stored, posted);
    }

    #[tokio::test]
    async fn persists_a_line_item_description() {
        let (db, repository) = ledger().await;

        let entry = entry_of(vec![
            JournalLineItem::new_with_description(
                AccountCode::parse("100").unwrap(),
                Amount::parse(1250).unwrap(),
                EntryType::Credit,
                "Out of petty cash".to_string(),
            )
            .unwrap(),
            line("300", 1250, EntryType::Debit),
        ]);

        repository.create(&entry).await.unwrap();

        let stored = fetch_line_items(&db.pool, entry.id()).await;
        assert_eq!(stored[0].description, Some("Out of petty cash".to_string()));
    }

    #[tokio::test]
    async fn refuses_an_entry_it_has_already_stored() {
        let (_db, repository) = ledger().await;
        let entry = coffee();

        repository.create(&entry).await.unwrap();

        let error = repository.create(&entry).await.unwrap_err();

        assert_matches!(
            error,
            CreateJournalEntryError::AlreadyExists(id) if id == *entry.id()
        );
    }

    #[tokio::test]
    async fn leaves_nothing_behind_when_a_line_item_is_refused() {
        let (db, repository) = ledger().await;

        let entry = entry_of(vec![
            line("100", 1250, EntryType::Credit),
            line("999", 1250, EntryType::Debit),
        ]);

        let error = repository.create(&entry).await.unwrap_err();

        assert_matches!(error, CreateJournalEntryError::Internal);
        assert!(fetch_entry(&db.pool, entry.id()).await.is_none());
        assert_eq!(count_line_items(&db.pool).await, 0);
    }

    #[tokio::test]
    async fn stores_entries_side_by_side() {
        let (db, repository) = ledger().await;

        let one = coffee();
        let other = entry_of(vec![
            line("100", 500, EntryType::Credit),
            line("300", 500, EntryType::Debit),
        ]);

        repository.create(&one).await.unwrap();
        repository.create(&other).await.unwrap();

        assert!(fetch_entry(&db.pool, one.id()).await.is_some());
        assert!(fetch_entry(&db.pool, other.id()).await.is_some());
        assert_eq!(count_line_items(&db.pool).await, 4);
    }
}
