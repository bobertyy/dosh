use dosh_domain::model::journal_line_item::EntryType;
use serde::{Deserialize, Serialize};

/// How an [`EntryType`] is represented on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryTypeJson {
    Credit,
    Debit,
}

impl From<EntryTypeJson> for EntryType {
    fn from(entry_type: EntryTypeJson) -> Self {
        match entry_type {
            EntryTypeJson::Credit => Self::Credit,
            EntryTypeJson::Debit => Self::Debit,
        }
    }
}

impl From<&EntryType> for EntryTypeJson {
    fn from(entry_type: &EntryType) -> Self {
        match entry_type {
            EntryType::Credit => Self::Credit,
            EntryType::Debit => Self::Debit,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const CASES: [(EntryTypeJson, EntryType, &str); 2] = [
        (EntryTypeJson::Credit, EntryType::Credit, "\"credit\""),
        (EntryTypeJson::Debit, EntryType::Debit, "\"debit\""),
    ];

    #[test]
    fn maps_every_entry_type_to_the_domain() {
        for (json, domain, _) in CASES {
            assert_eq!(EntryType::from(json), domain);
        }
    }

    #[test]
    fn maps_every_entry_type_from_the_domain() {
        for (json, domain, _) in CASES {
            assert_eq!(EntryTypeJson::from(&domain), json);
        }
    }

    #[test]
    fn serialises_every_entry_type_in_lowercase() {
        for (json, _, expected) in CASES {
            assert_eq!(serde_json::to_string(&json).unwrap(), expected);
        }
    }

    #[test]
    fn deserialises_every_entry_type_from_lowercase() {
        for (json, _, encoded) in CASES {
            assert_eq!(
                serde_json::from_str::<EntryTypeJson>(encoded).unwrap(),
                json
            );
        }
    }

    #[test]
    fn rejects_an_unknown_entry_type() {
        assert!(serde_json::from_str::<EntryTypeJson>("\"sideways\"").is_err());
    }
}
