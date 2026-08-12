use dosh_domain::model::journal_entry::JournalEntry;
use serde::{Deserialize, Serialize};

use crate::adapter::http::dto::journal_line_item::JournalLineItemJson;

/// A [`JournalEntry`] as returned to a client.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct JournalEntryJson {
    pub id: String,
    pub date: String,
    pub description: String,
    pub line_items: Vec<JournalLineItemJson>,
}

impl From<&JournalEntry> for JournalEntryJson {
    fn from(entry: &JournalEntry) -> Self {
        Self {
            id: entry.id().to_string(),
            date: entry.date().to_string(),
            description: entry.description().to_string(),
            line_items: entry
                .line_items()
                .iter()
                .map(JournalLineItemJson::from)
                .collect(),
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

    use crate::adapter::http::dto::entry_type::EntryTypeJson;

    use super::*;

    fn entry() -> JournalEntry {
        JournalEntry::new(
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
        .unwrap()
    }

    #[test]
    fn maps_an_entry_and_its_line_items() {
        let entry = entry();

        assert_eq!(
            JournalEntryJson::from(&entry),
            JournalEntryJson {
                id: entry.id().to_string(),
                date: "2026-08-09".to_string(),
                description: "Coffee for the office".to_string(),
                line_items: vec![
                    JournalLineItemJson {
                        id: entry.line_items()[0].id().to_string(),
                        account_code: "100".to_string(),
                        amount: 1250,
                        entry_type: EntryTypeJson::Credit,
                        description: None,
                    },
                    JournalLineItemJson {
                        id: entry.line_items()[1].id().to_string(),
                        account_code: "300".to_string(),
                        amount: 1250,
                        entry_type: EntryTypeJson::Debit,
                        description: None,
                    },
                ],
            }
        );
    }

    #[test]
    fn reports_the_generated_identity_of_the_entry() {
        let entry = entry();

        assert_eq!(
            JournalEntryJson::from(&entry).id,
            entry.id().uuid().to_string()
        );
    }

    #[test]
    fn serialises_a_null_line_item_description() {
        let entry = entry();
        let body = serde_json::to_value(JournalEntryJson::from(&entry)).unwrap();

        assert_eq!(
            body["line_items"][0]["description"],
            serde_json::Value::Null
        );
        assert_eq!(body["date"], "2026-08-09");
    }
}
