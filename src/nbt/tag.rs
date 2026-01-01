//! NBT tag type definitions.
//!
//! This module defines zero-sized marker types for each NBT tag type.
//! These types are used as type parameters to specify the NBT tag at compile time.
//!
//! # Tag Types
//!
//! ## Primitive Tags
//!
//! | Type | Tag ID | Rust Type | Description |
//! |------|--------|-----------|-------------|
//! | [`End`] | 0 | `()` | Marks the end of a compound tag |
//! | [`Byte`] | 1 | `i8` | Signed 8-bit integer |
//! | [`Short`] | 2 | `I16<O>` | Signed 16-bit integer (endianness-aware) |
//! | [`Int`] | 3 | `I32<O>` | Signed 32-bit integer (endianness-aware) |
//! | [`Long`] | 4 | `I64<O>` | Signed 64-bit integer (endianness-aware) |
//! | [`Float`] | 5 | `F32<O>` | 32-bit floating point (endianness-aware) |
//! | [`Double`] | 6 | `F64<O>` | 64-bit floating point (endianness-aware) |
//!
//! ## Array Tags
//!
//! | Type | Tag ID | Element Type | Description |
//! |------|--------|--------------|-------------|
//! | [`ByteArray`] | 7 | `i8` | Array of signed bytes |
//! | [`IntArray`] | 11 | `I32<O>` | Array of 32-bit integers |
//! | [`LongArray`] | 12 | `I64<O>` | Array of 64-bit integers |
//!
//! ## Container Tags
//!
//! | Type | Tag ID | Description |
//! |------|--------|-------------|
//! | [`String`] | 8 | MUTF-8 encoded string |
//! | [`List`] | 9 | Homogeneous  list |
//! | [`Compound`] | 10 | Key-value map (string → NBT value) |
//!
//! ## Typed Lists
//!
//! [`TypedList<T>`] represents a homogeneous list where elements are of type `T`.

use std::marker::PhantomData;

use zerocopy::byteorder;

use crate::{
    ByteOrder, ConfigMut, ConfigRef, StringMut, VecMut, NBT, NBTBase, CompoundOwn, ListOwn,
    StringOwn, TypedListOwn, VecOwn, TagID,
};

/// Marker type for the End tag (Tag ID: 0).
///
/// Used to mark the end of a compound tag's entries.
/// Has no associated value (represented as `()`).
#[derive(Clone, Copy)]
pub struct End;

/// Marker type for the Byte tag (Tag ID: 1).
///
/// Represents a signed 8-bit integer.
#[derive(Clone, Copy)]
pub struct Byte;

/// Marker type for the Short tag (Tag ID: 2).
///
/// Represents a signed 16-bit integer with endianness awareness.
#[derive(Clone, Copy)]
pub struct Short;

/// Marker type for the Int tag (Tag ID: 3).
///
/// Represents a signed 32-bit integer with endianness awareness.
#[derive(Clone, Copy)]
pub struct Int;

/// Marker type for the Long tag (Tag ID: 4).
///
/// Represents a signed 64-bit integer with endianness awareness.
#[derive(Clone, Copy)]
pub struct Long;

/// Marker type for the Float tag (Tag ID: 5).
///
/// Represents a 32-bit floating point value with endianness awareness.
#[derive(Clone, Copy)]
pub struct Float;

/// Marker type for the Double tag (Tag ID: 6).
///
/// Represents a 64-bit floating point value with endianness awareness.
#[derive(Clone, Copy)]
pub struct Double;

/// Marker type for the ByteArray tag (Tag ID: 7).
///
/// Represents an array of signed bytes (`i8`).
#[derive(Clone, Copy)]
pub struct ByteArray;

/// Marker type for the String tag (Tag ID: 8).
///
/// Represents a MUTF-8 encoded string.
#[derive(Clone, Copy)]
pub struct String;

/// Marker type for the List tag (Tag ID: 9).
///
/// Represents a homogeneous list.
/// For type-safe homogeneous lists, see [`TypedList<T>`].
#[derive(Clone, Copy)]
pub struct List;

/// Marker type for the Compound tag (Tag ID: 10).
///
/// Represents a key-value map where keys are strings and values are NBT tags.
#[derive(Clone, Copy)]
pub struct Compound;

/// Marker type for the IntArray tag (Tag ID: 11).
///
/// Represents an array of signed 32-bit integers with endianness awareness.
#[derive(Clone, Copy)]
pub struct IntArray;

