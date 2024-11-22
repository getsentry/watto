use core::fmt;
use core::hash::BuildHasher;
use core::str::Utf8Error;
use std::io::Cursor;

use hashbrown::hash_table::Entry;
use hashbrown::{DefaultHashBuilder, HashTable};
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
#[derive(Clone, Default)]
pub struct StringTable {
    hasher: DefaultHashBuilder,
    offsets: HashTable<usize>,
    buffer: Vec<u8>,
}

impl fmt::Debug for StringTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let iter = self
            .offsets
            .iter()
            .map(|&offset| (offset, Self::read(&self.buffer, offset).unwrap()));
        f.debug_map().entries(iter).finish()
    }
}

impl StringTable {
    /// Initializes an empty `StringTable`.
    pub fn new() -> Self {
        Self::default()
    }

    fn read_bytes(buffer: &[u8], offset: usize) -> Result<(&[u8], usize), ReadStringError> {
        let mut cursor = Cursor::new(buffer.get(offset..).ok_or(ReadStringError::OutOfBounds)?);
        let len = leb128::read::unsigned(&mut cursor)? as usize;
        // it would be nice if `leb128` would directly return this as well,
        // so one wouldn't have to use a `Cursor`.
        let leb_len = cursor.position() as usize;

        let start = offset + leb_len;
        let end = start + len;

        let string_bytes = buffer.get(start..end).ok_or(ReadStringError::OutOfBounds)?;

        Ok((string_bytes, end))
    }

    fn raw_entry(&mut self, string_bytes: &[u8]) -> (Entry<'_, usize>, &mut Vec<u8>) {
        let hasher = |val: &_| self.hasher.hash_one(val);
        let hash = hasher(string_bytes);

        let entry = self.offsets.entry(
            hash,
            |&offset| Self::read_bytes(&self.buffer, offset).unwrap().0 == string_bytes,
            |&offset| hasher(Self::read_bytes(&self.buffer, offset).unwrap().0),
        );
        (entry, &mut self.buffer)
    }

    /// Initializes a [`StringTable`] from a previously serialized representation.
    ///
    /// This essentially reverses the [`as_bytes`](Self::as_bytes) call.
    pub fn from_bytes(buffer: &[u8]) -> Result<Self, ReadStringError> {
        let mut slf = Self {
            buffer: buffer.into(),
            ..Default::default()
        };

        let mut offset = 0;
        while offset < buffer.len() {
            let (string_bytes, next_offset) = Self::read_bytes(buffer, offset)?;
            std::str::from_utf8(string_bytes)?;

            let (entry, _buffer) = slf.raw_entry(string_bytes);
            entry.insert(offset);

            offset = next_offset;
        }

        Ok(slf)
    }

    /// Insert a string into this `StringTable`.
    ///
    /// Returns an offset that can be used to retrieve the inserted string
    /// with [`read`](Self::read) after serializing this table with [`as_bytes`](Self::as_bytes).
    pub fn insert(&mut self, s: &str) -> usize {
        let string_bytes = s.as_bytes();
        let (entry, buffer) = self.raw_entry(string_bytes);

        let entry = entry.or_insert_with(|| {
            let offset = buffer.len();

            let string_len = string_bytes.len() as u64;
            leb128::write::unsigned(buffer, string_len).unwrap();
            buffer.extend_from_slice(string_bytes);

            offset
        });

        *entry.get()
    }

    /// Returns a byte slice containing the concatenation of the strings that have been
    /// added to this `StringTable`.
    pub fn as_bytes(&self) -> &[u8] {
        &self.buffer
    }

    /// Returns a byte vector containing the concatenation of the strings that have been
    /// added to this `StringTable`.
    ///
    /// This consumes the `StringTable`.
    pub fn into_bytes(self) -> Vec<u8> {
        self.buffer
    }

    /// Returns the string stored at the given offset in the byte slice, if any.
    ///
    /// Use this to retrieve a string that was previously [inserted](StringTable::insert) into a `StringTable`.
    pub fn read(buffer: &[u8], offset: usize) -> Result<&str, ReadStringError> {
        let bytes = Self::read_bytes(buffer, offset)?.0;
        Ok(std::str::from_utf8(bytes)?)
    }
}
