use std::sync::Arc;

use axum::{Router, routing::post};
use dosh_domain::use_case::create_account::CreateAccountUseCase;

use crate::adapter::http::handler::create_account;

/// Wires the use cases the API exposes onto their routes.
pub fn router(create_account_use_case: Arc<CreateAccountUseCase>) -> Router {
    Router::new()
        .route("/accounts", post(create_account::create_account))
        .with_state(create_account_use_case)
}
