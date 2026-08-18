use dosh_domain::model::account::Account;
use serde::{Deserialize, Serialize};

use crate::adapter::http::dto::account_class::{
    AccountClassJson, AccountSubclassJson, account_class_json,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct AccountJson {
    pub code: String,
    pub class: AccountClassJson,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subclass: Option<AccountSubclassJson>,
    pub description: Option<String>,
}

impl From<&Account> for AccountJson {
    fn from(account: &Account) -> Self {
        let (class, subclass) = account_class_json(account.class());

        Self {
            code: account.code().to_string(),
            class,
            subclass,
            description: account.description().clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use dosh_domain::model::{
        account::{AccountClass, AssetClass, RevenueClass},
        account_code::AccountCode,
    };

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
            AccountJson::from(&account),
            AccountJson {
                code: "200".to_string(),
                class: AccountClassJson::Revenue,
                subclass: Some(AccountSubclassJson::Sales),
                description: Some("Sales revenue".to_string()),
            }
        );
    }

    #[test]
    fn maps_an_account_without_a_description() {
        let account = Account::new(
            AccountCode::parse("100").unwrap(),
            AccountClass::Asset(AssetClass::Bank),
        );

        assert_eq!(
            AccountJson::from(&account),
            AccountJson {
                code: "100".to_string(),
                class: AccountClassJson::Asset,
                subclass: Some(AccountSubclassJson::Bank),
                description: None,
            }
        );
    }

    #[test]
    fn serialises_a_null_description() {
        let account = Account::new(
            AccountCode::parse("100").unwrap(),
            AccountClass::Asset(AssetClass::Bank),
        );

        assert_eq!(
            serde_json::to_string(&AccountJson::from(&account)).unwrap(),
            r#"{"code":"100","class":"asset","subclass":"bank","description":null}"#
        );
    }

    #[test]
    fn serialises_a_class_with_no_subclasses_without_one() {
        let account = Account::new(AccountCode::parse("960").unwrap(), AccountClass::Equity);

        assert_eq!(
            serde_json::to_string(&AccountJson::from(&account)).unwrap(),
            r#"{"code":"960","class":"equity","description":null}"#
        );
    }
}
