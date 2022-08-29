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
