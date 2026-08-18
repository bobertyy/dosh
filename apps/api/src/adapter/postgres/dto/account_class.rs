use dosh_domain::model::{
    account::{AccountClass, AssetClass, ExpenseClass, LiabilityClass, RevenueClass},
    account_filter::AccountClassFilter,
};

/// A stored class the schema should have made impossible.
#[derive(Debug, thiserror::Error)]
#[error("unknown stored account class: {0}")]
pub struct UnknownAccountClassPgValue(String);

/// How an [`AccountClass`] is stored in Postgres: the class, and where the
/// class has subclasses, the subclass after a dot.
///
/// The values must match the `class` check constraint on the `accounts` table.
pub fn account_class_value(class: &AccountClass) -> &'static str {
    match class {
        AccountClass::Asset(AssetClass::Bank) => "asset.bank",
        AccountClass::Asset(AssetClass::Current) => "asset.current",
        AccountClass::Asset(AssetClass::Fixed) => "asset.fixed",
        AccountClass::Asset(AssetClass::Inventory) => "asset.inventory",
        AccountClass::Asset(AssetClass::NonCurrent) => "asset.non_current",
        AccountClass::Asset(AssetClass::Prepayment) => "asset.prepayment",
        AccountClass::Equity => "equity",
        AccountClass::Expense(ExpenseClass::Depreciation) => "expense.depreciation",
        AccountClass::Expense(ExpenseClass::DirectCosts) => "expense.direct_costs",
        AccountClass::Expense(ExpenseClass::General) => "expense.general",
        AccountClass::Expense(ExpenseClass::Overhead) => "expense.overhead",
        AccountClass::Liability(LiabilityClass::Current) => "liability.current",
        AccountClass::Liability(LiabilityClass::NonCurrent) => "liability.non_current",
        AccountClass::Revenue(RevenueClass::OtherIncome) => "revenue.other_income",
        AccountClass::Revenue(RevenueClass::Sales) => "revenue.sales",
    }
}

/// The inverse of [`account_class_value`].
pub fn parse_account_class(value: &str) -> Result<AccountClass, UnknownAccountClassPgValue> {
    match value {
        "asset.bank" => Ok(AccountClass::Asset(AssetClass::Bank)),
        "asset.current" => Ok(AccountClass::Asset(AssetClass::Current)),
        "asset.fixed" => Ok(AccountClass::Asset(AssetClass::Fixed)),
        "asset.inventory" => Ok(AccountClass::Asset(AssetClass::Inventory)),
        "asset.non_current" => Ok(AccountClass::Asset(AssetClass::NonCurrent)),
        "asset.prepayment" => Ok(AccountClass::Asset(AssetClass::Prepayment)),
        "equity" => Ok(AccountClass::Equity),
        "expense.depreciation" => Ok(AccountClass::Expense(ExpenseClass::Depreciation)),
        "expense.direct_costs" => Ok(AccountClass::Expense(ExpenseClass::DirectCosts)),
        "expense.general" => Ok(AccountClass::Expense(ExpenseClass::General)),
        "expense.overhead" => Ok(AccountClass::Expense(ExpenseClass::Overhead)),
        "liability.current" => Ok(AccountClass::Liability(LiabilityClass::Current)),
        "liability.non_current" => Ok(AccountClass::Liability(LiabilityClass::NonCurrent)),
        "revenue.other_income" => Ok(AccountClass::Revenue(RevenueClass::OtherIncome)),
        "revenue.sales" => Ok(AccountClass::Revenue(RevenueClass::Sales)),
        unknown => Err(UnknownAccountClassPgValue(unknown.to_string())),
    }
}

/// The stored prefix an [`AccountClassFilter`] matches: a stored value equal to
/// it, or beginning with it and a dot.
pub fn account_class_filter_prefix(filter: &AccountClassFilter) -> &'static str {
    match filter {
        AccountClassFilter::Asset(None) => "asset",
        AccountClassFilter::Asset(Some(subclass)) => {
            account_class_value(&AccountClass::Asset(*subclass))
        }
        AccountClassFilter::Equity => "equity",
        AccountClassFilter::Expense(None) => "expense",
        AccountClassFilter::Expense(Some(subclass)) => {
            account_class_value(&AccountClass::Expense(*subclass))
        }
        AccountClassFilter::Liability(None) => "liability",
        AccountClassFilter::Liability(Some(subclass)) => {
            account_class_value(&AccountClass::Liability(*subclass))
        }
        AccountClassFilter::Revenue(None) => "revenue",
        AccountClassFilter::Revenue(Some(subclass)) => {
            account_class_value(&AccountClass::Revenue(*subclass))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const CASES: [(AccountClass, &str); 15] = [
        (AccountClass::Asset(AssetClass::Bank), "asset.bank"),
        (AccountClass::Asset(AssetClass::Current), "asset.current"),
        (AccountClass::Asset(AssetClass::Fixed), "asset.fixed"),
        (
            AccountClass::Asset(AssetClass::Inventory),
            "asset.inventory",
        ),
        (
            AccountClass::Asset(AssetClass::NonCurrent),
            "asset.non_current",
        ),
        (
            AccountClass::Asset(AssetClass::Prepayment),
            "asset.prepayment",
        ),
        (AccountClass::Equity, "equity"),
        (
            AccountClass::Expense(ExpenseClass::Depreciation),
            "expense.depreciation",
        ),
        (
            AccountClass::Expense(ExpenseClass::DirectCosts),
            "expense.direct_costs",
        ),
        (
            AccountClass::Expense(ExpenseClass::General),
            "expense.general",
        ),
        (
            AccountClass::Expense(ExpenseClass::Overhead),
            "expense.overhead",
        ),
        (
            AccountClass::Liability(LiabilityClass::Current),
            "liability.current",
        ),
        (
            AccountClass::Liability(LiabilityClass::NonCurrent),
            "liability.non_current",
        ),
        (
            AccountClass::Revenue(RevenueClass::OtherIncome),
            "revenue.other_income",
        ),
        (AccountClass::Revenue(RevenueClass::Sales), "revenue.sales"),
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

    #[test]
    fn rejects_a_stored_value_that_names_a_class_without_its_subclass() {
        assert!(parse_account_class("asset").is_err());
    }

    mod filter_prefix {
        use super::*;

        #[test]
        fn a_whole_class_is_the_class_alone() {
            assert_eq!(
                account_class_filter_prefix(&AccountClassFilter::Asset(None)),
                "asset"
            );
        }

        #[test]
        fn a_class_without_subclasses_is_its_own_stored_value() {
            assert_eq!(
                account_class_filter_prefix(&AccountClassFilter::Equity),
                "equity"
            );
        }

        #[test]
        fn one_subclass_is_the_stored_value_of_that_class() {
            for (class, stored) in CASES {
                assert_eq!(
                    account_class_filter_prefix(&AccountClassFilter::from(&class)),
                    stored
                );
            }
        }
    }
}
