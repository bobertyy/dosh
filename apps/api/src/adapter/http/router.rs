use std::sync::Arc;

use axum::{Router, extract::FromRef, routing::post};
use dosh_domain::use_case::{
    create_account::CreateAccountUseCase, list_accounts::ListAccountsUseCase,
};

use crate::adapter::http::handler::{create_account, list_accounts};

#[derive(Clone)]
pub struct AppState {
    create_account: Arc<CreateAccountUseCase>,
    list_accounts: Arc<ListAccountsUseCase>,
}

impl AppState {
    pub fn new(
        create_account: Arc<CreateAccountUseCase>,
        list_accounts: Arc<ListAccountsUseCase>,
    ) -> Self {
        Self {
            create_account,
            list_accounts,
        }
    }
}

impl FromRef<AppState> for Arc<CreateAccountUseCase> {
    fn from_ref(state: &AppState) -> Self {
        state.create_account.clone()
    }
}

impl FromRef<AppState> for Arc<ListAccountsUseCase> {
    fn from_ref(state: &AppState) -> Self {
        state.list_accounts.clone()
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route(
            "/accounts",
            post(create_account::create_account).get(list_accounts::list_accounts),
        )
        .with_state(state)
}
