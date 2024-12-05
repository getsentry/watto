use std::mem;

use watto::Pod;

#[test]
fn test_ref() {
    let num = u64::from_ne_bytes([0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7]);
    let bytes = num.as_bytes();
    assert_eq!(bytes, &[0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7]);
    assert_eq!(*u64::ref_from_bytes(bytes).unwrap(), num);

    // buffer too big
    let n = u32::ref_from_bytes(bytes);
    assert_eq!(n, None);
    // buffer not aligned
    let n = u32::ref_from_bytes(&bytes[1..]);
    assert_eq!(n, None);
}

#[test]
fn test_slice() {
    let num = u64::from_ne_bytes([0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7]);
    let bytes = num.as_bytes();
    let nums = u32::slice_from_bytes(bytes).unwrap();

    assert_eq!(nums.as_bytes(), &[0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7]);
    assert_eq!(nums.len(), 2);
    assert_eq!(nums[0], u32::from_ne_bytes([0x0, 0x1, 0x2, 0x3]));
    assert_eq!(nums[1], u32::from_ne_bytes([0x4, 0x5, 0x6, 0x7]));

    // buffer not a multiple of the element size
    let n = u32::slice_from_bytes(&bytes[..7]);
    assert_eq!(n, None);
    // buffer not aligned
    let n = u32::slice_from_bytes(&bytes[1..]);
    assert_eq!(n, None);
}

#[test]
fn test_ref_from_prefix() {
    let nums = [
        u64::from_ne_bytes([0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7]),
        u64::from_ne_bytes([0x8, 0x9, 0xa, 0xb, 0xc, 0xd, 0xe, 0xf]),
    ];
    let bytes = nums.as_bytes();
    let (num, rest) = u64::ref_from_prefix(bytes).unwrap();

    assert_eq!(num.as_bytes(), &[0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7]);

    assert_eq!(*num, nums[0]);

    assert_eq!(rest, &[0x8, 0x9, 0xa, 0xb, 0xc, 0xd, 0xe, 0xf]);

    // buffer not aligned
    let n = u64::ref_from_prefix(&bytes[1..]);
    assert_eq!(n, None);
}

#[test]
fn test_slice_from_prefix() {
    let nums = [
        u64::from_ne_bytes([0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7]),
        u64::from_ne_bytes([0x8, 0x9, 0xa, 0xb, 0xc, 0xd, 0xe, 0xf]),
    ];
    let bytes = nums.as_bytes();
    let (nums, rest) = u32::slice_from_prefix(bytes, 2).unwrap();

    assert_eq!(nums.as_bytes(), &[0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7]);

    assert_eq!(nums.len(), 2);
    assert_eq!(nums[0], u32::from_ne_bytes([0x0, 0x1, 0x2, 0x3]));
    assert_eq!(nums[1], u32::from_ne_bytes([0x4, 0x5, 0x6, 0x7]));

    assert_eq!(rest, &[0x8, 0x9, 0xa, 0xb, 0xc, 0xd, 0xe, 0xf]);

    // buffer not aligned
    let n = u32::slice_from_prefix(&bytes[1..], 2);
    assert_eq!(n, None);
}

#[test]
fn test_align_to() {
    let num = u64::from_ne_bytes([0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7]);
    let bytes = num.as_bytes();
    let (num, bytes) = u16::ref_from_prefix(bytes).unwrap();

    assert_eq!(*num, u16::from_ne_bytes([0x0, 0x1]));

    let (_, bytes) = watto::align_to(bytes, mem::size_of::<u32>()).unwrap();

    let (nums, bytes) = u32::slice_from_prefix(bytes, 1).unwrap();

    assert_eq!(nums, &[u32::from_ne_bytes([0x4, 0x5, 0x6, 0x7])]);

    assert_eq!(bytes, &[]);
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

#[cfg(feature = "strings")]
mod string_tests {
    use watto::StringTable;

    #[test]
    fn test_string_table() {
        let mut string_table = StringTable::new();

        let offset_empty = string_table.insert("");
        let offset_abc = string_table.insert("abc");
        let offset_def = string_table.insert("def");

        assert_eq!(string_table.insert("abc"), offset_abc);

        let string_bytes = string_table.as_bytes();
        let read_empty = StringTable::read(string_bytes, offset_empty).unwrap();
        let read_abc = StringTable::read(string_bytes, offset_abc).unwrap();
        let read_def = StringTable::read(string_bytes, offset_def).unwrap();
        assert_eq!(read_empty, "");
        assert_eq!(read_abc, "abc");
        assert_eq!(read_def, "def");

        // re-create the table using a serialized buffer
        let mut string_table = StringTable::from_bytes(string_bytes).unwrap();

        assert_eq!(string_table.insert("abc"), offset_abc);

        let string_bytes = string_table.as_bytes();
        let read_abc = StringTable::read(string_bytes, offset_abc).unwrap();
        let read_def = StringTable::read(string_bytes, offset_def).unwrap();
        assert_eq!(read_abc, "abc");
        assert_eq!(read_def, "def");
    }
}
