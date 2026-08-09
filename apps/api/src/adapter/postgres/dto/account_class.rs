use dosh_domain::model::account::AccountClass;

/// A stored class the schema should have made impossible.
#[derive(Debug, thiserror::Error)]
#[error("unknown stored account class: {0}")]
pub struct UnknownAccountClassPgValue(String);

/// How an [`AccountClass`] is stored in Postgres.
///
/// The values must match the `class` check constraint on the `accounts` table.
pub fn account_class_value(class: &AccountClass) -> &'static str {
    match class {
        AccountClass::Asset => "asset",
        AccountClass::Equity => "equity",
        AccountClass::Expense => "expense",
        AccountClass::Liability => "liability",
        AccountClass::Revenue => "revenue",
    }
}

/// The inverse of [`account_class_value`].
pub fn parse_account_class(value: &str) -> Result<AccountClass, UnknownAccountClassPgValue> {
    match value {
        "asset" => Ok(AccountClass::Asset),
        "equity" => Ok(AccountClass::Equity),
        "expense" => Ok(AccountClass::Expense),
        "liability" => Ok(AccountClass::Liability),
        "revenue" => Ok(AccountClass::Revenue),
        unknown => Err(UnknownAccountClassPgValue(unknown.to_string())),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const CASES: [(AccountClass, &str); 5] = [
        (AccountClass::Asset, "asset"),
        (AccountClass::Equity, "equity"),
        (AccountClass::Expense, "expense"),
        (AccountClass::Liability, "liability"),
        (AccountClass::Revenue, "revenue"),
    ];

    #[test]
    fn maps_every_class_to_its_stored_value() {
        for (class, expected) in CASES {
            assert_eq!(account_class_value(&class), expected);
        }
    }

    #[test]
    fn maps_every_stored_value_back_to_its_class() {
        for (class, stored) in CASES {
            assert_eq!(parse_account_class(stored).unwrap(), class);
        }
    }

    #[test]
    fn rejects_a_stored_value_it_does_not_know() {
        assert!(parse_account_class("pizza").is_err());
    }
}
