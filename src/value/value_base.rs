//! Base traits for NBT values.
//!
//! This module defines foundational traits that are used throughout
//! the value system:
//!
//! - [`Writable`] - Trait for serializing NBT values to bytes
//! - [`ValueBase`] - Base trait for all NBT values
//! - [`ListBase`] - Base trait for NBT lists
//! - [`TypedListBase`] - Base trait for typed NBT lists
//! - [`CompoundBase`] - Base trait for NBT compounds

use std::io::Write;

use crate::{
    ByteOrder, ListMut, ListRef, MapMut, MapRef, NBT, NBTBase, Result, TagID, ValueMut, ValueRef,
    VisitMut, VisitMutShared, VisitRef,
    tag::{
        Byte, ByteArray, Compound, Double, End, Float, Int, IntArray, List, Long, LongArray, Short,
        String, TypedList,
    },
};

/// Trait for writing NBT values to bytes.
///
/// This trait provides methods for serializing NBT values to different
/// output targets with configurable endianness.
pub trait Writable {
    fn write_to_vec<TARGET: ByteOrder>(&self) -> Vec<u8>;

    fn write_to_writer<TARGET: ByteOrder>(&self, writer: impl Write) -> Result<()>;
}

/// Base trait for all NBT values.
///
/// This trait provides the minimal interface shared by all NBT value types,
/// including methods for getting the tag ID and type checking.
pub trait ValueBase: Writable + Send + Sync + Sized {
    fn tag_id(&self) -> TagID;

    #[inline]
    fn is_<T: NBT>(&self) -> bool {
        self.tag_id() == T::TAG_ID
    }
}

/// Base trait for NBT lists.
///
/// This trait provides the minimal interface shared by all NBT list types,
/// including methods for getting the element tag ID, list length, and checking emptiness.
pub trait ListBase: Writable + Send + Sync + Sized {
    fn element_tag_id(&self) -> TagID;

    #[inline]
    fn element_is_<T: NBT>(&self) -> bool {
        self.element_tag_id() == T::TAG_ID
            || (self.element_tag_id() == TagID::End && self.is_empty())
    }

    fn len(&self) -> usize;

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Base trait for typed NBT lists.
///
/// This trait provides the minimal interface for homogeneous NBT lists
/// where all elements have the same type `T`. The element tag ID is
/// available as a constant for compile-time type checking.
///
/// # Type Parameters
///
/// * `T` - The NBT tag type of all elements
pub trait TypedListBase<T: NBT>: Writable + Send + Sync + Sized {
    const ELEMENT_TAG_ID: TagID = T::TAG_ID;

    fn len(&self) -> usize;

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Base trait for NBT compounds.
///
/// This trait provides the minimal interface for NBT compound values,
/// which are key-value maps where keys are MUTF-8 strings and values
/// are any NBT type.
pub trait CompoundBase: Writable + Send + Sync + Sized {}

pub trait NBTInto: NBTBase {
    fn ref_into_<'s, V: ValueRef<'s>>(value: V) -> Option<Self::TypeRef<'s, V::Config>>;

    fn mut_into_<'s, V: ValueMut<'s>>(value: V) -> Option<Self::TypeMut<'s, V::Config>>;
}

pub trait NBTRef: NBTBase {
    fn ref_<'a, 's: 'a, V: ValueRef<'s>>(value: &'a V) -> Option<&'a Self::TypeRef<'s, V::Config>>;

    fn mut_<'a, 's: 'a, V: ValueMut<'s>>(
        value: &'a mut V,
    ) -> Option<&'a mut Self::TypeMut<'s, V::Config>>;

    fn mut_shared_<'a, 's: 'a, V: ValueMut<'s>>(
        value: &'a V,
    ) -> Option<&'a Self::TypeMut<'s, V::Config>>;
}

macro_rules! impl_value_ref_dispatch {
    ($($t:ident),*) => {
        $(
            impl NBTInto for $t {
                #[inline]
                fn ref_into_<'s, V: ValueRef<'s>>(value: V) -> Option<Self::TypeRef<'s, V::Config>> {
                    value.map(|value| match value {
                        MapRef::$t(v) => Some(v),
                        _ => None,
                    })
                }

                #[inline]
                fn mut_into_<'s, V: ValueMut<'s>>(value: V) -> Option<Self::TypeMut<'s, V::Config>> {
                    value.map(|value| match value {
                        MapMut::$t(v) => Some(v),
                        _ => None,
                    })
                }
            }

            impl NBTRef for $t {
                #[inline]
                fn ref_<'a, 's: 'a, V:ValueRef<'s>>(value: &'a V) -> Option<&'a Self::TypeRef<'s, V::Config>> {
                    value.visit(|value| match value {
                        VisitRef::$t(v) => Some(v),
                        _ => None,
                    })
                }

                #[inline]
                fn mut_<'a, 's: 'a, V: ValueMut<'s>>(value: &'a mut V) -> Option<&'a mut Self::TypeMut<'s, V::Config>> {
                    value.visit(|value| match value {
                        VisitMut::$t(v) => Some(v),
                        _ => None,
                    })
                }

                #[inline]
                fn mut_shared_<'a, 's: 'a, V: ValueMut<'s>>(value: &'a V) -> Option<&'a Self::TypeMut<'s, V::Config>> {
                    value.visit_shared(|value| match value {
                        VisitMutShared::$t(v) => Some(v),
                        _ => None,
                    })
                }
            }
        )*
    };
}

impl_value_ref_dispatch!(
    End, Byte, Short, Int, Long, Float, Double, ByteArray, String, List, Compound, IntArray,
    LongArray
);

impl<T: NBT> NBTInto for TypedList<T> {
    #[inline]
    fn ref_into_<'s, V: ValueRef<'s>>(value: V) -> Option<Self::TypeRef<'s, V::Config>> {
        value.map(|value| match value {
            MapRef::List(v) => v.typed_::<T>(),
            _ => None,
        })
    }

    #[inline]
    fn mut_into_<'s, V: ValueMut<'s>>(value: V) -> Option<Self::TypeMut<'s, V::Config>> {
        value.map(|value| match value {
            MapMut::List(v) => v.typed_::<T>(),
            _ => None,
        })
    }
}
