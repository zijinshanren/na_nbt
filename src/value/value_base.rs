use std::io::Write;

use crate::{
    ByteOrder, ListMut, ListRef, MapMut, MapRef, NBT, NBTBase, Result, TagID, ValueMut, ValueRef,
    VisitMut, VisitRef,
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
    fn tag_id(&self) -> TagID;

    #[inline]
    fn is_<T: NBT>(&self) -> bool {
        self.tag_id() == T::TAG_ID
    }
}

pub trait ListBase: Send + Sync + Sized {
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

pub trait TypedListBase<T: NBT>: Send + Sync + Sized {
    const ELEMENT_TAG_ID: TagID = T::TAG_ID;

    fn len(&self) -> usize;

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait CompoundBase: Send + Sync + Sized {}

pub trait NBTInto: NBTBase {
    fn ref_into_<'s, V: ValueRef<'s>>(value: V) -> Option<Self::TypeRef<'s, V::Config>>;

    fn mut_into_<'s, V: ValueMut<'s>>(value: V) -> Option<Self::TypeMut<'s, V::Config>>;
}

pub trait NBTRef: NBTBase {
    fn ref_<'a, 's: 'a, V: ValueRef<'s>>(value: &'a V) -> Option<&'a Self::TypeRef<'s, V::Config>>;

    fn mut_<'a, 's: 'a, V: ValueMut<'s>>(
        value: &'a mut V,
    ) -> Option<&'a mut Self::TypeMut<'s, V::Config>>;
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
            }
        )*
    };
}

impl_value_ref_dispatch!(
    End, Byte, Short, Int, Long, Float, Double, ByteArray, String, List, Compound, IntArray,
    LongArray
);

impl<T: NBT> NBTInto for TypedList<T> {
    fn ref_into_<'s, V: ValueRef<'s>>(value: V) -> Option<Self::TypeRef<'s, V::Config>> {
        value.map(|value| match value {
            MapRef::List(v) => v.typed_::<T>(),
            _ => None,
        })
    }

    fn mut_into_<'s, V: ValueMut<'s>>(value: V) -> Option<Self::TypeMut<'s, V::Config>> {
        value.map(|value| match value {
            MapMut::List(v) => v.typed_::<T>(),
            _ => None,
        })
    }
}
