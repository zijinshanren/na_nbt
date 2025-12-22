use std::io::Write;

use crate::{ByteOrder, NBT, ReadableConfig, Result, TagID, Value, index::Index};

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
    type Config: ReadableConfig;

    fn tag_id(&self) -> TagID;

    /// .
    ///
    /// # Safety
    ///
    /// .
    unsafe fn to_unchecked<'a, T: NBT>(&'a self) -> T::Type<'a, Self::Config>
    where
        'doc: 'a;

    fn to<'a, T: NBT>(&'a self) -> Option<T::Type<'a, Self::Config>>
    where
        'doc: 'a;

    #[inline]
    fn is<T: NBT>(&self) -> bool {
        self.tag_id() == T::TAG_ID
    }

    fn to_readable<'a>(&'a self) -> <Self::Config as ReadableConfig>::Value<'a>
    where
        'doc: 'a;

    fn get_scoped<'a, I: Index>(
        &'a self,
        index: I,
    ) -> Option<<Self::Config as ReadableConfig>::Value<'a>>
    where
        'doc: 'a;

    fn with<'a, R>(&'a self, match_fn: impl FnOnce(Value<'a, Self::Config>) -> R) -> R
    where
        'doc: 'a;

    fn write_to_vec<TARGET: ByteOrder>(&self) -> Result<Vec<u8>>;

    fn write_to_writer<TARGET: ByteOrder>(&self, writer: impl Write) -> Result<()>;
}

/// A trait for NBT lists with scoped lifetimes.
pub trait ScopedReadableList<'doc>: IntoIterator + Send + Sync + Sized {
    type Config: ReadableConfig;

    fn tag_id(&self) -> TagID;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool;

    fn get_scoped<'a>(
        &'a self,
        index: usize,
    ) -> Option<<Self::Config as ReadableConfig>::Value<'a>>
    where
        'doc: 'a;

    fn iter_scoped<'a>(&'a self) -> <Self::Config as ReadableConfig>::ListIter<'a>
    where
        'doc: 'a;

    /// .
    ///
    /// # Safety
    ///
    /// .
    unsafe fn to_typed_list_unchecked<'a, T: NBT>(
        &'a self,
    ) -> <Self::Config as ReadableConfig>::TypedList<'a, T>
    where
        'doc: 'a;

    fn to_typed_list<'a, T: NBT>(
        &'a self,
    ) -> Option<<Self::Config as ReadableConfig>::TypedList<'a, T>>
    where
        'doc: 'a;
}

pub trait ScopedReadableTypedList<'doc, T: NBT>: IntoIterator + Send + Sync + Sized {
    type Config: ReadableConfig;

    #[inline]
    fn tag_id(&self) -> TagID {
        T::TAG_ID
    }

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool;

    fn get_scoped<'a>(&'a self, index: usize) -> Option<T::Type<'a, Self::Config>>
    where
        'doc: 'a;

    fn iter_scoped<'a>(&'a self) -> <Self::Config as ReadableConfig>::TypedListIter<'a, T>
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
