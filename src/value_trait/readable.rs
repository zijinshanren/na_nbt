use crate::{
    ReadableConfig, ScopedReadableByteArrayList, ScopedReadableByteList, ScopedReadableCompound,
    ScopedReadableCompoundList, ScopedReadableDoubleList, ScopedReadableEndList,
    ScopedReadableFloatList, ScopedReadableIntArrayList, ScopedReadableIntList, ScopedReadableList,
    ScopedReadableListList, ScopedReadableLongArrayList, ScopedReadableLongList,
    ScopedReadableShortList, ScopedReadableStringList, ScopedReadableValue, Value, index::Index,
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

    fn as_end_list<'a>(&'a self) -> Option<<Self::Config as ReadableConfig>::EndList<'doc>>
    where
        'doc: 'a;

    fn as_byte_list<'a>(&'a self) -> Option<<Self::Config as ReadableConfig>::ByteList<'doc>>
    where
        'doc: 'a;

    fn as_short_list<'a>(&'a self) -> Option<<Self::Config as ReadableConfig>::ShortList<'doc>>
    where
        'doc: 'a;

    fn as_int_list<'a>(&'a self) -> Option<<Self::Config as ReadableConfig>::IntList<'doc>>
    where
        'doc: 'a;

    fn as_long_list<'a>(&'a self) -> Option<<Self::Config as ReadableConfig>::LongList<'doc>>
    where
        'doc: 'a;

    fn as_float_list<'a>(&'a self) -> Option<<Self::Config as ReadableConfig>::FloatList<'doc>>
    where
        'doc: 'a;

    fn as_double_list<'a>(&'a self) -> Option<<Self::Config as ReadableConfig>::DoubleList<'doc>>
    where
        'doc: 'a;

    fn as_byte_array_list<'a>(
        &'a self,
    ) -> Option<<Self::Config as ReadableConfig>::ByteArrayList<'doc>>
    where
        'doc: 'a;

    fn as_string_list<'a>(&'a self) -> Option<<Self::Config as ReadableConfig>::StringList<'doc>>
    where
        'doc: 'a;

    fn as_list_list<'a>(&'a self) -> Option<<Self::Config as ReadableConfig>::ListList<'doc>>
    where
        'doc: 'a;

    fn as_compound_list<'a>(
        &'a self,
    ) -> Option<<Self::Config as ReadableConfig>::CompoundList<'doc>>
    where
        'doc: 'a;

    fn as_int_array_list<'a>(
        &'a self,
    ) -> Option<<Self::Config as ReadableConfig>::IntArrayList<'doc>>
    where
        'doc: 'a;

    fn as_long_array_list<'a>(
        &'a self,
    ) -> Option<<Self::Config as ReadableConfig>::LongArrayList<'doc>>
    where
        'doc: 'a;
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

pub trait ReadableEndList<'doc>:
    ScopedReadableEndList<'doc> + Send + Sync + Sized + Clone + Default
{
    fn iter(&self) -> <Self::Config as ReadableConfig>::EndListIter<'doc>;
}

pub trait ReadableByteList<'doc>:
    ScopedReadableByteList<'doc> + Send + Sync + Sized + Clone + Default
{
    fn iter(&self) -> <Self::Config as ReadableConfig>::ByteListIter<'doc>;
}

pub trait ReadableShortList<'doc>:
    ScopedReadableShortList<'doc> + Send + Sync + Sized + Clone + Default
{
    fn iter(&self) -> <Self::Config as ReadableConfig>::ShortListIter<'doc>;
}

pub trait ReadableIntList<'doc>:
    ScopedReadableIntList<'doc> + Send + Sync + Sized + Clone + Default
{
    fn iter(&self) -> <Self::Config as ReadableConfig>::IntListIter<'doc>;
}

pub trait ReadableLongList<'doc>:
    ScopedReadableLongList<'doc> + Send + Sync + Sized + Clone + Default
{
    fn iter(&self) -> <Self::Config as ReadableConfig>::LongListIter<'doc>;
}

pub trait ReadableFloatList<'doc>:
    ScopedReadableFloatList<'doc> + Send + Sync + Sized + Clone + Default
{
    fn iter(&self) -> <Self::Config as ReadableConfig>::FloatListIter<'doc>;
}

pub trait ReadableDoubleList<'doc>:
    ScopedReadableDoubleList<'doc> + Send + Sync + Sized + Clone + Default
{
    fn iter(&self) -> <Self::Config as ReadableConfig>::DoubleListIter<'doc>;
}

pub trait ReadableByteArrayList<'doc>:
    ScopedReadableByteArrayList<'doc> + Send + Sync + Sized + Clone + Default
{
    fn get(&self, index: usize) -> Option<<Self::Config as ReadableConfig>::ByteArray<'doc>>;
    fn iter(&self) -> <Self::Config as ReadableConfig>::ByteArrayListIter<'doc>;
}

pub trait ReadableStringList<'doc>:
    ScopedReadableStringList<'doc> + Send + Sync + Sized + Clone + Default
{
    fn get(&self, index: usize) -> Option<<Self::Config as ReadableConfig>::String<'doc>>;
    fn iter(&self) -> <Self::Config as ReadableConfig>::StringListIter<'doc>;
}

pub trait ReadableListList<'doc>:
    ScopedReadableListList<'doc> + Send + Sync + Sized + Clone + Default
{
    fn get(&self, index: usize) -> Option<<Self::Config as ReadableConfig>::List<'doc>>;
    fn iter(&self) -> <Self::Config as ReadableConfig>::ListListIter<'doc>;
}

pub trait ReadableCompoundList<'doc>:
    ScopedReadableCompoundList<'doc> + Send + Sync + Sized + Clone + Default
{
    fn get(&self, index: usize) -> Option<<Self::Config as ReadableConfig>::Compound<'doc>>;
    fn iter(&self) -> <Self::Config as ReadableConfig>::CompoundListIter<'doc>;
}

pub trait ReadableIntArrayList<'doc>:
    ScopedReadableIntArrayList<'doc> + Send + Sync + Sized + Clone + Default
{
    fn get(&self, index: usize) -> Option<<Self::Config as ReadableConfig>::IntArray<'doc>>;
    fn iter(&self) -> <Self::Config as ReadableConfig>::IntArrayListIter<'doc>;
}

pub trait ReadableLongArrayList<'doc>:
    ScopedReadableLongArrayList<'doc> + Send + Sync + Sized + Clone + Default
{
    fn get(&self, index: usize) -> Option<<Self::Config as ReadableConfig>::LongArray<'doc>>;
    fn iter(&self) -> <Self::Config as ReadableConfig>::LongArrayListIter<'doc>;
}
