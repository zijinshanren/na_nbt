//! Utility types and functions.
//!
//! This module contains utility types used throughout the crate.
//!
//! # ByteOrder Trait
//!
//! The [`ByteOrder`] trait is a convenience trait that combines [`zerocopy::ByteOrder`]
//! with `Send + Sync + 'static` bounds. Use this trait when you need to accept
//! any byte order in generic code.
//!
//! ```
//! use na_nbt::ByteOrder;
//! use zerocopy::byteorder::{BigEndian, LittleEndian};
//!
//! fn process_nbt<O: ByteOrder>(data: &[u8]) {
//!     // Works with any byte order
//! }
//!
//! // Can be called with either endianness
//! # let data = [];
//! process_nbt::<BigEndian>(&data);
//! process_nbt::<LittleEndian>(&data);
//! ```

#[inline(always)]
#[cold]
pub(crate) fn cold_path() {}

/// A trait for byte order types.
///
/// This trait is automatically implemented for all types that implement
/// [`zerocopy::ByteOrder`] and are `Send + Sync + 'static`.
///
/// The two main implementations are:
/// - [`BigEndian`](zerocopy::byteorder::BigEndian) - Used by Minecraft Java Edition
/// - [`LittleEndian`](zerocopy::byteorder::LittleEndian) - Used by Minecraft Bedrock Edition
pub trait ByteOrder: zerocopy::ByteOrder + Send + Sync + 'static {}

impl<T: zerocopy::ByteOrder + Send + Sync + 'static> ByteOrder for T {}

pub(crate) static EMPTY_LIST: [u8; 5] = [0; 5];
pub(crate) static EMPTY_COMPOUND: [u8; 1] = [0];
