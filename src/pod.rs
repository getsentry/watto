use core::{mem, slice};

/// Plain Old Data
///
/// This `unsafe` trait signifies that a POD type can be converted to and from
/// a slice of bytes.
///
/// # Safety
///
/// The concrete type needs to have a stable binary layout, and every raw bit
/// pattern has to be a valid representation for the type.
///
/// You can consult the sections about type layouts of the
/// [Rust Reference](https://doc.rust-lang.org/reference/type-layout.html),
/// [Unsafe Code Guidelines](https://rust-lang.github.io/unsafe-code-guidelines/layout/structs-and-tuples.html), or
/// [The Rustonomicon](https://doc.rust-lang.org/nomicon/other-reprs.html)
/// for more information.
pub unsafe trait Pod {
    /// This gives the raw bytes of a certain POD.
    fn as_bytes(&self) -> &[u8] {
        unsafe {
            let len = mem::size_of_val(self);
            slice::from_raw_parts(self as *const Self as *const u8, len)
        }
    }

    /// Creates a reference to [`Self`] from a slice of bytes.
    ///
    /// This checks that `bytes` has proper alignment and exact size.
    fn ref_from_bytes(bytes: &[u8]) -> Option<&Self>
    where
        Self: Sized,
    {
        if bytes.len() != mem::size_of::<Self>()
            || !crate::utils::is_aligned_to(bytes, mem::align_of::<Self>())
        {
            return None;
        }

        // SAFETY:
        // We have checked size and alignment, and our type is a `Pod`.
        Some(unsafe { &*(bytes.as_ptr() as *const Self) })
    }

    /// Creates a reference to [`Self`] from a slice of bytes.
    ///
    /// This checks that `bytes` has proper alignment and is large enough.
    /// It also returns the trailing bytes as a new slice.
    fn ref_from_prefix(bytes: &[u8]) -> Option<(&Self, &[u8])>
    where
        Self: Sized,
    {
        if bytes.len() < mem::size_of::<Self>()
            || !crate::utils::is_aligned_to(bytes, mem::align_of::<Self>())
        {
            return None;
        }

        let (bytes, suffix) = bytes.split_at(mem::size_of::<Self>());

        // SAFETY:
        // We have checked size and alignment, and our type is a `Pod`.
        Some((unsafe { &*(bytes.as_ptr() as *const Self) }, suffix))
    }

    /// Creates a slice of [`Self`] from a slice of bytes.
    ///
    /// This checks that `bytes` has proper alignment and its size is a multiple
    /// of the size of [`Self`].
    /// The resulting slice will hold exactly the number of elements that fit in
    /// the underlying buffer.
    fn slice_from_bytes(bytes: &[u8]) -> Option<&[Self]>
    where
        Self: Sized,
    {
        assert_ne!(mem::size_of::<Self>(), 0);

        let len = bytes.len();
        let elem_size = mem::size_of::<Self>();

        if len % elem_size != 0 || !crate::utils::is_aligned_to(bytes, mem::align_of::<Self>()) {
            return None;
        }

        let elems = len / elem_size;

        // SAFETY:
        // We have checked size and alignment, and our type is a `Pod`.
        Some(unsafe { slice::from_raw_parts(bytes.as_ptr() as *const Self, elems) })
    }

    /// Creates a slice of [`Self`] from a slice of bytes.
    ///
    /// This checks that `bytes` has proper alignment and is large enough to hold
    /// `elems` elements of [`Self`].
    ///
    /// It also returns the trailing bytes as a new slice.
    fn slice_from_prefix(bytes: &[u8], elems: usize) -> Option<(&[Self], &[u8])>
    where
        Self: Sized,
    {
        assert_ne!(mem::size_of::<Self>(), 0);

        let elem_size = mem::size_of::<Self>();
        let expected_len = elem_size.checked_mul(elems)?;

        if bytes.len() < expected_len
            || !crate::utils::is_aligned_to(bytes, mem::align_of::<Self>())
        {
            return None;
        }

        let (bytes, suffix) = bytes.split_at(expected_len);

        // SAFETY:
        // We have checked size and alignment, and our type is a `Pod`.
        Some((
            unsafe { slice::from_raw_parts(bytes.as_ptr() as *const Self, elems) },
            suffix,
        ))
    }
}

unsafe impl<T: Pod> Pod for [T] {}
unsafe impl<T: Pod, const N: usize> Pod for [T; N] {}

/// Implements `$trait` for one or more `$type`s.
macro_rules! impl_for_types {
    ($trait:ident, $type:ty) => (
        unsafe impl $trait for $type {}
    );
    ($trait:ident, $type:ty, $($types:ty),*) => (
        unsafe impl $trait for $type {}
        impl_for_types!($trait, $($types),*);
    );
}

impl_for_types!(Pod, u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize, f32, f64);
