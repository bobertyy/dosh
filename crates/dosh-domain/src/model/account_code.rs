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

        let all_digits = !code.is_empty() && code.bytes().all(|b| b.is_ascii_digit());
        let is_first_digit_zero = code.starts_with("0");

        match all_digits && !is_first_digit_zero {
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
}
