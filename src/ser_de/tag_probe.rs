//! Zero-cost tag type detection for serde values.
//!
//! This module provides `tag_of`, a function that determines the NBT tag
//! type of a Rust value without performing actual serialization. This is useful for
//! pre-flight type checking and determining the appropriate NBT representation before
//! encoding.
//!
//! # Serde-to-NBT Type Mapping
//!
//! The following sections show how each serde data model type maps to NBT tags,
//! organized by the serde data model categories.
//!
//! ## Primitive Types (14 types)
//!
//! Signed and unsigned integers map directly to their NBT counterparts, with
//! unsigned values reinterpreted as signed. Floating-point and character types
//! map directly.
//!
//! | Serde Type | NBT Tag | Representation |
//! |------------|---------|----------------|
//! | `bool` | `TAG_Byte` | `0` or `1` |
//! | `i8` | `TAG_Byte` | Direct |
//! | `i16` | `TAG_Short` | Direct |
//! | `i32` | `TAG_Int` | Direct |
//! | `i64` | `TAG_Long` | Direct |
//! | `i128` | `TAG_Int_Array` | 4-element array (Minecraft UUID) |
//! | `u8` | `TAG_Byte` | Reinterpreted as signed |
//! | `u16` | `TAG_Short` | Reinterpreted as signed |
//! | `u32` | `TAG_Int` | Reinterpreted as signed |
//! | `u64` | `TAG_Long` | Reinterpreted as signed |
//! | `u128` | `TAG_Int_Array` | Reinterpreted as i128, then 4-element array |
//! | `f32` | `TAG_Float` | Direct |
//! | `f64` | `TAG_Double` | Direct |
//! | `char` | `TAG_Int` | UTF-32 code point |
//!
//! ## `string`
//!
//! Strings are encoded as MUTF-8 (modified UTF-8), NBT's string encoding format.
//!
//! | Serde Type | NBT Tag |
//! |------------|---------|
//! | `string` | `TAG_String` |
//!
//! ## `byte array`
//!
//! Byte arrays serialize as length-prefixed byte sequences.
//!
//! | Serde Type | NBT Tag |
//! |------------|---------|
//! | `byte array` | `TAG_Byte_Array` |
//!
//! ## `option`
//!
//! Options serialize as compounds, using an empty string key to wrap the inner value.
//!
//! | Serde Type | NBT Tag | Representation |
//! |------------|---------|----------------|
//! | `option` (Some) | `TAG_Compound` | `Compound { "" : <value> }` |
//! | `option` (None) | `TAG_Compound` | `Compound { }` (empty) |
//!
//! Empty compounds have a payload consisting of a single `TAG_End` terminator byte.
//!
//! ## `unit` and `unit_struct`
//!
//! Unit types serialize as empty compounds.
//!
//! | Serde Type | NBT Tag | Representation |
//! |------------|---------|----------------|
//! | `unit` | `TAG_Compound` | `Compound { }` (empty) |
//! | `unit_struct` | `TAG_Compound` | `Compound { }` (empty) |
//!
//! ## `unit_variant`
//!
//! Enum unit variants (variants without associated data) serialize as their discriminant index.
//!
//! | Serde Type | NBT Tag | Representation |
//! |------------|---------|----------------|
//! | `unit_variant` | `TAG_Int` | Variant index as integer |
//!
//! ## `newtype_struct`
//!
//! By default, newtype structs serialize as their inner value. Special marker names
//! trigger native NBT array/list serialization (see [Special Markers](#special-markers) below).
//!
//! | Serde Type | NBT Tag | Representation |
//! |------------|---------|----------------|
//! | `newtype_struct` | Varies | Inner value, or special marker |
//!
//! ## `newtype_variant`
//!
//! Enum newtype variants serialize as compounds with the variant name as the key.
//!
//! | Serde Type | NBT Tag | Representation |
//! |------------|---------|----------------|
//! | `newtype_variant` | `TAG_Compound` | `Compound { "<variant>" : <value> }` |
//!
//! ## `seq`
//!
//! Sequences serialize as heterogeneous wrapped lists. For homogeneous lists,
//! use the markers in [`crate::marker`].
//!
//! | Serde Type | NBT Tag | Representation |
//! |------------|---------|----------------|
//! | `seq` | `TAG_List` | Wrapped heterogeneous |
//!
//! Heterogeneous sequences use the "wrapped list" representation:
//! ```text
//! [value1, value2, ...] → List [ Compound { "" : value1 }, Compound { "" : value2 }, ... ]
//! ```
//!
//! ## `tuple` and `tuple_struct`
//!
//! Tuples and tuple structs serialize as heterogeneous wrapped lists.
//!
//! | Serde Type | NBT Tag | Representation |
//! |------------|---------|----------------|
//! | `tuple` | `TAG_List` | Wrapped heterogeneous |
//! | `tuple_struct` | `TAG_List` | Wrapped heterogeneous |
//!
//! ## `tuple_variant`
//!
//! Enum tuple variants serialize as compounds containing a list.
//!
//! | Serde Type | NBT Tag | Representation |
//! |------------|---------|----------------|
//! | `tuple_variant` | `TAG_Compound` | `Compound { "<variant>" : List [...] }` |
//!
//! ## `map`
//!
//! Maps serialize as compounds. NBT requires compound keys to be strings, so all
//! map keys are encoded as MUTF-8 strings. The key encoding rules are:
//!
//! | Key Type | String Encoding | Examples |
//! |----------|----------------|----------|
//! | `string` | Direct | `"hello"` → `"hello"` |
//! | `bool` | "true" or "false" | `true` → `"true"` |
//! | Integers | Decimal string | `42` → `"42"`, `-1` → `"-1"` |
//! | Floats | Decimal string | `3.14` → `"3.14"` |
//! | `char` | Single character | `'a'` → `"a"` |
//! | `bytes` | Base64 encoding | `b"\xff"` → `"/w=="` |
//! | `None` | "None" | `None` → `"None"` |
//! | `Some(T)` | "Some(value)" | `Some(42)` → `"Some(42)"` |
//! | `newtype_struct` | "Name(value)" | `Millimeters(5)` → `"Millimeters(5)"` |
//! | `unit_struct` | Struct name | `Unit` → `"Unit"` |
//! | `unit` | Empty string | `()` → `""` |
//! | `unit_variant` | Not allowed | Returns `Error::KEY` |
//! | `newtype_variant` | Not allowed | Returns `Error::KEY` |
//! | `seq`, `tuple`, `tuple_struct` | Not allowed | Returns `Error::KEY` |
//! | `map`, `struct`, `struct_variant` | Not allowed | Returns `Error::KEY` |
//!
//! The `Some(value)` and `newtype_struct` encodings preserve type information,
//! allowing values like `Option<String>` to be distinguishable during deserialization.
//! For example, `Some("None")` encodes as `"Some(None)"` while `None` encodes as
//! `"None"`, so both can coexist as keys in the same map.
//!
//! | Serde Type | NBT Tag | Representation |
//! |------------|---------|----------------|
//! | `map` | `TAG_Compound` | Key-value pairs as compound entries |
//!
//! ## `struct`
//!
//! Structs serialize as compounds with field names as keys.
//!
//! | Serde Type | NBT Tag | Representation |
//! |------------|---------|----------------|
//! | `struct` | `TAG_Compound` | Fields as named compound entries |
//!
//! ## `struct_variant`
//!
//! Enum struct variants serialize as compounds containing a nested compound.
//!
//! | Serde Type | NBT Tag | Representation |
//! |------------|---------|----------------|
//! | `struct_variant` | `TAG_Compound` | `Compound { "<variant>" : Compound {...} }` |
//!
//! ## Special Markers
//!
//! The following newtype struct names trigger native NBT array/list serialization:
//!
//! | Marker Name | NBT Tag | Purpose |
//! |-------------|---------|---------|
//! | `"na_nbt:byte_array"` | `TAG_Byte_Array` | Direct byte array serialization |
//! | `"na_nbt:int_array"` | `TAG_Int_Array` | Direct i32 array serialization |
//! | `"na_nbt:long_array"` | `TAG_Long_Array` | Direct i64 array serialization |
//! | `"na_nbt:list"` | `TAG_List` | Homogeneous list serialization |
//!
//! These are internal implementation details used by the public marker modules in
//! [`crate::marker`].

