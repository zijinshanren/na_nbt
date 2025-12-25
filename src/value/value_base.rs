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

pub trait FromNBTRef<'a, T: GenericNBT, C: ConfigRef>: From<T::TypeRef<'a, C>> {}

pub trait FromAnyNBTRefl<'a, C: ConfigRef>:
    FromNBTRef<'a, End, C>
    + FromNBTRef<'a, Byte, C>
    + FromNBTRef<'a, Short, C>
    + FromNBTRef<'a, Int, C>
    + FromNBTRef<'a, Long, C>
    + FromNBTRef<'a, Float, C>
    + FromNBTRef<'a, Double, C>
    + FromNBTRef<'a, ByteArray, C>
    + FromNBTRef<'a, String, C>
    + FromNBTRef<'a, List, C>
    + FromNBTRef<'a, Compound, C>
    + FromNBTRef<'a, IntArray, C>
    + FromNBTRef<'a, LongArray, C>
    + FromNBTRef<'a, TypedList<End>, C>
    + FromNBTRef<'a, TypedList<Byte>, C>
    + FromNBTRef<'a, TypedList<Short>, C>
    + FromNBTRef<'a, TypedList<Int>, C>
    + FromNBTRef<'a, TypedList<Long>, C>
    + FromNBTRef<'a, TypedList<Float>, C>
    + FromNBTRef<'a, TypedList<Double>, C>
    + FromNBTRef<'a, TypedList<ByteArray>, C>
    + FromNBTRef<'a, TypedList<String>, C>
    + FromNBTRef<'a, TypedList<List>, C>
    + FromNBTRef<'a, TypedList<Compound>, C>
    + FromNBTRef<'a, TypedList<IntArray>, C>
    + FromNBTRef<'a, TypedList<LongArray>, C>
{
}

pub trait ValueBase: Send + Sync + Sized {
    type ConfigRef: ConfigRef;

    fn tag_id(&self) -> TagID;

    #[inline]
    fn is_<T: NBTBase>(&self) -> bool {
        self.tag_id() == T::TAG_ID
    }
}

pub trait ListBase: Send + Sync + Sized {
    type ConfigRef: ConfigRef;

    fn element_tag_id(&self) -> TagID;

    #[inline]
    fn element_is_<T: NBTBase>(&self) -> bool {
        self.element_tag_id() == T::TAG_ID
            || (self.element_tag_id() == TagID::End && self.is_empty())
    }

    fn len(&self) -> usize;

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait TypedListBase<T: NBTBase>: Send + Sync + Sized {
    type ConfigRef: ConfigRef;

    const ELEMENT_TAG_ID: TagID = T::TAG_ID;

    fn len(&self) -> usize;

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait CompoundBase: Send + Sync + Sized {
    type ConfigRef: ConfigRef;

    fn compound_get_impl<'a, 'doc>(
        &'a self,
        key: &str,
    ) -> Option<(TagID, <Self::ConfigRef as ConfigRef>::ReadParams<'a>)>
    where
        'doc: 'a;
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
