//! NBT tag type enumeration.
//!
//! This module contains the [`Tag`] enum which represents all NBT tag types.
//! Each tag type corresponds to a different kind of value in the NBT format.
//!
//! # Tag Types
//!
//! | ID | Tag | Rust Type |
//! |----|-----|-----------|
//! | 0 | End | `()` |
//! | 1 | Byte | `i8` |
//! | 2 | Short | `i16` |
//! | 3 | Int | `i32` |
//! | 4 | Long | `i64` |
//! | 5 | Float | `f32` |
//! | 6 | Double | `f64` |
//! | 7 | ByteArray | `[i8]` |
//! | 8 | String | Modified UTF-8 string |
//! | 9 | List | Homogeneous list of values |
//! | 10 | Compound | Map of string keys to values |
//! | 11 | IntArray | `[i32]` |
//! | 12 | LongArray | `[i64]` |
//!
//! # Example
//!
//! ```
//! use na_nbt::{Tag, read_borrowed};
//! use zerocopy::byteorder::BigEndian;
//!
//! let data = [0x0a, 0x00, 0x00, 0x00]; // Empty compound
//! let doc = read_borrowed::<BigEndian>(&data).unwrap();
//! let root = doc.root();
//!
//! assert_eq!(root.tag_id(), Tag::Compound);
//! assert!(root.tag_id().is_composite());
//! ```

use std::marker::PhantomData;

use crate::ReadableConfig;

/// Represents an NBT tag type.
///
/// This enum corresponds to the tag type byte in the NBT binary format.
/// Each variant represents a different kind of value that can be stored.
///
/// # Categories
///
/// Tags can be categorized using the helper methods:
///
/// - **Primitive** ([`is_primitive`](Tag::is_primitive)): End, Byte, Short, Int, Long, Float, Double
/// - **Array** ([`is_array`](Tag::is_array)): ByteArray, IntArray, LongArray
/// - **Composite** ([`is_composite`](Tag::is_composite)): List, Compound
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum TagID {
    /// End tag (0) - Marks the end of a compound.
    End = 0,
    /// Byte tag (1) - A signed 8-bit integer.
    Byte = 1,
    /// Short tag (2) - A signed 16-bit integer.
    Short = 2,
    /// Int tag (3) - A signed 32-bit integer.
    Int = 3,
    /// Long tag (4) - A signed 64-bit integer.
    Long = 4,
    /// Float tag (5) - A 32-bit IEEE 754 floating point number.
    Float = 5,
    /// Double tag (6) - A 64-bit IEEE 754 floating point number.
    Double = 6,
    /// ByteArray tag (7) - An array of signed bytes.
    ByteArray = 7,
    /// String tag (8) - A Modified UTF-8 encoded string.
    String = 8,
    /// List tag (9) - A list of values, all of the same type.
    List = 9,
    /// Compound tag (10) - A map of string keys to NBT values.
    Compound = 10,
    /// IntArray tag (11) - An array of signed 32-bit integers.
    IntArray = 11,
    /// LongArray tag (12) - An array of signed 64-bit integers.
    LongArray = 12,
}

impl TagID {
    /// Creates a `Tag` from a raw byte value without validation.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `value` is a valid tag type (0-12).
    /// Passing an invalid value results in undefined behavior.
    pub(crate) unsafe fn from_u8_unchecked(value: u8) -> Self {
        unsafe { std::mem::transmute(value) }
    }

    /// Returns `true` if this is a primitive tag type.
    ///
    /// Primitive tags are: End, Byte, Short, Int, Long, Float, Double.
    /// These tags store their values directly without additional structure.
    ///
    /// # Example
    ///
    /// ```
    /// use na_nbt::Tag;
    ///
    /// assert!(Tag::Int.is_primitive());
    /// assert!(Tag::Double.is_primitive());
    /// assert!(!Tag::List.is_primitive());
    /// assert!(!Tag::ByteArray.is_primitive());
    /// ```
    pub const fn is_primitive(self) -> bool {
        matches!(
            self,
            Self::End
                | Self::Byte
                | Self::Short
                | Self::Int
                | Self::Long
                | Self::Float
                | Self::Double
        )
    }

