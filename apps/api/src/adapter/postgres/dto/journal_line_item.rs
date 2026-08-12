use dosh_domain::model::{journal_entry::JournalEntryId, journal_line_item::JournalLineItem};
use uuid::Uuid;

use crate::adapter::postgres::dto::entry_type::entry_type_value;

/// A row of the `journal_line_items` table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalLineItemPgRecord {
    pub id: Uuid,
    pub journal_entry_id: Uuid,
    pub account_code: String,
    pub amount: i64,
    pub entry_type: String,
    pub description: Option<String>,
}

impl From<(&JournalEntryId, &JournalLineItem)> for JournalLineItemPgRecord {
    fn from((journal_entry_id, item): (&JournalEntryId, &JournalLineItem)) -> Self {
        Self {
            id: item.id().uuid(),
            journal_entry_id: journal_entry_id.uuid(),
            account_code: item.account_code().to_string(),
            amount: item.amount().minor_units(),
            entry_type: entry_type_value(item.entry_type()).to_string(),
            description: item.description().clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use dosh_domain::model::{
        account_code::AccountCode,
        amount::Amount,
        journal_line_item::{EntryType, JournalLineItem},
    };

    use super::*;

    #[test]
    fn maps_a_line_item_with_a_description() {
        let entry_id = JournalEntryId::generate();
        let item = JournalLineItem::new_with_description(
            AccountCode::parse("300").unwrap(),
            Amount::parse(1250).unwrap(),
            EntryType::Debit,
            "Beans".to_string(),
        )
        .unwrap();

        assert_eq!(
            JournalLineItemPgRecord::from((&entry_id, &item)),
            JournalLineItemPgRecord {
                id: item.id().uuid(),
                journal_entry_id: entry_id.uuid(),
                account_code: "300".to_string(),
                amount: 1250,
                entry_type: "debit".to_string(),
                description: Some("Beans".to_string()),
            }
        );
    }

    #[test]
    fn maps_a_line_item_without_a_description() {
        let entry_id = JournalEntryId::generate();
        let item = JournalLineItem::new(
            AccountCode::parse("100").unwrap(),
            Amount::parse(1250).unwrap(),
            EntryType::Credit,
        );

        assert_eq!(
            JournalLineItemPgRecord::from((&entry_id, &item)),
            JournalLineItemPgRecord {
                id: item.id().uuid(),
                journal_entry_id: entry_id.uuid(),
                account_code: "100".to_string(),
                amount: 1250,
                entry_type: "credit".to_string(),
                description: None,
            }
        );
    }
}
