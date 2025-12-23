use std::io::Write;

use crate::{ByteOrder, GenericNBT, Index, NBT, ReadableConfig, Result, TagID, Value};

pub trait ScopedReadableValue<'doc>: Send + Sync + Sized {
    type Config: ReadableConfig;

    fn to_readable<'a>(&'a self) -> <Self::Config as ReadableConfig>::Value<'a>
    where
        'doc: 'a;

    fn tag_id(&self) -> TagID;

    fn is<T: NBT>(&self) -> bool;

    fn to_<'a, T: GenericNBT>(&'a self) -> Option<T::Type<'a, Self::Config>>
    where
        'doc: 'a;

    fn at<'a>(&'a self, index: impl Index) -> Option<<Self::Config as ReadableConfig>::Value<'a>>
    where
        'doc: 'a;

    fn at_<'a, T: GenericNBT>(&'a self, index: impl Index) -> Option<T::Type<'a, Self::Config>>
    where
        'doc: 'a;

    fn visit<'a, R>(&'a self, match_fn: impl FnOnce(Value<'a, Self::Config>) -> R) -> R
    where
        'doc: 'a;

    fn write_to_vec<TARGET: ByteOrder>(&self) -> Result<Vec<u8>>;

    fn write_to_writer<TARGET: ByteOrder>(&self, writer: impl Write) -> Result<()>;
}

pub trait ScopedReadableList<'doc>: Send + Sync + Sized {
    type Config: ReadableConfig;

    fn to_readable<'a>(&'a self) -> <Self::Config as ReadableConfig>::List<'a>
    where
        'doc: 'a;

    fn tag_id(&self) -> TagID;

    fn is<T: NBT>(&self) -> bool;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool;

    fn at_<'a, T: GenericNBT>(&'a self, index: usize) -> Option<T::Type<'a, Self::Config>>
    where
        'doc: 'a;

    fn at<'a>(&'a self, index: usize) -> Option<<Self::Config as ReadableConfig>::Value<'a>>
    where
        'doc: 'a;

    fn iter_scoped<'a>(&'a self) -> <Self::Config as ReadableConfig>::ListIter<'a>
    where
        'doc: 'a;

    fn to_typed_<'a, T: NBT>(
        &'a self,
    ) -> Option<<Self::Config as ReadableConfig>::TypedList<'a, T>>
    where
        'doc: 'a;
}

pub trait ScopedReadableTypedList<'doc, T: NBT>: Send + Sync + Sized {
    type Config: ReadableConfig;

    const TAG_ID: TagID = T::TAG_ID;

    fn to_readable<'a>(&'a self) -> <Self::Config as ReadableConfig>::TypedList<'a, T>
    where
        'doc: 'a;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool;

    fn at<'a>(&'a self, index: usize) -> Option<T::Type<'a, Self::Config>>
    where
        'doc: 'a;

    fn iter_scoped<'a>(&'a self) -> <Self::Config as ReadableConfig>::TypedListIter<'a, T>
    where
        'doc: 'a;
}

pub trait ScopedReadableCompound<'doc>: Send + Sync + Sized {
    type Config: ReadableConfig;

    fn to_readable<'a>(&'a self) -> <Self::Config as ReadableConfig>::Compound<'a>
    where
        'doc: 'a;

    fn at_<'a, T: GenericNBT>(&'a self, key: &str) -> Option<T::Type<'a, Self::Config>>
    where
        'doc: 'a;

    fn at<'a>(&'a self, key: &str) -> Option<<Self::Config as ReadableConfig>::Value<'a>>
    where
        'doc: 'a;

    fn iter_scoped<'a>(&'a self) -> <Self::Config as ReadableConfig>::CompoundIter<'a>
    where
        'doc: 'a;
}
