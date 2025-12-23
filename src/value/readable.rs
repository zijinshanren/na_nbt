use crate::{
    GenericNBT, Index, NBT, ReadableConfig, ScopedReadableCompound, ScopedReadableList,
    ScopedReadableTypedList, ScopedReadableValue, Value,
};

pub trait ReadableValue<'doc>: ScopedReadableValue<'doc> + Clone + Default {
    fn ref_<'a, T: NBT>(&'a self) -> Option<&'a T::Type<'doc, Self::Config>>
    where
        'doc: 'a;

    fn into_<T: GenericNBT>(self) -> Option<T::Type<'doc, Self::Config>>;

    fn get(&self, index: impl Index) -> Option<<Self::Config as ReadableConfig>::Value<'doc>>;

    fn get_<T: GenericNBT>(&self, index: impl Index) -> Option<T::Type<'doc, Self::Config>>;

    fn map<R>(self, match_fn: impl FnOnce(Value<'doc, Self::Config>) -> R) -> R;
}

pub trait ReadableList<'doc>:
    ScopedReadableList<'doc>
    + IntoIterator<Item = <Self::Config as ReadableConfig>::Value<'doc>>
    + Clone
    + Default
{
    fn get_<T: GenericNBT>(&self, index: usize) -> Option<T::Type<'doc, Self::Config>>;

    fn get(&self, index: usize) -> Option<<Self::Config as ReadableConfig>::Value<'doc>>;

    fn iter(&self) -> <Self::Config as ReadableConfig>::ListIter<'doc>;

    fn into_typed_<T: NBT>(self) -> Option<<Self::Config as ReadableConfig>::TypedList<'doc, T>>;
}

pub trait ReadableTypedList<'doc, T: NBT>:
    ScopedReadableTypedList<'doc, T>
    + IntoIterator<Item = T::Type<'doc, Self::Config>>
    + Clone
    + Default
{
    fn get(&self, index: usize) -> Option<T::Type<'doc, Self::Config>>;

    fn iter(&self) -> <Self::Config as ReadableConfig>::TypedListIter<'doc, T>;
}

pub trait ReadableCompound<'doc>:
    ScopedReadableCompound<'doc>
    + IntoIterator<
        Item = (
            <Self::Config as ReadableConfig>::String<'doc>,
            <Self::Config as ReadableConfig>::Value<'doc>,
        ),
    > + Clone
    + Default
{
    fn get_<T: GenericNBT>(&self, key: &str) -> Option<T::Type<'doc, Self::Config>>;

    fn get(&self, key: &str) -> Option<<Self::Config as ReadableConfig>::Value<'doc>>;

    fn iter(&self) -> <Self::Config as ReadableConfig>::CompoundIter<'doc>;
}
