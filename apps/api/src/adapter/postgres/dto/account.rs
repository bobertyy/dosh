use dosh_domain::model::account::Account;

use crate::adapter::postgres::dto::account_class::AccountClassPgValue;

/// A row of the `accounts` table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountPgRecord {
    pub code: String,
    pub class: AccountClassPgValue,
    pub description: Option<String>,
}

impl From<&Account> for AccountPgRecord {
    fn from(account: &Account) -> Self {
        Self {
            code: account.code().to_string(),
            class: account.class().into(),
            description: account.description().clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use dosh_domain::model::{account::AccountClass, account_code::AccountCode};

    use super::*;

    #[test]
    fn maps_an_account_with_a_description() {
        let account = Account::new_with_description(
            AccountCode::parse("200").unwrap(),
            AccountClass::Revenue,
            "Sales revenue".to_string(),
        )
        .unwrap();

        assert_eq!(
            AccountPgRecord::from(&account),
            AccountPgRecord {
                code: "200".to_string(),
                class: AccountClassPgValue::from(AccountClass::Revenue),
                description: Some("Sales revenue".to_string()),
            }
        );
    }

    #[test]
    fn maps_an_account_without_a_description() {
        let account = Account::new(AccountCode::parse("100").unwrap(), AccountClass::Asset);

        assert_eq!(
            AccountPgRecord::from(&account),
            AccountPgRecord {
                code: "100".to_string(),
                class: AccountClassPgValue::from(AccountClass::Asset),
                description: None,
            }
        );
    }
}
