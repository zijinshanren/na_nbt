use std::ops::Deref;

use zerocopy::byteorder;

use crate::{
    ByteOrder, NBT, ReadableCompound, ReadableList, ReadableString, ReadableTypedList,
    ReadableValue,
};

pub trait ReadableConfig: Send + Sync + Sized + Clone + 'static {
    type ByteOrder: ByteOrder;
    type Value<'doc>: ReadableValue<'doc, Config = Self>;
    type ByteArray<'doc>: Deref<Target = [i8]> + Clone;
    type String<'doc>: ReadableString<'doc>;
    type List<'doc>: ReadableList<'doc, Config = Self, Item = Self::Value<'doc>>;
    type ListIter<'doc>: Iterator<Item = Self::Value<'doc>> + ExactSizeIterator + Clone + Default;
    type TypedList<'doc, T: NBT>: ReadableTypedList<'doc, T, Config = Self, Item = T::Type<'doc, Self>>;
    type TypedListIter<'doc, T: NBT>: Iterator<Item = T::Type<'doc, Self>>
        + ExactSizeIterator
        + Clone
        + Default;
    type Compound<'doc>: ReadableCompound<'doc, Config = Self, Item = (Self::String<'doc>, Self::Value<'doc>)>;
    type CompoundIter<'doc>: Iterator<Item = (Self::String<'doc>, Self::Value<'doc>)>
        + Clone
        + Default;
    type IntArray<'doc>: Deref<Target = [byteorder::I32<Self::ByteOrder>]> + Clone;
    type LongArray<'doc>: Deref<Target = [byteorder::I64<Self::ByteOrder>]> + Clone;
}
