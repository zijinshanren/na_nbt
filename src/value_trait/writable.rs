use zerocopy::byteorder;

use crate::{
    ReadableConfig, ScopedWritableCompound, ScopedWritableList, ScopedWritableValue, ValueMut,
    WritableConfig,
    view::{StringViewMut, VecViewMut},
};

/// A trait for values that can be written to or modified as NBT data.
///
/// This trait extends [`ScopedWritableValue`] and allows for more extensive modifications,
/// including accessing mutable references to complex types like lists and compounds.
pub trait WritableValue<'s>: ScopedWritableValue<'s> + Send + Sync + Sized {
    /// Returns a mutable reference to the byte array, if it is one.
    fn as_byte_array_mut<'a>(&'a mut self) -> Option<&'a mut VecViewMut<'s, i8>>
    where
        's: 'a;

    /// Returns a mutable reference to the string, if it is one.
    fn as_string_mut<'a>(&'a mut self) -> Option<&'a mut StringViewMut<'s>>
    where
        's: 'a;

    /// Returns a mutable reference to the list, if it is one.
    fn as_list_mut<'a>(
        &'a mut self,
    ) -> Option<&'a mut <Self::ConfigMut as WritableConfig>::ListMut<'s>>
    where
        's: 'a;

    /// Returns a mutable reference to the compound, if it is one.
    fn as_compound_mut<'a>(
        &'a mut self,
    ) -> Option<&'a mut <Self::ConfigMut as WritableConfig>::CompoundMut<'s>>
    where
        's: 'a;

    /// Returns a mutable reference to the int array, if it is one.
    fn as_int_array_mut<'a>(
        &'a mut self,
    ) -> Option<
        &'a mut VecViewMut<'s, byteorder::I32<<Self::ConfigMut as ReadableConfig>::ByteOrder>>,
    >
    where
        's: 'a;

    /// Returns a mutable reference to the long array, if it is one.
    fn as_long_array_mut<'a>(
        &'a mut self,
    ) -> Option<
        &'a mut VecViewMut<'s, byteorder::I64<<Self::ConfigMut as ReadableConfig>::ByteOrder>>,
    >
    where
        's: 'a;

    /// Visits the mutable value with a closure, allowing for efficient pattern matching and modification.
    fn visit_mut<'a, R>(
        &'a mut self,
        match_fn: impl FnOnce(ValueMut<'a, 's, Self::ConfigMut>) -> R,
    ) -> R
    where
        's: 'a;
}

/// A trait for writable NBT lists.
pub trait WritableList<'s>: ScopedWritableList<'s> + Send + Sync + Sized {}

/// A trait for writable NBT compounds.
pub trait WritableCompound<'s>: ScopedWritableCompound<'s> + Send + Sync + Sized {}
