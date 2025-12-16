use crate::{
    index::Index,
    value_trait::{
        config::ReadableConfig,
        scoped_readable::{ScopedReadableCompound, ScopedReadableList, ScopedReadableValue},
        value::Value,
    },
};

/// A trait for values that can be read as NBT data.
///
/// This trait abstracts over different NBT representations (e.g., [`ReadonlyValue`](crate::ReadonlyValue),
/// [`OwnedValue`](crate::OwnedValue)), allowing you to write generic code that works with any of them.
pub trait ReadableValue<'doc>: ScopedReadableValue<'doc> + Send + Sync + Sized + Clone {
    /// Returns the value as a string, if it is one.
    fn as_string<'a>(&'a self) -> Option<&'a <Self::Config as ReadableConfig>::String<'doc>>
    where
        'doc: 'a;

    /// Returns the value as a list, if it is one.
    fn as_list<'a>(&'a self) -> Option<&'a <Self::Config as ReadableConfig>::List<'doc>>
    where
        'doc: 'a;

    /// Returns the value as a compound, if it is one.
    fn as_compound<'a>(&'a self) -> Option<&'a <Self::Config as ReadableConfig>::Compound<'doc>>
    where
        'doc: 'a;

    /// Gets a value at the specified index (for lists) or key (for compounds).
    fn get<I: Index>(&self, index: I) -> Option<<Self::Config as ReadableConfig>::Value<'doc>>;

    /// Visits the value with a closure, allowing for efficient pattern matching.
    fn visit<'a, R>(&'a self, match_fn: impl FnOnce(Value<'a, 'doc, Self::Config>) -> R) -> R
    where
        'doc: 'a;
}

/// A trait for NBT lists.
pub trait ReadableList<'doc>: ScopedReadableList<'doc> + Send + Sync + Sized + Clone {
    /// Gets the element at the given index.
    fn get(&self, index: usize) -> Option<<Self::Config as ReadableConfig>::Value<'doc>>;

    /// Returns an iterator over the elements of the list.
    fn iter(&self) -> <Self::Config as ReadableConfig>::ListIter<'doc>;
}

/// A trait for NBT compounds.
pub trait ReadableCompound<'doc>:
    ScopedReadableCompound<'doc> + Send + Sync + Sized + Clone
{
    /// Gets the value associated with the given key.
    fn get(&self, key: &str) -> Option<<Self::Config as ReadableConfig>::Value<'doc>>;

    /// Returns an iterator over the entries of the compound.
    fn iter(&self) -> <Self::Config as ReadableConfig>::CompoundIter<'doc>;
}
