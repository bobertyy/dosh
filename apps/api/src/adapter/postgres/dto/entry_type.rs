use dosh_domain::model::journal_line_item::EntryType;

/// How an [`EntryType`] is stored. The values must match the `entry_type`
/// check constraint on the `journal_line_items` table.
pub fn entry_type_value(entry_type: &EntryType) -> &'static str {
    match entry_type {
        EntryType::Credit => "credit",
        EntryType::Debit => "debit",
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn maps_every_entry_type_to_its_stored_value() {
        assert_eq!(entry_type_value(&EntryType::Credit), "credit");
        assert_eq!(entry_type_value(&EntryType::Debit), "debit");
    }
}
