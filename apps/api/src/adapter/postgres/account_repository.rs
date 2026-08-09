use std::pin::Pin;

use dosh_domain::{
    model::{account::Account, account_code::AccountCode, account_filter::AccountFilter},
    port::account_repository::{AccountRepository, CreateAccountError, ListAccountsError},
};
use sqlx::PgPool;

use crate::adapter::postgres::dto::{
    account::AccountPgRecord, account_class::account_class_value,
};

pub struct PostgresAccountRepository {
    pool: PgPool,
}

impl PostgresAccountRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl AccountRepository for PostgresAccountRepository {
    fn create<'a>(
        &'a self,
        account: &'a Account,
    ) -> Pin<Box<dyn Future<Output = Result<(), CreateAccountError>> + Send + 'a>> {
        Box::pin(async move {
            let record = AccountPgRecord::from(account);

            let result = sqlx::query!(
                "INSERT INTO accounts (code, class, description) VALUES ($1, $2, $3)",
                record.code,
                record.class,
                record.description,
            )
            .execute(&self.pool)
            .await;

            match result {
                Ok(_) => Ok(()),
                Err(sqlx::Error::Database(err)) if err.is_unique_violation() => {
                    Err(CreateAccountError::AlreadyExists(account.code().clone()))
                }
                Err(_) => Err(CreateAccountError::Internal),
            }
        })
    }

    fn list<'a>(
        &'a self,
        filter: &'a AccountFilter,
        after: Option<&'a AccountCode>,
        limit: u32,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Account>, ListAccountsError>> + Send + 'a>> {
        Box::pin(async move {
            let class = filter.class().map(account_class_value);
            let code_prefix = filter.code_starts_with().map(ToString::to_string);
            let after = after.map(ToString::to_string);

            // Each filter is one nullable parameter that stands aside when it
            // is null, so every combination of them is the same statement.
            // `starts_with` needs no escaping: a prefix is digits only.
            let records = sqlx::query_as!(
                AccountPgRecord,
                r#"
                SELECT code AS "code!", class AS "class!", description
                FROM accounts
                WHERE ($1::text IS NULL OR class = $1)
                  AND ($2::text IS NULL OR starts_with(code, $2))
                  AND ($3::text IS NULL OR code > $3)
                ORDER BY code
                LIMIT $4
                "#,
                class,
                code_prefix,
                after,
                i64::from(limit),
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|_| ListAccountsError::Internal)?;

            records
                .into_iter()
                .map(|record| Account::try_from(record).map_err(|_| ListAccountsError::Internal))
                .collect()
        })
    }
}
