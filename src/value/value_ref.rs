use crate::{CompoundBase, ConfigRef, Index, ListBase, NBT, TypedListBase, Value, ValueBase};

pub trait ValueRef<'s>: ValueBase + Clone + Default {
    fn ref_<'a, T: NBT>(&'a self) -> Option<&'a T::Type<'s, Self::Config>>
    where
        's: 'a;

    fn into_<'a, T: NBT>(self) -> Option<T::Type<'s, Self::Config>>;

    fn get(&self, index: impl Index) -> Option<<Self::Config as ConfigRef>::Value<'s>>;

    fn get_<'a, T: NBT>(&self, index: impl Index) -> Option<T::Type<'s, Self::Config>>;

    fn map<R>(self, match_fn: impl FnOnce(Value<'s, Self::Config>) -> R) -> R;
}

pub trait ListRef<'s>:
    ListBase + IntoIterator<Item = <Self::Config as ConfigRef>::Value<'s>> + Clone + Default
{
    fn get(&self, index: usize) -> Option<<Self::Config as ConfigRef>::Value<'s>>;

    fn get_<'a, T: NBT>(&self, index: usize) -> Option<T::Type<'s, Self::Config>>;

    fn typed_<'a, T: NBT>(&self) -> Option<<Self::Config as ConfigRef>::TypedList<'s, T>>;

    fn iter(&self) -> <Self::Config as ConfigRef>::ListIter<'s>;
}

pub trait TypedListRef<'s, T: NBT>:
    TypedListBase<T> + IntoIterator<Item = T::Type<'s, Self::Config>> + Clone + Default
{
    fn get(&self, index: usize) -> Option<T::Type<'s, Self::Config>>;

    fn iter(&self) -> <Self::Config as ConfigRef>::TypedListIter<'s, T>;
}

pub trait CompoundRef<'s>:
    CompoundBase
    + IntoIterator<
        Item = (
            <Self::Config as ConfigRef>::String<'s>,
            <Self::Config as ConfigRef>::Value<'s>,
        ),
    > + Clone
    + Default
{
    fn get(&self, key: &str) -> Option<<Self::Config as ConfigRef>::Value<'s>>;

    fn get_<'a, T: NBT>(&self, key: &str) -> Option<T::Type<'s, Self::Config>>;

    fn iter(&self) -> <Self::Config as ConfigRef>::CompoundIter<'s>;
}
