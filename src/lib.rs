//! # na_nbt
//!
//! A high-performance NBT (Named Binary Tag) parser with serde support.
//! NBT is a binary format used by Minecraft.
//!
//! # Operating on untyped NBT values
//!
//! There are three ways to parse NBT values:
//! 1. **Borrowed**: [`read_borrowed`] - Zero-copy reading from a borrowed slice.
//! ```rust
//! use na_nbt::{BE, ValueRef, read_borrowed, tag};
//! let data = include_bytes!("../fuzz/in/level");
//! let doc = read_borrowed::<BE>(data).unwrap();
//! let nbt = doc.root();
//! let o = nbt
//!     .get("Data")
//!     .and_then(|d| d.get("Player"))
//!     .and_then(|p| p.get("Attributes"))
//!     .and_then(|a| a.get(2))
//!     .and_then(|a| a.get("Modifiers"))
//!     .and_then(|m| m.get(0))
//!     .and_then(|m| m.get_::<tag::Int>("Operation"))
//!     // or:
//!     //  .and_then(|m| m.get("Operation"))
//!     //  .and_then(|o| o.into_::<tag::Int>())
//!     .unwrap();
//! assert_eq!(o, 2);
//! ```
//! There is a constraint that `doc` must not outlive the source, and `nbt` must not outlive `doc`.
//!
//! 2. **Shared**: [`read_shared`] - Zero-copy reading from a [`bytes::Bytes`] with shared ownership.
//! ```rust
//! use na_nbt::{BE, ValueRef, read_shared, tag};
//! let data = include_bytes!("../fuzz/in/level");
//! let nbt = read_shared::<BE>(bytes::Bytes::copy_from_slice(data)).unwrap();
//! let player = nbt
//!     .get("Data")
//!     .and_then(|d| d.get_::<tag::Compound>("Player"))
//!     .unwrap();
//! ```
//! Similar to borrowed, but there is not a `doc`, and `nbt`'s lifetime is not tied to the source.
//!
//! 3. **Owned**: [`read_owned`] - Parses NBT into fully owned structures that can be modified.
//! ```rust
//! use na_nbt::{BE, ValueMut, Writable, read_owned, tag};
//! let data = include_bytes!("../fuzz/in/level");
//! let mut nbt = read_owned::<BE, BE>(data).unwrap();
//! let mut data = nbt.get_mut("Data").unwrap();
//! let mut player = data.get_mut("Player").unwrap();
//! player.get_mut("Health").unwrap().into_::<tag::Short>().unwrap().set(10);
//! player.get_mut_::<tag::Short>("Health").unwrap().set(10);
//! *player.get_mut_::<tag::Byte>("Sleeping").unwrap() = 1;
//! let file = std::fs::File::create("new.nbt").unwrap();
//! nbt.write_to_writer::<BE>(file).unwrap();
//! ```
//!
//! # Generic functions and constructing NBT values
//!
//! There are three groups of traits and structs provided to write generic functions and construct NBT values.
//!
//! in fact, methods on the result of `read_xxx` are provided by these traits.
//!
//! 1. Ref: [`ValueRef`], [`ListRef`], [`CompoundRef`] and [`TypedListRef`].
//! 2. Mut: [`ValueMut`], [`ListMut`], [`CompoundMut`] and [`TypedListMut`].
//! 3. Own: [`ValueOwn`], [`ListOwn`], [`CompoundOwn`] and [`TypedListOwn`].
//!
//! ```rust
//! fn example<'s>(nbt: &mut impl CompoundMut<'s>) {
//!     let mut comp = CompoundOwn::default();
//!     comp.insert("Int", 1);
//!     comp.insert("String", "2");
//!     comp.insert("ByteArray", [1i8, 2, 3]);
//!     nbt.insert("Compound", comp);
//!     let mut list = TypedListOwn::default();
//!     list.push(1);
//!     list.push(2);
//!     list.push(3);
//!     nbt.insert("List", list);
//! }
//! ```
//!
//! ```rust
//! use na_nbt::{ValueRef, VisitRef, ListBase, ValueBase};
//!
//! fn dump<'s>(value: &impl ValueRef<'s>, indent: usize) -> String {
//!     let pad = "  ".repeat(indent);
//!     value.visit(|v| match v {
//!         VisitRef::End(_) => format!("{pad}End"),
//!         VisitRef::Byte(v) => format!("{pad}Byte({v})"),
//!         VisitRef::Short(v) => format!("{pad}Short({v})"),
//!         VisitRef::Int(v) => format!("{pad}Int({v})"),
//!         VisitRef::Long(v) => format!("{pad}Long({v})"),
//!         VisitRef::Float(v) => format!("{pad}Float({v})"),
//!         VisitRef::Double(v) => format!("{pad}Double({v})"),
//!         VisitRef::ByteArray(v) => format!("{pad}ByteArray({} bytes)", v.len()),
//!         VisitRef::String(v) => format!("{pad}String({:?})", v.decode_lossy()),
//!         VisitRef::IntArray(v) => format!("{pad}IntArray({} ints)", v.len()),
//!         VisitRef::LongArray(v) => format!("{pad}LongArray({} longs)", v.len()),
//!         VisitRef::List(list) => {
//!             let mut out = format!("{pad}List[{}] {{\n", list.len());
//!             for item in list.iter() {
//!                 out.push_str(&dump(&item, indent + 1));
//!                 out.push('\n');
//!             }
//!             out.push_str(&format!("{pad}}}"));
//!             out
//!         }
//!         VisitRef::Compound(compound) => {
//!             let mut out = format!("{pad}Compound {{\n");
//!             for (key, val) in compound.iter() {
//!                 let nested = dump(&val, indent + 1);
//!                 out.push_str(&format!(
//!                     "{}  {:?}: {}\n",
//!                     pad,
//!                     key.decode_lossy(),
//!                     nested.trim_start()
//!                 ));
//!             }
//!             out.push_str(&format!("{pad}}}"));
//!             out
//!         }
//!     })
//! }
//! ```
//!
//! # Serde support
//!
//! na_nbt provides serde support. For serde-to-NBT type mapping, see [`tag_probe`].
//! ```rust
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize, Deserialize, Debug, PartialEq)]
//! struct Address {
//!     street: String,
//!     city: String,
//! }
//!
//! fn an_address() -> na_nbt::Result<()> {
//!     let address = Address {
//!         street: "10 Downing Street".to_owned(),
//!         city: "London".to_owned(),
//!     };
//!
//!     let j = na_nbt::to_vec_be(&address)?;
//!     let addr: Address = na_nbt::from_slice_be(&j)?;
//!     assert_eq!(addr, address);
//!
//!     Ok(())
//! }
//! ```

pub use zerocopy::BigEndian;
pub use zerocopy::LittleEndian;
pub use zerocopy::NativeEndian;
pub use zerocopy::byteorder::{BE, F32, F64, I16, I32, I64, I128, LE, U16, U32, U64, U128};

pub mod error;
mod immutable;
mod index;
mod mutable;
mod nbt;
#[cfg(feature = "serde")]
mod ser_de;
mod util;
mod value;
mod view;
mod write;

pub use error::{Error, Result};
pub use immutable::{read_borrowed, read_shared};
use mutable::IntoNBT;
pub use mutable::{
    CompoundOwn, ListOwn, TypedListOwn, ValueOwn, read_owned, read_owned_from_reader,
};
use nbt::*;
pub use nbt::{TagID, tag};
#[cfg(feature = "serde")]
pub use ser_de::*;
pub use util::MUTF8Str;
use util::*;
use value::*;
pub use value::{
    CompoundBase, CompoundMut, CompoundRef, ListBase, ListMut, ListRef, MapMut, MapRef,
    TypedListBase, TypedListMut, TypedListRef, ValueBase, ValueMut, ValueRef, VisitMut,
    VisitMutShared, VisitRef, Writable,
};
use view::*;
