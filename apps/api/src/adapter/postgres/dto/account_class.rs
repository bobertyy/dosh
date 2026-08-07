use dosh_domain::model::account::AccountClass;

/// How an [`AccountClass`] is stored in Postgres.
///
/// The values must match the `class` check constraint on the `accounts` table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccountClassPgValue(&'static str);

impl AccountClassPgValue {
    pub fn value(&self) -> &'static str {
        self.0
    }
}

impl From<&AccountClass> for AccountClassPgValue {
    fn from(class: &AccountClass) -> Self {
        Self(match class {
            AccountClass::Asset => "asset",
            AccountClass::Equity => "equity",
            AccountClass::Expense => "expense",
            AccountClass::Liability => "liability",
            AccountClass::Revenue => "revenue",
        })
    }
}

impl From<AccountClass> for AccountClassPgValue {
    fn from(class: AccountClass) -> Self {
        Self::from(&class)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn maps_every_class_to_its_stored_value() {
        let cases = [
            (AccountClass::Asset, "asset"),
            (AccountClass::Equity, "equity"),
            (AccountClass::Expense, "expense"),
            (AccountClass::Liability, "liability"),
            (AccountClass::Revenue, "revenue"),
        ];

        for (class, expected) in cases {
            assert_eq!(AccountClassPgValue::from(class).value(), expected);
        }
    }
}
