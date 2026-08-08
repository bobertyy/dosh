use std::sync::Arc;

use crate::{
    model::{account::Account, account_code::AccountCode},
    port::account_repository::{self, AccountRepository},
};

pub struct CreateAccountUseCase {
    account_repo: Arc<dyn AccountRepository>,
}

#[derive(thiserror::Error, Debug)]
pub enum CreateAccountUseCaseError {
    #[error("repository encountered an issue")]
    Repository,
    #[error("account with code {0} already exists")]
    AlreadyExists(AccountCode),
}

impl From<account_repository::CreateAccountError> for CreateAccountUseCaseError {
    fn from(value: account_repository::CreateAccountError) -> Self {
        match value {
            account_repository::CreateAccountError::Internal => Self::Repository,
            account_repository::CreateAccountError::AlreadyExists(account_code) => {
                Self::AlreadyExists(account_code)
            }
        }
    }
}

impl CreateAccountUseCase {
    pub fn new(account_repo: Arc<dyn AccountRepository>) -> Self {
        Self { account_repo }
    }

    pub async fn execute(&self, account: &Account) -> Result<(), CreateAccountUseCaseError> {
        self.account_repo
            .create(account)
            .await
            .map_err(|err| err.into())
    }
}
