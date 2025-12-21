//! Traits for generic NBT value access.
//!
//! This module defines the trait hierarchy that enables writing generic code
//! that works with any NBT value type. The traits are organized by capability:
//!
//! # Trait Hierarchy
//!
//! ```text
//! ScopedReadableValue (all types)
//!         ▲
//!         │
//! ┌───────┴───────┐
//! │               │
//! ReadableValue   ScopedWritableValue
//! (immutable)      (scoped mutation)
//!                         ▲
//!                         │
//!                   WritableValue
//!                   (full mutation)
//! ```
//!
//! # Trait Overview
//!
//! | Trait | Capability | Implemented By |
//! |-------|------------|----------------|
//! | [`ScopedReadableValue`] | Read primitives and containers (scoped) | All value types |
//! | [`ReadableValue`] | + Document-lifetime references | Borrowed, Shared, Immutable |
//! | [`ScopedWritableValue`] | + Mutation (scoped) | Owned, Mutable |
//! | [`WritableValue`] | + Mutable references to containers | Mutable |
//!
//! # Scoped vs Unscoped Methods
//!
//! The "scoped" suffix indicates **bounded lifetime** and **indirection**: scoped methods return by value,
//! while unscoped methods return references to stored data.
//!
//! ## Reading (given `value: &'a ReadonlyValue<'doc, O, D>`)
//!
//! | Method | Returns | Notes |
//! |--------|---------|-------|
//! | `get()` | `ReadonlyValue<'doc, ...>` | With document lifetime |
//! | `get_scoped()` | `ReadonlyValue<'a, ...>` | With lifetime `'a` |
//! | `as_byte_array()` | `&'a ReadonlyArray<'doc, ...>` | Reference to existing field |
//! | `as_byte_array_scoped()` | `ReadonlyArray<'a, ...>` | Constructed view |
//!
//! ## Writing (given `value: &'a mut MutableValue<'s, O>`)
//!
//! | Method | Returns | Notes |
//! |--------|---------|-------|
//! | `as_byte_array_mut()` | `&'a mut VecViewMut<'s, i8>` | Reference to existing field |
//! | `as_byte_array_mut_scoped()` | `VecViewMut<'a, i8>` | Constructed view |
//!
//! Note: Although unscoped `as_byte_array_mut()` returns `&'a mut VecViewMut<'s, i8>` with
//! the storage lifetime `'s`, the `VecViewMut` cannot be cloned, so the effective lifetime
//! is bounded by `'a` anyway.
//!
//! **Why the difference?**
//!
//! - **Unscoped** methods return references to data already stored in the value type.
//!   For [`ReadableValue`], the returned value has document lifetime `'doc` and can
//!   be stored independently.
//!
//! - **Scoped** methods construct new view types on demand. This is necessary for types
//!   like [`OwnedValue`](crate::OwnedValue) that don't directly contain the container types (e.g.,
//!   `OwnedValue` doesn't store a `VecViewMut`, so `as_byte_array_mut_scoped()` must
//!   construct one from the raw data).
//!
//! **When to use which:**
//!
//! - Use **scoped** methods when writing generic code that works with all value types
//! - Use **unscoped** methods when you need direct access to stored fields
//!
//! **Note:** Scoped methods return types that implement the corresponding unscoped traits.
//! For example, `get_scoped()` on [`ScopedReadableValue`] returns a type implementing
//! [`ReadableValue`], and `iter_scoped()` yields items implementing [`ReadableValue`].
//! This allows you to call unscoped methods on the returned values.
//!
//! # Example: Generic Function
//!
//! ```
//! use na_nbt::{ScopedReadableValue, ReadableString, ValueScoped};
//!
//! fn describe_value<'doc>(value: &impl ScopedReadableValue<'doc>) -> String {
//!     value.visit_scoped(|v| match v {
//!         ValueScoped::Byte(n) => format!("Byte: {n}"),
//!         ValueScoped::Int(n) => format!("Int: {n}"),
//!         ValueScoped::String(s) => format!("String: {:?}", s.decode()),
//!         ValueScoped::Compound(_) => "Compound { ... }".to_string(),
//!         ValueScoped::List(_) => "List [ ... ]".to_string(),
//!         _ => "Other".to_string(),
//!     })
//! }
//! ```
//!
//! # Pattern Matching with visit
//!
//! Use [`ScopedReadableValue::visit_scoped`] or [`ReadableValue::visit`] for
//! exhaustive pattern matching:
//!
//! ```
//! use na_nbt::{ScopedReadableValue, ValueScoped, read_borrowed};
//! use zerocopy::byteorder::BigEndian;
//!
//! fn sum_numbers<'doc>(value: &impl ScopedReadableValue<'doc>) -> i64 {
//!     value.visit_scoped(|v| match v {
//!         ValueScoped::Byte(n) => n as i64,
//!         ValueScoped::Short(n) => n as i64,
//!         ValueScoped::Int(n) => n as i64,
//!         ValueScoped::Long(n) => n,
//!         _ => 0,
//!     })
//! }
//! ```

mod config;
mod readable;
mod scoped_readable;
mod scoped_writable;
mod string;
mod value;
mod writable;

pub use config::*;
pub use readable::*;
pub use scoped_readable::*;
pub use scoped_writable::*;
pub use string::*;
pub use value::*;
pub use writable::*;
