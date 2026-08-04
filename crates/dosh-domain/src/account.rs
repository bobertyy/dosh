use crate::account_code::AccountCode;

#[derive(Debug, PartialEq)]
pub enum AccountClass {
    Asset,
    Equity,
    Expense,
    Liability,
    Revenue,
}

#[derive(Debug)]
pub struct Account {
    code: AccountCode,
    class: AccountClass,
    description: Option<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum AccountCreationError {
    #[error("description cannot be empty")]
    EmptyDescription,
}

impl Account {
    pub fn new(code: AccountCode, class: AccountClass) -> Self {
        Self {
            code,
            class,
            description: None,
        }
    }

    pub fn new_with_description(
        code: AccountCode,
        class: AccountClass,
        description: String,
    ) -> Result<Self, AccountCreationError> {
        if description.is_empty() {
            return Err(AccountCreationError::EmptyDescription);
        }

        Ok(Self {
            code,
            class,
            description: Some(description),
        })
    }

    pub fn code(&self) -> &AccountCode {
        &self.code
    }

    pub fn class(&self) -> &AccountClass {
        &self.class
    }

    pub fn description(&self) -> &Option<String> {
        &self.description
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::account_code::AccountCode;

    mod new_with_description {
        use super::*;
        use std::assert_matches;

        #[test]
        fn returns_account() {
            let account = Account::new_with_description(
                AccountCode::parse("200").unwrap(),
                AccountClass::Revenue,
                "Sales revenue".to_string(),
            )
            .unwrap();

            assert_eq!(account.code, AccountCode::parse("200").unwrap());
            assert_eq!(account.class, AccountClass::Revenue);
            assert_eq!(account.description, Some("Sales revenue".to_string()));
        }

        #[test]
        fn returns_error_when_description_is_empty() {
            let error = Account::new_with_description(
                AccountCode::parse("200").unwrap(),
                AccountClass::Revenue,
                "".to_string(),
            )
            .unwrap_err();

            assert_matches!(error, AccountCreationError::EmptyDescription);
        }
    }
}
