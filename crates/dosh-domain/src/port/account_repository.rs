use std::pin::Pin;

use crate::model::{account::Account, account_code::AccountCode};

#[derive(Debug, thiserror::Error)]
pub enum CreateAccountError {
    #[error("internal account repository error")]
    Internal,
    #[error("account with code {0} already exists")]
    AlreadyExists(AccountCode),
}

pub trait AccountRepository {
    fn create(
        &self,
        account: &Account,
    ) -> Pin<Box<dyn Future<Output = Result<(), CreateAccountError>> + Send>>;
}
