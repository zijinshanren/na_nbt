use crate::{
    NBT, ReadableConfig, ScopedReadableCompound, ScopedReadableList, ScopedReadableTypedList,
    ScopedReadableValue, Value, index::Index,
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
    /// .
    ///
    /// # Safety
    ///
    /// .
    unsafe fn peek_unchecked<'a, T: NBT>(&'a self) -> &'a T::Type<'doc, Self::Config>
    where
        'doc: 'a;

    fn peek<'a, T: NBT>(&'a self) -> Option<&'a T::Type<'doc, Self::Config>>
    where
        'doc: 'a;

    /// .
    ///
    /// # Safety
    ///
    /// .
    unsafe fn extract_unchecked<T: NBT>(self) -> T::Type<'doc, Self::Config>;

    fn extract<T: NBT>(self) -> Option<T::Type<'doc, Self::Config>>;

    fn get<I: Index>(&self, index: I) -> Option<<Self::Config as ReadableConfig>::Value<'doc>>;

    fn visit<R>(self, match_fn: impl FnOnce(Value<'doc, Self::Config>) -> R) -> R;
}

/// A trait for NBT lists.
pub trait ReadableList<'doc>:
    ScopedReadableList<'doc> + Send + Sync + Sized + Clone + Default
{
    /// Gets the element at the given index.
    fn get(&self, index: usize) -> Option<<Self::Config as ReadableConfig>::Value<'doc>>;

    /// Returns an iterator over the elements of the list.
    fn iter(&self) -> <Self::Config as ReadableConfig>::ListIter<'doc>;

    fn extract_typed_list<T: NBT>(
        self,
    ) -> Option<<Self::Config as ReadableConfig>::TypedList<'doc, T>>;
}

pub trait ReadableTypedList<'doc, T: NBT>:
    ScopedReadableTypedList<'doc, T> + Send + Sync + Sized + Clone + Default
{
    fn get(&self, index: usize) -> Option<T::Type<'doc, Self::Config>>;

    fn iter(&self) -> <Self::Config as ReadableConfig>::TypedListIter<'doc, T>;
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
