use dosh_domain::model::{
    account::{Account, AccountCreationError},
    account_code::{AccountCode, AccountCodeParseError},
};

use crate::adapter::postgres::dto::account_class::{
    UnknownAccountClassPgValue, account_class_value, parse_account_class,
};

/// A row of the `accounts` table.
///
/// `class` is a plain string so `sqlx::query_as!` can decode a row straight
/// into it; [`account_class_value`] pins what that string may be.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountPgRecord {
    pub code: String,
    pub class: String,
    pub description: Option<String>,
}

impl From<&Account> for AccountPgRecord {
    fn from(account: &Account) -> Self {
        Self {
            code: account.code().to_string(),
            class: account_class_value(account.class()).to_string(),
            description: account.description().clone(),
        }
    }
}

/// A stored row the schema should have made impossible.
#[derive(Debug, thiserror::Error)]
pub enum AccountPgRecordError {
    #[error(transparent)]
    Code(#[from] AccountCodeParseError),
    #[error(transparent)]
    Class(#[from] UnknownAccountClassPgValue),
    #[error(transparent)]
    Account(#[from] AccountCreationError),
}

impl TryFrom<AccountPgRecord> for Account {
    type Error = AccountPgRecordError;

    fn try_from(record: AccountPgRecord) -> Result<Self, Self::Error> {
        let code = AccountCode::parse(record.code)?;
        let class = parse_account_class(&record.class)?;

        match record.description {
            Some(description) => Ok(Account::new_with_description(code, class, description)?),
            None => Ok(Account::new(code, class)),
        }
    }
}

#[cfg(test)]
mod test {
    use std::assert_matches;

    use dosh_domain::model::account::{AccountClass, AssetClass, RevenueClass};

    use super::*;

    #[test]
    fn maps_an_account_with_a_description() {
        let account = Account::new_with_description(
            AccountCode::parse("200").unwrap(),
            AccountClass::Revenue(RevenueClass::Sales),
            "Sales revenue".to_string(),
        )
        .unwrap();

        assert_eq!(
            AccountPgRecord::from(&account),
            AccountPgRecord {
                code: "200".to_string(),
                class: "revenue.sales".to_string(),
                description: Some("Sales revenue".to_string()),
            }
        );
    }

    #[test]
    fn maps_an_account_without_a_description() {
        let account = Account::new(
            AccountCode::parse("100").unwrap(),
            AccountClass::Asset(AssetClass::Current),
        );

        assert_eq!(
            AccountPgRecord::from(&account),
            AccountPgRecord {
                code: "100".to_string(),
                class: "asset.current".to_string(),
                description: None,
            }
        );
    }

    mod stored {
        use super::*;

        fn record(code: &str, class: &str, description: Option<&str>) -> AccountPgRecord {
            AccountPgRecord {
                code: code.to_string(),
                class: class.to_string(),
                description: description.map(ToString::to_string),
            }
        }

        #[test]
        fn maps_a_row_with_a_description() {
            let account =
                Account::try_from(record("200", "revenue.sales", Some("Sales revenue"))).unwrap();

            assert_eq!(account.code(), &AccountCode::parse("200").unwrap());
            assert_eq!(account.class(), &AccountClass::Revenue(RevenueClass::Sales));
            assert_eq!(account.description(), &Some("Sales revenue".to_string()));
        }

        #[test]
        fn maps_a_row_without_a_description() {
            let account = Account::try_from(record("100", "asset.current", None)).unwrap();

            assert_eq!(account.code(), &AccountCode::parse("100").unwrap());
            assert_eq!(account.class(), &AccountClass::Asset(AssetClass::Current));
            assert_eq!(account.description(), &None);
        }

        #[test]
        fn returns_error_when_the_stored_code_is_not_a_code() {
            let error = Account::try_from(record("0123", "asset.current", None)).unwrap_err();

            assert_matches!(error, AccountPgRecordError::Code(_));
        }

        #[test]
        fn returns_error_when_the_stored_class_is_unknown() {
            let error = Account::try_from(record("100", "pizza", None)).unwrap_err();

            assert_matches!(error, AccountPgRecordError::Class(_));
        }

        #[test]
        fn returns_error_when_the_stored_description_is_empty() {
            let error = Account::try_from(record("100", "asset.current", Some(""))).unwrap_err();

            assert_matches!(error, AccountPgRecordError::Account(_));
        }
    }
}