/// Marker type for the LongArray tag (Tag ID: 12).
///
/// Represents an array of signed 64-bit integers with endianness awareness.
#[derive(Clone, Copy)]
pub struct LongArray;

impl NBTBase for End {
    const TAG_ID: TagID = TagID::End;
    type Element = Self;
    type TypeRef<'a, Config: ConfigRef> = ();
    type TypeMut<'a, Config: ConfigMut> = &'a mut ();
    type Type<O: ByteOrder> = ();

    #[inline]
    fn dispatch<A, R>(
        a: A,
        end: impl FnOnce(A) -> R,
        _: impl FnOnce(A) -> R,
        _: impl FnOnce(A) -> R,
    ) -> R {
        end(a)
    }
}

impl NBTBase for Byte {
    const TAG_ID: TagID = TagID::Byte;
    type Element = Self;
    type TypeRef<'a, Config: ConfigRef> = i8;
    type TypeMut<'a, Config: ConfigMut> = &'a mut i8;
    type Type<O: ByteOrder> = i8;

    #[inline]
    fn dispatch<A, R>(
        a: A,
        _: impl FnOnce(A) -> R,
        normal: impl FnOnce(A) -> R,
        _: impl FnOnce(A) -> R,
    ) -> R {
        normal(a)
    }
}

impl NBTBase for Short {
    const TAG_ID: TagID = TagID::Short;
    type Element = Self;
    type TypeRef<'a, Config: ConfigRef> = i16;
    type TypeMut<'a, Config: ConfigMut> = &'a mut byteorder::I16<Config::ByteOrder>;
    type Type<O: ByteOrder> = byteorder::I16<O>;

    #[inline]
    fn dispatch<A, R>(
        a: A,
        _: impl FnOnce(A) -> R,
        normal: impl FnOnce(A) -> R,
        _: impl FnOnce(A) -> R,
    ) -> R {
        normal(a)
    }
}

impl NBTBase for Int {
    const TAG_ID: TagID = TagID::Int;
    type Element = Self;
    type TypeRef<'a, Config: ConfigRef> = i32;
    type TypeMut<'a, Config: ConfigMut> = &'a mut byteorder::I32<Config::ByteOrder>;
    type Type<O: ByteOrder> = byteorder::I32<O>;

    #[inline]
    fn dispatch<A, R>(
        a: A,
        _: impl FnOnce(A) -> R,
        normal: impl FnOnce(A) -> R,
        _: impl FnOnce(A) -> R,
    ) -> R {
        normal(a)
    }
}

impl NBTBase for Long {
    const TAG_ID: TagID = TagID::Long;
    type Element = Self;
    type TypeRef<'a, Config: ConfigRef> = i64;
    type TypeMut<'a, Config: ConfigMut> = &'a mut byteorder::I64<Config::ByteOrder>;
    type Type<O: ByteOrder> = byteorder::I64<O>;

    #[inline]
    fn dispatch<A, R>(
        a: A,
        _: impl FnOnce(A) -> R,
        normal: impl FnOnce(A) -> R,
        _: impl FnOnce(A) -> R,
    ) -> R {
        normal(a)
    }
}

impl NBTBase for Float {
    const TAG_ID: TagID = TagID::Float;
    type Element = Self;
    type TypeRef<'a, Config: ConfigRef> = f32;
    type TypeMut<'a, Config: ConfigMut> = &'a mut byteorder::F32<Config::ByteOrder>;
    type Type<O: ByteOrder> = byteorder::F32<O>;

    #[inline]
    fn dispatch<A, R>(
        a: A,
        _: impl FnOnce(A) -> R,
        normal: impl FnOnce(A) -> R,
        _: impl FnOnce(A) -> R,
    ) -> R {
        normal(a)
    }
}

impl NBTBase for Double {
    const TAG_ID: TagID = TagID::Double;
    type Element = Self;
    type TypeRef<'a, Config: ConfigRef> = f64;
    type TypeMut<'a, Config: ConfigMut> = &'a mut byteorder::F64<Config::ByteOrder>;
    type Type<O: ByteOrder> = byteorder::F64<O>;

    #[inline]
    fn dispatch<A, R>(
        a: A,
        _: impl FnOnce(A) -> R,
        normal: impl FnOnce(A) -> R,
        _: impl FnOnce(A) -> R,
    ) -> R {
        normal(a)
    }
}

