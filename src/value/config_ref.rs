use std::ops::Deref;

use zerocopy::byteorder;

use crate::{ByteOrder, CompoundRef, ListRef, NBT, StringRef, TypedListRef, ValueRef};

pub trait ConfigRef: Send + Sync + Sized + Clone + 'static {
    type ByteOrder: ByteOrder;
    type Value<'doc>: ValueRef<'doc, Config = Self>;
    type ByteArray<'doc>: Deref<Target = [i8]> + Clone;
    type String<'doc>: StringRef<'doc>;
    type List<'doc>: ListRef<'doc, Config = Self>;
    type ListIter<'doc>: Iterator<Item = Self::Value<'doc>> + ExactSizeIterator + Clone + Default;
    type TypedList<'doc, T: NBT>: TypedListRef<'doc, T, Config = Self>;
    type TypedListIter<'doc, T: NBT>: Iterator<Item = T::Type<'doc, Self>>
        + ExactSizeIterator
        + Clone
        + Default;
    type Compound<'doc>: CompoundRef<'doc, Config = Self>;
    type CompoundIter<'doc>: Iterator<Item = (Self::String<'doc>, Self::Value<'doc>)>
        + Clone
        + Default;
    type IntArray<'doc>: Deref<Target = [byteorder::I32<Self::ByteOrder>]> + Clone;
    type LongArray<'doc>: Deref<Target = [byteorder::I64<Self::ByteOrder>]> + Clone;
}
