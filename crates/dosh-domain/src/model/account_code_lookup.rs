use super::account_code::AccountCode;

/// The answer to asking after a set of account codes: the ones naming an
/// account, and the ones naming none. Each half names a code once and is
/// ordered by code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountCodeLookup {
    found: Vec<AccountCode>,
    missing: Vec<AccountCode>,
}

impl AccountCodeLookup {
    /// Splits the codes `asked_after` by whether `existing` names them.
    pub fn split(asked_after: &[AccountCode], existing: &[AccountCode]) -> Self {
        let mut asked_after = asked_after.to_vec();
        asked_after.sort();
        asked_after.dedup();

        let (found, missing) = asked_after
            .into_iter()
            .partition(|code| existing.contains(code));

        Self { found, missing }
    }

    pub fn found(&self) -> &[AccountCode] {
        &self.found
    }

    pub fn missing(&self) -> &[AccountCode] {
        &self.missing
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn codes(codes: &[&str]) -> Vec<AccountCode> {
        codes
            .iter()
            .map(|code| AccountCode::parse(*code).unwrap())
            .collect()
    }

    #[test]
    fn splits_the_codes_asked_after() {
        let lookup = AccountCodeLookup::split(&codes(&["100", "999"]), &codes(&["100"]));

        assert_eq!(lookup.found(), codes(&["100"]));
        assert_eq!(lookup.missing(), codes(&["999"]));
    }

    #[test]
    fn orders_each_half_by_code() {
        let lookup = AccountCodeLookup::split(
            &codes(&["999", "110", "998", "100"]),
            &codes(&["110", "100"]),
        );

        assert_eq!(lookup.found(), codes(&["100", "110"]));
        assert_eq!(lookup.missing(), codes(&["998", "999"]));
    }

    #[test]
    fn names_a_code_once_however_often_it_was_asked_after() {
        let lookup =
            AccountCodeLookup::split(&codes(&["100", "100", "999", "999"]), &codes(&["100"]));

        assert_eq!(lookup.found(), codes(&["100"]));
        assert_eq!(lookup.missing(), codes(&["999"]));
    }

    #[test]
    fn leaves_out_an_existing_code_that_was_not_asked_after() {
        let lookup = AccountCodeLookup::split(&codes(&["100"]), &codes(&["100", "200"]));

        assert_eq!(lookup.found(), codes(&["100"]));
        assert!(lookup.missing().is_empty());
    }

    #[test]
    fn answers_for_no_codes_at_all() {
        let lookup = AccountCodeLookup::split(&[], &codes(&["100"]));

        assert!(lookup.found().is_empty());
        assert!(lookup.missing().is_empty());
    }
}
