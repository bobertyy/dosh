mod common;

use std::{assert_matches, sync::Arc};

use api::adapter::postgres::account_repository::PostgresAccountRepository;
use common::migrated_database;
use dosh_domain::{
    model::{
        account::{Account, AccountClass},
        account_code::{AccountCode, AccountCodePrefix},
        account_filter::AccountFilter,
        page::Page,
        page_cursor::PageCursor,
        page_limit::PageLimit,
    },
    port::account_repository::AccountRepository,
    use_case::list_accounts::{ListAccountsQuery, ListAccountsUseCase, ListAccountsUseCaseError},
};

use crate::common::TestDb;

async fn seeded_use_case(cases: &[(&str, AccountClass)]) -> (TestDb, ListAccountsUseCase) {
    let db = migrated_database().await;
    let repository = PostgresAccountRepository::new(db.pool.clone());

    for (code, class) in cases {
        let account = Account::new(AccountCode::parse(*code).unwrap(), *class);
        repository.create(&account).await.unwrap();
    }

    (db, ListAccountsUseCase::new(Arc::new(repository)))
}

fn asset(code: &str) -> (&str, AccountClass) {
    (code, AccountClass::Asset)
}

fn first_page_of(limit: u32) -> ListAccountsQuery {
    ListAccountsQuery::new(
        AccountFilter::default(),
        None,
        PageLimit::parse(limit).unwrap(),
    )
}

fn codes(page: &Page<Account>) -> Vec<String> {
    page.items()
        .iter()
        .map(|account| account.code().to_string())
        .collect()
}

#[tokio::test]
async fn returns_every_account_when_they_fit_on_one_page() {
    let (_db, use_case) = seeded_use_case(&[asset("100"), asset("110"), asset("200")]).await;

    let page = use_case.execute(&first_page_of(20)).await.unwrap();

    assert_eq!(codes(&page), vec!["100", "110", "200"]);
    assert!(!page.has_more());
    assert_eq!(page.next_cursor(), None);
}

#[tokio::test]
async fn returns_no_more_accounts_than_the_page_holds() {
    let (_db, use_case) = seeded_use_case(&[asset("100"), asset("110"), asset("200")]).await;

    let page = use_case.execute(&first_page_of(2)).await.unwrap();

    assert_eq!(codes(&page), vec!["100", "110"]);
    assert!(page.has_more());
}

#[tokio::test]
async fn a_page_that_exactly_empties_the_collection_has_no_more() {
    let (_db, use_case) = seeded_use_case(&[asset("100"), asset("110"), asset("200")]).await;

    let page = use_case.execute(&first_page_of(3)).await.unwrap();

    assert_eq!(codes(&page), vec!["100", "110", "200"]);
    assert!(!page.has_more());
    assert_eq!(page.next_cursor(), None);
}

#[tokio::test]
async fn the_cursor_leads_to_the_accounts_that_did_not_fit() {
    let (_db, use_case) = seeded_use_case(&[asset("100"), asset("110"), asset("200")]).await;

    let first = use_case.execute(&first_page_of(2)).await.unwrap();

    let next = use_case
        .execute(&ListAccountsQuery::new(
            AccountFilter::default(),
            first.next_cursor().cloned(),
            PageLimit::parse(2).unwrap(),
        ))
        .await
        .unwrap();

    assert_eq!(codes(&next), vec!["200"]);
    assert!(!next.has_more());
}

#[tokio::test]
async fn walking_the_cursor_sees_every_account_exactly_once() {
    let seeded = [
        asset("100"),
        asset("110"),
        asset("120"),
        asset("200"),
        asset("300"),
    ];
    let (_db, use_case) = seeded_use_case(&seeded).await;

    let mut seen: Vec<String> = Vec::new();
    let mut cursor: Option<PageCursor> = None;

    loop {
        let page = use_case
            .execute(&ListAccountsQuery::new(
                AccountFilter::default(),
                cursor,
                PageLimit::parse(2).unwrap(),
            ))
            .await
            .unwrap();

        seen.extend(codes(&page));

        match page.next_cursor() {
            Some(next) => cursor = Some(next.clone()),
            None => break,
        }
    }

    assert_eq!(seen, vec!["100", "110", "120", "200", "300"]);
}

