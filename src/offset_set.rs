use core::hash::{BuildHasher, Hash};
use core::marker::PhantomData;
use core::{fmt, mem};
use std::io::Cursor;

use hashbrown::hash_table::Entry;
use hashbrown::{DefaultHashBuilder, HashTable};
use thiserror::Error;

use crate::Pod;

/// An error when trying to read a slice from a serialized [`OffsetSet`].
#[derive(Debug, Error)]
pub enum ReadOffsetSetError {
    /// The entry's length prefix is not valid LEB128.
    #[error("error reading LEB128 encoded number")]
    Leb128(#[from] leb128::read::Error),
    /// The entry's offset or length is outside the bounds of the data blob.
    #[error("element offset or length is out of bounds")]
    OutOfBounds,
}

/// A struct for storing arbitrary slices without duplicates.
///
/// The [`OffsetSet`] can be thought of as a specialized version of
/// [`IndexSet<&[T]>`](https://docs.rs/indexmap/latest/indexmap/set/struct.IndexSet.html),
/// with the following differences:
///
/// - It is specialized to store slices of `T`.
/// - It does not return an *index*, but rather an *offset* of the encoded slice
///   within the buffer.
/// - It is intended to be serialized as an opaque buffer, and data to be loaded
///   from it with minimal overhead.
#[derive(Clone)]
pub struct OffsetSet<T> {
    hasher: DefaultHashBuilder,
    offsets: HashTable<usize>,
    buffer: Vec<u8>,
    _t: PhantomData<T>,
}

impl<T: fmt::Debug + Pod> fmt::Debug for OffsetSet<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.entries()).finish()
    }
}

impl<T> Default for OffsetSet<T> {
    fn default() -> Self {
        Self {
            hasher: Default::default(),
            offsets: Default::default(),
            buffer: Default::default(),
            _t: Default::default(),
        }
    }
}

impl<T: Pod> OffsetSet<T> {
    #[doc(hidden)]
    const _ALIGN_OF_T: () = {
        // TODO: this is not a hard requirement for now, and we can lift this in the future.
        // - using `leb128` encoding might not make sense at all for types with larger alignment
        // - otherwise this might be missing a couple of places that need explicit alignment
        // - and as we have found out, miri is particularly picky about alignment as well :-)
        assert!(
            mem::align_of::<T>() == 1,
            "T is currently limited to alignment `1`"
        )
    };

    /// Initializes an empty [`OffsetSet`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the slice stored at the given offset in the byte slice, if any.
    ///
    /// Use this to retrieve a slice that was previously [inserted](OffsetSet::insert) into an [`OffsetSet`].
    pub fn read(buffer: &[u8], offset: usize) -> Result<&[T], ReadOffsetSetError> {
        Ok(OffsetSet::read_internal(buffer, offset)?.0)
    }

    fn read_internal(buffer: &[u8], offset: usize) -> Result<(&[T], usize), ReadOffsetSetError> {
        let mut cursor = Cursor::new(
            buffer
                .get(offset..)
                .ok_or(ReadOffsetSetError::OutOfBounds)?,
        );
        let len = leb128::read::unsigned(&mut cursor)? as usize;
        // it would be nice if `leb128` would directly return this as well,
        // so one wouldn't have to use a `Cursor`.
        let leb_len = cursor.position() as usize;

        let start = offset + leb_len;
        let end = start + len * mem::size_of::<T>();

        let bytes = buffer
            .get(start..end)
            .ok_or(ReadOffsetSetError::OutOfBounds)?;
        let slice = T::slice_from_bytes(bytes).ok_or(ReadOffsetSetError::OutOfBounds)?;

        Ok((slice, end))
    }

    /// Iterates over all the entries is this [`OffsetSet`].
    ///
    /// This yields `(offset, slice)` pairs.
    pub fn entries(&self) -> impl Iterator<Item = (usize, &[T])> + '_ {
        self.offsets
            .iter()
            .map(|&offset| (offset, Self::read(&self.buffer, offset).unwrap()))
    }

    /// Returns a byte slice containing the serialized representation of this [`OffsetSet`].
    pub fn as_bytes(&self) -> &[u8] {
        &self.buffer
    }

    /// Returns a byte vector containing the serialized representation of this [`OffsetSet`].
    ///
    /// This consumes the [`OffsetSet`].
    pub fn into_bytes(self) -> Vec<u8> {
        self.buffer
    }
}

impl<T: Pod + PartialEq + Hash> OffsetSet<T> {
    fn raw_entry(&mut self, items: &[T]) -> (Entry<'_, usize>, &mut Vec<u8>) {
        let hasher = |val: &_| self.hasher.hash_one(val);
        let hash = hasher(items);

        let entry = self.offsets.entry(
            hash,
            |&offset| Self::read(&self.buffer, offset).unwrap() == items,
            |&offset| hasher(Self::read(&self.buffer, offset).unwrap()),
        );
        (entry, &mut self.buffer)
    }

    /// Initializes an [`OffsetSet`] from a previously serialized representation.
    ///
    /// This essentially reverses the [`as_bytes`](Self::as_bytes) call.
    pub fn from_bytes(buffer: &[u8]) -> Result<Self, ReadOffsetSetError> {
        Self::from_bytes_validated(buffer, |_| Ok(()))
    }

    /// Initializes an [`OffsetSet`] from a previously serialized representation,
    /// running each loaded slice through a validation function.
    pub fn from_bytes_validated<V, E>(buffer: &[u8], validate: V) -> Result<Self, E>
    where
        E: From<ReadOffsetSetError>,
        V: Fn(&[T]) -> Result<(), E>,
    {
        let mut slf = Self {
            buffer: buffer.into(),
            ..Default::default()
        };

        let mut offset = 0;
        while offset < buffer.len() {
            let (item, next_offset) = Self::read_internal(buffer, offset)?;
            validate(item)?;

            let (entry, _buffer) = slf.raw_entry(item);
            entry.insert(offset);

            offset = next_offset;
        }

        Ok(slf)
    }

    /// Insert a string into this [`OffsetSet`].
    ///
    /// Returns an offset that can be used to retrieve the inserted input
    /// with [`read`](Self::read) after serializing this table with [`as_bytes`](Self::as_bytes).
    pub fn insert(&mut self, input: &[T]) -> usize {
        let (entry, buffer) = self.raw_entry(input);

        let entry = entry.or_insert_with(|| {
            let offset = buffer.len();

            let len = input.len() as u64;
            leb128::write::unsigned(buffer, len).unwrap();
            buffer.extend_from_slice(input.as_bytes());

            offset
        });

        *entry.get()
    }
}
