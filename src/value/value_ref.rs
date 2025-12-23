use crate::{CompoundBase, ConfigRef, Index, ListBase, NBT, TypedListBase, Value, ValueBase};

pub trait ValueRef<'s>: ValueBase + Clone + Default {
    fn ref_<'a, T: NBT>(&'a self) -> Option<&'a T::Type<'s, Self::ConfigRef>>
    where
        's: 'a;

    fn into_<T: NBT>(self) -> Option<T::Type<'s, Self::ConfigRef>>;

    fn get(&self, index: impl Index) -> Option<<Self::ConfigRef as ConfigRef>::Value<'s>>;

    fn get_<T: NBT>(&self, index: impl Index) -> Option<T::Type<'s, Self::ConfigRef>>;

    fn map<R>(self, match_fn: impl FnOnce(Value<'s, Self::ConfigRef>) -> R) -> R;
}

pub trait ListRef<'s>:
    ListBase + IntoIterator<Item = <Self::ConfigRef as ConfigRef>::Value<'s>> + Clone + Default
{
    fn get(&self, index: usize) -> Option<<Self::ConfigRef as ConfigRef>::Value<'s>>;

    fn get_<T: NBT>(&self, index: usize) -> Option<T::Type<'s, Self::ConfigRef>>;

    fn typed_<T: NBT>(self) -> Option<<Self::ConfigRef as ConfigRef>::TypedList<'s, T>>;

    fn iter(&self) -> <Self::ConfigRef as ConfigRef>::ListIter<'s>;
}

pub trait TypedListRef<'s, T: NBT>:
    TypedListBase<T> + IntoIterator<Item = T::Type<'s, Self::ConfigRef>> + Clone + Default
{
    fn get(&self, index: usize) -> Option<T::Type<'s, Self::ConfigRef>>;

    fn iter(&self) -> <Self::ConfigRef as ConfigRef>::TypedListIter<'s, T>;
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

    fn get_<T: NBT>(&self, key: &str) -> Option<T::Type<'s, Self::Config>>;

    fn iter(&self) -> <Self::Config as ConfigRef>::CompoundIter<'s>;
}
