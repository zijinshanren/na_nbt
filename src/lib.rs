//! # na_nbt
//!
//! `na_nbt` is a high-performance, flexible NBT (Named Binary Tag) library for Rust.
//!
//! It provides multiple ways to interact with NBT data:
//!
//! *   **Immutable (Zero-Copy)**: Extremely fast parsing and reading with minimal memory allocation.
//!     See [`ReadonlyValue`] and [`read_borrowed`].
//! *   **Mutable (Owned)**: Full modification capabilities with an owned data structure.
//!     See [`OwnedValue`] and [`read_owned`].
//! *   **Generic Abstraction**: Traits like [`ReadableValue`] and [`WritableValue`] allow writing code
//!     that works with both immutable and mutable representations.
//!
//! ## Example
//!
//! ```rust
//! use na_nbt::read_borrowed;
//! use zerocopy::byteorder::BigEndian;
//!
//! let data = [
//!     0x0a, // Compound
//!     0x00, 0x00, // Name length 0
//!     0x00, // End
//! ];
//!
//! let doc = read_borrowed::<BigEndian>(&data).unwrap();
//! let root = doc.root();
//! assert!(root.as_compound().is_some());
//! ```

mod error;
mod immutable;
mod index;
mod mutable;
mod tag;
mod util;
mod value_trait;
mod view;

pub use error::*;
pub use immutable::*;
pub use mutable::*;
pub use tag::*;
pub use util::*;
pub use value_trait::*;
