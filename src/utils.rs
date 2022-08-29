/// Essentially re-implements nightly-only [`ptr::is_aligned_to`].
///
/// See:
/// https://doc.rust-lang.org/core/primitive.pointer.html#method.is_aligned_to
pub fn is_aligned_to(slice: &[u8], align: usize) -> bool {
    if !align.is_power_of_two() {
        panic!("is_aligned_to: align is not a power-of-two");
    }

    slice.as_ptr() as usize & (align - 1) == 0
}
