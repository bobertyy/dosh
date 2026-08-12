use dosh_domain::model::{
    account_code::{AccountCode, AccountCodeParseError},
    amount::{Amount, AmountParseError},
    journal_date::{JournalDate, JournalDateParseError},
    journal_entry::{JournalEntry, JournalEntryCreationError},
    journal_line_item::{JournalLineItem, JournalLineItemCreationError},
};
use serde::{Deserialize, Serialize};

use crate::adapter::http::dto::entry_type::EntryTypeJson;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct CreateJournalEntryRequest {
    pub date: String,
    pub description: String,
    pub line_items: Vec<JournalLineItemRequest>,
}

/// One line of a posting, with `amount` in minor units.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct JournalLineItemRequest {
    pub account_code: String,
    pub amount: i64,
    pub entry_type: EntryTypeJson,
    #[serde(default)]
    pub description: Option<String>,
}

/// A request that deserialised cleanly but does not describe a valid posting.
#[derive(Debug, thiserror::Error)]
pub enum CreateJournalEntryRequestError {
    #[error(transparent)]
    Date(#[from] JournalDateParseError),
    #[error(transparent)]
    AccountCode(#[from] AccountCodeParseError),
    #[error(transparent)]
    Amount(#[from] AmountParseError),
    #[error(transparent)]
    LineItem(#[from] JournalLineItemCreationError),
    #[error(transparent)]
    Entry(#[from] JournalEntryCreationError),
}

impl TryFrom<CreateJournalEntryRequest> for JournalEntry {
    type Error = CreateJournalEntryRequestError;

    fn try_from(request: CreateJournalEntryRequest) -> Result<Self, Self::Error> {
        let date = JournalDate::parse(request.date)?;

        let line_items = request
            .line_items
            .into_iter()
            .map(JournalLineItem::try_from)
            .collect::<Result<Vec<JournalLineItem>, Self::Error>>()?;

        Ok(JournalEntry::new(date, request.description, line_items)?)
    }
}

impl TryFrom<JournalLineItemRequest> for JournalLineItem {
    type Error = CreateJournalEntryRequestError;

    fn try_from(request: JournalLineItemRequest) -> Result<Self, Self::Error> {
        let account_code = AccountCode::parse(request.account_code)?;
        let amount = Amount::parse(request.amount)?;
        let entry_type = request.entry_type.into();

        match request.description {
            Some(description) => Ok(JournalLineItem::new_with_description(
                account_code,
                amount,
                entry_type,
                description,
            )?),
            None => Ok(JournalLineItem::new(account_code, amount, entry_type)),
        }
    }
}

#[cfg(test)]
mod test {
    use std::assert_matches;

    use dosh_domain::model::journal_line_item::EntryType;

    use super::*;

    fn line(account_code: &str, amount: i64, entry_type: EntryTypeJson) -> JournalLineItemRequest {
        JournalLineItemRequest {
            account_code: account_code.to_string(),
            amount,
            entry_type,
            description: None,
        }
    }

    fn request(line_items: Vec<JournalLineItemRequest>) -> CreateJournalEntryRequest {
        CreateJournalEntryRequest {
            date: "2026-08-09".to_string(),
            description: "Coffee for the office".to_string(),
            line_items,
        }
    }

    fn balanced() -> CreateJournalEntryRequest {
        request(vec![
            line("100", 1250, EntryTypeJson::Credit),
            line("300", 1250, EntryTypeJson::Debit),
        ])
    }

    #[test]
    fn maps_a_balanced_request() {
        let entry = JournalEntry::try_from(balanced()).unwrap();

        assert_eq!(entry.date(), &JournalDate::parse("2026-08-09").unwrap());
        assert_eq!(entry.description(), "Coffee for the office");
        assert_eq!(entry.line_items().len(), 2);

        let credit = &entry.line_items()[0];
        assert_eq!(credit.account_code(), &AccountCode::parse("100").unwrap());
        assert_eq!(credit.amount(), &Amount::parse(1250).unwrap());
        assert_eq!(credit.entry_type(), &EntryType::Credit);
        assert_eq!(credit.description(), &None);
    }

    #[test]
    fn maps_a_line_item_with_a_description() {
        let entry = JournalEntry::try_from(request(vec![
            JournalLineItemRequest {
                description: Some("Beans".to_string()),
                ..line("100", 1250, EntryTypeJson::Credit)
            },
            line("300", 1250, EntryTypeJson::Debit),
        ]))
        .unwrap();

        assert_eq!(
            entry.line_items()[0].description(),
            &Some("Beans".to_string())
        );
    }

    #[test]
    fn gives_the_entry_and_every_line_item_an_identity() {
        let entry = JournalEntry::try_from(balanced()).unwrap();

        let ids: Vec<String> = entry
            .line_items()
            .iter()
            .map(|item| item.id().to_string())
            .collect();

        assert_ne!(ids[0], ids[1]);
        assert!(!entry.id().to_string().is_empty());
    }

    #[test]
    fn returns_error_when_the_date_is_not_a_date() {
        let error = JournalEntry::try_from(CreateJournalEntryRequest {
            date: "the ninth".to_string(),
            ..balanced()
        })
        .unwrap_err();

        assert_matches!(error, CreateJournalEntryRequestError::Date(_));
    }

    #[test]
    fn returns_error_when_an_account_code_is_not_a_code() {
        let error = JournalEntry::try_from(request(vec![
            line("nought", 1250, EntryTypeJson::Credit),
            line("300", 1250, EntryTypeJson::Debit),
        ]))
        .unwrap_err();

        assert_matches!(error, CreateJournalEntryRequestError::AccountCode(_));
    }

    #[test]
    fn returns_error_when_an_amount_is_not_positive() {
        let error = JournalEntry::try_from(request(vec![
            line("100", 0, EntryTypeJson::Credit),
            line("300", 0, EntryTypeJson::Debit),
        ]))
        .unwrap_err();

        assert_matches!(error, CreateJournalEntryRequestError::Amount(_));
    }

    #[test]
    fn returns_error_when_a_line_item_description_is_empty() {
        let error = JournalEntry::try_from(request(vec![
            JournalLineItemRequest {
                description: Some("".to_string()),
                ..line("100", 1250, EntryTypeJson::Credit)
            },
            line("300", 1250, EntryTypeJson::Debit),
        ]))
        .unwrap_err();

        assert_matches!(error, CreateJournalEntryRequestError::LineItem(_));
    }

    #[test]
    fn returns_error_when_the_credits_do_not_match_the_debits() {
        let error = JournalEntry::try_from(request(vec![
            line("100", 1250, EntryTypeJson::Credit),
            line("300", 1000, EntryTypeJson::Debit),
        ]))
        .unwrap_err();

        assert_matches!(error, CreateJournalEntryRequestError::Entry(_));
    }

    #[test]
    fn returns_error_when_there_are_no_line_items() {
        let error = JournalEntry::try_from(request(Vec::new())).unwrap_err();

        assert_matches!(
            error,
            CreateJournalEntryRequestError::Entry(JournalEntryCreationError::NoLineItems)
        );
    }

    #[test]
    fn deserialises_a_body_without_line_item_descriptions() {
        let body = r#"{
            "date": "2026-08-09",
            "description": "Coffee for the office",
            "line_items": [
                {"account_code": "100", "amount": 1250, "entry_type": "credit"},
                {"account_code": "300", "amount": 1250, "entry_type": "debit"}
            ]
        }"#;

        assert_eq!(
            serde_json::from_str::<CreateJournalEntryRequest>(body).unwrap(),
            balanced()
        );
    }

    #[test]
    fn rejects_a_body_with_an_unknown_entry_type() {
        let body = r#"{
            "date": "2026-08-09",
            "description": "Coffee for the office",
            "line_items": [{"account_code": "100", "amount": 1250, "entry_type": "sideways"}]
        }"#;

        assert!(serde_json::from_str::<CreateJournalEntryRequest>(body).is_err());
    }
}
