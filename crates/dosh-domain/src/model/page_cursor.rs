use std::fmt::Display;

/// Where the next page starts: the position of the last item on the page just
/// read. A client receives one and hands it back untouched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageCursor(String);

#[derive(thiserror::Error, Debug)]
pub enum PageCursorParseError {
    #[error("expected cursor to be non-empty")]
    Empty,
}

impl PageCursor {
    pub fn parse<Input: Into<String>>(input: Input) -> Result<Self, PageCursorParseError> {
        let cursor = input.into();

        match cursor.is_empty() {
            true => Err(PageCursorParseError::Empty),
            false => Ok(Self(cursor)),
        }
    }
}

/// An item a collection can be paged through with a keyset cursor.
///
/// The implementor designates the value its collection is ordered by — an
/// account's code, elsewhere something else — and says how a cursor names one.
/// Paging needs nothing more: with a key it can write a cursor, and with a
/// cursor it can read the key back.
pub trait Pageable {
    /// The value that positions an item in the collection.
    type Key: Display;

    /// What a cursor that names no key comes back as.
    type KeyError;

    /// Where this item sits in the collection.
    fn page_key(&self) -> &Self::Key;

    /// The key `cursor` names, or an error if it names none.
    fn key_from_cursor(cursor: &PageCursor) -> Result<Self::Key, Self::KeyError>;
}

/// The cursor to the page after an item is that item's own position.
impl<Item: Pageable> From<&Item> for PageCursor {
    fn from(item: &Item) -> Self {
        Self(item.page_key().to_string())
    }
}

impl Display for PageCursor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod test {
    use std::assert_matches;

    use super::*;

    struct Numbered(u32);

    #[derive(Debug)]
    struct NotANumber;

    impl Pageable for Numbered {
        type Key = u32;
        type KeyError = NotANumber;

        fn page_key(&self) -> &u32 {
            &self.0
        }

        fn key_from_cursor(cursor: &PageCursor) -> Result<u32, NotANumber> {
            cursor.to_string().parse().map_err(|_| NotANumber)
        }
    }

    #[test]
    fn parse_should_return_cursor() {
        assert_eq!(PageCursor::parse("200").unwrap().to_string(), "200");
    }

    #[test]
    fn parse_should_return_error_when_cursor_is_empty() {
        assert_matches!(
            PageCursor::parse("").unwrap_err(),
            PageCursorParseError::Empty
        );
    }

    #[test]
    fn is_built_from_the_key_of_a_pageable_item() {
        assert_eq!(
            PageCursor::from(&Numbered(200)),
            PageCursor::parse("200").unwrap()
        );
    }

    #[test]
    fn reads_the_key_back_out_of_a_cursor() {
        let cursor = PageCursor::from(&Numbered(200));

        assert_eq!(Numbered::key_from_cursor(&cursor).unwrap(), 200);
    }

    #[test]
    fn reports_a_cursor_that_names_no_key() {
        let cursor = PageCursor::parse("not a key").unwrap();

        assert_matches!(Numbered::key_from_cursor(&cursor), Err(NotANumber));
    }
}
