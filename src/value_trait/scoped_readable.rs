use std::io::Write;

use crate::{ByteOrder, ReadableConfig, Result, Tag, ValueScoped, index::Index};

/// Core trait for reading NBT values.
///
/// This is the primary trait for generic NBT code. All value types implement it:
/// - [`BorrowedValue`](crate::BorrowedValue)
/// - [`SharedValue`](crate::SharedValue)
/// - [`OwnedValue`](crate::OwnedValue)
/// - [`MutableValue`](crate::MutableValue)
/// - [`ImmutableValue`](crate::ImmutableValue)
///
/// # Usage
///
/// Accept `impl ScopedReadableValue<'doc>` to write functions that work with any value type:
///
/// ```rust
/// use na_nbt::{ScopedReadableValue, Tag};
///
/// fn is_number<'doc>(value: &impl ScopedReadableValue<'doc>) -> bool {
///     matches!(value.tag_id(), Tag::Byte | Tag::Short | Tag::Int | Tag::Long | Tag::Float | Tag::Double)
/// }
/// ```
///
/// # Key Methods
///
/// - `tag_id()` - Get the NBT tag type
/// - `as_*()` - Try to get a specific type (returns `Option`)
/// - `is_*()` - Check if value is a specific type
/// - `visit_scoped()` - Pattern match on the value type
pub trait ScopedReadableValue<'doc>: Send + Sync + Sized {
    /// The configuration associated with this value.
    type Config: ReadableConfig;

    /// Returns the tag ID of the value.
    fn tag_id(&self) -> Tag;

    /// Returns `Some(())` if the value is an End tag.
    fn as_end(&self) -> Option<()>;
    /// Returns `true` if the value is an End tag.
    fn is_end(&self) -> bool;

    /// Returns the value as a byte, if it is one.
    fn as_byte(&self) -> Option<i8>;
    /// Returns `true` if the value is a byte.
    fn is_byte(&self) -> bool;

    /// Returns the value as a short, if it is one.
    fn as_short(&self) -> Option<i16>;
    /// Returns `true` if the value is a short.
    fn is_short(&self) -> bool;

    /// Returns the value as an int, if it is one.
    fn as_int(&self) -> Option<i32>;
    /// Returns `true` if the value is an int.
    fn is_int(&self) -> bool;

    /// Returns the value as a long, if it is one.
    fn as_long(&self) -> Option<i64>;
    /// Returns `true` if the value is a long.
    fn is_long(&self) -> bool;

    /// Returns the value as a float, if it is one.
    fn as_float(&self) -> Option<f32>;
    /// Returns `true` if the value is a float.
    fn is_float(&self) -> bool;

    /// Returns the value as a double, if it is one.
    fn as_double(&self) -> Option<f64>;
    /// Returns `true` if the value is a double.
    fn is_double(&self) -> bool;

    fn as_byte_array_scoped<'a>(
        &'a self,
    ) -> Option<<Self::Config as ReadableConfig>::ByteArray<'a>>
    where
        'doc: 'a;

    /// Returns `true` if the value is a byte array.
    fn is_byte_array(&self) -> bool;

    /// Returns the value as a string with a scoped lifetime, if it is one.
    fn as_string_scoped<'a>(&'a self) -> Option<<Self::Config as ReadableConfig>::String<'a>>
    where
        'doc: 'a;
    /// Returns `true` if the value is a string.
    fn is_string(&self) -> bool;

    /// Returns the value as a list with a scoped lifetime, if it is one.
    fn as_list_scoped<'a>(&'a self) -> Option<<Self::Config as ReadableConfig>::List<'a>>
    where
        'doc: 'a;
    /// Returns `true` if the value is a list.
    fn is_list(&self) -> bool;

    /// Returns the value as a compound with a scoped lifetime, if it is one.
    fn as_compound_scoped<'a>(&'a self) -> Option<<Self::Config as ReadableConfig>::Compound<'a>>
    where
        'doc: 'a;
    /// Returns `true` if the value is a compound.
    fn is_compound(&self) -> bool;

    fn as_int_array_scoped<'a>(&'a self) -> Option<<Self::Config as ReadableConfig>::IntArray<'a>>
    where
        'doc: 'a;

    /// Returns `true` if the value is an int array.
    fn is_int_array(&self) -> bool;

    fn as_long_array_scoped<'a>(
        &'a self,
    ) -> Option<<Self::Config as ReadableConfig>::LongArray<'a>>
    where
        'doc: 'a;

    /// Returns `true` if the value is a long array.
    fn is_long_array(&self) -> bool;

    /// Gets a value at the specified index (for lists) or key (for compounds) with a scoped lifetime.
    fn get_scoped<'a, I: Index>(
        &'a self,
        index: I,
    ) -> Option<<Self::Config as ReadableConfig>::Value<'a>>
    where
        'doc: 'a;

    /// Visits the value with a closure, allowing for efficient pattern matching with scoped lifetimes.
    fn visit_scoped<'a, R>(
        &'a self,
        match_fn: impl FnOnce(ValueScoped<'a, Self::Config>) -> R,
    ) -> R
    where
        'doc: 'a;

    /// Writes the value to a byte vector.
    fn write_to_vec<TARGET: ByteOrder>(&self) -> Result<Vec<u8>>;

    /// Writes the value to a writer.
    fn write_to_writer<TARGET: ByteOrder>(&self, writer: impl Write) -> Result<()>;
}

