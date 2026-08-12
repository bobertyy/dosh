use std::pin::Pin;

use dosh_domain::{
    model::journal_entry::JournalEntry,
    port::journal_entry_repository::{CreateJournalEntryError, JournalEntryRepository},
};
use sqlx::{PgPool, Postgres, Transaction};

use crate::adapter::postgres::dto::{
    journal_entry::JournalEntryPgRecord, journal_line_item::JournalLineItemPgRecord,
};

pub struct PostgresJournalEntryRepository {
    pool: PgPool,
}

impl PostgresJournalEntryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl JournalEntryRepository for PostgresJournalEntryRepository {
    /// The entry and its line items land in one transaction.
    fn create<'a>(
        &'a self,
        entry: &'a JournalEntry,
    ) -> Pin<Box<dyn Future<Output = Result<(), CreateJournalEntryError>> + Send + 'a>> {
        Box::pin(async move {
            let mut transaction = self
                .pool
                .begin()
                .await
                .map_err(|_| CreateJournalEntryError::Internal)?;

            insert_entry(&mut transaction, entry)
                .await
                .map_err(|err| classify(err, entry))?;

            for item in entry.line_items() {
                let record = JournalLineItemPgRecord::from((entry.id(), item));

                insert_line_item(&mut transaction, &record)
                    .await
                    .map_err(|err| classify(err, entry))?;
            }

            transaction
                .commit()
                .await
                .map_err(|_| CreateJournalEntryError::Internal)
        })
    }
}

async fn insert_entry(
    transaction: &mut Transaction<'_, Postgres>,
    entry: &JournalEntry,
) -> Result<(), sqlx::Error> {
    let record = JournalEntryPgRecord::from(entry);

    sqlx::query!(
        "INSERT INTO journal_entries (id, date, description) VALUES ($1, $2, $3)",
        record.id,
        record.date,
        record.description,
    )
    .execute(&mut **transaction)
    .await
    .map(|_| ())
}

async fn insert_line_item(
    transaction: &mut Transaction<'_, Postgres>,
    record: &JournalLineItemPgRecord,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO journal_line_items
            (id, journal_entry_id, account_code, amount, entry_type, description)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        record.id,
        record.journal_entry_id,
        record.account_code,
        record.amount,
        record.entry_type,
        record.description,
    )
    .execute(&mut **transaction)
    .await
    .map(|_| ())
}

/// A foreign key violation is internal: the use case has already reported any
/// account we do not keep, so reaching one here means the accounts moved.
fn classify(error: sqlx::Error, entry: &JournalEntry) -> CreateJournalEntryError {
    match error {
        sqlx::Error::Database(err) if err.is_unique_violation() => {
            CreateJournalEntryError::AlreadyExists(*entry.id())
        }
        _ => CreateJournalEntryError::Internal,
    }
}
