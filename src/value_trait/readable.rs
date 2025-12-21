use crate::{
    index::Index,
    value_trait::{
        config::ReadableConfig,
        scoped_readable::{ScopedReadableCompound, ScopedReadableList, ScopedReadableValue},
        value::Value,
    },
};

/// Extended trait for reading NBT values with full lifetime access.
///
/// This trait extends [`ScopedReadableValue`] with methods that return references tied
/// to the document lifetime rather than the borrow scope. This allows storing references
/// to nested values.
///
/// Most code should use [`ScopedReadableValue`] instead, as it is implemented by more types.
pub trait ReadableValue<'doc>:
    ScopedReadableValue<'doc> + Send + Sync + Sized + Clone + Default
{
    /// Returns the value as a byte array, if it is one.
    fn as_byte_array<'a>(&'a self) -> Option<&'a <Self::Config as ReadableConfig>::ByteArray<'doc>>
    where
        'doc: 'a;

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

    /// Returns the value as an int array, if it is one.
    fn as_int_array<'a>(&'a self) -> Option<&'a <Self::Config as ReadableConfig>::IntArray<'doc>>
    where
        'doc: 'a;

    /// Returns the value as a long array, if it is one.
    fn as_long_array<'a>(&'a self) -> Option<&'a <Self::Config as ReadableConfig>::LongArray<'doc>>
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
pub trait ReadableList<'doc>:
    ScopedReadableList<'doc> + Send + Sync + Sized + Clone + Default
{
    /// Gets the element at the given index.
    fn get(&self, index: usize) -> Option<<Self::Config as ReadableConfig>::Value<'doc>>;

    /// Returns an iterator over the elements of the list.
    fn iter(&self) -> <Self::Config as ReadableConfig>::ListIter<'doc>;
}

/// A trait for NBT compounds.
pub trait ReadableCompound<'doc>:
    ScopedReadableCompound<'doc> + Send + Sync + Sized + Clone + Default
{
    /// Gets the value associated with the given key.
    fn get(&self, key: &str) -> Option<<Self::Config as ReadableConfig>::Value<'doc>>;

    /// Returns an iterator over the entries of the compound.
    fn iter(&self) -> <Self::Config as ReadableConfig>::CompoundIter<'doc>;
}
