use std::mem;

use watto::Pod;

#[test]
fn test_ref() {
    let bytes = vec![0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8];
    let num = u64::ref_from_bytes(&bytes[0..8]).unwrap();

    assert_eq!(num.as_bytes(), &[0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7]);

    assert_eq!(
        *num,
        u64::from_ne_bytes([0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7])
    );

    // buffer too big
    let n = u64::ref_from_bytes(&bytes);
    assert_eq!(n, None);
    // buffer not aligned
    let n = u64::ref_from_bytes(&bytes[1..]);
    assert_eq!(n, None);
}

#[test]
fn test_slice() {
    let bytes = vec![0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8];
    let nums = u32::slice_from_bytes(&bytes[0..8]).unwrap();

    assert_eq!(nums.as_bytes(), &[0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7]);

    assert_eq!(nums.len(), 2);
    assert_eq!(nums[0], u32::from_ne_bytes([0x0, 0x1, 0x2, 0x3]));
    assert_eq!(nums[1], u32::from_ne_bytes([0x4, 0x5, 0x6, 0x7]));

    // buffer not a multiple of the element size
    let n = u32::slice_from_bytes(&bytes);
    assert_eq!(n, None);
    // buffer not aligned
    let n = u32::slice_from_bytes(&bytes[1..]);
    assert_eq!(n, None);
}

#[test]
fn test_ref_from_prefix() {
    let bytes = vec![0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9];
    let (num, rest) = u64::ref_from_prefix(&bytes).unwrap();

    assert_eq!(num.as_bytes(), &[0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7]);

    assert_eq!(
        *num,
        u64::from_ne_bytes([0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7])
    );

    assert_eq!(rest, &[0x8, 0x9]);

    // buffer not aligned
    let n = u64::ref_from_prefix(&bytes[1..]);
    assert_eq!(n, None);
}

#[test]
fn test_slice_from_prefix() {
    let bytes = vec![0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9];
    let (nums, rest) = u32::slice_from_prefix(&bytes, 2).unwrap();

    assert_eq!(nums.as_bytes(), &[0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7]);

    assert_eq!(nums.len(), 2);
    assert_eq!(nums[0], u32::from_ne_bytes([0x0, 0x1, 0x2, 0x3]));
    assert_eq!(nums[1], u32::from_ne_bytes([0x4, 0x5, 0x6, 0x7]));

    assert_eq!(rest, &[0x8, 0x9]);

    // buffer not aligned
    let n = u32::slice_from_prefix(&bytes[1..], 2);
    assert_eq!(n, None);
}

#[test]
fn test_align_to() {
    let bytes = &[0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9];
    let (num, bytes) = u16::ref_from_prefix(bytes).unwrap();

    assert_eq!(*num, u16::from_ne_bytes([0x0, 0x1]));

    let (_, bytes) = watto::align_to(bytes, mem::size_of::<u32>()).unwrap();

    let (nums, bytes) = u32::slice_from_prefix(bytes, 1).unwrap();

    assert_eq!(nums, &[u32::from_ne_bytes([0x4, 0x5, 0x6, 0x7])]);

    assert_eq!(bytes, &[0x8, 0x9]);
}

#[cfg(feature = "writer")]
mod writer_tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn test_writer() {
        let mut writer = watto::Writer::new(vec![]);

        let num = u16::from_ne_bytes([0x0, 0x1]);
        writer.write_all(num.as_bytes()).unwrap();

        writer.align_to(mem::align_of::<u32>()).unwrap();

        let nums = &[
            u32::from_ne_bytes([0x2, 0x3, 0x4, 0x5]),
            u32::from_ne_bytes([0x6, 0x7, 0x8, 0x9]),
        ];
        writer.write_all(nums.as_bytes()).unwrap();

        writer.align_to(32).unwrap();

        let buffer = writer.into_inner();

        assert_eq!(
            buffer,
            &[
                0x0, 0x1, 0x0, 0x0, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0x0, 0x0, 0x0, 0x0,
                0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0
            ]
        )
    }
}
