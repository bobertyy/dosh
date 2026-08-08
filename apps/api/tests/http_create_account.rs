use std::{
    pin::Pin,
    sync::{Arc, Mutex},
};

use api::adapter::http::{
    dto::{account::AccountJson, account_class::AccountClassJson, error::ErrorJson},
    router::router,
};
use axum::{
    body::Body,
    http::{StatusCode, header},
};
use dosh_domain::{
    model::account::Account,
    port::account_repository::{AccountRepository, CreateAccountError},
    use_case::create_account::CreateAccountUseCase,
};
use tower::ServiceExt;

/// What the stub repository does when a handler reaches it.
enum Outcome {
    Created,
    AlreadyExists,
    Internal,
}

/// Stands in for a real adapter so these tests only cover the HTTP layer.
struct StubAccountRepository {
    outcome: Outcome,
    created: Mutex<Vec<AccountJson>>,
}

impl StubAccountRepository {
    fn new(outcome: Outcome) -> Arc<Self> {
        Arc::new(Self {
            outcome,
            created: Mutex::new(Vec::new()),
        })
    }

    /// The accounts the handler asked to be persisted, as the wire sees them.
    fn created(&self) -> Vec<AccountJson> {
        self.created.lock().unwrap().clone()
    }
}

impl AccountRepository for StubAccountRepository {
    fn create<'a>(
        &'a self,
        account: &'a Account,
    ) -> Pin<Box<dyn Future<Output = Result<(), CreateAccountError>> + Send + 'a>> {
        Box::pin(async move {
            self.created
                .lock()
                .unwrap()
                .push(AccountJson::from(account));

            match self.outcome {
                Outcome::Created => Ok(()),
                Outcome::AlreadyExists => {
                    Err(CreateAccountError::AlreadyExists(account.code().clone()))
                }
                Outcome::Internal => Err(CreateAccountError::Internal),
            }
        })
    }
}

struct TestResponse {
    status: StatusCode,
    body: Vec<u8>,
}

impl TestResponse {
    fn account(&self) -> AccountJson {
        serde_json::from_slice(&self.body).expect("body should be an account")
    }

    fn error(&self) -> ErrorJson {
        serde_json::from_slice(&self.body).expect("body should be an error")
    }
}

async fn post_accounts(repository: Arc<StubAccountRepository>, body: &'static str) -> TestResponse {
    post_accounts_with_content_type(repository, body, Some("application/json")).await
}

async fn post_accounts_with_content_type(
    repository: Arc<StubAccountRepository>,
    body: &'static str,
    content_type: Option<&'static str>,
) -> TestResponse {
    let app = router(Arc::new(CreateAccountUseCase::new(repository)));

    let mut request = axum::http::Request::builder()
        .method("POST")
        .uri("/accounts");

    if let Some(content_type) = content_type {
        request = request.header(header::CONTENT_TYPE, content_type);
    }

    let response = app
        .oneshot(request.body(Body::from(body)).unwrap())
        .await
        .expect("the router is infallible");

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("failed to read the response body")
        .to_vec();

    TestResponse { status, body }
}

mod create_account {
    use super::*;

    #[tokio::test]
    async fn creates_an_account_with_a_description() {
        let repository = StubAccountRepository::new(Outcome::Created);

        let response = post_accounts(
            repository.clone(),
            r#"{"code":"200","class":"revenue","description":"Sales revenue"}"#,
        )
        .await;

        let expected = AccountJson {
            code: "200".to_string(),
            class: AccountClassJson::Revenue,
            description: Some("Sales revenue".to_string()),
        };

        assert_eq!(response.status, StatusCode::CREATED);
        assert_eq!(response.account(), expected);
        assert_eq!(repository.created(), vec![expected]);
    }

    #[tokio::test]
    async fn creates_an_account_without_a_description() {
        let repository = StubAccountRepository::new(Outcome::Created);

        let response = post_accounts(repository.clone(), r#"{"code":"100","class":"asset"}"#).await;

        let expected = AccountJson {
            code: "100".to_string(),
            class: AccountClassJson::Asset,
            description: None,
        };

        assert_eq!(response.status, StatusCode::CREATED);
        assert_eq!(response.account(), expected);
        assert_eq!(repository.created(), vec![expected]);
    }

    #[tokio::test]
    async fn returns_conflict_when_the_code_is_taken() {
        let repository = StubAccountRepository::new(Outcome::AlreadyExists);

        let response =
            post_accounts(repository.clone(), r#"{"code":"200","class":"revenue"}"#).await;

        assert_eq!(response.status, StatusCode::CONFLICT);
        assert_eq!(
            response.error(),
            ErrorJson::new("account with code 200 already exists")
        );
    }

    #[tokio::test]
    async fn hides_the_cause_of_a_repository_failure() {
        let repository = StubAccountRepository::new(Outcome::Internal);

        let response =
            post_accounts(repository.clone(), r#"{"code":"200","class":"revenue"}"#).await;

        assert_eq!(response.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(response.error(), ErrorJson::new("internal server error"));
    }

    #[tokio::test]
    async fn rejects_an_invalid_code_without_reaching_the_repository() {
        let repository = StubAccountRepository::new(Outcome::Created);

        let response =
            post_accounts(repository.clone(), r#"{"code":"0123","class":"asset"}"#).await;

        assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            response.error(),
            ErrorJson::new(
                "expected code to be string of digits starting with non-zero digit, got: 0123"
            )
        );
        assert!(repository.created().is_empty());
    }

    #[tokio::test]
    async fn rejects_an_empty_description_without_reaching_the_repository() {
        let repository = StubAccountRepository::new(Outcome::Created);

        let response = post_accounts(
            repository.clone(),
            r#"{"code":"100","class":"asset","description":""}"#,
        )
        .await;

        assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            response.error(),
            ErrorJson::new("description cannot be empty")
        );
        assert!(repository.created().is_empty());
    }

    #[tokio::test]
    async fn rejects_an_unknown_class_without_reaching_the_repository() {
        let repository = StubAccountRepository::new(Outcome::Created);

        let response = post_accounts(repository.clone(), r#"{"code":"100","class":"pizza"}"#).await;

        assert_eq!(response.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(repository.created().is_empty());
    }

    #[tokio::test]
    async fn rejects_a_body_that_is_not_json() {
        let repository = StubAccountRepository::new(Outcome::Created);

        let response = post_accounts(repository.clone(), "not json").await;

        assert_eq!(response.status, StatusCode::BAD_REQUEST);
        assert!(repository.created().is_empty());
    }

    #[tokio::test]
    async fn rejects_a_body_sent_without_a_json_content_type() {
        let repository = StubAccountRepository::new(Outcome::Created);

        let response = post_accounts_with_content_type(
            repository.clone(),
            r#"{"code":"100","class":"asset"}"#,
            None,
        )
        .await;

        assert_eq!(response.status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
        assert!(repository.created().is_empty());
    }

    #[tokio::test]
    async fn reports_every_rejection_as_a_json_error_body() {
        let repository = StubAccountRepository::new(Outcome::Created);

        let response = post_accounts(repository, "not json").await;

        assert!(!response.error().error.is_empty());
    }
}
