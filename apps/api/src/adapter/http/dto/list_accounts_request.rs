use dosh_domain::{
    model::{
        account_code::{AccountCodePrefix, AccountCodePrefixParseError},
        account_filter::AccountFilter,
        page_cursor::{PageCursor, PageCursorParseError},
        page_limit::{PageLimit, PageLimitParseError},
    },
    use_case::list_accounts::ListAccountsQuery,
};
use serde::{Deserialize, Serialize};

use crate::adapter::http::dto::account_class::AccountClassJson;

/// The query string of `GET /accounts`. Every parameter is optional: with none
/// of them the client gets the first page of every account.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ListAccountsRequest {
    #[serde(default)]
    pub limit: Option<u32>,
    #[serde(default)]
    pub page_cursor: Option<String>,
    /// The wire calls a class a type; the domain calls it a class.
    #[serde(default)]
    pub account_type: Option<AccountClassJson>,
    /// Bracketed so more account_code operators can join it later:
    /// `?account_code[starts_with]=2`.
    #[serde(default, rename = "account_code[starts_with]")]
    pub account_code_starts_with: Option<String>,
}

/// A query string that deserialised cleanly but does not describe a page the
/// domain will serve.
#[derive(Debug, thiserror::Error)]
pub enum ListAccountsRequestError {
    #[error(transparent)]
    Limit(#[from] PageLimitParseError),
    #[error(transparent)]
    Cursor(#[from] PageCursorParseError),
    #[error(transparent)]
    CodePrefix(#[from] AccountCodePrefixParseError),
}

impl TryFrom<ListAccountsRequest> for ListAccountsQuery {
    type Error = ListAccountsRequestError;

    fn try_from(request: ListAccountsRequest) -> Result<Self, Self::Error> {
        let limit = request
            .limit
            .map(PageLimit::parse)
            .transpose()?
            .unwrap_or_default();

        let cursor = request.page_cursor.map(PageCursor::parse).transpose()?;

        let code_starts_with = request
            .account_code_starts_with
            .map(AccountCodePrefix::parse)
            .transpose()?;

        let filter = AccountFilter::new(request.account_type.map(Into::into), code_starts_with);

        Ok(ListAccountsQuery::new(filter, cursor, limit))
    }
}

#[cfg(test)]
mod test {
    use std::assert_matches;

    use dosh_domain::model::account::AccountClass;

    use super::*;

    #[test]
    fn maps_an_empty_request_to_the_first_page_of_everything() {
        let query = ListAccountsQuery::try_from(ListAccountsRequest::default()).unwrap();

        assert_eq!(query, ListAccountsQuery::default());
        assert_eq!(query.limit(), PageLimit::default());
        assert_eq!(query.cursor(), None);
        assert_eq!(query.filter().class(), None);
        assert_eq!(query.filter().code_starts_with(), None);
    }

    #[test]
    fn maps_a_fully_specified_request() {
        let query = ListAccountsQuery::try_from(ListAccountsRequest {
            limit: Some(5),
            page_cursor: Some("200".to_string()),
            account_type: Some(AccountClassJson::Revenue),
            account_code_starts_with: Some("2".to_string()),
        })
        .unwrap();

        assert_eq!(query.limit(), PageLimit::parse(5).unwrap());
        assert_eq!(query.cursor(), Some(&PageCursor::parse("200").unwrap()));
        assert_eq!(query.filter().class(), Some(&AccountClass::Revenue));
        assert_eq!(
            query.filter().code_starts_with(),
            Some(&AccountCodePrefix::parse("2").unwrap())
        );
    }

    #[test]
    fn returns_error_when_the_limit_is_out_of_range() {
        let error = ListAccountsQuery::try_from(ListAccountsRequest {
            limit: Some(0),
            ..Default::default()
        })
        .unwrap_err();

        assert_matches!(error, ListAccountsRequestError::Limit(_));
    }

    #[test]
    fn returns_error_when_the_cursor_is_empty() {
        let error = ListAccountsQuery::try_from(ListAccountsRequest {
            page_cursor: Some("".to_string()),
            ..Default::default()
        })
        .unwrap_err();

        assert_matches!(error, ListAccountsRequestError::Cursor(_));
    }

    #[test]
    fn returns_error_when_the_code_prefix_is_not_a_prefix() {
        let error = ListAccountsQuery::try_from(ListAccountsRequest {
            account_code_starts_with: Some("2%".to_string()),
            ..Default::default()
        })
        .unwrap_err();

        assert_matches!(error, ListAccountsRequestError::CodePrefix(_));
    }

    #[test]
    fn deserialises_every_parameter() {
        let request: ListAccountsRequest = serde_urlencoded::from_str(
            "limit=5&page_cursor=200&account_type=revenue&account_code%5Bstarts_with%5D=2",
        )
        .unwrap();

        assert_eq!(
            request,
            ListAccountsRequest {
                limit: Some(5),
                page_cursor: Some("200".to_string()),
                account_type: Some(AccountClassJson::Revenue),
                account_code_starts_with: Some("2".to_string()),
            }
        );
    }

    #[test]
    fn deserialises_an_empty_query_string() {
        let request: ListAccountsRequest = serde_urlencoded::from_str("").unwrap();

        assert_eq!(request, ListAccountsRequest::default());
    }

    #[test]
    fn rejects_a_limit_that_is_not_a_number() {
        assert!(serde_urlencoded::from_str::<ListAccountsRequest>("limit=lots").is_err());
    }
}
