use dosh_domain::model::account::AccountClass;
use serde::{Deserialize, Serialize};

/// How an [`AccountClass`] is represented on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AccountClassJson {
    Asset,
    Equity,
    Expense,
    Liability,
    Revenue,
}

impl From<AccountClassJson> for AccountClass {
    fn from(class: AccountClassJson) -> Self {
        match class {
            AccountClassJson::Asset => Self::Asset,
            AccountClassJson::Equity => Self::Equity,
            AccountClassJson::Expense => Self::Expense,
            AccountClassJson::Liability => Self::Liability,
            AccountClassJson::Revenue => Self::Revenue,
        }
    }
}

impl From<&AccountClass> for AccountClassJson {
    fn from(class: &AccountClass) -> Self {
        match class {
            AccountClass::Asset => Self::Asset,
            AccountClass::Equity => Self::Equity,
            AccountClass::Expense => Self::Expense,
            AccountClass::Liability => Self::Liability,
            AccountClass::Revenue => Self::Revenue,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const CASES: [(AccountClassJson, AccountClass, &str); 5] = [
        (AccountClassJson::Asset, AccountClass::Asset, "\"asset\""),
        (AccountClassJson::Equity, AccountClass::Equity, "\"equity\""),
        (
            AccountClassJson::Expense,
            AccountClass::Expense,
            "\"expense\"",
        ),
        (
            AccountClassJson::Liability,
            AccountClass::Liability,
            "\"liability\"",
        ),
        (
            AccountClassJson::Revenue,
            AccountClass::Revenue,
            "\"revenue\"",
        ),
    ];

    #[test]
    fn maps_every_class_to_the_domain() {
        for (json, domain, _) in CASES {
            assert_eq!(AccountClass::from(json), domain);
        }
    }

    #[test]
    fn maps_every_class_from_the_domain() {
        for (json, domain, _) in CASES {
            assert_eq!(AccountClassJson::from(&domain), json);
        }
    }

    #[test]
    fn serialises_every_class_in_lowercase() {
        for (json, _, expected) in CASES {
            assert_eq!(serde_json::to_string(&json).unwrap(), expected);
        }
    }

    #[test]
    fn deserialises_every_class_from_lowercase() {
        for (json, _, encoded) in CASES {
            assert_eq!(
                serde_json::from_str::<AccountClassJson>(encoded).unwrap(),
                json
            );
        }
    }

    #[test]
    fn rejects_an_unknown_class() {
        assert!(serde_json::from_str::<AccountClassJson>("\"pizza\"").is_err());
    }
}
