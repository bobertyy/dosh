use dosh_domain::model::{
    account::{AccountClass, AssetClass, ExpenseClass, LiabilityClass, RevenueClass},
    account_filter::AccountClassFilter,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountClassJson {
    Asset,
    Equity,
    Expense,
    Liability,
    Revenue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountSubclassJson {
    Bank,
    Current,
    Depreciation,
    DirectCosts,
    Fixed,
    General,
    Inventory,
    NonCurrent,
    OtherIncome,
    Overhead,
    Prepayment,
    Sales,
}

#[derive(Debug, thiserror::Error)]
pub enum AccountClassJsonError {
    #[error("this class needs a subclass")]
    MissingSubclass,
    #[error("this subclass does not belong to this class")]
    MismatchedSubclass,
}

pub fn account_class_json(class: &AccountClass) -> (AccountClassJson, Option<AccountSubclassJson>) {
    use AccountSubclassJson as Sub;

    match class {
        AccountClass::Asset(AssetClass::Bank) => (AccountClassJson::Asset, Some(Sub::Bank)),
        AccountClass::Asset(AssetClass::Current) => (AccountClassJson::Asset, Some(Sub::Current)),
        AccountClass::Asset(AssetClass::Fixed) => (AccountClassJson::Asset, Some(Sub::Fixed)),
        AccountClass::Asset(AssetClass::Inventory) => {
            (AccountClassJson::Asset, Some(Sub::Inventory))
        }
        AccountClass::Asset(AssetClass::NonCurrent) => {
            (AccountClassJson::Asset, Some(Sub::NonCurrent))
        }
        AccountClass::Asset(AssetClass::Prepayment) => {
            (AccountClassJson::Asset, Some(Sub::Prepayment))
        }
        AccountClass::Equity => (AccountClassJson::Equity, None),
        AccountClass::Expense(ExpenseClass::Depreciation) => {
            (AccountClassJson::Expense, Some(Sub::Depreciation))
        }
        AccountClass::Expense(ExpenseClass::DirectCosts) => {
            (AccountClassJson::Expense, Some(Sub::DirectCosts))
        }
        AccountClass::Expense(ExpenseClass::General) => {
            (AccountClassJson::Expense, Some(Sub::General))
        }
        AccountClass::Expense(ExpenseClass::Overhead) => {
            (AccountClassJson::Expense, Some(Sub::Overhead))
        }
        AccountClass::Liability(LiabilityClass::Current) => {
            (AccountClassJson::Liability, Some(Sub::Current))
        }
        AccountClass::Liability(LiabilityClass::NonCurrent) => {
            (AccountClassJson::Liability, Some(Sub::NonCurrent))
        }
        AccountClass::Revenue(RevenueClass::OtherIncome) => {
            (AccountClassJson::Revenue, Some(Sub::OtherIncome))
        }
        AccountClass::Revenue(RevenueClass::Sales) => (AccountClassJson::Revenue, Some(Sub::Sales)),
    }
}

pub fn parse_account_class(
    class: AccountClassJson,
    subclass: Option<AccountSubclassJson>,
) -> Result<AccountClass, AccountClassJsonError> {
    use AccountClassJson as Class;
    use AccountSubclassJson as Sub;

    Ok(match (class, subclass) {
        (Class::Asset, Some(Sub::Bank)) => AccountClass::Asset(AssetClass::Bank),
        (Class::Asset, Some(Sub::Current)) => AccountClass::Asset(AssetClass::Current),
        (Class::Asset, Some(Sub::Fixed)) => AccountClass::Asset(AssetClass::Fixed),
        (Class::Asset, Some(Sub::Inventory)) => AccountClass::Asset(AssetClass::Inventory),
        (Class::Asset, Some(Sub::NonCurrent)) => AccountClass::Asset(AssetClass::NonCurrent),
        (Class::Asset, Some(Sub::Prepayment)) => AccountClass::Asset(AssetClass::Prepayment),
        (Class::Equity, None) => AccountClass::Equity,
        (Class::Expense, Some(Sub::Depreciation)) => {
            AccountClass::Expense(ExpenseClass::Depreciation)
        }
        (Class::Expense, Some(Sub::DirectCosts)) => {
            AccountClass::Expense(ExpenseClass::DirectCosts)
        }
        (Class::Expense, Some(Sub::General)) => AccountClass::Expense(ExpenseClass::General),
        (Class::Expense, Some(Sub::Overhead)) => AccountClass::Expense(ExpenseClass::Overhead),
        (Class::Liability, Some(Sub::Current)) => AccountClass::Liability(LiabilityClass::Current),
        (Class::Liability, Some(Sub::NonCurrent)) => {
            AccountClass::Liability(LiabilityClass::NonCurrent)
        }
        (Class::Revenue, Some(Sub::OtherIncome)) => {
            AccountClass::Revenue(RevenueClass::OtherIncome)
        }
        (Class::Revenue, Some(Sub::Sales)) => AccountClass::Revenue(RevenueClass::Sales),
        (_, None) => return Err(AccountClassJsonError::MissingSubclass),
        (_, Some(_)) => return Err(AccountClassJsonError::MismatchedSubclass),
    })
}

pub fn parse_account_class_filter(
    class: AccountClassJson,
    subclass: Option<AccountSubclassJson>,
) -> Result<AccountClassFilter, AccountClassJsonError> {
    match (class, subclass) {
        (AccountClassJson::Asset, None) => Ok(AccountClassFilter::Asset(None)),
        (AccountClassJson::Expense, None) => Ok(AccountClassFilter::Expense(None)),
        (AccountClassJson::Liability, None) => Ok(AccountClassFilter::Liability(None)),
        (AccountClassJson::Revenue, None) => Ok(AccountClassFilter::Revenue(None)),
        (class, subclass) => parse_account_class(class, subclass).map(|class| (&class).into()),
    }
}

#[cfg(test)]
mod test {
    use std::assert_matches;

    use super::*;

    const CASES: [(AccountClass, AccountClassJson, Option<AccountSubclassJson>); 15] = [
        (
            AccountClass::Asset(AssetClass::Bank),
            AccountClassJson::Asset,
            Some(AccountSubclassJson::Bank),
        ),
        (
            AccountClass::Asset(AssetClass::Current),
            AccountClassJson::Asset,
            Some(AccountSubclassJson::Current),
        ),
        (
            AccountClass::Asset(AssetClass::Fixed),
            AccountClassJson::Asset,
            Some(AccountSubclassJson::Fixed),
        ),
        (
            AccountClass::Asset(AssetClass::Inventory),
            AccountClassJson::Asset,
            Some(AccountSubclassJson::Inventory),
        ),
        (
            AccountClass::Asset(AssetClass::NonCurrent),
            AccountClassJson::Asset,
            Some(AccountSubclassJson::NonCurrent),
        ),
        (
            AccountClass::Asset(AssetClass::Prepayment),
            AccountClassJson::Asset,
            Some(AccountSubclassJson::Prepayment),
        ),
        (AccountClass::Equity, AccountClassJson::Equity, None),
        (
            AccountClass::Expense(ExpenseClass::Depreciation),
            AccountClassJson::Expense,
            Some(AccountSubclassJson::Depreciation),
        ),
        (
            AccountClass::Expense(ExpenseClass::DirectCosts),
            AccountClassJson::Expense,
            Some(AccountSubclassJson::DirectCosts),
        ),
        (
            AccountClass::Expense(ExpenseClass::General),
            AccountClassJson::Expense,
            Some(AccountSubclassJson::General),
        ),
        (
            AccountClass::Expense(ExpenseClass::Overhead),
            AccountClassJson::Expense,
            Some(AccountSubclassJson::Overhead),
        ),
        (
            AccountClass::Liability(LiabilityClass::Current),
            AccountClassJson::Liability,
            Some(AccountSubclassJson::Current),
        ),
        (
            AccountClass::Liability(LiabilityClass::NonCurrent),
            AccountClassJson::Liability,
            Some(AccountSubclassJson::NonCurrent),
        ),
        (
            AccountClass::Revenue(RevenueClass::OtherIncome),
            AccountClassJson::Revenue,
            Some(AccountSubclassJson::OtherIncome),
        ),
        (
            AccountClass::Revenue(RevenueClass::Sales),
            AccountClassJson::Revenue,
            Some(AccountSubclassJson::Sales),
        ),
    ];

    const CLASS_WIRE_NAMES: [(AccountClassJson, &str); 5] = [
        (AccountClassJson::Asset, "asset"),
        (AccountClassJson::Equity, "equity"),
        (AccountClassJson::Expense, "expense"),
        (AccountClassJson::Liability, "liability"),
        (AccountClassJson::Revenue, "revenue"),
    ];

    const SUBCLASS_WIRE_NAMES: [(AccountSubclassJson, &str); 12] = [
        (AccountSubclassJson::Bank, "bank"),
        (AccountSubclassJson::Current, "current"),
        (AccountSubclassJson::Depreciation, "depreciation"),
        (AccountSubclassJson::DirectCosts, "direct_costs"),
        (AccountSubclassJson::Fixed, "fixed"),
        (AccountSubclassJson::General, "general"),
        (AccountSubclassJson::Inventory, "inventory"),
        (AccountSubclassJson::NonCurrent, "non_current"),
        (AccountSubclassJson::OtherIncome, "other_income"),
        (AccountSubclassJson::Overhead, "overhead"),
        (AccountSubclassJson::Prepayment, "prepayment"),
        (AccountSubclassJson::Sales, "sales"),
    ];

    #[test]
    fn writes_every_class_to_the_wire() {
        for (domain, class, subclass) in CASES {
            assert_eq!(account_class_json(&domain), (class, subclass));
        }
    }

    #[test]
    fn reads_every_class_back_from_the_wire() {
        for (domain, class, subclass) in CASES {
            assert_eq!(parse_account_class(class, subclass).unwrap(), domain);
        }
    }

    #[test]
    fn returns_error_when_a_class_with_subclasses_has_none() {
        assert_matches!(
            parse_account_class(AccountClassJson::Asset, None).unwrap_err(),
            AccountClassJsonError::MissingSubclass
        );
    }

    #[test]
    fn returns_error_when_a_subclass_belongs_to_another_class() {
        assert_matches!(
            parse_account_class(AccountClassJson::Asset, Some(AccountSubclassJson::Sales))
                .unwrap_err(),
            AccountClassJsonError::MismatchedSubclass
        );
    }

    #[test]
    fn returns_error_when_a_class_without_subclasses_is_given_one() {
        assert_matches!(
            parse_account_class(AccountClassJson::Equity, Some(AccountSubclassJson::Bank))
                .unwrap_err(),
            AccountClassJsonError::MismatchedSubclass
        );
    }

    #[test]
    fn serialises_every_class_and_subclass_in_snake_case() {
        for (class, wire) in CLASS_WIRE_NAMES {
            assert_eq!(
                serde_json::to_string(&class).unwrap(),
                format!("\"{wire}\"")
            );
        }

        for (subclass, wire) in SUBCLASS_WIRE_NAMES {
            assert_eq!(
                serde_json::to_string(&subclass).unwrap(),
                format!("\"{wire}\"")
            );
        }
    }

    #[test]
    fn deserialises_every_class_and_subclass_from_snake_case() {
        for (class, wire) in CLASS_WIRE_NAMES {
            assert_eq!(
                serde_json::from_str::<AccountClassJson>(&format!("\"{wire}\"")).unwrap(),
                class
            );
        }

        for (subclass, wire) in SUBCLASS_WIRE_NAMES {
            assert_eq!(
                serde_json::from_str::<AccountSubclassJson>(&format!("\"{wire}\"")).unwrap(),
                subclass
            );
        }
    }

    #[test]
    fn rejects_an_unknown_class() {
        assert!(serde_json::from_str::<AccountClassJson>("\"pizza\"").is_err());
        assert!(serde_json::from_str::<AccountSubclassJson>("\"pizza\"").is_err());
    }

    mod filter {
        use super::*;

        #[test]
        fn a_class_alone_asks_for_every_account_in_it() {
            assert_eq!(
                parse_account_class_filter(AccountClassJson::Asset, None).unwrap(),
                AccountClassFilter::Asset(None)
            );
        }

        #[test]
        fn a_class_without_subclasses_asks_for_itself() {
            assert_eq!(
                parse_account_class_filter(AccountClassJson::Equity, None).unwrap(),
                AccountClassFilter::Equity
            );
        }

        #[test]
        fn a_class_and_subclass_ask_for_that_subclass_alone() {
            assert_eq!(
                parse_account_class_filter(
                    AccountClassJson::Revenue,
                    Some(AccountSubclassJson::Sales)
                )
                .unwrap(),
                AccountClassFilter::Revenue(Some(RevenueClass::Sales))
            );
        }

        #[test]
        fn returns_error_when_a_subclass_belongs_to_another_class() {
            assert_matches!(
                parse_account_class_filter(
                    AccountClassJson::Revenue,
                    Some(AccountSubclassJson::Bank)
                )
                .unwrap_err(),
                AccountClassJsonError::MismatchedSubclass
            );
        }
    }
}