#[tokio::test]
async fn filters_apply_to_every_page() {
    let (_db, use_case) = seeded_use_case(&[
        ("200", AccountClass::Revenue),
        ("210", AccountClass::Revenue),
        ("220", AccountClass::Revenue),
        ("230", AccountClass::Asset),
        ("300", AccountClass::Revenue),
    ])
    .await;

    let filter = || {
        AccountFilter::new(
            Some(AccountClass::Revenue),
            Some(AccountCodePrefix::parse("2").unwrap()),
        )
    };

    let first = use_case
        .execute(&ListAccountsQuery::new(
            filter(),
            None,
            PageLimit::parse(2).unwrap(),
        ))
        .await
        .unwrap();

    assert_eq!(codes(&first), vec!["200", "210"]);
    assert!(first.has_more());

    let next = use_case
        .execute(&ListAccountsQuery::new(
            filter(),
            first.next_cursor().cloned(),
            PageLimit::parse(2).unwrap(),
        ))
        .await
        .unwrap();

    // 230 is an asset and 300 is outside the prefix, so the revenue 2xx
    // accounts run out here rather than the collection doing so.
    assert_eq!(codes(&next), vec!["220"]);
    assert!(!next.has_more());
}

#[tokio::test]
async fn returns_an_empty_page_when_nothing_matches() {
    let (_db, use_case) = seeded_use_case(&[asset("100")]).await;

    let page = use_case
        .execute(&ListAccountsQuery::new(
            AccountFilter::new(Some(AccountClass::Revenue), None),
            None,
            PageLimit::default(),
        ))
        .await
        .unwrap();

    assert!(page.items().is_empty());
    assert!(!page.has_more());
    assert_eq!(page.next_cursor(), None);
}

#[tokio::test]
async fn reports_the_limit_the_page_was_read_under() {
    let (_db, use_case) = seeded_use_case(&[asset("100")]).await;

    let page = use_case.execute(&first_page_of(3)).await.unwrap();

    assert_eq!(page.limit(), PageLimit::parse(3).unwrap());
}

#[tokio::test]
async fn returns_the_accounts_as_they_were_stored() {
    let db = migrated_database().await;
    let repository = PostgresAccountRepository::new(db.pool.clone());

    let stored = Account::new_with_description(
        AccountCode::parse("200").unwrap(),
        AccountClass::Revenue,
        "Sales revenue".to_string(),
    )
    .unwrap();
    repository.create(&stored).await.unwrap();

    let use_case = ListAccountsUseCase::new(Arc::new(repository));

    let page = use_case.execute(&first_page_of(20)).await.unwrap();

    let account = page.items().first().unwrap();
    assert_eq!(account.code(), &AccountCode::parse("200").unwrap());
    assert_eq!(account.class(), &AccountClass::Revenue);
    assert_eq!(account.description(), &Some("Sales revenue".to_string()));
}

#[tokio::test]
async fn rejects_a_cursor_that_names_no_position() {
    let (_db, use_case) = seeded_use_case(&[asset("100")]).await;

    let error = use_case
        .execute(&ListAccountsQuery::new(
            AccountFilter::default(),
            Some(PageCursor::parse("not a position").unwrap()),
            PageLimit::default(),
        ))
        .await
        .unwrap_err();

    assert_matches!(error, ListAccountsUseCaseError::InvalidCursor);
}

#[tokio::test]
async fn a_cursor_past_the_end_of_the_collection_reads_an_empty_page() {
    let (_db, use_case) = seeded_use_case(&[asset("100")]).await;

    let page = use_case
        .execute(&ListAccountsQuery::new(
            AccountFilter::default(),
            Some(PageCursor::parse("900").unwrap()),
            PageLimit::default(),
        ))
        .await
        .unwrap();

    assert!(page.items().is_empty());
    assert!(!page.has_more());
}
