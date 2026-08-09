use std::sync::Arc;

use crate::{
    model::{
        account::Account,
        account_filter::AccountFilter,
        page::Page,
        page_cursor::{PageCursor, Pageable},
        page_limit::PageLimit,
    },
    port::account_repository::{self, AccountRepository},
};

#[derive(Debug, Default, PartialEq)]
pub struct ListAccountsQuery {
    filter: AccountFilter,
    cursor: Option<PageCursor>,
    limit: PageLimit,
}

impl ListAccountsQuery {
    pub fn new(filter: AccountFilter, cursor: Option<PageCursor>, limit: PageLimit) -> Self {
        Self {
            filter,
            cursor,
            limit,
        }
    }

    pub fn filter(&self) -> &AccountFilter {
        &self.filter
    }

    pub fn cursor(&self) -> Option<&PageCursor> {
        self.cursor.as_ref()
    }

    pub fn limit(&self) -> PageLimit {
        self.limit
    }
}

pub struct ListAccountsUseCase {
    account_repo: Arc<dyn AccountRepository>,
}

#[derive(thiserror::Error, Debug)]
pub enum ListAccountsUseCaseError {
    #[error("repository encountered an issue")]
    Repository,
    #[error("page cursor does not name a position in this collection")]
    InvalidCursor,
}

impl From<account_repository::ListAccountsError> for ListAccountsUseCaseError {
    fn from(value: account_repository::ListAccountsError) -> Self {
        match value {
            account_repository::ListAccountsError::Internal => Self::Repository,
        }
    }
}

impl ListAccountsUseCase {
    pub fn new(account_repo: Arc<dyn AccountRepository>) -> Self {
        Self { account_repo }
    }

    pub async fn execute(
        &self,
        query: &ListAccountsQuery,
    ) -> Result<Page<Account>, ListAccountsUseCaseError> {
        let limit = query.limit();

        let after = query
            .cursor()
            .map(Account::key_from_cursor)
            .transpose()
            .map_err(|_| ListAccountsUseCaseError::InvalidCursor)?;

        let mut accounts = self
            .account_repo
            .list(query.filter(), after.as_ref(), limit.get() + 1)
            .await?;

        let has_more = accounts.len() > limit.get() as usize;
        if has_more {
            accounts.pop();
        }

        let next_cursor = match has_more {
            true => accounts.last().map(PageCursor::from),
            false => None,
        };

        Ok(Page::new(accounts, limit, next_cursor))
    }
}
