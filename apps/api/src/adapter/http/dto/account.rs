use dosh_domain::model::account::Account;
use serde::{Deserialize, Serialize};

use crate::adapter::http::dto::account_class::AccountClassJson;

/// An [`Account`] as returned to a client.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct AccountJson {
    pub code: String,
    pub class: AccountClassJson,
    pub description: Option<String>,
}

impl From<&Account> for AccountJson {
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
            AccountJson::from(&account),
            AccountJson {
                code: "200".to_string(),
                class: AccountClassJson::Revenue,
                description: Some("Sales revenue".to_string()),
            }
        );
    }

    #[test]
    fn maps_an_account_without_a_description() {
        let account = Account::new(AccountCode::parse("100").unwrap(), AccountClass::Asset);

        assert_eq!(
            AccountJson::from(&account),
            AccountJson {
                code: "100".to_string(),
                class: AccountClassJson::Asset,
                description: None,
            }
        );
    }

    #[test]
    fn serialises_a_null_description() {
        let account = Account::new(AccountCode::parse("100").unwrap(), AccountClass::Asset);

        assert_eq!(
            serde_json::to_string(&AccountJson::from(&account)).unwrap(),
            r#"{"code":"100","class":"asset","description":null}"#
        );
    }
}
