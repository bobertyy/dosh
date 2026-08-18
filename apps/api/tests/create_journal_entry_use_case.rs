mod common;

use std::{assert_matches, sync::Arc};

use api::adapter::postgres::{
    account_repository::PostgresAccountRepository,
    journal_entry_repository::PostgresJournalEntryRepository,
};
use common::{TestDb, migrated_database};
use dosh_domain::{
    model::{
        account::{Account, AccountClass, AssetClass},
        account_code::AccountCode,
        amount::Amount,
        journal_date::JournalDate,
        journal_entry::{JournalEntry, JournalEntryId},
        journal_line_item::{EntryType, JournalLineItem},
    },
    port::account_repository::AccountRepository,
    use_case::create_journal_entry::{CreateJournalEntryUseCase, CreateJournalEntryUseCaseError},
};
use sqlx::PgPool;

async fn use_case_with_seeded_accounts(
    account_codes: &[&str],
) -> (TestDb, CreateJournalEntryUseCase) {
    let db = migrated_database().await;
    let accounts = PostgresAccountRepository::new(db.pool.clone());

    for code in account_codes {
        accounts
            .create(&Account::new(
                AccountCode::parse(*code).unwrap(),
                AccountClass::Asset(AssetClass::Bank),
            ))
            .await
            .unwrap();
    }

    let use_case = CreateJournalEntryUseCase::new(
        Arc::new(accounts),
        Arc::new(PostgresJournalEntryRepository::new(db.pool.clone())),
    );

    (db, use_case)
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

#[derive(Debug, PartialEq, Eq)]
struct StoredLineItem {
    account_code: String,
    amount: i64,
    entry_type: String,
}

fn stored(account_code: &str, amount: i64, entry_type: &str) -> StoredLineItem {
    StoredLineItem {
        account_code: account_code.to_string(),
        amount,
        entry_type: entry_type.to_string(),
    }
}

async fn line_items_under(pool: &PgPool, id: &JournalEntryId) -> Vec<StoredLineItem> {
    sqlx::query_as!(
        StoredLineItem,
        r#"
        SELECT account_code, amount, entry_type
        FROM journal_line_items
        WHERE journal_entry_id = $1
        ORDER BY account_code, amount
        "#,
        id.uuid()
    )
    .fetch_all(pool)
    .await
    .expect("failed to query journal line items")
}

async fn entry_count(pool: &PgPool) -> i64 {
    sqlx::query_scalar!("SELECT COUNT(*) FROM journal_entries")
        .fetch_one(pool)
        .await
        .expect("failed to count journal entries")
        .unwrap_or_default()
}

#[tokio::test]
async fn posts_an_entry_to_accounts_the_ledger_keeps() {
    let (db, use_case) = use_case_with_seeded_accounts(&["100", "300"]).await;

    let entry = entry_of(vec![
        line("100", 1250, EntryType::Credit),
        line("300", 1250, EntryType::Debit),
    ]);

    use_case.execute(&entry).await.unwrap();

    assert_eq!(
        line_items_under(&db.pool, entry.id()).await,
        vec![stored("100", 1250, "credit"), stored("300", 1250, "debit"),]
    );
}

#[tokio::test]
async fn posts_several_lines_to_the_same_account() {
    let (db, use_case) = use_case_with_seeded_accounts(&["100", "300"]).await;

    let entry = entry_of(vec![
        line("100", 1000, EntryType::Credit),
        line("100", 250, EntryType::Credit),
        line("300", 1250, EntryType::Debit),
    ]);

    use_case.execute(&entry).await.unwrap();

    assert_eq!(
        line_items_under(&db.pool, entry.id()).await,
        vec![
            stored("100", 250, "credit"),
            stored("100", 1000, "credit"),
            stored("300", 1250, "debit"),
        ]
    );
}

#[tokio::test]
async fn refuses_an_entry_posting_to_an_account_the_ledger_does_not_keep() {
    let (_db, use_case) = use_case_with_seeded_accounts(&["100"]).await;

    let entry = entry_of(vec![
        line("100", 1250, EntryType::Credit),
        line("300", 1250, EntryType::Debit),
    ]);

    let error = use_case.execute(&entry).await.unwrap_err();

    assert_matches!(
        error,
        CreateJournalEntryUseCaseError::UnknownAccountCodes(codes)
            if codes == vec![AccountCode::parse("300").unwrap()]
    );
}

#[tokio::test]
async fn names_every_account_the_ledger_does_not_keep() {
    let (_db, use_case) = use_case_with_seeded_accounts(&["100"]).await;

    let entry = entry_of(vec![
        line("998", 1000, EntryType::Credit),
        line("999", 250, EntryType::Credit),
        line("100", 1250, EntryType::Debit),
    ]);

    let error = use_case.execute(&entry).await.unwrap_err();

    assert_eq!(error.to_string(), "no account exists with code: 998, 999");
}

#[tokio::test]
async fn stores_nothing_when_it_refuses_an_entry() {
    let (db, use_case) = use_case_with_seeded_accounts(&["100"]).await;

    let entry = entry_of(vec![
        line("100", 1250, EntryType::Credit),
        line("300", 1250, EntryType::Debit),
    ]);

    use_case.execute(&entry).await.unwrap_err();

    assert_eq!(entry_count(&db.pool).await, 0);
    assert!(line_items_under(&db.pool, entry.id()).await.is_empty());
}

#[tokio::test]
async fn posts_entries_of_the_same_shape_side_by_side() {
    let (db, use_case) = use_case_with_seeded_accounts(&["100", "300"]).await;

    let one = entry_of(vec![
        line("100", 1250, EntryType::Credit),
        line("300", 1250, EntryType::Debit),
    ]);
    let other = entry_of(vec![
        line("100", 1250, EntryType::Credit),
        line("300", 1250, EntryType::Debit),
    ]);

    use_case.execute(&one).await.unwrap();
    use_case.execute(&other).await.unwrap();

    assert_ne!(one.id(), other.id());
    assert_eq!(entry_count(&db.pool).await, 2);
}
