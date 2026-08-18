use crate::model::{
    account::{AccountClass, AssetClass, ExpenseClass, LiabilityClass, RevenueClass},
    account_code::AccountCodePrefix,
};

/// Which classes a caller is interested in: a whole class, or one subclass of
/// it. A class that has no subclasses names itself.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AccountClassFilter {
    Asset(Option<AssetClass>),
    Equity,
    Expense(Option<ExpenseClass>),
    Liability(Option<LiabilityClass>),
    Revenue(Option<RevenueClass>),
}

impl From<&AccountClass> for AccountClassFilter {
    fn from(class: &AccountClass) -> Self {
        match class {
            AccountClass::Asset(subclass) => Self::Asset(Some(*subclass)),
            AccountClass::Equity => Self::Equity,
            AccountClass::Expense(subclass) => Self::Expense(Some(*subclass)),
            AccountClass::Liability(subclass) => Self::Liability(Some(*subclass)),
            AccountClass::Revenue(subclass) => Self::Revenue(Some(*subclass)),
        }
    }
}

/// Which accounts a caller is interested in. A field left out does not filter,
/// so the default filter matches every account.
#[derive(Debug, Default, PartialEq)]
pub struct AccountFilter {
    class: Option<AccountClassFilter>,
    code_starts_with: Option<AccountCodePrefix>,
}

impl AccountFilter {
    pub fn new(
        class: Option<AccountClassFilter>,
        code_starts_with: Option<AccountCodePrefix>,
    ) -> Self {
        Self {
            class,
            code_starts_with,
        }
    }

    pub fn class(&self) -> Option<&AccountClassFilter> {
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
            Some(AccountClassFilter::Revenue(Some(RevenueClass::Sales))),
            Some(AccountCodePrefix::parse("2").unwrap()),
        );

        assert_eq!(
            filter.class(),
            Some(&AccountClassFilter::Revenue(Some(RevenueClass::Sales)))
        );
        assert_eq!(
            filter.code_starts_with(),
            Some(&AccountCodePrefix::parse("2").unwrap())
        );
    }

    mod from_a_class {
        use super::*;

        #[test]
        fn narrows_to_that_class_and_subclass() {
            assert_eq!(
                AccountClassFilter::from(&AccountClass::Asset(AssetClass::Bank)),
                AccountClassFilter::Asset(Some(AssetClass::Bank))
            );
        }

        #[test]
        fn narrows_to_a_class_that_has_no_subclasses() {
            assert_eq!(
                AccountClassFilter::from(&AccountClass::Equity),
                AccountClassFilter::Equity
            );
        }
    }
}