impl NBTBase for ByteArray {
    const TAG_ID: TagID = TagID::ByteArray;
    type Element = Byte;
    type TypeRef<'a, Config: ConfigRef> = Config::ByteArray<'a>;
    type TypeMut<'a, Config: ConfigMut> = VecMut<'a, i8>;
    type Type<O: ByteOrder> = VecOwn<i8>;

    #[inline]
    fn dispatch<A, R>(
        a: A,
        _: impl FnOnce(A) -> R,
        normal: impl FnOnce(A) -> R,
        _: impl FnOnce(A) -> R,
    ) -> R {
        normal(a)
    }
}

impl NBTBase for String {
    const TAG_ID: TagID = TagID::String;
    type Element = Byte;
    type TypeRef<'a, Config: ConfigRef> = Config::String<'a>;
    type TypeMut<'a, Config: ConfigMut> = StringMut<'a>;
    type Type<O: ByteOrder> = StringOwn;

    #[inline]
    fn dispatch<A, R>(
        a: A,
        _: impl FnOnce(A) -> R,
        normal: impl FnOnce(A) -> R,
        _: impl FnOnce(A) -> R,
    ) -> R {
        normal(a)
    }
}

impl NBTBase for List {
    const TAG_ID: TagID = TagID::List;
    type Element = Self;
    type TypeRef<'a, Config: ConfigRef> = Config::List<'a>;
    type TypeMut<'a, Config: ConfigMut> = Config::ListMut<'a>;
    type Type<O: ByteOrder> = ListOwn<O>;

    #[inline]
    fn dispatch<A, R>(
        a: A,
        _: impl FnOnce(A) -> R,
        normal: impl FnOnce(A) -> R,
        _: impl FnOnce(A) -> R,
    ) -> R {
        normal(a)
    }
}

impl NBTBase for Compound {
    const TAG_ID: TagID = TagID::Compound;
    type Element = Self;
    type TypeRef<'a, Config: ConfigRef> = Config::Compound<'a>;
    type TypeMut<'a, Config: ConfigMut> = Config::CompoundMut<'a>;
    type Type<O: ByteOrder> = CompoundOwn<O>;

    #[inline]
    fn dispatch<A, R>(
        a: A,
        _: impl FnOnce(A) -> R,
        normal: impl FnOnce(A) -> R,
        _: impl FnOnce(A) -> R,
    ) -> R {
        normal(a)
    }
}

impl NBTBase for IntArray {
    const TAG_ID: TagID = TagID::IntArray;
    type Element = Int;
    type TypeRef<'a, Config: ConfigRef> = Config::IntArray<'a>;
    type TypeMut<'a, Config: ConfigMut> = VecMut<'a, byteorder::I32<Config::ByteOrder>>;
    type Type<O: ByteOrder> = VecOwn<byteorder::I32<O>>;

    #[inline]
    fn dispatch<A, R>(
        a: A,
        _: impl FnOnce(A) -> R,
        normal: impl FnOnce(A) -> R,
        _: impl FnOnce(A) -> R,
    ) -> R {
        normal(a)
    }
}

impl NBTBase for LongArray {
    const TAG_ID: TagID = TagID::LongArray;
    type Element = Long;
    type TypeRef<'a, Config: ConfigRef> = Config::LongArray<'a>;
    type TypeMut<'a, Config: ConfigMut> = VecMut<'a, byteorder::I64<Config::ByteOrder>>;
    type Type<O: ByteOrder> = VecOwn<byteorder::I64<O>>;

    #[inline]
    fn dispatch<A, R>(
        a: A,
        _: impl FnOnce(A) -> R,
        normal: impl FnOnce(A) -> R,
        _: impl FnOnce(A) -> R,
    ) -> R {
        normal(a)
    }
}

/// Marker type for a typed list (Tag ID: 9, same as List).
#[derive(Clone, Copy)]
pub struct TypedList<T: NBTBase>(PhantomData<T>);

impl<T: NBT> NBTBase for TypedList<T> {
    const TAG_ID: TagID = TagID::List;
    type Element = T;
    type TypeRef<'a, Config: ConfigRef> = Config::TypedList<'a, T>;
    type TypeMut<'a, Config: ConfigMut> = Config::TypedListMut<'a, T>;
    type Type<O: ByteOrder> = TypedListOwn<O, T>;

    #[inline]
    fn dispatch<A, R>(
        a: A,
        _: impl FnOnce(A) -> R,
        _: impl FnOnce(A) -> R,
        typed_list: impl FnOnce(A) -> R,
    ) -> R {
        typed_list(a)
    }
}
