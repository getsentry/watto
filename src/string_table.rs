use core::fmt;
use core::str::Utf8Error;

use thiserror::Error;

use crate::{OffsetSet, ReadOffsetSetError};

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

impl From<ReadOffsetSetError> for ReadStringError {
    fn from(value: ReadOffsetSetError) -> Self {
        match value {
            ReadOffsetSetError::Leb128(error) => Self::Leb128(error),
            ReadOffsetSetError::OutOfBounds => Self::OutOfBounds,
        }
    }
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
#[derive(Clone, Default)]
pub struct StringTable {
    inner: OffsetSet<u8>,
}

impl fmt::Debug for StringTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let iter = self
            .inner
            .entries()
            .map(|(offset, string_bytes)| (offset, std::str::from_utf8(string_bytes).unwrap()));
        f.debug_map().entries(iter).finish()
    }
}

impl StringTable {
    /// Initializes an empty `StringTable`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Initializes a [`StringTable`] from a previously serialized representation.
    ///
    /// This essentially reverses the [`as_bytes`](Self::as_bytes) call.
    pub fn from_bytes(buffer: &[u8]) -> Result<Self, ReadStringError> {
        let inner =
            OffsetSet::from_bytes_validated(buffer, |string_bytes| {
                match std::str::from_utf8(string_bytes) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(ReadStringError::Utf8(err)),
                }
            })?;
        Ok(Self { inner })
    }

    /// Insert a string into this `StringTable`.
    ///
    /// Returns an offset that can be used to retrieve the inserted string
    /// with [`read`](Self::read) after serializing this table with [`as_bytes`](Self::as_bytes).
    pub fn insert(&mut self, s: &str) -> usize {
        self.inner.insert(s.as_bytes())
    }

    /// Returns a byte slice containing the concatenation of the strings that have been
    /// added to this `StringTable`.
    pub fn as_bytes(&self) -> &[u8] {
        self.inner.as_bytes()
    }

    /// Returns a byte vector containing the concatenation of the strings that have been
    /// added to this `StringTable`.
    ///
    /// This consumes the `StringTable`.
    pub fn into_bytes(self) -> Vec<u8> {
        self.inner.into_bytes()
    }

    /// Returns the string stored at the given offset in the byte slice, if any.
    ///
    /// Use this to retrieve a string that was previously [inserted](StringTable::insert) into a `StringTable`.
    pub fn read(buffer: &[u8], offset: usize) -> Result<&str, ReadStringError> {
        let bytes = OffsetSet::read(buffer, offset)?;
        Ok(std::str::from_utf8(bytes)?)
    }
}