/// A trait for NBT lists with scoped lifetimes.
pub trait ScopedReadableList<'doc>: IntoIterator + Send + Sync + Sized {
    /// The configuration associated with this list.
    type Config: ReadableConfig;

    /// Returns the tag ID of the elements in the list.
    fn tag_id(&self) -> Tag;

    /// Returns the number of elements in the list.
    fn len(&self) -> usize;

    /// Returns `true` if the list is empty.
    fn is_empty(&self) -> bool;

    /// Gets the element at the given index with a scoped lifetime.
    fn get_scoped<'a>(
        &'a self,
        index: usize,
    ) -> Option<<Self::Config as ReadableConfig>::Value<'a>>
    where
        'doc: 'a;

    fn iter_scoped<'a>(&'a self) -> <Self::Config as ReadableConfig>::ListIter<'a>
    where
        'doc: 'a;

    fn as_end_list_scoped<'a>(&'a self) -> Option<<Self::Config as ReadableConfig>::EndList<'a>>
    where
        'doc: 'a;

    fn as_byte_list_scoped<'a>(&'a self) -> Option<<Self::Config as ReadableConfig>::ByteList<'a>>
    where
        'doc: 'a;

    fn as_short_list_scoped<'a>(
        &'a self,
    ) -> Option<<Self::Config as ReadableConfig>::ShortList<'a>>
    where
        'doc: 'a;

    fn as_int_list_scoped<'a>(&'a self) -> Option<<Self::Config as ReadableConfig>::IntList<'a>>
    where
        'doc: 'a;

    fn as_long_list_scoped<'a>(&'a self) -> Option<<Self::Config as ReadableConfig>::LongList<'a>>
    where
        'doc: 'a;

    fn as_float_list_scoped<'a>(
        &'a self,
    ) -> Option<<Self::Config as ReadableConfig>::FloatList<'a>>
    where
        'doc: 'a;

    fn as_double_list_scoped<'a>(
        &'a self,
    ) -> Option<<Self::Config as ReadableConfig>::DoubleList<'a>>
    where
        'doc: 'a;

    fn as_byte_array_list_scoped<'a>(
        &'a self,
    ) -> Option<<Self::Config as ReadableConfig>::ByteArrayList<'a>>
    where
        'doc: 'a;

    fn as_string_list_scoped<'a>(
        &'a self,
    ) -> Option<<Self::Config as ReadableConfig>::StringList<'a>>
    where
        'doc: 'a;

    fn as_list_list_scoped<'a>(&'a self) -> Option<<Self::Config as ReadableConfig>::ListList<'a>>
    where
        'doc: 'a;

    fn as_compound_list_scoped<'a>(
        &'a self,
    ) -> Option<<Self::Config as ReadableConfig>::CompoundList<'a>>
    where
        'doc: 'a;

    fn as_int_array_list_scoped<'a>(
        &'a self,
    ) -> Option<<Self::Config as ReadableConfig>::IntArrayList<'a>>
    where
        'doc: 'a;

    fn as_long_array_list_scoped<'a>(
        &'a self,
    ) -> Option<<Self::Config as ReadableConfig>::LongArrayList<'a>>
    where
        'doc: 'a;
}

