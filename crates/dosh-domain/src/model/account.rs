use super::{
    account_code::{AccountCode, AccountCodeParseError},
    page_cursor::{PageCursor, Pageable},
};

#[derive(Debug, Clone, Copy, PartialEq)]
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

impl Pageable for Account {
    type Key = AccountCode;
    type KeyError = AccountCodeParseError;

    fn page_key(&self) -> &AccountCode {
        &self.code
    }

    fn key_from_cursor(cursor: &PageCursor) -> Result<AccountCode, AccountCodeParseError> {
        AccountCode::parse(cursor.to_string())
    }
}

#[cfg(test)]
mod test {
    use super::*;

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

    mod pageable {
        use super::*;
        use std::assert_matches;

        #[test]
        fn is_positioned_by_its_code() {
            let account = Account::new(AccountCode::parse("200").unwrap(), AccountClass::Revenue);

            assert_eq!(
                PageCursor::from(&account),
                PageCursor::parse("200").unwrap()
            );
        }

        #[test]
        fn reads_a_code_back_out_of_a_cursor() {
            let cursor = PageCursor::parse("200").unwrap();

            assert_eq!(
                Account::key_from_cursor(&cursor).unwrap(),
                AccountCode::parse("200").unwrap()
            );
        }

        #[test]
        fn returns_error_when_a_cursor_is_not_a_code() {
            let cursor = PageCursor::parse("not a code").unwrap();

            assert_matches!(
                Account::key_from_cursor(&cursor).unwrap_err(),
                AccountCodeParseError::InvalidFormat(input) if input == "not a code"
            );
        }
    }
}
