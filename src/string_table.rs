use std::collections::HashMap;

/// A struct for storing strings without duplicates.
///
/// Add strings to the table with [`insert`](StringTable::insert). The
/// returned `usize` can be used to [`lookup`](StringTable::lookup) the string
/// in the table's serialized representation.
/// # Example
/// ```
/// use watto::StringTable;
/// let mut table = StringTable::new();
/// let foo_idx = table.insert("foo");
/// let bar_idx = table.insert("bar");
/// let string_bytes = table.as_bytes();
/// assert_eq!(StringTable::lookup(string_bytes, foo_idx).unwrap(), "foo");
/// assert_eq!(StringTable::lookup(string_bytes, bar_idx).unwrap(), "bar");
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
    /// with [`lookup`](Self::lookup) after serializing this table with [`as_bytes`](Self::as_bytes).
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
    /// Use this to look up a string that was previously [inserted](StringTable::insert) into a `StringTable`.
    pub fn lookup(string_bytes: &[u8], offset: usize) -> Option<&str> {
        let reader = &mut string_bytes.get(offset as usize..)?;
        let len = leb128::read::unsigned(reader).ok()? as usize;

        let bytes = reader.get(..len)?;

        std::str::from_utf8(bytes).ok()
    }

    /// Returns the total length of the strings in this table (including length prefixes).
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Returns true if this table is empty.
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}