use api::adapter::postgres::{account_repository::PostgresAccountRepository, db};

const DATABASE_URL: &str = "postgres://dosh:dosh@localhost:5432/dosh";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = db::connect(DATABASE_URL).await?;
    db::migrate(&pool).await?;

    let _account_repository = PostgresAccountRepository::new(pool);

    println!("Aup, duck!");

    Ok(())
}
