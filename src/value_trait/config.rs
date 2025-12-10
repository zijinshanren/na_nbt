use crate::{
    util::ByteOrder,
    value_trait::{
        readable::{ReadableCompound, ReadableList, ReadableValue},
        string::ReadableString,
    },
};

pub trait ReadableConfig: Send + Sync + Sized + 'static {
    type ByteOrder: ByteOrder;
    type Value<'doc>: ReadableValue<'doc, Config = Self>;
    type String<'doc>: ReadableString<'doc>;
    type Name<'doc>: ReadableString<'doc>;
    type List<'doc>: ReadableList<'doc, Config = Self, IterValue = Self::Value<'doc>>;
    type ListIter<'doc>: Iterator<Item = Self::Value<'doc>> + ExactSizeIterator + Clone;
    type Compound<'doc>: ReadableCompound<'doc, Config = Self, IterValue = Self::Value<'doc>>;
    type CompoundIter<'doc>: Iterator<Item = (Self::Name<'doc>, Self::Value<'doc>)> + Clone;
}
