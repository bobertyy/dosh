use std::fmt::Display;

use uuid::Uuid;

use super::{account_code::AccountCode, amount::Amount};

/// Which side of the ledger a line sits on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    Credit,
    Debit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JournalLineItemId(Uuid);

impl JournalLineItemId {
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn uuid(&self) -> Uuid {
        self.0
    }
}

impl Display for JournalLineItemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// One side of one movement: an amount posted to an account, as a credit or a
/// debit.
#[derive(Debug)]
pub struct JournalLineItem {
    id: JournalLineItemId,
    account_code: AccountCode,
    amount: Amount,
    entry_type: EntryType,
    description: Option<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum JournalLineItemCreationError {
    #[error("line item description cannot be empty")]
    EmptyDescription,
}

impl JournalLineItem {
    pub fn new(account_code: AccountCode, amount: Amount, entry_type: EntryType) -> Self {
        Self {
            id: JournalLineItemId::generate(),
            account_code,
            amount,
            entry_type,
            description: None,
        }
    }

    pub fn new_with_description(
        account_code: AccountCode,
        amount: Amount,
        entry_type: EntryType,
        description: String,
    ) -> Result<Self, JournalLineItemCreationError> {
        if description.is_empty() {
            return Err(JournalLineItemCreationError::EmptyDescription);
        }

        Ok(Self {
            description: Some(description),
            ..Self::new(account_code, amount, entry_type)
        })
    }

    pub fn id(&self) -> &JournalLineItemId {
        &self.id
    }

    pub fn account_code(&self) -> &AccountCode {
        &self.account_code
    }

    pub fn amount(&self) -> &Amount {
        &self.amount
    }

    pub fn entry_type(&self) -> &EntryType {
        &self.entry_type
    }

    pub fn description(&self) -> &Option<String> {
        &self.description
    }
}

#[cfg(test)]
mod test {
    use std::assert_matches;

    use super::*;

    fn credit_of(minor_units: i64) -> JournalLineItem {
        JournalLineItem::new(
            AccountCode::parse("200").unwrap(),
            Amount::parse(minor_units).unwrap(),
            EntryType::Credit,
        )
    }

    mod new {
        use super::*;

        #[test]
        fn returns_a_line_item_without_a_description() {
            let item = credit_of(1250);

            assert_eq!(item.account_code(), &AccountCode::parse("200").unwrap());
            assert_eq!(item.amount(), &Amount::parse(1250).unwrap());
            assert_eq!(item.entry_type(), &EntryType::Credit);
            assert_eq!(item.description(), &None);
        }

        #[test]
        fn gives_every_line_item_an_identity_of_its_own() {
            assert_ne!(credit_of(1250).id(), credit_of(1250).id());
        }
    }

    mod new_with_description {
        use super::*;

        #[test]
        fn returns_a_line_item() {
            let item = JournalLineItem::new_with_description(
                AccountCode::parse("100").unwrap(),
                Amount::parse(1250).unwrap(),
                EntryType::Debit,
                "Coffee".to_string(),
            )
            .unwrap();

            assert_eq!(item.account_code(), &AccountCode::parse("100").unwrap());
            assert_eq!(item.amount(), &Amount::parse(1250).unwrap());
            assert_eq!(item.entry_type(), &EntryType::Debit);
            assert_eq!(item.description(), &Some("Coffee".to_string()));
        }

        #[test]
        fn returns_error_when_description_is_empty() {
            let error = JournalLineItem::new_with_description(
                AccountCode::parse("100").unwrap(),
                Amount::parse(1250).unwrap(),
                EntryType::Debit,
                "".to_string(),
            )
            .unwrap_err();

            assert_matches!(error, JournalLineItemCreationError::EmptyDescription);
        }
    }
}
