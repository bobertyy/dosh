use std::sync::Arc;

use api::adapter::{
    http::{
        router::{self, AppState},
        server,
    },
    postgres::{
        account_repository::PostgresAccountRepository, db,
        journal_entry_repository::PostgresJournalEntryRepository,
    },
};
use dosh_domain::{
    port::{
        account_repository::AccountRepository, journal_entry_repository::JournalEntryRepository,
    },
    use_case::{
        create_account::CreateAccountUseCase, create_journal_entry::CreateJournalEntryUseCase,
        list_accounts::ListAccountsUseCase,
    },
};

const DATABASE_URL: &str = "postgres://dosh:dosh@localhost:5432/dosh";
const BIND_ADDRESS: &str = "0.0.0.0:3000";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = db::connect(DATABASE_URL).await?;
    db::migrate(&pool).await?;

    let account_repository: Arc<dyn AccountRepository> =
        Arc::new(PostgresAccountRepository::new(pool.clone()));
    let journal_entry_repository: Arc<dyn JournalEntryRepository> =
        Arc::new(PostgresJournalEntryRepository::new(pool));

    let state = AppState::new(
        Arc::new(CreateAccountUseCase::new(account_repository.clone())),
        Arc::new(ListAccountsUseCase::new(account_repository.clone())),
        Arc::new(CreateJournalEntryUseCase::new(
            account_repository,
            journal_entry_repository,
        )),
    );

    let listener = server::bind(BIND_ADDRESS).await?;
    println!("Aup, duck! Listening on {}", listener.local_addr()?);

    server::serve(listener, router::router(state)).await?;

    Ok(())
}
