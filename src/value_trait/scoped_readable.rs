use std::io::Write;

use zerocopy::byteorder;

use crate::{
    ByteOrder, Result, Tag,
    index::Index,
    value_trait::{ReadableConfig, ValueScoped},
};

/// A trait for values that can be read as NBT data with scoped lifetimes.
///
/// This trait is the base for [`ReadableValue`](crate::ReadableValue) and provides methods
/// for checking types and accessing primitive values. It is designed to work with values
/// that may have lifetimes tied to a specific scope.
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

    /// Returns the value as a byte array, if it is one.
    fn as_byte_array<'a>(&'a self) -> Option<&'a [i8]>
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

    /// Returns the value as an int array, if it is one.
    fn as_int_array<'a>(
        &'a self,
    ) -> Option<&'a [byteorder::I32<<Self::Config as ReadableConfig>::ByteOrder>]>
    where
        'doc: 'a;
    /// Returns `true` if the value is an int array.
    fn is_int_array(&self) -> bool;

    /// Returns the value as a long array, if it is one.
    fn as_long_array<'a>(
        &'a self,
    ) -> Option<&'a [byteorder::I64<<Self::Config as ReadableConfig>::ByteOrder>]>
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
