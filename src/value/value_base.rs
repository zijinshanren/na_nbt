use std::io::Write;

use crate::{
    ByteOrder, ConfigRef, GenericNBT, ListRef, MapRef, NBT, NBTBase, Result, TagID, ValueRef,
    VisitRef,
    tag::{
        Byte, ByteArray, Compound, Double, End, Float, Int, IntArray, List, Long, LongArray, Short,
        String, TypedList,
    },
};

pub trait Writable {
    fn write_to_vec<TARGET: ByteOrder>(&self, buf: &mut Vec<u8>);

    fn write_to_writer<TARGET: ByteOrder>(&self, writer: impl Write) -> Result<()>;
}

pub trait ValueBase: Send + Sync + Sized {
    type ConfigRef: ConfigRef;

    fn tag_id(&self) -> TagID;

    #[inline]
    fn is_<T: NBT>(&self) -> bool {
        self.tag_id() == T::TAG_ID
    }
}

pub trait ListBase: Send + Sync + Sized {
    type ConfigRef: ConfigRef;

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

    fn list_get_impl<'a, T: GenericNBT>(
        &'a self,
        index: usize,
    ) -> <Self::ConfigRef as ConfigRef>::ReadParams<'a>;
}

pub trait TypedListBase<T: NBT>: Send + Sync + Sized {
    type ConfigRef: ConfigRef;

    const ELEMENT_TAG_ID: TagID = T::TAG_ID;

    fn len(&self) -> usize;

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn typed_list_get_impl<'a>(
        &'a self,
        index: usize,
    ) -> <Self::ConfigRef as ConfigRef>::ReadParams<'a>;
}

pub trait CompoundBase: Send + Sync + Sized {
    type ConfigRef: ConfigRef;

    fn compound_get_impl<'a>(
        &'a self,
        key: &str,
    ) -> Option<(TagID, <Self::ConfigRef as ConfigRef>::ReadParams<'a>)>;
}

pub trait NBTInto: NBTBase {
    fn into_<'s, V: ValueRef<'s>>(value: V) -> Option<Self::TypeRef<'s, V::ConfigRef>>;
}

pub trait NBTRef: NBTBase {
    fn ref_<'a, 's: 'a, V: ValueRef<'s>>(
        value: &'a V,
    ) -> Option<&'a Self::TypeRef<'s, V::ConfigRef>>;
}

macro_rules! impl_value_ref_dispatch {
    ($($t:ident),*) => {
        $(
            impl NBTInto for $t {
                #[inline]
                fn into_<'s, V: ValueRef<'s>>(value: V) -> Option<Self::TypeRef<'s, V::ConfigRef>> {
                    value.map(|value| match value {
                        MapRef::$t(v) => Some(v),
                        _ => None,
                    })
                }
            }

            impl NBTRef for $t {
                #[inline]
                fn ref_<'a, 's: 'a, V:ValueRef<'s>>(value: &'a V) -> Option<&'a Self::TypeRef<'s, V::ConfigRef>> {
                    value.visit(|value| match value {
                        VisitRef::$t(v) => Some(v),
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
    fn into_<'s, V: ValueRef<'s>>(value: V) -> Option<Self::TypeRef<'s, V::ConfigRef>> {
        value.map(|value| match value {
            MapRef::List(v) => v.typed_::<T>(),
            _ => None,
        })
    }
}
