use std::ops::Deref;

use zerocopy::byteorder;

use crate::{
    ByteOrder, CompoundRef, GenericNBT, ListRef, NBTBase, StringRef, TypedListRef, ValueRef,
};

pub trait ConfigRef: Send + Sync + Sized + Clone + 'static {
    type ByteOrder: ByteOrder;
    type Value<'doc>: ValueRef<'doc, ConfigRef = Self>;
    type ByteArray<'doc>: Deref<Target = [i8]> + Clone + Default;
    type String<'doc>: StringRef<'doc>;
    type List<'doc>: ListRef<'doc, ConfigRef = Self>;
    type ListIter<'doc>: Iterator<Item = Self::Value<'doc>> + ExactSizeIterator + Clone + Default;
    type TypedList<'doc, T: NBTBase>: TypedListRef<'doc, T, ConfigRef = Self>;
    type TypedListIter<'doc, T: NBTBase>: Iterator<Item = T::TypeRef<'doc, Self>>
        + ExactSizeIterator
        + Clone
        + Default;
    type Compound<'doc>: CompoundRef<'doc, ConfigRef = Self>;
    type CompoundIter<'doc>: Iterator<Item = (Self::String<'doc>, Self::Value<'doc>)>
        + Clone
        + Default;
    type IntArray<'doc>: Deref<Target = [byteorder::I32<Self::ByteOrder>]> + Clone + Default;
    type LongArray<'doc>: Deref<Target = [byteorder::I64<Self::ByteOrder>]> + Clone + Default;

    type ReadParams<'a>: Sized;

    unsafe fn read<'a, 'doc, T: GenericNBT>(
        params: Self::ReadParams<'a>,
    ) -> Option<T::TypeRef<'doc, Self>>;
}
