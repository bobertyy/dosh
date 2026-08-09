#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageLimit(u32);

#[derive(thiserror::Error, Debug)]
pub enum PageLimitParseError {
    #[error("expected limit between {min} and {max}, got: {got}")]
    OutOfRange { min: u32, max: u32, got: u32 },
}

impl PageLimit {
    pub const MIN: u32 = 1;
    pub const MAX: u32 = 100;
    pub const DEFAULT: u32 = 20;

    pub fn parse(limit: u32) -> Result<Self, PageLimitParseError> {
        match (Self::MIN..=Self::MAX).contains(&limit) {
            true => Ok(Self(limit)),
            false => Err(PageLimitParseError::OutOfRange {
                min: Self::MIN,
                max: Self::MAX,
                got: limit,
            }),
        }
    }

    pub fn get(&self) -> u32 {
        self.0
    }
}

impl Default for PageLimit {
    fn default() -> Self {
        Self(Self::DEFAULT)
    }
}

#[cfg(test)]
mod test {
    use std::assert_matches;

    use super::*;

    #[test]
    fn parse_should_return_limit() {
        assert_eq!(PageLimit::parse(25).unwrap().get(), 25);
    }

    #[test]
    fn parse_should_accept_the_bounds() {
        assert_eq!(PageLimit::parse(PageLimit::MIN).unwrap().get(), 1);
        assert_eq!(PageLimit::parse(PageLimit::MAX).unwrap().get(), 100);
    }

    #[test]
    fn parse_should_return_error_when_limit_is_zero() {
        assert_matches!(
            PageLimit::parse(0).unwrap_err(),
            PageLimitParseError::OutOfRange { got: 0, .. }
        );
    }

    #[test]
    fn parse_should_return_error_when_limit_is_above_the_maximum() {
        assert_matches!(
            PageLimit::parse(101).unwrap_err(),
            PageLimitParseError::OutOfRange { got: 101, .. }
        );
    }

    #[test]
    fn defaults_to_a_modest_page() {
        assert_eq!(PageLimit::default().get(), PageLimit::DEFAULT);
    }
}
