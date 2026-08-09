mod common;

use std::{assert_matches, sync::Arc};

use api::adapter::postgres::account_repository::PostgresAccountRepository;
use common::migrated_database;
use dosh_domain::{
    model::{
        account::{Account, AccountClass},
        account_code::AccountCode,
        account_filter::AccountFilter,
        page_limit::PageLimit,
    },
    use_case::{
        create_account::{CreateAccountUseCase, CreateAccountUseCaseError},
        list_accounts::{ListAccountsQuery, ListAccountsUseCase},
    },
};

#[tokio::test]
async fn an_account_it_creates_can_be_listed_back() {
    let db = migrated_database().await;
    let repository = Arc::new(PostgresAccountRepository::new(db.pool.clone()));

    let account = Account::new_with_description(
        AccountCode::parse("200").unwrap(),
        AccountClass::Revenue,
        "Sales revenue".to_string(),
    )
    .unwrap();

    CreateAccountUseCase::new(repository.clone())
        .execute(&account)
        .await
        .unwrap();

    let page = ListAccountsUseCase::new(repository)
        .execute(&ListAccountsQuery::new(
            AccountFilter::default(),
            None,
            PageLimit::default(),
        ))
        .await
        .unwrap();

    let listed = page.items().first().unwrap();
    assert_eq!(listed.code(), &AccountCode::parse("200").unwrap());
    assert_eq!(listed.class(), &AccountClass::Revenue);
    assert_eq!(listed.description(), &Some("Sales revenue".to_string()));
}

#[tokio::test]
async fn refuses_a_code_that_is_already_taken() {
    let db = migrated_database().await;
    let use_case =
        CreateAccountUseCase::new(Arc::new(PostgresAccountRepository::new(db.pool.clone())));

    let existing = Account::new(AccountCode::parse("200").unwrap(), AccountClass::Revenue);
    use_case.execute(&existing).await.unwrap();

    let duplicate = Account::new(AccountCode::parse("200").unwrap(), AccountClass::Asset);

    let error = use_case.execute(&duplicate).await.unwrap_err();

    assert_matches!(
        error,
        CreateAccountUseCaseError::AlreadyExists(code) if code == AccountCode::parse("200").unwrap()
    );
}