use serde::{Serialize, ser};

use crate::{Error, Result, TagID};

/// Zero-allocation probe serializer for type detection.
///
/// `TagProbe` implements `serde::Serializer` but produces only a `TagID` instead
/// of serialized bytes. Every method returns `Ok(TagID)` and never fails,
/// which enables the use of `unwrap_unchecked()` in `tag_of` without safety concerns.
struct TagProbe;

impl ser::Serializer for TagProbe {
    type Ok = TagID;
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        // Booleans serialize as bytes (0 or 1) to match NBT conventions.
        self.serialize_i8(v as i8)
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok> {
        Ok(TagID::Byte)
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok> {
        Ok(TagID::Short)
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok> {
        Ok(TagID::Int)
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok> {
        Ok(TagID::Long)
    }

    #[cfg(feature = "i128")]
    fn serialize_i128(self, _v: i128) -> Result<Self::Ok> {
        // i128 values are encoded as IntArray to match Minecraft's UUID representation.
        // A UUID (128 bits) is stored as an IntArray of 4 integers.
        Ok(TagID::IntArray)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        // Unsigned integers are reinterpreted as their signed equivalents.
        self.serialize_i8(v as i8)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        self.serialize_i16(v as i16)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        self.serialize_i32(v as i32)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        self.serialize_i64(v as i64)
    }

    #[cfg(feature = "i128")]
    fn serialize_u128(self, v: u128) -> Result<Self::Ok> {
        self.serialize_i128(v as i128)
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok> {
        Ok(TagID::Float)
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok> {
        Ok(TagID::Double)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        // Chars are serialized as their UTF-32 code point (i32).
        self.serialize_i32(v as i32)
    }

    fn serialize_str(self, _v: &str) -> Result<Self::Ok> {
        Ok(TagID::String)
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok> {
        Ok(TagID::ByteArray)
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        // None serializes as an empty Compound. The payload is a single TAG_End.
        self.serialize_unit()
    }

    fn serialize_some<T: ?Sized + Serialize>(self, _v: &T) -> Result<Self::Ok> {
        // Some(value) serializes as Compound { "" : value }.
        // The empty string key allows the inner value to be any NBT type,
        // since NBT compounds require string keys and we need a single
        // well-defined key to hold the wrapped value.
        Ok(TagID::Compound)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        // Unit types serialize as an empty Compound. The payload is a single TAG_End.
        Ok(TagID::Compound)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        idx: u32,
        _var: &'static str,
    ) -> Result<Self::Ok> {
        // Unit variants (enum discriminants) serialize as their integer index.
        self.serialize_u32(idx)
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok> {
        // Special marker names are recognized to signal native NBT array types.
        // These correspond to the public markers in crate::marker.
        match name {
            "na_nbt:byte_array" => Ok(TagID::ByteArray),
            "na_nbt:int_array" => Ok(TagID::IntArray),
            "na_nbt:long_array" => Ok(TagID::LongArray),
            "na_nbt:list" => Ok(TagID::List),
            // For other newtype structs, probe the inner value.
            _ => value.serialize(self),
        }
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _idx: u32,
        _var: &'static str,
        _value: &T,
    ) -> Result<Self::Ok> {
        // Newtype variants (enum with associated data) serialize as Compound.
        Ok(TagID::Compound)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _idx: u32,
        _var: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(self)
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _idx: u32,
        _var: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Ok(self)
    }
}

impl ser::SerializeSeq for TagProbe {
    type Ok = TagID;
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<()> {
        // Elements are ignored; we only care about the container type.
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        // Heterogeneous sequences serialize as a wrapped List.
        Ok(TagID::List)
    }
}

impl ser::SerializeTuple for TagProbe {
    type Ok = TagID;
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<()> {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        // Tuples also serialize as a wrapped List.
        Ok(TagID::List)
    }
}

impl ser::SerializeTupleStruct for TagProbe {
    type Ok = TagID;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<()> {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(TagID::List)
    }
}

impl ser::SerializeTupleVariant for TagProbe {
    type Ok = TagID;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<()> {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        // Tuple variants (enum variants with tuple fields) serialize as Compound.
        Ok(TagID::Compound)
    }
}

impl ser::SerializeMap for TagProbe {
    type Ok = TagID;
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, _key: &T) -> Result<()> {
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<()> {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(TagID::Compound)
    }
}

impl ser::SerializeStruct for TagProbe {
    type Ok = TagID;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        _key: &'static str,
        _value: &T,
    ) -> Result<()> {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(TagID::Compound)
    }
}

impl ser::SerializeStructVariant for TagProbe {
    type Ok = TagID;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        _key: &'static str,
        _value: &T,
    ) -> Result<()> {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        // Struct variants (enum variants with struct fields) serialize as Compound.
        Ok(TagID::Compound)
    }
}

/// Returns the NBT tag type that a value would serialize to.
///
/// This function performs a zero-copy probe of the value's serialization
/// without generating any serialized output. The type detection is performed
/// at compile time where possible, with zero runtime allocation overhead.
///
/// # Common Use Cases
///
/// - **Pre-flight validation**: Determine the NBT representation before encoding
/// - **Type checking**: Validate that a type serializes as expected
/// - **Conditional logic**: Branch based on the resulting tag type
///
/// # Examples
///
/// Basic type detection:
///
/// ```rust
/// use na_nbt::tag_of;
/// use na_nbt::TagID;
///
/// assert_eq!(tag_of(&42i32), TagID::Int);
/// assert_eq!(tag_of(&"hello"), TagID::String);
/// assert_eq!(tag_of(&vec![1i32, 2, 3]), TagID::List);
/// ```
///
/// Option and unit types serialize as compounds:
///
/// ```rust
/// use na_nbt::tag_of;
/// use na_nbt::TagID;
///
/// assert_eq!(tag_of(&Some(42i32)), TagID::Compound);
/// assert_eq!(tag_of(&Option::<i32>::None), TagID::Compound);
/// ```
///
/// # Safety
///
/// The function uses `unsafe { ... }.unwrap_unchecked()` internally. This is safe
/// because `TagProbe` is designed to never return an error—all serialization
/// methods unconditionally return `Ok(TagID)`. Since the `Result` can never be
/// `Err`, `unwrap_unchecked()` will never trigger undefined behavior.
#[inline]
pub fn tag_of<T: ?Sized + Serialize>(value: &T) -> TagID {
    unsafe { value.serialize(TagProbe).unwrap_unchecked() }
}
