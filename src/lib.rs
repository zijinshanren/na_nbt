//! # na_nbt
//!
//! NBT (Named Binary Tag) is a binary format used by Minecraft to store structured
//! game data including worlds, player inventories, and entity information.
//!
//! An NBT file contains a tree of typed, named values. For example, a simple
//! player data structure might contain:
//!
//! | Tag Type | Name | Value |
//! |----------|------|-------|
//! | Compound | (root) | |
//! | ├─ String | "name" | "Steve" |
//! | ├─ Int | "score" | 100 |
//! | └─ List | "inventory" | [Compound, ...] |
//!
//! There are three common ways to work with NBT data in Rust:
//!
//! - **As raw bytes.** Binary NBT data you read from a file, receive over the
//!   network, or prepare to send to a Minecraft client/server.
//!
//! - **As a zero-copy representation.** When you need maximum performance for
//!   read-only access, parsing directly from the source bytes without allocations.
//!
//! - **As an owned mutable structure.** When you need to modify NBT data,
//!   build structures from scratch, or keep values after the source is dropped.
//!
//! This crate provides efficient, flexible, and safe ways to work with each of
//! these representations.
//!
//! # Reading NBT from bytes
//!
//! Any valid NBT data can be parsed into a value type. Choose the parsing mode
//! based on your needs:
//!
//! | Mode | Function | Type | Use Case |
//! |------|----------|------|----------|
//! | **Borrowed** | [`read_borrowed`] | [`BorrowedValue`] | Fast read-only access |
//! | **Shared** | [`read_shared`] | [`SharedValue`] | Multi-threaded access |
//! | **Owned** | [`read_owned`] | [`OwnedValue`] | Modification required |
//!
//! ```
//! use na_nbt::{Result, read_borrowed};
//! use zerocopy::byteorder::BigEndian;
//!
//! fn read_example() -> Result<()> {
//!     // NBT binary data - maybe from a file or network
//!     let data = [
//!         0x0a, 0x00, 0x00,                           // Compound with empty name
//!         0x01, 0x00, 0x05, b'l', b'e', b'v', b'e', b'l', 42u8, // Byte "level" = 42
//!         0x00,                                       // End tag
//!     ];
//!
//!     // Parse the NBT data
//!     let doc = read_borrowed::<BigEndian>(&data)?;
//!     let root = doc.root();
//!
//!     // Access data directly using get() on the value
//!     // Returns None if root is not a compound or key doesn't exist
//!     if let Some(level) = root.get("level") {
//!         println!("Level: {}", level.as_byte().unwrap_or(0));
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! The result of methods like `as_compound()` returns an `Option` - `None` if
//! the value is not the expected type. You can also use `get()` directly on
//! any value, which will return `None` if the value is not indexable.
//!
//! # Zero-copy parsing (borrowed)
//!
//! For maximum read performance, use [`read_borrowed`]. The parsed values
//! reference the original byte slice directly - no data is copied.
//!
//! ```
//! use na_nbt::read_borrowed;
//! use zerocopy::byteorder::BigEndian;
//!
//! let data = [0x0a, 0x00, 0x00, 0x00]; // Empty compound
//! let doc = read_borrowed::<BigEndian>(&data).unwrap();
//! let root = doc.root(); // Zero-copy reference into `data`
//!
//! assert!(root.is_compound());
//! ```
//!
//! # Zero-copy parsing (shared)
//!
//! When you need to share NBT data across threads or don't want to deal with
//! lifetimes, use [`read_shared`]. The data is wrapped in `Arc` for shared
//! ownership.
//!
//! ```
//! use na_nbt::read_shared;
//! use bytes::Bytes;
//! use zerocopy::byteorder::BigEndian;
//!
//! let data = Bytes::from_static(&[0x0a, 0x00, 0x00, 0x00]);
//! let root = read_shared::<BigEndian>(data).unwrap();
//!
//! // Clone and send to another thread
//! let cloned = root.clone();
//! std::thread::spawn(move || {
//!     assert!(cloned.as_compound().is_some());
//! }).join().unwrap();
//! ```
//!
//! # Owned parsing for mutation
//!
//! When you need to modify NBT data, use [`read_owned`]. This creates an
//! owned structure that can be modified in place.
//!
//! ```
//! use na_nbt::{read_owned, OwnedValue};
//! use zerocopy::byteorder::BigEndian;
//!
//! let data = [0x0a, 0x00, 0x00, 0x00]; // Empty compound
//! let mut root: OwnedValue<BigEndian> = read_owned::<BigEndian, BigEndian>(&data).unwrap();
//!
//! // Modify the value
//! if let OwnedValue::Compound(ref mut compound) = root {
//!     compound.insert("score", 100i32);
//!     compound.insert("name", "Steve");
//! }
//! ```
//!
//! # Building NBT from scratch
//!
//! You can construct NBT values programmatically using the owned types:
//!
//! ```
//! use na_nbt::{OwnedValue, OwnedCompound};
//! use zerocopy::byteorder::BigEndian;
//!
//! // Create a compound from scratch
//! let mut compound: OwnedCompound<BigEndian> = OwnedCompound::default();
//! compound.insert("name", "Alex");
//! compound.insert("health", 20i32);
//! compound.insert("score", 100i64);
//!
//! let root = OwnedValue::Compound(compound);
//! ```
//!
//! # Writing NBT to bytes
//!
//! All value types can be serialized back to NBT binary format:
//!
//! ```
//! use na_nbt::{read_owned, OwnedValue};
//! use zerocopy::byteorder::{BigEndian, LittleEndian};
//!
//! let data = [0x0a, 0x00, 0x00, 0x00]; // Empty compound (BigEndian)
//! let root: OwnedValue<BigEndian> = read_owned::<BigEndian, BigEndian>(&data).unwrap();
//!
//! // Write back as BigEndian
//! let bytes = root.write_to_vec::<BigEndian>().unwrap();
//!
//! // Or write to any io::Write
//! let mut buffer = Vec::new();
//! root.write_to_writer::<BigEndian>(&mut buffer).unwrap();
//! ```
//!
//! # Endianness
//!
//! NBT data can be stored in either big-endian (Java Edition) or little-endian
//! (Bedrock Edition) format. This crate supports both through the generic
//! byte order parameter.
//!
//! ```
//! use na_nbt::{read_owned, OwnedValue};
//! use zerocopy::byteorder::{BigEndian, LittleEndian};
//!
//! // Read Java Edition NBT (BigEndian)
//! # let java_data = [0x0a, 0x00, 0x00, 0x00];
//! let java_value: OwnedValue<BigEndian> = read_owned::<BigEndian, BigEndian>(&java_data).unwrap();
//!
//! // Read Bedrock Edition NBT (LittleEndian)
//! # let bedrock_data = [0x0a, 0x00, 0x00, 0x00];
//! let bedrock_value: OwnedValue<LittleEndian> = read_owned::<LittleEndian, LittleEndian>(&bedrock_data).unwrap();
//!
//! // Convert between formats by reading as one endianness and writing as another
//! # let java_data = [0x0a, 0x00, 0x00, 0x00];
//! let value: OwnedValue<LittleEndian> = read_owned::<BigEndian, LittleEndian>(&java_data).unwrap();
//! let bedrock_bytes = value.write_to_vec::<LittleEndian>().unwrap();
//! ```
//!
//! # Trait hierarchy for generic code
//!
//! This crate provides a trait hierarchy for writing generic code that works
//! with any value type:
//!
//! ```text
//! ScopedReadableValue (all types)
//!         ▲
//!         │
//! ┌───────┴───────┐
//! │               │
//! ReadableValue   ScopedWritableValue
//! (borrowed,      (owned, mutable)
//!  shared,                ▲
//!  immutable)             │
//!                   WritableValue
//!                   (mutable)
//! ```
//!
//! - [`ScopedReadableValue`] - Core reading trait, implemented by all value types.
//!   Scoped methods like `get_scoped()` construct views on demand with borrow lifetime.
//!
//! - [`ReadableValue`] - Adds unscoped methods like `get()` that return references to
//!   stored data with document lifetime `'doc`.
//!
//! - [`ScopedWritableValue`] - Adds mutation via scoped methods like `as_byte_array_mut_scoped()`
//!   that construct mutable views on demand.
//!
//! - [`WritableValue`] - Adds unscoped mutable access like `as_byte_array_mut()` returning
//!   references to stored mutable views.
//!
//! Use these traits to write generic functions:
//!
//! ```
//! use na_nbt::{
//!     ScopedReadableValue, ScopedReadableCompound, ScopedReadableList,
//!     ReadableString, ValueScoped,
//! };
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
//! ```
//!
//! # Types overview
//!
//! ## Value types
//!
//! | Type | Description |
//! |------|-------------|
//! | [`BorrowedValue`] | Zero-copy value borrowing from a slice |
//! | [`SharedValue`] | Zero-copy value with `Arc` shared ownership |
//! | [`OwnedValue`] | Fully owned, mutable value |
//! | [`MutableValue`] | Mutable view into an [`OwnedValue`] |
//! | [`ImmutableValue`] | Immutable view into an [`OwnedValue`] |
//!
//! ## Container types
//!
//! Each value type has associated container types for compounds, lists, and strings:
//!
//! - `ReadonlyCompound`, `ReadonlyList`, `ReadonlyString` - For borrowed/shared values
//! - `OwnedCompound`, `OwnedList` - For owned values  
//! - `MutableCompound`, `MutableList` - For mutable views
//! - `ImmutableCompound`, `ImmutableList`, `ImmutableString` - For immutable views
//!
//! ## Other types
//!
//! | Type | Description |
//! |------|-------------|
//! | [`Tag`] | NBT tag type enumeration (End, Byte, Short, Int, etc.) |
//! | [`Error`] | Error type for parsing and writing |
//! | [`Result`] | Result type alias using [`Error`] |
//!
//! # Feature comparison
//!
//! | Feature | `BorrowedValue` | `SharedValue` | `OwnedValue` |
//! |---------|-----------------|---------------|--------------|
//! | Zero-copy parsing | ✅ | ✅ | ❌ |
//! | Modify values | ❌ | ❌ | ✅ |
//! | Outlives source | ❌ | ✅ | ✅ |
//! | Thread-safe sharing | ✅* | ✅ | ✅ |
//! | Clone | ✅ | ✅ | ❌† |
//!
//! \* With appropriate lifetime management  
//! † Use [`MutableValue`] or [`ImmutableValue`] for borrowed access

pub mod de;
pub mod error;
pub mod immutable;
mod index;
pub mod mutable;
pub mod ser;
pub mod tag;
pub mod util;
pub mod value_trait;
mod view;

pub use error::*;
pub use immutable::*;
pub use mutable::*;
pub use ser::{Serializer, to_vec, to_writer};
pub use tag::*;
pub use util::*;
pub use value_trait::*;
