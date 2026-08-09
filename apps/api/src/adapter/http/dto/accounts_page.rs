use dosh_domain::model::{account::Account, page::Page};
use serde::{Deserialize, Serialize};

use crate::adapter::http::dto::{account::AccountJson, pagination::PaginationJson};

/// A page of accounts as returned to a client.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct AccountsPageJson {
    pub accounts: Vec<AccountJson>,
    pub pagination: PaginationJson,
}

impl From<&Page<Account>> for AccountsPageJson {
    fn from(page: &Page<Account>) -> Self {
        Self {
            accounts: page.items().iter().map(AccountJson::from).collect(),
            pagination: PaginationJson::from(page),
        }
    }
}

#[cfg(test)]
mod test {
    use dosh_domain::model::{
        account::AccountClass, account_code::AccountCode, page_cursor::PageCursor,
        page_limit::PageLimit,
    };

    use crate::adapter::http::dto::account_class::AccountClassJson;

    use super::*;

    fn account(code: &str) -> Account {
        Account::new(AccountCode::parse(code).unwrap(), AccountClass::Asset)
    }

    #[test]
    fn maps_the_accounts_and_the_paging_of_a_page() {
        let page = Page::new(
            vec![account("100"), account("110")],
            PageLimit::parse(2).unwrap(),
            Some(PageCursor::parse("110").unwrap()),
        );

        assert_eq!(
            AccountsPageJson::from(&page),
            AccountsPageJson {
                accounts: vec![
                    AccountJson {
                        code: "100".to_string(),
                        class: AccountClassJson::Asset,
                        description: None,
                    },
                    AccountJson {
                        code: "110".to_string(),
                        class: AccountClassJson::Asset,
                        description: None,
                    },
                ],
                pagination: PaginationJson {
                    limit: 2,
                    has_more: true,
                    next_page_cursor: Some("110".to_string()),
                },
            }
        );
    }

    #[test]
    fn serialises_an_empty_page_as_an_empty_list() {
        let page = Page::new(Vec::new(), PageLimit::parse(20).unwrap(), None);

        assert_eq!(
            serde_json::to_string(&AccountsPageJson::from(&page)).unwrap(),
            r#"{"accounts":[],"pagination":{"limit":20,"has_more":false,"next_page_cursor":null}}"#
        );
    }
}
