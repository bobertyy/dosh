use std::{collections::HashSet, sync::Arc};

use crate::{
    model::{account_code::AccountCode, journal_entry::JournalEntry},
    port::{
        account_repository::{self, AccountRepository},
        journal_entry_repository::{self, JournalEntryRepository},
    },
};

pub struct CreateJournalEntryUseCase {
    account_repo: Arc<dyn AccountRepository>,
    journal_entry_repo: Arc<dyn JournalEntryRepository>,
}

#[derive(thiserror::Error, Debug)]
pub enum CreateJournalEntryUseCaseError {
    #[error("repository encountered an issue")]
    Repository,
    #[error("no account exists with code: {}", join(.0))]
    UnknownAccountCodes(Vec<AccountCode>),
}

impl From<account_repository::FindAccountCodesError> for CreateJournalEntryUseCaseError {
    fn from(value: account_repository::FindAccountCodesError) -> Self {
        match value {
            account_repository::FindAccountCodesError::Internal => Self::Repository,
        }
    }
}

impl From<journal_entry_repository::CreateJournalEntryError> for CreateJournalEntryUseCaseError {
    fn from(value: journal_entry_repository::CreateJournalEntryError) -> Self {
        match value {
            journal_entry_repository::CreateJournalEntryError::Internal => Self::Repository,
            journal_entry_repository::CreateJournalEntryError::AlreadyExists(_) => Self::Repository,
        }
    }
}

impl CreateJournalEntryUseCase {
    pub fn new(
        account_repo: Arc<dyn AccountRepository>,
        journal_entry_repo: Arc<dyn JournalEntryRepository>,
    ) -> Self {
        Self {
            account_repo,
            journal_entry_repo,
        }
    }

    pub async fn execute(
        &self,
        entry: &JournalEntry,
    ) -> Result<(), CreateJournalEntryUseCaseError> {
        self.check_accounts_exist(entry).await?;

        self.journal_entry_repo.create(entry).await?;

        Ok(())
    }

    async fn check_accounts_exist(
        &self,
        entry: &JournalEntry,
    ) -> Result<(), CreateJournalEntryUseCaseError> {
        let posted_to: Vec<AccountCode> = posted_accounts(entry).into_iter().collect();

        let lookup = self.account_repo.look_up_codes(&posted_to).await?;

        match lookup.missing().is_empty() {
            true => Ok(()),
            false => Err(CreateJournalEntryUseCaseError::UnknownAccountCodes(
                lookup.missing().to_vec(),
            )),
        }
    }
}

fn posted_accounts(entry: &JournalEntry) -> HashSet<AccountCode> {
    entry
        .line_items()
        .iter()
        .map(|item| item.account_code().clone())
        .collect()
}

fn join(codes: &[AccountCode]) -> String {
    codes
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join(", ")
}

#[cfg(test)]
mod test {
    use crate::model::{
        amount::Amount,
        journal_date::JournalDate,
        journal_line_item::{EntryType, JournalLineItem},
    };

    use super::*;

    #[test]
    fn asks_after_each_account_once_however_many_lines_post_to_it() {
        let line = |code: &str, minor_units: i64, entry_type| {
            JournalLineItem::new(
                AccountCode::parse(code).unwrap(),
                Amount::parse(minor_units).unwrap(),
                entry_type,
            )
        };

        let entry = JournalEntry::new(
            JournalDate::parse("2026-08-09").unwrap(),
            "Coffee for the office".to_string(),
            vec![
                line("100", 1000, EntryType::Credit),
                line("100", 250, EntryType::Credit),
                line("300", 1250, EntryType::Debit),
            ],
        )
        .unwrap();

        assert_eq!(
            posted_accounts(&entry),
            HashSet::from([
                AccountCode::parse("100").unwrap(),
                AccountCode::parse("300").unwrap()
            ])
        );
    }

    #[test]
    fn names_every_unknown_code_in_one_message() {
        let error = CreateJournalEntryUseCaseError::UnknownAccountCodes(vec![
            AccountCode::parse("998").unwrap(),
            AccountCode::parse("999").unwrap(),
        ]);

        assert_eq!(error.to_string(), "no account exists with code: 998, 999");
    }
}
