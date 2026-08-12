use dosh_domain::model::journal_line_item::JournalLineItem;
use serde::{Deserialize, Serialize};

use crate::adapter::http::dto::entry_type::EntryTypeJson;

/// A [`JournalLineItem`] as returned to a client, with `amount` in minor units.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct JournalLineItemJson {
    pub id: String,
    pub account_code: String,
    pub amount: i64,
    pub entry_type: EntryTypeJson,
    pub description: Option<String>,
}

impl From<&JournalLineItem> for JournalLineItemJson {
    fn from(item: &JournalLineItem) -> Self {
        Self {
            id: item.id().to_string(),
            account_code: item.account_code().to_string(),
            amount: item.amount().minor_units(),
            entry_type: item.entry_type().into(),
            description: item.description().clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use dosh_domain::model::{
        account_code::AccountCode, amount::Amount, journal_line_item::EntryType,
    };

    use super::*;

    #[test]
    fn maps_a_line_item_with_a_description() {
        let item = JournalLineItem::new_with_description(
            AccountCode::parse("300").unwrap(),
            Amount::parse(1250).unwrap(),
            EntryType::Debit,
            "Beans".to_string(),
        )
        .unwrap();

        assert_eq!(
            JournalLineItemJson::from(&item),
            JournalLineItemJson {
                id: item.id().to_string(),
                account_code: "300".to_string(),
                amount: 1250,
                entry_type: EntryTypeJson::Debit,
                description: Some("Beans".to_string()),
            }
        );
    }

    #[test]
    fn maps_a_line_item_without_a_description() {
        let item = JournalLineItem::new(
            AccountCode::parse("100").unwrap(),
            Amount::parse(1250).unwrap(),
            EntryType::Credit,
        );

        assert_eq!(
            JournalLineItemJson::from(&item),
            JournalLineItemJson {
                id: item.id().to_string(),
                account_code: "100".to_string(),
                amount: 1250,
                entry_type: EntryTypeJson::Credit,
                description: None,
            }
        );
    }

    #[test]
    fn reports_the_generated_identity_of_the_line_item() {
        let item = JournalLineItem::new(
            AccountCode::parse("100").unwrap(),
            Amount::parse(1250).unwrap(),
            EntryType::Credit,
        );

        assert_eq!(
            JournalLineItemJson::from(&item).id,
            item.id().uuid().to_string()
        );
    }
}
