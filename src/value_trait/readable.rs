use crate::{
    index::Index,
    value_trait::{
        config::ReadableConfig,
        scoped_readable::{ScopedReadableCompound, ScopedReadableList, ScopedReadableValue},
        value::Value,
    },
};

pub trait ReadableValue<'doc>: ScopedReadableValue<'doc> + Send + Sync + Sized + Clone {
    fn as_string<'a>(&'a self) -> Option<&'a <Self::Config as ReadableConfig>::String<'doc>>
    where
        'doc: 'a;

    fn as_list<'a>(&'a self) -> Option<&'a <Self::Config as ReadableConfig>::List<'doc>>
    where
        'doc: 'a;

    fn as_compound<'a>(&'a self) -> Option<&'a <Self::Config as ReadableConfig>::Compound<'doc>>
    where
        'doc: 'a;

    fn get<I: Index>(&self, index: I) -> Option<<Self::Config as ReadableConfig>::Value<'doc>>;

    fn visit<'a, R>(&'a self, match_fn: impl FnOnce(Value<'a, 'doc, Self::Config>) -> R) -> R
    where
        'doc: 'a;
}

pub trait ReadableList<'doc>: ScopedReadableList<'doc> + Send + Sync + Sized + Clone {
    fn get(&self, index: usize) -> Option<<Self::Config as ReadableConfig>::Value<'doc>>;

    fn iter(&self) -> <Self::Config as ReadableConfig>::ListIter<'doc>;
}

pub trait ReadableCompound<'doc>:
    ScopedReadableCompound<'doc> + Send + Sync + Sized + Clone
{
    fn get(&self, key: &str) -> Option<<Self::Config as ReadableConfig>::Value<'doc>>;

    fn iter(&self) -> <Self::Config as ReadableConfig>::CompoundIter<'doc>;
}
