use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use dosh_domain::{
    model::account::Account,
    use_case::create_account::{CreateAccountUseCase, CreateAccountUseCaseError},
};

use crate::adapter::http::dto::{
    account::AccountJson,
    create_account_request::{CreateAccountRequest, CreateAccountRequestError},
    error::ErrorJson,
};

/// `POST /accounts`
///
/// The extractor is taken as a [`Result`] so a malformed body becomes a
/// [`CreateAccountApiError`] like every other failure, rather than axum's plain
/// text.
pub async fn create_account(
    State(use_case): State<Arc<CreateAccountUseCase>>,
    request: Result<Json<CreateAccountRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<AccountJson>), CreateAccountApiError> {
    let Json(request) = request?;

    let account = Account::try_from(request)?;

    use_case.execute(&account).await?;

    Ok((StatusCode::CREATED, Json(AccountJson::from(&account))))
}

#[derive(Debug, thiserror::Error)]
pub enum CreateAccountApiError {
    #[error(transparent)]
    Rejection(#[from] JsonRejection),
    #[error(transparent)]
    Request(#[from] CreateAccountRequestError),
    #[error(transparent)]
    UseCase(#[from] CreateAccountUseCaseError),
}

impl IntoResponse for CreateAccountApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::Rejection(rejection) => (rejection.status(), rejection.body_text()),
            Self::Request(error) => (StatusCode::UNPROCESSABLE_ENTITY, error.to_string()),
            Self::UseCase(error) => match error {
                CreateAccountUseCaseError::AlreadyExists(_) => {
                    (StatusCode::CONFLICT, error.to_string())
                }
                // The client can do nothing about this, so the cause is hidden.
                CreateAccountUseCaseError::Repository => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_string(),
                ),
            },
        };

        (status, Json(ErrorJson::new(message))).into_response()
    }
}

#[cfg(test)]
mod test {
    use dosh_domain::model::account_code::AccountCode;

    use super::*;

    async fn answer_to(error: CreateAccountApiError) -> (StatusCode, ErrorJson) {
        let response = error.into_response();
        let status = response.status();

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("failed to read the response body");

        (
            status,
            serde_json::from_slice(&body).expect("body should be an error"),
        )
    }

    #[tokio::test]
    async fn maps_an_already_existing_account_to_conflict() {
        let (status, body) = answer_to(CreateAccountApiError::UseCase(
            CreateAccountUseCaseError::AlreadyExists(AccountCode::parse("200").unwrap()),
        ))
        .await;

        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(body, ErrorJson::new("account with code 200 already exists"));
    }

    #[tokio::test]
    async fn hides_the_cause_of_a_repository_failure() {
        let (status, body) = answer_to(CreateAccountApiError::UseCase(
            CreateAccountUseCaseError::Repository,
        ))
        .await;

        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body, ErrorJson::new("internal server error"));
    }

    #[tokio::test]
    async fn maps_an_invalid_request_to_unprocessable_entity() {
        let (status, body) = answer_to(CreateAccountApiError::Request(
            CreateAccountRequestError::Code(AccountCode::parse("0123").unwrap_err()),
        ))
        .await;

        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            body,
            ErrorJson::new(
                "expected code to be string of digits starting with non-zero digit, got: 0123"
            )
        );
    }
}