pub trait ScopedReadableCompound<'doc>: IntoIterator + Send + Sync + Sized {
    type Config: ReadableConfig;

    fn get_scoped<'a>(&'a self, key: &str) -> Option<<Self::Config as ReadableConfig>::Value<'a>>
    where
        'doc: 'a;

    fn iter_scoped<'a>(&'a self) -> <Self::Config as ReadableConfig>::CompoundIter<'a>
    where
        'doc: 'a;
}

pub trait ScopedReadableEndList<'doc>: IntoIterator + Send + Sync + Sized {
    type Config: ReadableConfig;

    #[inline]
    fn tag_id(&self) -> Tag {
        Tag::End
    }

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool;

    fn get(&self, index: usize) -> Option<()>;

    fn iter_scoped<'a>(&'a self) -> <Self::Config as ReadableConfig>::EndListIter<'a>
    where
        'doc: 'a;
}

pub trait ScopedReadableByteList<'doc>: IntoIterator + Send + Sync + Sized {
    type Config: ReadableConfig;

    #[inline]
    fn tag_id(&self) -> Tag {
        Tag::Byte
    }
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn get(&self, index: usize) -> Option<i8>;
    fn iter_scoped<'a>(&'a self) -> <Self::Config as ReadableConfig>::ByteListIter<'a>
    where
        'doc: 'a;
}

pub trait ScopedReadableShortList<'doc>: IntoIterator + Send + Sync + Sized {
    type Config: ReadableConfig;

    #[inline]
    fn tag_id(&self) -> Tag {
        Tag::Short
    }
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn get(&self, index: usize) -> Option<i16>;
    fn iter_scoped<'a>(&'a self) -> <Self::Config as ReadableConfig>::ShortListIter<'a>
    where
        'doc: 'a;
}

pub trait ScopedReadableIntList<'doc>: IntoIterator + Send + Sync + Sized {
    type Config: ReadableConfig;

    #[inline]
    fn tag_id(&self) -> Tag {
        Tag::Int
    }
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn get(&self, index: usize) -> Option<i32>;
    fn iter_scoped<'a>(&'a self) -> <Self::Config as ReadableConfig>::IntListIter<'a>
    where
        'doc: 'a;
}

pub trait ScopedReadableLongList<'doc>: IntoIterator + Send + Sync + Sized {
    type Config: ReadableConfig;

    #[inline]
    fn tag_id(&self) -> Tag {
        Tag::Long
    }
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn get(&self, index: usize) -> Option<i64>;
    fn iter_scoped<'a>(&'a self) -> <Self::Config as ReadableConfig>::LongListIter<'a>
    where
        'doc: 'a;
}

pub trait ScopedReadableFloatList<'doc>: IntoIterator + Send + Sync + Sized {
    type Config: ReadableConfig;

    #[inline]
    fn tag_id(&self) -> Tag {
        Tag::Float
    }
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn get(&self, index: usize) -> Option<f32>;
    fn iter_scoped<'a>(&'a self) -> <Self::Config as ReadableConfig>::FloatListIter<'a>
    where
        'doc: 'a;
}

