use dosh_domain::model::{
    account::{Account, AccountCreationError},
    account_code::{AccountCode, AccountCodeParseError},
};
use serde::{Deserialize, Serialize};

use crate::adapter::http::dto::account_class::AccountClassJson;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct CreateAccountRequest {
    pub code: String,
    pub class: AccountClassJson,
    #[serde(default)]
    pub description: Option<String>,
}

/// A request that deserialised cleanly but does not describe a valid account.
#[derive(Debug, thiserror::Error)]
pub enum CreateAccountRequestError {
    #[error(transparent)]
    Code(#[from] AccountCodeParseError),
    #[error(transparent)]
    Account(#[from] AccountCreationError),
}

impl TryFrom<CreateAccountRequest> for Account {
    type Error = CreateAccountRequestError;

    fn try_from(request: CreateAccountRequest) -> Result<Self, Self::Error> {
        let code = AccountCode::parse(request.code)?;
        let class = request.class.into();

        match request.description {
            Some(description) => Ok(Account::new_with_description(code, class, description)?),
            None => Ok(Account::new(code, class)),
        }
    }
}

#[cfg(test)]
mod test {
    use std::assert_matches;

    use dosh_domain::model::account::AccountClass;

    use super::*;

    #[test]
    fn maps_a_request_with_a_description() {
        let account = Account::try_from(CreateAccountRequest {
            code: "200".to_string(),
            class: AccountClassJson::Revenue,
            description: Some("Sales revenue".to_string()),
        })
        .unwrap();

        assert_eq!(account.code(), &AccountCode::parse("200").unwrap());
        assert_eq!(account.class(), &AccountClass::Revenue);
        assert_eq!(account.description(), &Some("Sales revenue".to_string()));
    }

    #[test]
    fn maps_a_request_without_a_description() {
        let account = Account::try_from(CreateAccountRequest {
            code: "100".to_string(),
            class: AccountClassJson::Asset,
            description: None,
        })
        .unwrap();

        assert_eq!(account.code(), &AccountCode::parse("100").unwrap());
        assert_eq!(account.class(), &AccountClass::Asset);
        assert_eq!(account.description(), &None);
    }

    #[test]
    fn returns_error_when_code_is_invalid() {
        let error = Account::try_from(CreateAccountRequest {
            code: "0123".to_string(),
            class: AccountClassJson::Asset,
            description: None,
        })
        .unwrap_err();

        assert_matches!(error, CreateAccountRequestError::Code(_));
    }

    #[test]
    fn returns_error_when_description_is_empty() {
        let error = Account::try_from(CreateAccountRequest {
            code: "100".to_string(),
            class: AccountClassJson::Asset,
            description: Some("".to_string()),
        })
        .unwrap_err();

        assert_matches!(error, CreateAccountRequestError::Account(_));
    }

    #[test]
    fn deserialises_a_body_without_a_description() {
        assert_eq!(
            serde_json::from_str::<CreateAccountRequest>(r#"{"code":"100","class":"asset"}"#)
                .unwrap(),
            CreateAccountRequest {
                code: "100".to_string(),
                class: AccountClassJson::Asset,
                description: None,
            }
        );
    }
}
