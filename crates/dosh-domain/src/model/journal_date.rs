use std::fmt::Display;

use chrono::NaiveDate;

/// The day a journal entry is posted on: a calendar date, no time and no zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct JournalDate(NaiveDate);

#[derive(thiserror::Error, Debug)]
pub enum JournalDateParseError {
    #[error("expected date in YYYY-MM-DD form, got: {0}")]
    InvalidFormat(String),
}

impl JournalDate {
    const FORMAT: &'static str = "%Y-%m-%d";

    pub fn parse<Input: AsRef<str>>(input: Input) -> Result<Self, JournalDateParseError> {
        let input = input.as_ref();

        NaiveDate::parse_from_str(input, Self::FORMAT)
            .map(Self)
            .map_err(|_| JournalDateParseError::InvalidFormat(input.to_string()))
    }

    pub fn date(&self) -> NaiveDate {
        self.0
    }
}

impl Display for JournalDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.format(Self::FORMAT).fmt(f)
    }
}

#[cfg(test)]
mod test {
    use std::assert_matches;

    use super::*;

    #[test]
    fn parse_should_return_date() {
        let date = JournalDate::parse("2026-08-09").unwrap();

        assert_eq!(date.date(), NaiveDate::from_ymd_opt(2026, 8, 9).unwrap());
    }

    #[test]
    fn parse_should_return_error_when_date_is_not_a_date() {
        assert_matches!(
            JournalDate::parse("the ninth").unwrap_err(),
            JournalDateParseError::InvalidFormat(input) if input == "the ninth"
        );
    }

    #[test]
    fn parse_should_return_error_when_the_day_does_not_exist() {
        assert_matches!(
            JournalDate::parse("2026-02-30").unwrap_err(),
            JournalDateParseError::InvalidFormat(input) if input == "2026-02-30"
        );
    }

    #[test]
    fn parse_should_return_error_when_the_date_carries_a_time() {
        assert_matches!(
            JournalDate::parse("2026-08-09T12:00:00Z").unwrap_err(),
            JournalDateParseError::InvalidFormat(_)
        );
    }

    #[test]
    fn prints_in_the_form_it_reads() {
        let printed = JournalDate::parse("2026-08-09").unwrap().to_string();

        assert_eq!(printed, "2026-08-09");
        assert_eq!(
            JournalDate::parse(printed).unwrap().to_string(),
            "2026-08-09"
        );
    }
}
