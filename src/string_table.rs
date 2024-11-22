use core::str::Utf8Error;
use std::collections::HashMap;
use std::io::Cursor;

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
/// back from the table's byte representation.
///
/// Use [`as_bytes`](StringTable::as_bytes) to obtain a byte representation of a `StringTable`.
/// The byte representation is the concatenation of the strings that have been added to the table,
/// with each individual string prefixed with its length in [LEB128 encoding](https://en.wikipedia.org/wiki/LEB128).
/// The byte representation contains each string only once.
///
/// # Example
/// ```
/// use watto::StringTable;
///
/// let mut table = StringTable::new();
/// let foo_offset = table.insert("foo");
/// let bar_offset = table.insert("bar");
///
/// let string_bytes = table.as_bytes();
/// assert_eq!(StringTable::read(string_bytes, foo_offset).unwrap(), "foo");
/// assert_eq!(StringTable::read(string_bytes, bar_offset).unwrap(), "bar");
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

    /// Initializes a [`StringTable`] from a previously serialized representation.
    ///
    /// This essentially reverses the [`as_bytes`](Self::as_bytes) call.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ReadStringError> {
        let mut rest = bytes;

        let mut string_offset = 0;
        let mut strings: HashMap<_, _> = Default::default();
        while !rest.is_empty() {
            let mut cursor = Cursor::new(rest);
            let len = leb128::read::unsigned(&mut cursor)? as usize;
            // it would be nice if `leb128` would directly return this as well,
            // so one wouldn't have to use a `Cursor`.
            let leb_len = cursor.position() as usize;

            let (string_bytes, new_rest) = rest
                .split_at_checked(leb_len + len)
                .ok_or(ReadStringError::OutOfBounds)?;

            let string =
                std::str::from_utf8(&string_bytes[leb_len..]).map_err(ReadStringError::from)?;
            strings.insert(string.to_owned(), string_offset);

            string_offset += leb_len + len;
            rest = new_rest;
        }

        Ok(Self {
            strings,
            bytes: bytes.into(),
        })
    }

    /// Insert a string into this `StringTable`.
    ///
    /// Returns an offset that can be used to retrieve the inserted string
    /// with [`read`](Self::read) after serializing this table with [`as_bytes`](Self::as_bytes).
    pub fn insert(&mut self, s: &str) -> usize {
        if let Some(&offset) = self.strings.get(s) {
            return offset;
        }
        let string_offset = self.bytes.len();
        let string_len = s.len() as u64;
        leb128::write::unsigned(&mut self.bytes, string_len).unwrap();
        self.bytes.extend_from_slice(s.as_bytes());

        self.strings.insert(s.to_owned(), string_offset);
        string_offset
    }

    /// Returns a byte slice containing the concatenation of the strings that have been
    /// added to this `StringTable`.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns a byte vector containing the concatenation of the strings that have been
    /// added to this `StringTable`.
    ///
    /// This consumes the `StringTable`.
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    /// Returns the string stored at the given offset in the byte slice, if any.
    ///
    /// Use this to retrieve a string that was previously [inserted](StringTable::insert) into a `StringTable`.
    pub fn read(string_bytes: &[u8], offset: usize) -> Result<&str, ReadStringError> {
        let reader = &mut string_bytes
            .get(offset..)
            .ok_or(ReadStringError::OutOfBounds)?;
        let len = leb128::read::unsigned(reader)? as usize;

        let bytes = reader.get(..len).ok_or(ReadStringError::OutOfBounds)?;

        std::str::from_utf8(bytes).map_err(ReadStringError::from)
    }
}
