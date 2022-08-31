# Watto

[![Build Status](https://github.com/getsentry/watto/workflows/CI/badge.svg)](https://github.com/getsentry/watto/actions?workflow=CI)
<a href="https://crates.io/crates/watto"><img src="https://img.shields.io/crates/v/watto.svg" alt=""></a>
<a href="https://github.com/getsentry/watto/blob/master/LICENSE"><img src="https://img.shields.io/crates/l/watto.svg" alt=""></a>
[![codecov](https://codecov.io/gh/getsentry/watto/branch/master/graph/badge.svg?token=R22XLVB7KP)](https://codecov.io/gh/getsentry/watto)

Utilities for parsing and serializing Plain Old Data.

## Pod

The API is primarily defined on the [`Pod`] trait, which can be implemented
for `#[repr(C)]` types. It is then possible to get a reference to that [`Pod`]
or a slice thereof directly from an underlying buffer.
Similarly, the [`Pod`] can also be turned into its underlying buffer as well,
for example to write it out into an output buffer.

## Features

`writer`: Exports an additional [`Writer`] wrapping a [`std::io::Write`]
which allows explicitly aligning the output buffer by adding padding bytes.

## End-to-End Example

```rust
use std::mem;
use std::io::Write;

use watto::Pod;

/// Our format looks like this:
/// * A header, containing the number of `A`s.
/// * An aligned slice of `A`s (length given by the header)
/// * An aligned slice of `B`s (length implicitly given by end of buffer)
#[repr(C)]
struct Header {
    version: u32,
    num_as: u32,
}
unsafe impl Pod for Header {}

#[repr(C)]
#[derive(Debug, PartialEq)]
struct A(u16);
unsafe impl Pod for A {}

#[repr(C)]
#[derive(Debug, PartialEq)]
struct B(u64);
unsafe impl Pod for B {}

// Writing into an output buffer:
let mut writer = watto::Writer::new(vec![]);

writer.write_all(Header { version: 1, num_as: 3 }.as_bytes()).unwrap();
writer.align_to(mem::align_of::<A>()).unwrap();
writer.write_all(&[A(1), A(2), A(3)].as_bytes()).unwrap();
writer.align_to(mem::align_of::<B>()).unwrap();
writer.write_all(&[B(4), B(5), B(6)].as_bytes()).unwrap();

let buffer = writer.into_inner();

// Reading from a buffer:
let buffer = &buffer;

let (header, buffer) = Header::ref_from_prefix(buffer).unwrap();
let (_, buffer) = watto::align_to(buffer, mem::align_of::<A>()).unwrap();
let (r#as, buffer) = A::slice_from_prefix(buffer, header.num_as as usize).unwrap();
let (_, buffer) = watto::align_to(buffer, mem::align_of::<B>()).unwrap();
let bs = B::slice_from_bytes(buffer).unwrap();

assert_eq!(header.num_as, 3);
assert_eq!(r#as, &[A(1), A(2), A(3)]);
assert_eq!(bs, &[B(4), B(5), B(6)]);
```

## Alternatives
Watto is strongly inspired by [`zerocopy`](https://docs.rs/zerocopy/0.6.1/zerocopy/).

Differences between the two include:

* `zerocopy` has two distinct traits for reading and writing bytes, `watto` only has one for both.
* In  `zerocopy`, reading a value requires wrapping it in `LayoutVerified`. In `watto`, types implementing
  `Pod` can be read directly.
* `watto` includes a `Writer` that allows explicit alignment of output.
* `zerocopy` includes endianness-aware integer types.
## Why Watto?

> Qui-Gon Jinn: I have... acquired a pod in a game of chance. The fastest ever built.
>
> Watto: I hope you didn't kill anyone I know for it.
>
> -- [Star Wars: Episode I - The Phantom Menace](https://www.imdb.com/title/tt0120915/quotes/qt0270694)

## License

Watto is licensed under the MIT license.