    /// Returns `true` if this is an array tag type.
    ///
    /// Array tags are: ByteArray, IntArray, LongArray.
    /// These store contiguous sequences of primitive values.
    ///
    /// # Example
    ///
    /// ```
    /// use na_nbt::Tag;
    ///
    /// assert!(Tag::ByteArray.is_array());
    /// assert!(Tag::IntArray.is_array());
    /// assert!(Tag::LongArray.is_array());
    /// assert!(!Tag::List.is_array());
    /// ```
    pub const fn is_array(self) -> bool {
        matches!(self, Self::ByteArray | Self::IntArray | Self::LongArray)
    }

    /// Returns `true` if this is a composite tag type.
    ///
    /// Composite tags are: List, Compound.
    /// These contain other NBT values as children.
    ///
    /// # Example
    ///
    /// ```
    /// use na_nbt::Tag;
    ///
    /// assert!(Tag::List.is_composite());
    /// assert!(Tag::Compound.is_composite());
    /// assert!(!Tag::Int.is_composite());
    /// assert!(!Tag::ByteArray.is_composite());
    /// ```
    pub const fn is_composite(self) -> bool {
        matches!(self, Self::List | Self::Compound)
    }
}

// todo: Sealed NBT
pub trait NBT {
    const TAG_ID: TagID;
    type Type<'a, Config: ReadableConfig>;
}

pub(crate) trait NBTExtract<'doc, Config: ReadableConfig, V>: NBT {
    fn extract(value: V) -> Option<Self::Type<'doc, Config>>;
}

pub struct TagEnd;

impl NBT for TagEnd {
    const TAG_ID: TagID = TagID::End;
    type Type<'a, Config: ReadableConfig> = ();
}

pub struct TagByte;

impl NBT for TagByte {
    const TAG_ID: TagID = TagID::Byte;
    type Type<'a, Config: ReadableConfig> = i8;
}

pub struct TagShort;

impl NBT for TagShort {
    const TAG_ID: TagID = TagID::Short;
    type Type<'a, Config: ReadableConfig> = i16;
}

pub struct TagInt;

impl NBT for TagInt {
    const TAG_ID: TagID = TagID::Int;
    type Type<'a, Config: ReadableConfig> = i32;
}

pub struct TagLong;

impl NBT for TagLong {
    const TAG_ID: TagID = TagID::Long;
    type Type<'a, Config: ReadableConfig> = i64;
}

pub struct TagFloat;

impl NBT for TagFloat {
    const TAG_ID: TagID = TagID::Float;
    type Type<'a, Config: ReadableConfig> = f32;
}

pub struct TagDouble;

impl NBT for TagDouble {
    const TAG_ID: TagID = TagID::Double;
    type Type<'a, Config: ReadableConfig> = f64;
}

pub struct TagByteArray;

impl NBT for TagByteArray {
    const TAG_ID: TagID = TagID::ByteArray;
    type Type<'a, Config: ReadableConfig> = Config::ByteArray<'a>;
}

pub struct TagString;

impl NBT for TagString {
    const TAG_ID: TagID = TagID::String;
    type Type<'a, Config: ReadableConfig> = Config::String<'a>;
}

pub struct TagList;

impl NBT for TagList {
    const TAG_ID: TagID = TagID::List;
    type Type<'a, Config: ReadableConfig> = Config::List<'a>;
}

pub struct TagTypedList<T: NBT>(PhantomData<T>);

impl<T: NBT> NBT for TagTypedList<T> {
    const TAG_ID: TagID = TagID::List;
    type Type<'a, Config: ReadableConfig> = Config::TypedList<'a, T>;
}

pub struct TagCompound;

impl NBT for TagCompound {
    const TAG_ID: TagID = TagID::Compound;
    type Type<'a, Config: ReadableConfig> = Config::Compound<'a>;
}

pub struct TagIntArray;

impl NBT for TagIntArray {
    const TAG_ID: TagID = TagID::IntArray;
    type Type<'a, Config: ReadableConfig> = Config::IntArray<'a>;
}

pub struct TagLongArray;

impl NBT for TagLongArray {
    const TAG_ID: TagID = TagID::LongArray;
    type Type<'a, Config: ReadableConfig> = Config::LongArray<'a>;
}
