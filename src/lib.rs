#![doc = include_str!("../README.md")]
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
