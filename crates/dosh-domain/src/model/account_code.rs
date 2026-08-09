use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountCode(String);

#[derive(thiserror::Error, Debug)]
pub enum AccountCodeParseError {
    #[error("expected code to be string of digits starting with non-zero digit, got: {0}")]
    InvalidFormat(String),
}

impl AccountCode {
    pub fn parse<Input: Into<String>>(input: Input) -> Result<Self, AccountCodeParseError> {
        let code = input.into();

        match is_code_shaped(&code) {
            true => Ok(Self(code)),
            false => Err(AccountCodeParseError::InvalidFormat(code)),
        }
    }
}

impl Display for AccountCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// The leading digits of an [`AccountCode`], for matching whole families of
/// accounts at once. A prefix is itself a well-formed code, so `"2"` selects
/// every account under `2`, and `"0"` — which no code can start with — is
/// rejected rather than quietly matching nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountCodePrefix(String);

#[derive(thiserror::Error, Debug)]
pub enum AccountCodePrefixParseError {
    #[error("expected code prefix to be string of digits starting with non-zero digit, got: {0}")]
    InvalidFormat(String),
}

impl AccountCodePrefix {
    pub fn parse<Input: Into<String>>(input: Input) -> Result<Self, AccountCodePrefixParseError> {
        let prefix = input.into();

        match is_code_shaped(&prefix) {
            true => Ok(Self(prefix)),
            false => Err(AccountCodePrefixParseError::InvalidFormat(prefix)),
        }
    }
}

impl Display for AccountCodePrefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

fn is_code_shaped(input: &str) -> bool {
    let all_digits = !input.is_empty() && input.bytes().all(|b| b.is_ascii_digit());
    let is_first_digit_zero = input.starts_with("0");

    all_digits && !is_first_digit_zero
}

#[cfg(test)]
mod test {

    use std::assert_matches;

    use super::*;

    #[test]
    fn parse_should_return_account_code() {
        let result = AccountCode::parse("201");

        assert_eq!(result.unwrap(), AccountCode("201".to_string()))
    }

    #[test]
    fn parse_should_return_error_when_code_is_not_digits() {
        let result = AccountCode::parse("123a");

        assert_matches!(
            result.unwrap_err(),
            AccountCodeParseError::InvalidFormat(expected_input) if expected_input == "123a"
        );
    }

    #[test]
    fn parse_should_return_error_when_code_starts_with_zero() {
        let result = AccountCode::parse("0123");

        assert_matches!(
            result.unwrap_err(),
                AccountCodeParseError::InvalidFormat(expected_input) if expected_input == "0123"
        )
    }

    mod prefix {
        use super::*;

        #[test]
        fn parse_should_return_prefix() {
            assert_eq!(AccountCodePrefix::parse("20").unwrap().to_string(), "20");
        }

        #[test]
        fn parse_should_return_error_when_prefix_is_empty() {
            assert_matches!(
                AccountCodePrefix::parse("").unwrap_err(),
                AccountCodePrefixParseError::InvalidFormat(expected_input) if expected_input.is_empty()
            );
        }

        #[test]
        fn parse_should_return_error_when_prefix_is_not_digits() {
            assert_matches!(
                AccountCodePrefix::parse("2%").unwrap_err(),
                AccountCodePrefixParseError::InvalidFormat(expected_input) if expected_input == "2%"
            );
        }

        #[test]
        fn parse_should_return_error_when_prefix_starts_with_zero() {
            assert_matches!(
                AccountCodePrefix::parse("0").unwrap_err(),
                AccountCodePrefixParseError::InvalidFormat(expected_input) if expected_input == "0"
            );
        }
    }
}
