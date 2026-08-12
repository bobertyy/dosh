use std::fmt::Display;

/// A sum of money, counted in the currency's smallest unit — pennies, cents.
/// Always positive: direction is the entry type's business.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Amount(i64);

#[derive(thiserror::Error, Debug)]
pub enum AmountParseError {
    #[error("expected amount in minor units to be greater than zero, got: {0}")]
    NotPositive(i64),
}

impl Amount {
    pub fn parse(minor_units: i64) -> Result<Self, AmountParseError> {
        match minor_units > 0 {
            true => Ok(Self(minor_units)),
            false => Err(AmountParseError::NotPositive(minor_units)),
        }
    }

    pub fn minor_units(&self) -> i64 {
        self.0
    }
}

impl Display for Amount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod test {
    use std::assert_matches;

    use super::*;

    #[test]
    fn parse_should_return_amount() {
        assert_eq!(Amount::parse(1250).unwrap().minor_units(), 1250);
    }

    #[test]
    fn parse_should_accept_the_smallest_movement() {
        assert_eq!(Amount::parse(1).unwrap().minor_units(), 1);
    }

    #[test]
    fn parse_should_return_error_when_amount_is_zero() {
        assert_matches!(
            Amount::parse(0).unwrap_err(),
            AmountParseError::NotPositive(0)
        );
    }

    #[test]
    fn parse_should_return_error_when_amount_is_negative() {
        assert_matches!(
            Amount::parse(-1250).unwrap_err(),
            AmountParseError::NotPositive(-1250)
        );
    }
}