pub trait ScopedReadableDoubleList<'doc>: IntoIterator + Send + Sync + Sized {
    type Config: ReadableConfig;

    #[inline]
    fn tag_id(&self) -> Tag {
        Tag::Double
    }
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn get(&self, index: usize) -> Option<f64>;
    fn iter_scoped<'a>(&'a self) -> <Self::Config as ReadableConfig>::DoubleListIter<'a>
    where
        'doc: 'a;
}

pub trait ScopedReadableByteArrayList<'doc>: IntoIterator + Send + Sync + Sized {
    type Config: ReadableConfig;

    #[inline]
    fn tag_id(&self) -> Tag {
        Tag::ByteArray
    }
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn get_scoped<'a>(
        &'a self,
        index: usize,
    ) -> Option<<Self::Config as ReadableConfig>::ByteArray<'a>>
    where
        'doc: 'a;
    fn iter_scoped<'a>(&'a self) -> <Self::Config as ReadableConfig>::ByteArrayListIter<'a>
    where
        'doc: 'a;
}

pub trait ScopedReadableStringList<'doc>: IntoIterator + Send + Sync + Sized {
    type Config: ReadableConfig;

    #[inline]
    fn tag_id(&self) -> Tag {
        Tag::String
    }
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn get_scoped<'a>(
        &'a self,
        index: usize,
    ) -> Option<<Self::Config as ReadableConfig>::String<'a>>
    where
        'doc: 'a;
    fn iter_scoped<'a>(&'a self) -> <Self::Config as ReadableConfig>::StringListIter<'a>
    where
        'doc: 'a;
}

pub trait ScopedReadableListList<'doc>: IntoIterator + Send + Sync + Sized {
    type Config: ReadableConfig;

    #[inline]
    fn tag_id(&self) -> Tag {
        Tag::List
    }
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn get_scoped<'a>(&'a self, index: usize) -> Option<<Self::Config as ReadableConfig>::List<'a>>
    where
        'doc: 'a;
    fn iter_scoped<'a>(&'a self) -> <Self::Config as ReadableConfig>::ListListIter<'a>
    where
        'doc: 'a;
}

pub trait ScopedReadableCompoundList<'doc>: IntoIterator + Send + Sync + Sized {
    type Config: ReadableConfig;

    #[inline]
    fn tag_id(&self) -> Tag {
        Tag::Compound
    }
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn get_scoped<'a>(
        &'a self,
        index: usize,
    ) -> Option<<Self::Config as ReadableConfig>::Compound<'a>>
    where
        'doc: 'a;
    fn iter_scoped<'a>(&'a self) -> <Self::Config as ReadableConfig>::CompoundListIter<'a>
    where
        'doc: 'a;
}

pub trait ScopedReadableIntArrayList<'doc>: IntoIterator + Send + Sync + Sized {
    type Config: ReadableConfig;

    #[inline]
    fn tag_id(&self) -> Tag {
        Tag::IntArray
    }
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn get_scoped<'a>(
        &'a self,
        index: usize,
    ) -> Option<<Self::Config as ReadableConfig>::IntArray<'a>>
    where
        'doc: 'a;
    fn iter_scoped<'a>(&'a self) -> <Self::Config as ReadableConfig>::IntArrayListIter<'a>
    where
        'doc: 'a;
}

pub trait ScopedReadableLongArrayList<'doc>: IntoIterator + Send + Sync + Sized {
    type Config: ReadableConfig;

    #[inline]
    fn tag_id(&self) -> Tag {
        Tag::LongArray
    }
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn get_scoped<'a>(
        &'a self,
        index: usize,
    ) -> Option<<Self::Config as ReadableConfig>::LongArray<'a>>
    where
        'doc: 'a;
    fn iter_scoped<'a>(&'a self) -> <Self::Config as ReadableConfig>::LongArrayListIter<'a>
    where
        'doc: 'a;
}
