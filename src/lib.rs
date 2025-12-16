//! # na_nbt
//!
//! A high-performance NBT (Named Binary Tag) library for Rust with zero-copy parsing
//! and full mutation support.
//!
//! ## Quick Start
//!
//! ```rust
//! use na_nbt::read_borrowed;
//! use zerocopy::byteorder::BigEndian;
//!
//! let data = [
//!     0x0a, 0x00, 0x00, // Compound with empty name
//!     0x01, 0x00, 0x03, b'f', b'o', b'o', 42u8, // Byte "foo" = 42
//!     0x00, // End
//! ];
//!
//! let doc = read_borrowed::<BigEndian>(&data).unwrap();
//! let root = doc.root();
//!
//! if let Some(compound) = root.as_compound() {
//!     if let Some(value) = compound.get("foo") {
//!         assert_eq!(value.as_byte(), Some(42));
//!     }
//! }
//! ```
//!
//! ## Two Parsing Modes
//!
//! | Mode | Function | Type | Use Case |
//! |------|----------|------|----------|
//! | **Zero-copy (borrowed)** | [`read_borrowed`] | [`BorrowedValue`] | Fast reads |
//! | **Zero-copy (shared)** | [`read_shared`] | [`SharedValue`] | Don't want to be bothered with lifetimes |
//! | **Owned** | [`read_owned`] | [`OwnedValue`] | Need to modify |
//!
//! ### Zero-Copy Mode (Borrowed)
//!
//! Parses NBT without copying data. Values reference the original byte slice directly.
//! Best for read-only access when performance matters.
//!
//! ```rust
//! use na_nbt::read_borrowed;
//! use zerocopy::byteorder::BigEndian;
//!
//! let data = [0x0a, 0x00, 0x00, 0x00]; // Empty compound
//! let doc = read_borrowed::<BigEndian>(&data).unwrap();
//! let root = doc.root(); // Zero-copy reference into `data`
//! ```
//!
//! ### Zero-Copy Mode (Shared)
//!
//! Like borrowed mode, but wraps data in `Arc` for shared ownership. Values are
//! `Clone`, `Send`, `Sync`, and `'static` - perfect for multi-threaded scenarios.
//!
//! ```rust
//! use na_nbt::read_shared;
//! use bytes::Bytes;
//! use zerocopy::byteorder::BigEndian;
//!
//! let data = Bytes::from_static(&[0x0a, 0x00, 0x00, 0x00]);
//! let root = read_shared::<BigEndian>(data).unwrap();
//!
//! // Can clone and send to other threads
//! let cloned = root.clone();
//! std::thread::spawn(move || {
//!     assert!(cloned.as_compound().is_some());
//! }).join().unwrap();
//! ```
//!
//! ### Owned Mode
//!
//! Parses NBT into an owned structure that can be modified and outlives the source.
//! Supports endianness conversion during parsing.
//!
//! ```rust
//! use na_nbt::{read_owned, OwnedValue};
//! use zerocopy::byteorder::{BigEndian, LittleEndian};
//!
//! let data = [0x0a, 0x00, 0x00, 0x00]; // Empty compound (BigEndian)
//!
//! // Convert from BigEndian source to LittleEndian storage
//! let mut root: OwnedValue<LittleEndian> = read_owned::<BigEndian, LittleEndian>(&data).unwrap();
//!
//! if let OwnedValue::Compound(ref mut compound) = root {
//!     compound.insert("score", 100i32);
//! }
//! ```
//!
//! ## Writing Generic Code
//!
//! ### Trait Hierarchy
//!
//! ```text
//! ReadableValue ───────► ScopedReadableValue
//!                              ▲
//!                              │
//! WritableValue ───► ScopedWritableValue
//! ```
//!
//! - `Readable` extends `ScopedReadable` with methods returning document-lifetime references
//! - `Writable` extends `ScopedWritable` with mutable access to complex types
//! - `ScopedWritable` extends `ScopedReadable` (writable values are also readable)
//!
//! **Scoped vs Unscoped**: When you call `get_scoped()` or `as_*_scoped()` methods, the returned
//! value has a lifetime tied to the borrow. The non-scoped versions (`get()`, `as_*()`) return
//! values with the document lifetime, allowing you to store them.
//!
//! | Trait | Capability | Implemented By |
//! |-------|------------|----------------|
//! | [`ScopedReadableValue`] | Read primitives, iterate | All value types |
//! | [`ReadableValue`] | + Store references to nested values | [`BorrowedValue`], [`SharedValue`], [`ImmutableValue`] |
//! | [`ScopedWritableValue`] | + Modify primitives | [`OwnedValue`], [`MutableValue`] |
//! | [`WritableValue`] | + Mutable access to arrays/lists/compounds | [`MutableValue`] |
//!
//! ### Example: Generic Dump Function
//!
//! ```rust
//! use na_nbt::{
//!     ScopedReadableValue, ScopedReadableList, ScopedReadableCompound,
//!     ReadableString, ValueScoped, read_borrowed, read_owned,
//! };
//! use zerocopy::byteorder::BigEndian;
//!
//! fn dump<'doc>(value: &impl ScopedReadableValue<'doc>) -> String {
//!     dump_inner(value, 0)
//! }
//!
//! fn dump_inner<'doc>(value: &impl ScopedReadableValue<'doc>, indent: usize) -> String {
//!     let pad = "  ".repeat(indent);
//!     value.visit_scoped(|v| match v {
//!         ValueScoped::End => format!("{pad}End"),
//!         ValueScoped::Byte(n) => format!("{pad}Byte({n})"),
//!         ValueScoped::Short(n) => format!("{pad}Short({n})"),
//!         ValueScoped::Int(n) => format!("{pad}Int({n})"),
//!         ValueScoped::Long(n) => format!("{pad}Long({n})"),
//!         ValueScoped::Float(n) => format!("{pad}Float({n})"),
//!         ValueScoped::Double(n) => format!("{pad}Double({n})"),
//!         ValueScoped::ByteArray(a) => format!("{pad}ByteArray[{}]", a.len()),
//!         ValueScoped::IntArray(a) => format!("{pad}IntArray[{}]", a.len()),
//!         ValueScoped::LongArray(a) => format!("{pad}LongArray[{}]", a.len()),
//!         ValueScoped::String(s) => format!("{pad}String({:?})", s.decode()),
//!         ValueScoped::List(list) => {
//!             let mut out = format!("{pad}List[{}] {{\n", list.len());
//!             for item in list.iter_scoped() {
//!                 out.push_str(&dump_inner(&item, indent + 1));
//!                 out.push('\n');
//!             }
//!             out.push_str(&format!("{pad}}}"));
//!             out
//!         }
//!         ValueScoped::Compound(compound) => {
//!             let mut out = format!("{pad}Compound {{\n");
//!             for (key, val) in compound.iter_scoped() {
//!                 let nested = dump_inner(&val, indent + 1);
//!                 out.push_str(&format!("{}  {:?}: {}\n", pad, key.decode(), nested.trim_start()));
//!             }
//!             out.push_str(&format!("{pad}}}"));
//!             out
//!         }
//!     })
//! }
//!
//! // Works with borrowed values
//! let data = [0x0a, 0x00, 0x00, 0x01, 0x00, 0x01, b'x', 5, 0x00];
//! let doc = read_borrowed::<BigEndian>(&data).unwrap();
//! let s = dump(&doc.root());
//! assert!(s.contains("Compound"));
//!
//! // Works with owned values
//! let owned = read_owned::<BigEndian, BigEndian>(&data).unwrap();
//! let s = dump(&owned);
//! assert!(s.contains("Compound"));
//! ```
//!
//! ## Type Overview
//!
//! ### Zero-Copy Types
//!
//! - [`ReadonlyValue`] - The underlying zero-copy value type
//! - [`BorrowedValue`] - Type alias for borrowed data (`ReadonlyValue<'s, O, ()>`)
//! - [`SharedValue`] - Type alias for `Arc`-wrapped data
//!
//! ### Owned Types
//!
//! - [`OwnedValue`] - Fully owned, mutable NBT value
//! - [`MutableValue`] - Mutable view into an `OwnedValue`
//! - [`ImmutableValue`] - Immutable view into an `OwnedValue`
//!
//! ## Feature Comparison
//!
//! | Feature | `BorrowedValue` | `OwnedValue` |
//! |---------|-----------------|--------------|
//! | Zero-copy parsing | ✅ | ❌ |
//! | Modify values | ❌ | ✅ |
//! | Outlives source | ❌ | ✅ |
//! | Endianness conversion | On write | On read or write |
//! | Memory usage | Minimal | Proportional to data |

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
