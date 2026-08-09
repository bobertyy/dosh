use crate::model::{account::AccountClass, account_code::AccountCodePrefix};

/// Which accounts a caller is interested in. A field left out does not filter,
/// so the default filter matches every account.
#[derive(Debug, Default, PartialEq)]
pub struct AccountFilter {
    class: Option<AccountClass>,
    code_starts_with: Option<AccountCodePrefix>,
}

impl AccountFilter {
    pub fn new(class: Option<AccountClass>, code_starts_with: Option<AccountCodePrefix>) -> Self {
        Self {
            class,
            code_starts_with,
        }
    }

    pub fn class(&self) -> Option<&AccountClass> {
        self.class.as_ref()
    }

    pub fn code_starts_with(&self) -> Option<&AccountCodePrefix> {
        self.code_starts_with.as_ref()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn the_default_filter_matches_everything() {
        let filter = AccountFilter::default();

        assert_eq!(filter.class(), None);
        assert_eq!(filter.code_starts_with(), None);
    }

    #[test]
    fn holds_the_criteria_it_was_built_with() {
        let filter = AccountFilter::new(
            Some(AccountClass::Revenue),
            Some(AccountCodePrefix::parse("2").unwrap()),
        );

        assert_eq!(filter.class(), Some(&AccountClass::Revenue));
        assert_eq!(
            filter.code_starts_with(),
            Some(&AccountCodePrefix::parse("2").unwrap())
        );
    }
}
