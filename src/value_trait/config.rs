use crate::{
    ReadableCompound, ReadableList, ReadableString, ReadableValue, WritableCompound, WritableList,
    WritableValue, util::ByteOrder,
};

pub trait ReadableConfig: Send + Sync + Sized + 'static {
    type ByteOrder: ByteOrder;
    type Value<'doc>: ReadableValue<'doc, Config = Self>;
    type String<'doc>: ReadableString<'doc>;
    type Name<'doc>: ReadableString<'doc>;
    type List<'doc>: ReadableList<'doc, Config = Self, Item = Self::Value<'doc>>;
    type ListIter<'doc>: Iterator<Item = Self::Value<'doc>> + ExactSizeIterator + Clone;
    type Compound<'doc>: ReadableCompound<'doc, Config = Self, Item = (Self::Name<'doc>, Self::Value<'doc>)>;
    type CompoundIter<'doc>: Iterator<Item = (Self::Name<'doc>, Self::Value<'doc>)> + Clone;
}

pub trait WritableConfig: ReadableConfig + Send + Sync + Sized + 'static {
    type ValueMut<'s>: WritableValue<'s, ConfigMut = Self>;
    type ListMut<'s>: WritableList<'s, ConfigMut = Self, Item = Self::ValueMut<'s>>;
    type ListIterMut<'s>: Iterator<Item = Self::ValueMut<'s>> + ExactSizeIterator;
    type CompoundMut<'s>: WritableCompound<'s, ConfigMut = Self, Item = (Self::Name<'s>, Self::ValueMut<'s>)>;
    type CompoundIterMut<'s>: Iterator<Item = (Self::Name<'s>, Self::ValueMut<'s>)>;
}
