use crate::{
    GenericNBT, Index, NBT, ReadableConfig, ScopedReadableCompound, ScopedReadableList,
    ScopedReadableTypedList, ScopedReadableValue, Value,
};

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
    unsafe fn extract_unchecked<T: GenericNBT>(self) -> T::Type<'doc, Self::Config>;

    fn extract<T: GenericNBT>(self) -> Option<T::Type<'doc, Self::Config>>;

    fn get(&self, index: impl Index) -> Option<<Self::Config as ReadableConfig>::Value<'doc>>;

    /// .
    ///
    /// # Safety
    ///
    /// .
    // will not check tag id
    unsafe fn get_typed_unchecked<T: GenericNBT>(
        &self,
        index: impl Index,
    ) -> Option<T::Type<'doc, Self::Config>>;

    fn get_typed<T: GenericNBT>(&self, index: impl Index) -> Option<T::Type<'doc, Self::Config>>;

    fn visit<R>(self, match_fn: impl FnOnce(Value<'doc, Self::Config>) -> R) -> R;
}

pub trait ReadableList<'doc>:
    ScopedReadableList<'doc> + Send + Sync + Sized + Clone + Default
{
    /// .
    ///
    /// # Safety
    ///
    /// .
    // will not check tag id
    unsafe fn get_typed_unchecked<T: GenericNBT>(
        &self,
        index: usize,
    ) -> Option<T::Type<'doc, Self::Config>>;

    fn get_typed<T: GenericNBT>(&self, index: usize) -> Option<T::Type<'doc, Self::Config>>;

    /// Gets the element at the given index.
    fn get(&self, index: usize) -> Option<<Self::Config as ReadableConfig>::Value<'doc>>;

    /// Returns an iterator over the elements of the list.
    fn iter(&self) -> <Self::Config as ReadableConfig>::ListIter<'doc>;

    /// .
    ///
    /// # Safety
    ///
    /// .
    unsafe fn extract_typed_list_unchecked<T: NBT>(
        self,
    ) -> <Self::Config as ReadableConfig>::TypedList<'doc, T>;

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

pub trait ReadableCompound<'doc>:
    ScopedReadableCompound<'doc> + Send + Sync + Sized + Clone + Default
{
    /// .
    ///
    /// # Safety
    ///
    /// .
    // will not check tag id
    unsafe fn get_typed_unchecked<T: GenericNBT>(
        &self,
        key: &str,
    ) -> Option<T::Type<'doc, Self::Config>>;

    fn get_typed<T: GenericNBT>(&self, key: &str) -> Option<T::Type<'doc, Self::Config>>;

    /// Gets the value associated with the given key.
    fn get(&self, key: &str) -> Option<<Self::Config as ReadableConfig>::Value<'doc>>;

    /// Returns an iterator over the entries of the compound.
    fn iter(&self) -> <Self::Config as ReadableConfig>::CompoundIter<'doc>;
}
