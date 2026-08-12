use chrono::NaiveDate;
use dosh_domain::model::journal_entry::JournalEntry;
use uuid::Uuid;

/// A row of the `journal_entries` table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalEntryPgRecord {
    pub id: Uuid,
    pub date: NaiveDate,
    pub description: String,
}

impl From<&JournalEntry> for JournalEntryPgRecord {
    fn from(entry: &JournalEntry) -> Self {
        Self {
            id: entry.id().uuid(),
            date: entry.date().date(),
            description: entry.description().to_string(),
        }
    }
}

#[cfg(test)]
mod test {
    use dosh_domain::model::{
        account_code::AccountCode,
        amount::Amount,
        journal_date::JournalDate,
        journal_line_item::{EntryType, JournalLineItem},
    };

    use super::*;

    #[test]
    fn maps_an_entry() {
        let entry = JournalEntry::new(
            JournalDate::parse("2026-08-09").unwrap(),
            "Coffee for the office".to_string(),
            vec![
                JournalLineItem::new(
                    AccountCode::parse("100").unwrap(),
                    Amount::parse(1250).unwrap(),
                    EntryType::Credit,
                ),
                JournalLineItem::new(
                    AccountCode::parse("300").unwrap(),
                    Amount::parse(1250).unwrap(),
                    EntryType::Debit,
                ),
            ],
        )
        .unwrap();

        assert_eq!(
            JournalEntryPgRecord::from(&entry),
            JournalEntryPgRecord {
                id: entry.id().uuid(),
                date: NaiveDate::from_ymd_opt(2026, 8, 9).unwrap(),
                description: "Coffee for the office".to_string(),
            }
        );
    }
}
