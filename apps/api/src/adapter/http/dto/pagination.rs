use dosh_domain::model::page::Page;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct PaginationJson {
    pub limit: u32,
    pub has_more: bool,
    pub next_page_cursor: Option<String>,
}

impl<Item> From<&Page<Item>> for PaginationJson {
    fn from(page: &Page<Item>) -> Self {
        Self {
            limit: page.limit().get(),
            has_more: page.has_more(),
            next_page_cursor: page.next_cursor().map(ToString::to_string),
        }
    }
}

#[cfg(test)]
mod test {
    use dosh_domain::model::{page_cursor::PageCursor, page_limit::PageLimit};

    use super::*;

    #[test]
    fn reports_the_cursor_of_a_page_that_has_more() {
        let page = Page::new(
            vec!["100", "110"],
            PageLimit::parse(2).unwrap(),
            Some(PageCursor::parse("110").unwrap()),
        );

        assert_eq!(
            PaginationJson::from(&page),
            PaginationJson {
                limit: 2,
                has_more: true,
                next_page_cursor: Some("110".to_string()),
            }
        );
    }

    #[test]
    fn reports_no_cursor_on_the_last_page() {
        let page = Page::new(vec!["100"], PageLimit::parse(2).unwrap(), None);

        assert_eq!(
            PaginationJson::from(&page),
            PaginationJson {
                limit: 2,
                has_more: false,
                next_page_cursor: None,
            }
        );
    }

    #[test]
    fn reports_the_limit_the_page_was_read_under_when_it_is_empty() {
        let page: Page<&str> = Page::new(Vec::new(), PageLimit::parse(20).unwrap(), None);

        assert_eq!(
            PaginationJson::from(&page),
            PaginationJson {
                limit: 20,
                has_more: false,
                next_page_cursor: None,
            }
        );
    }
}
