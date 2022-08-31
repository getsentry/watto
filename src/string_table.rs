use core::str::Utf8Error;
use std::collections::HashMap;

use thiserror::Error;

/// An error when trying to read a string from a serialized [`StringTable`].
#[derive(Debug, Error)]
pub enum ReadStringError {
    /// The string's length prefix is not valid LEB128.
    #[error("error reading LEB128 encoded number")]
    Leb128(#[from] leb128::read::Error),
    /// The string data is not valid UTF-8.
    #[error("error reading UTF-8 string data")]
    Utf8(#[from] Utf8Error),
    /// The string's offset or length is outside the bounds of the data blob.
    #[error("string offset or length is out of bounds")]
    OutOfBounds,
}
/// A struct for storing strings without duplicates.
///
/// Add strings to the table with [`insert`](StringTable::insert). The
/// returned `usize` can be used to [`read`](StringTable::read) the string
/// back from the table's serialized representation.
///
/// # Example
/// ```
/// use watto::StringTable;
/// let mut table = StringTable::new();
/// let foo_idx = table.insert("foo");
/// let bar_idx = table.insert("bar");
/// let string_bytes = table.as_bytes();
/// assert_eq!(StringTable::read(string_bytes, foo_idx).unwrap(), "foo");
/// assert_eq!(StringTable::read(string_bytes, bar_idx).unwrap(), "bar");
/// ```
#[derive(Debug, Clone, Default)]
pub struct StringTable {
    strings: HashMap<String, usize>,
    bytes: Vec<u8>,
}

impl StringTable {
    /// Initializes an empty `StringTable`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a string into this `StringTable`.
    ///
    /// Returns an offset that can be used to retrieve the inserted string
    /// with [`read`](Self::read) after serializing this table with [`as_bytes`](Self::as_bytes).
    pub fn insert(&mut self, s: &str) -> usize {
        let Self {
            ref mut strings,
            ref mut bytes,
        } = self;
        if s.is_empty() {
            return usize::MAX;
        }
        if let Some(&offset) = strings.get(s) {
            return offset;
        }
        let string_offset = bytes.len() as usize;
        let string_len = s.len() as u64;
        leb128::write::unsigned(bytes, string_len).unwrap();
        bytes.extend(s.bytes());

        strings.insert(s.to_owned(), string_offset);
        string_offset
    }

    /// Returns a byte slice containing the concatenation of the strings that have been
    /// added to this `StringTable`.
    ///
    /// Each string is prefixed with its length in [LEB128 encoding](https://en.wikipedia.org/wiki/LEB128).
    ///
    /// Strings can be looked up using the `usize` returned by [`insert`](Self::insert).
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the string stored at the given offset in the byte slice, if any.
    ///
    /// Use this to retrieve a string that was previously [inserted](StringTable::insert) into a `StringTable`.
    pub fn read(string_bytes: &[u8], offset: usize) -> Result<&str, ReadStringError> {
        let reader = &mut string_bytes
            .get(offset as usize..)
            .ok_or(ReadStringError::OutOfBounds)?;
        let len = leb128::read::unsigned(reader)? as usize;

        let bytes = reader.get(..len).ok_or(ReadStringError::OutOfBounds)?;

        std::str::from_utf8(bytes).map_err(ReadStringError::from)
    }
}
