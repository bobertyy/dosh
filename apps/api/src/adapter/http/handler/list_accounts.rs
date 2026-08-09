use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    extract::{Query, rejection::QueryRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use dosh_domain::use_case::list_accounts::{
    ListAccountsQuery, ListAccountsUseCase, ListAccountsUseCaseError,
};

use crate::adapter::http::dto::{
    accounts_page::AccountsPageJson,
    error::ErrorJson,
    list_accounts_request::{ListAccountsRequest, ListAccountsRequestError},
};

/// `GET /accounts`
///
/// The extractor is taken as a [`Result`] so a query string that will not
/// deserialise becomes a [`ListAccountsApiError`] like every other failure,
/// rather than axum's plain text.
pub async fn list_accounts(
    State(use_case): State<Arc<ListAccountsUseCase>>,
    request: Result<Query<ListAccountsRequest>, QueryRejection>,
) -> Result<Json<AccountsPageJson>, ListAccountsApiError> {
    let Query(request) = request?;

    let query = ListAccountsQuery::try_from(request)?;

    let page = use_case.execute(&query).await?;

    Ok(Json(AccountsPageJson::from(&page)))
}

#[derive(Debug, thiserror::Error)]
pub enum ListAccountsApiError {
    #[error(transparent)]
    Rejection(#[from] QueryRejection),
    #[error(transparent)]
    Request(#[from] ListAccountsRequestError),
    #[error(transparent)]
    UseCase(#[from] ListAccountsUseCaseError),
}

impl IntoResponse for ListAccountsApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::Rejection(rejection) => (rejection.status(), rejection.body_text()),
            Self::Request(error) => (StatusCode::UNPROCESSABLE_ENTITY, error.to_string()),
            Self::UseCase(error) => match error {
                // The client can do nothing about this, so the cause is hidden.
                ListAccountsUseCaseError::Repository => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_string(),
                ),
                ListAccountsUseCaseError::InvalidCursor => {
                    (StatusCode::UNPROCESSABLE_ENTITY, error.to_string())
                }
            },
        };

        (status, Json(ErrorJson::new(message))).into_response()
    }
}

#[cfg(test)]
mod test {
    use dosh_domain::model::page_limit::PageLimit;

    use super::*;

    async fn answer_to(error: ListAccountsApiError) -> (StatusCode, ErrorJson) {
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
    async fn hides_the_cause_of_a_repository_failure() {
        let (status, body) = answer_to(ListAccountsApiError::UseCase(
            ListAccountsUseCaseError::Repository,
        ))
        .await;

        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body, ErrorJson::new("internal server error"));
    }

    #[tokio::test]
    async fn maps_a_cursor_that_names_no_account_to_unprocessable_entity() {
        let (status, body) = answer_to(ListAccountsApiError::UseCase(
            ListAccountsUseCaseError::InvalidCursor,
        ))
        .await;

        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            body,
            ErrorJson::new("page cursor does not name a position in this collection")
        );
    }

    #[tokio::test]
    async fn maps_an_out_of_range_limit_to_unprocessable_entity() {
        let (status, body) = answer_to(ListAccountsApiError::Request(
            ListAccountsRequestError::Limit(PageLimit::parse(0).unwrap_err()),
        ))
        .await;

        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            body,
            ErrorJson::new("expected limit between 1 and 100, got: 0")
        );
    }
}
