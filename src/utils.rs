/// Essentially re-implements nightly-only [`ptr::is_aligned_to`].
///
/// See:
/// https://doc.rust-lang.org/core/primitive.pointer.html#method.is_aligned_to
pub(crate) fn is_aligned_to(bytes: &[u8], align: usize) -> bool {
    if !align.is_power_of_two() {
        panic!("is_aligned_to: align is not a power-of-two");
    }

    bytes.as_ptr() as usize & (align - 1) == 0
}

/// Splits the given `bytes` into padding and a slice that is properly aligned
/// to `align` bytes.
///
/// Returns [`None`] when `bytes` is not large enough.
pub fn align_to(bytes: &[u8], align: usize) -> Option<(&[u8], &[u8])> {
    let offset = bytes.as_ptr().align_offset(align);

    if bytes.len() < offset {
        return None;
    }

    Some(bytes.split_at(offset))
}

/// Splits the given `bytes` into padding and a slice that is properly aligned
/// for `T`.
///
/// Returns [`None`] when `bytes` is not large enough.
pub fn align_to_type<T>(bytes: &[u8]) -> Option<(&[u8], &[u8])> {
    align_to(bytes, core::mem::align_of::<T>())
}
