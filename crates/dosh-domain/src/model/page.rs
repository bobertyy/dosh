use crate::model::{page_cursor::PageCursor, page_limit::PageLimit};

#[derive(Debug, PartialEq)]
pub struct Page<Item> {
    items: Vec<Item>,
    limit: PageLimit,
    next_cursor: Option<PageCursor>,
}

impl<Item> Page<Item> {
    pub fn new(items: Vec<Item>, limit: PageLimit, next_cursor: Option<PageCursor>) -> Self {
        Self {
            items,
            limit,
            next_cursor,
        }
    }

    pub fn items(&self) -> &[Item] {
        &self.items
    }

    /// The page size the results were read under, not the number of results.
    pub fn limit(&self) -> PageLimit {
        self.limit
    }

    pub fn next_cursor(&self) -> Option<&PageCursor> {
        self.next_cursor.as_ref()
    }

    pub fn has_more(&self) -> bool {
        self.next_cursor.is_some()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn a_page_with_a_cursor_has_more() {
        let page = Page::new(
            vec!["100"],
            PageLimit::default(),
            Some(PageCursor::parse("100").unwrap()),
        );

        assert!(page.has_more());
        assert_eq!(page.next_cursor(), Some(&PageCursor::parse("100").unwrap()));
    }

    #[test]
    fn a_page_without_a_cursor_is_the_last_one() {
        let page = Page::new(vec!["100"], PageLimit::default(), None);

        assert!(!page.has_more());
        assert_eq!(page.next_cursor(), None);
    }

    #[test]
    fn an_empty_page_keeps_its_limit() {
        let page: Page<&str> = Page::new(Vec::new(), PageLimit::parse(5).unwrap(), None);

        assert!(page.items().is_empty());
        assert_eq!(page.limit(), PageLimit::parse(5).unwrap());
    }
}
