use std::pin::Pin;

use crate::model::{account::Account, account_code::AccountCode, account_filter::AccountFilter};

#[derive(Debug, thiserror::Error)]
pub enum CreateAccountError {
    #[error("internal account repository error")]
    Internal,
    #[error("account with code {0} already exists")]
    AlreadyExists(AccountCode),
}

#[derive(Debug, thiserror::Error)]
pub enum ListAccountsError {
    #[error("internal account repository error")]
    Internal,
}

pub trait AccountRepository: Send + Sync {
    fn create<'a>(
        &'a self,
        account: &'a Account,
    ) -> Pin<Box<dyn Future<Output = Result<(), CreateAccountError>> + Send + 'a>>;

    /// The accounts matching `filter`, ordered by code, starting at the first
    /// code after `after`, and never more than `limit` of them.
    ///
    /// Ordering is by code as stored, so a caller walking the collection a page
    /// at a time sees every account exactly once.
    fn list<'a>(
        &'a self,
        filter: &'a AccountFilter,
        after: Option<&'a AccountCode>,
        limit: u32,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Account>, ListAccountsError>> + Send + 'a>>;
}
