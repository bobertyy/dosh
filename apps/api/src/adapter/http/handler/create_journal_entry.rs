use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use dosh_domain::{
    model::journal_entry::JournalEntry,
    use_case::create_journal_entry::{CreateJournalEntryUseCase, CreateJournalEntryUseCaseError},
};

use crate::adapter::http::dto::{
    create_journal_entry_request::{CreateJournalEntryRequest, CreateJournalEntryRequestError},
    error::ErrorJson,
    journal_entry::JournalEntryJson,
};

/// `POST /journal-entries`
pub async fn create_journal_entry(
    State(use_case): State<Arc<CreateJournalEntryUseCase>>,
    request: Result<Json<CreateJournalEntryRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<JournalEntryJson>), CreateJournalEntryApiError> {
    let Json(request) = request?;

    let entry = JournalEntry::try_from(request)?;

    use_case.execute(&entry).await?;

    Ok((StatusCode::CREATED, Json(JournalEntryJson::from(&entry))))
}

#[derive(Debug, thiserror::Error)]
pub enum CreateJournalEntryApiError {
    #[error(transparent)]
    Rejection(#[from] JsonRejection),
    #[error(transparent)]
    Request(#[from] CreateJournalEntryRequestError),
    #[error(transparent)]
    UseCase(#[from] CreateJournalEntryUseCaseError),
}

impl IntoResponse for CreateJournalEntryApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::Rejection(rejection) => (rejection.status(), rejection.body_text()),
            Self::Request(error) => (StatusCode::UNPROCESSABLE_ENTITY, error.to_string()),
            Self::UseCase(error) => match error {
                CreateJournalEntryUseCaseError::UnknownAccountCodes(_) => {
                    (StatusCode::UNPROCESSABLE_ENTITY, error.to_string())
                }
                CreateJournalEntryUseCaseError::Repository => (
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
    use dosh_domain::model::{
        account_code::AccountCode, journal_date::JournalDate,
        journal_entry::JournalEntryCreationError,
    };

    use super::*;

    async fn answer_to(error: CreateJournalEntryApiError) -> (StatusCode, ErrorJson) {
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
    async fn maps_unknown_account_codes_to_unprocessable_entity() {
        let (status, body) = answer_to(CreateJournalEntryApiError::UseCase(
            CreateJournalEntryUseCaseError::UnknownAccountCodes(vec![
                AccountCode::parse("998").unwrap(),
                AccountCode::parse("999").unwrap(),
            ]),
        ))
        .await;

        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            body,
            ErrorJson::new("no account exists with code: 998, 999")
        );
    }

    #[tokio::test]
    async fn hides_the_cause_of_a_repository_failure() {
        let (status, body) = answer_to(CreateJournalEntryApiError::UseCase(
            CreateJournalEntryUseCaseError::Repository,
        ))
        .await;

        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body, ErrorJson::new("internal server error"));
    }

    #[tokio::test]
    async fn maps_an_unbalanced_entry_to_unprocessable_entity() {
        let (status, body) = answer_to(CreateJournalEntryApiError::Request(
            CreateJournalEntryRequestError::Entry(JournalEntryCreationError::Unbalanced {
                credits: 1250,
                debits: 1000,
            }),
        ))
        .await;

        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            body,
            ErrorJson::new(
                "credits and debits must balance, got credits of 1250 against debits of 1000"
            )
        );
    }

    #[tokio::test]
    async fn maps_an_invalid_date_to_unprocessable_entity() {
        let (status, body) = answer_to(CreateJournalEntryApiError::Request(
            CreateJournalEntryRequestError::Date(JournalDate::parse("the ninth").unwrap_err()),
        ))
        .await;

        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            body,
            ErrorJson::new("expected date in YYYY-MM-DD form, got: the ninth")
        );
    }
}
