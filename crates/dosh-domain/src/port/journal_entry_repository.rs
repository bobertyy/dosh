use std::pin::Pin;

use crate::model::journal_entry::{JournalEntry, JournalEntryId};

#[derive(Debug, thiserror::Error)]
pub enum CreateJournalEntryError {
    #[error("internal journal entry repository error")]
    Internal,
    #[error("journal entry with id {0} already exists")]
    AlreadyExists(JournalEntryId),
}

pub trait JournalEntryRepository: Send + Sync {
    /// Stores `entry` whole — every line item of it, or none.
    fn create<'a>(
        &'a self,
        entry: &'a JournalEntry,
    ) -> Pin<Box<dyn Future<Output = Result<(), CreateJournalEntryError>> + Send + 'a>>;
}
