//! Utilities for parsing and serializing Plain Old Data.
//!
//! The API is primarily defined on the [`Pod`] trait, which can be implemented
//! for `#[repr(C)]` types. It is then possible to get a reference to that [`Pod`]
//! or a slice thereof directly from an underlying buffer.
//! Similarly, the [`Pod`] can also be turned into its underlying buffer as well,
//! for example to write it out into an output buffer.
//!
//! # Features
//!
//! `writer`: Exports an additional [`Writer`] wrapping a [`std::io::Write`]
//! which allows explicitly aligning the output buffer by adding padding bytes.
//!
//! # Example
//!
//! ```
//! use std::mem;
//! use std::io::Write;
//!
//! use watto::Pod;
//!
//! /// Our format looks like this:
//! /// * A header, containing the number of `A`s.
//! /// * An aligned slice of `A`s (length given by the header)
//! /// * An aligned slice of `B`s (length implicitly given by end of buffer)
//! #[repr(C)]
//! struct Header {
//!     version: u32,
//!     num_as: u32,
//! }
//! unsafe impl Pod for Header {}
//!
//! #[repr(C)]
//! #[derive(Debug, PartialEq)]
//! struct A(u16);
//! unsafe impl Pod for A {}
//!
//! #[repr(C)]
//! #[derive(Debug, PartialEq)]
//! struct B(u64);
//! unsafe impl Pod for B {}
//!
//! // Writing into an output buffer:
//! let mut writer = watto::Writer::new(vec![]);
//!
//! writer.write_all(Header { version: 1, num_as: 3 }.as_bytes()).unwrap();
//! writer.align_to(mem::align_of::<A>()).unwrap();
//! writer.write_all(&[A(1), A(2), A(3)].as_bytes()).unwrap();
//! writer.align_to(mem::align_of::<B>()).unwrap();
//! writer.write_all(&[B(4), B(5), B(6)].as_bytes()).unwrap();
//!
//! let buffer = writer.into_inner();
//!
//! // Reading from a buffer:
//! let buffer = &buffer;
//!
//! let (header, buffer) = Header::ref_from_prefix(buffer).unwrap();
//! let (_, buffer) = watto::align_to(buffer, mem::align_of::<A>()).unwrap();
//! let (r#as, buffer) = A::slice_from_prefix(buffer, header.num_as as usize).unwrap();
//! let (_, buffer) = watto::align_to(buffer, mem::align_of::<B>()).unwrap();
//! let bs = B::slice_from_bytes(buffer).unwrap();
//!
//! assert_eq!(header.num_as, 3);
//! assert_eq!(r#as, &[A(1), A(2), A(3)]);
//! assert_eq!(bs, &[B(4), B(5), B(6)]);
//! ```

#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(docsrs, feature(doc_cfg_hide))]
#![cfg_attr(docsrs, doc(cfg_hide(doc)))]

mod helpers;
mod pod;
#[cfg(feature = "strings")]
mod string_table;
mod utils;
#[cfg(feature = "writer")]
mod writer;

pub use helpers::*;
pub use pod::*;
#[cfg(feature = "strings")]
pub use string_table::*;
#[cfg(feature = "writer")]
pub use writer::*;
