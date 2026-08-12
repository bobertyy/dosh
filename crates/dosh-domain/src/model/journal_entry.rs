use std::fmt::Display;

use uuid::Uuid;

use super::{
    journal_date::JournalDate,
    journal_line_item::{EntryType, JournalLineItem},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JournalEntryId(Uuid);

impl JournalEntryId {
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn uuid(&self) -> Uuid {
        self.0
    }
}

impl Display for JournalEntryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// One posting to the ledger: a dated, described set of line items whose
/// credits and debits cancel out. It cannot be built out of balance.
#[derive(Debug)]
pub struct JournalEntry {
    id: JournalEntryId,
    date: JournalDate,
    description: String,
    line_items: Vec<JournalLineItem>,
}

#[derive(thiserror::Error, Debug)]
pub enum JournalEntryCreationError {
    #[error("description cannot be empty")]
    EmptyDescription,
    #[error("an entry must have at least one line item")]
    NoLineItems,
    #[error("credits and debits must balance, got credits of {credits} against debits of {debits}")]
    Unbalanced { credits: i128, debits: i128 },
}

impl JournalEntry {
    pub fn new(
        date: JournalDate,
        description: String,
        line_items: Vec<JournalLineItem>,
    ) -> Result<Self, JournalEntryCreationError> {
        if description.is_empty() {
            return Err(JournalEntryCreationError::EmptyDescription);
        }

        if line_items.is_empty() {
            return Err(JournalEntryCreationError::NoLineItems);
        }

        let credits = total_of(&line_items, EntryType::Credit);
        let debits = total_of(&line_items, EntryType::Debit);

        if credits != debits {
            return Err(JournalEntryCreationError::Unbalanced { credits, debits });
        }

        Ok(Self {
            id: JournalEntryId::generate(),
            date,
            description,
            line_items,
        })
    }

    pub fn id(&self) -> &JournalEntryId {
        &self.id
    }

    pub fn date(&self) -> &JournalDate {
        &self.date
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn line_items(&self) -> &[JournalLineItem] {
        &self.line_items
    }
}

fn total_of(line_items: &[JournalLineItem], entry_type: EntryType) -> i128 {
    line_items
        .iter()
        .filter(|item| item.entry_type() == &entry_type)
        .map(|item| i128::from(item.amount().minor_units()))
        .sum()
}

#[cfg(test)]
mod test {
    use std::assert_matches;

    use crate::model::{account_code::AccountCode, amount::Amount};

    use super::*;

    fn line(code: &str, minor_units: i64, entry_type: EntryType) -> JournalLineItem {
        JournalLineItem::new(
            AccountCode::parse(code).unwrap(),
            Amount::parse(minor_units).unwrap(),
            entry_type,
        )
    }

    fn entry_of(
        line_items: Vec<JournalLineItem>,
    ) -> Result<JournalEntry, JournalEntryCreationError> {
        JournalEntry::new(
            JournalDate::parse("2026-08-09").unwrap(),
            "Coffee for the office".to_string(),
            line_items,
        )
    }

    #[test]
    fn returns_a_balanced_entry() {
        let entry = entry_of(vec![
            line("100", 1250, EntryType::Credit),
            line("300", 1250, EntryType::Debit),
        ])
        .unwrap();

        assert_eq!(entry.date(), &JournalDate::parse("2026-08-09").unwrap());
        assert_eq!(entry.description(), "Coffee for the office");
        assert_eq!(entry.line_items().len(), 2);
    }

    #[test]
    fn balances_many_lines_against_one() {
        let entry = entry_of(vec![
            line("100", 1250, EntryType::Credit),
            line("300", 1000, EntryType::Debit),
            line("310", 250, EntryType::Debit),
        ]);

        assert!(entry.is_ok());
    }

    #[test]
    fn gives_every_entry_an_identity_of_its_own() {
        let one = entry_of(vec![
            line("100", 1250, EntryType::Credit),
            line("300", 1250, EntryType::Debit),
        ])
        .unwrap();
        let other = entry_of(vec![
            line("100", 1250, EntryType::Credit),
            line("300", 1250, EntryType::Debit),
        ])
        .unwrap();

        assert_ne!(one.id(), other.id());
    }

    #[test]
    fn keeps_the_line_items_it_was_given() {
        let entry = entry_of(vec![
            line("100", 1250, EntryType::Credit),
            line("300", 1250, EntryType::Debit),
        ])
        .unwrap();

        let codes: Vec<String> = entry
            .line_items()
            .iter()
            .map(|item| item.account_code().to_string())
            .collect();

        assert_eq!(codes, vec!["100", "300"]);
    }

    #[test]
    fn returns_error_when_credits_do_not_match_debits() {
        let error = entry_of(vec![
            line("100", 1250, EntryType::Credit),
            line("300", 1000, EntryType::Debit),
        ])
        .unwrap_err();

        assert_matches!(
            error,
            JournalEntryCreationError::Unbalanced {
                credits: 1250,
                debits: 1000
            }
        );
    }

    #[test]
    fn returns_error_when_every_line_is_on_the_same_side() {
        let error = entry_of(vec![
            line("100", 1250, EntryType::Debit),
            line("300", 1250, EntryType::Debit),
        ])
        .unwrap_err();

        assert_matches!(
            error,
            JournalEntryCreationError::Unbalanced {
                credits: 0,
                debits: 2500
            }
        );
    }

    #[test]
    fn returns_error_when_there_are_no_line_items() {
        assert_matches!(
            entry_of(Vec::new()).unwrap_err(),
            JournalEntryCreationError::NoLineItems
        );
    }

    #[test]
    fn returns_error_when_the_description_is_empty() {
        let error = JournalEntry::new(
            JournalDate::parse("2026-08-09").unwrap(),
            "".to_string(),
            vec![
                line("100", 1250, EntryType::Credit),
                line("300", 1250, EntryType::Debit),
            ],
        )
        .unwrap_err();

        assert_matches!(error, JournalEntryCreationError::EmptyDescription);
    }

    #[test]
    fn totals_that_overflow_a_single_amount_still_balance() {
        let entry = entry_of(vec![
            line("100", i64::MAX, EntryType::Credit),
            line("110", i64::MAX, EntryType::Credit),
            line("300", i64::MAX, EntryType::Debit),
            line("310", i64::MAX, EntryType::Debit),
        ]);

        assert!(entry.is_ok());
    }
}
