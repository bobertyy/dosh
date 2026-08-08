use std::sync::Arc;

use api::adapter::{
    http::{router, server},
    postgres::{account_repository::PostgresAccountRepository, db},
};
use dosh_domain::use_case::create_account::CreateAccountUseCase;

const DATABASE_URL: &str = "postgres://dosh:dosh@localhost:5432/dosh";
const BIND_ADDRESS: &str = "0.0.0.0:3000";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = db::connect(DATABASE_URL).await?;
    db::migrate(&pool).await?;

    let account_repository = Arc::new(PostgresAccountRepository::new(pool));
    let create_account_use_case = Arc::new(CreateAccountUseCase::new(account_repository));

    let listener = server::bind(BIND_ADDRESS).await?;
    println!("Aup, duck! Listening on {}", listener.local_addr()?);

    server::serve(listener, router::router(create_account_use_case)).await?;

    Ok(())
}
