use std::pin::Pin;

use dosh_domain::{
    model::account::Account,
    port::account_repository::{AccountRepository, CreateAccountError},
};
use sqlx::PgPool;

use crate::adapter::postgres::dto::account::AccountPgRecord;

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
                record.class.value(),
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
}
