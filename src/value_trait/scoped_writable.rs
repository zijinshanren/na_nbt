use zerocopy::byteorder;

use crate::{
    IntoOwnedValue, OwnedValue, ReadableConfig, ScopedReadableCompound, ScopedReadableList,
    ScopedReadableValue,
    index::Index,
    value_trait::{config::WritableConfig, value::ValueMutScoped},
    view::{StringViewMut, VecViewMut},
};

/// A trait for values that can be written to or modified as NBT data with scoped lifetimes.
///
/// This trait extends [`ScopedReadableValue`] and provides methods for modifying primitive values
/// and accessing mutable references to complex types with scoped lifetimes.
pub trait ScopedWritableValue<'s>:
    ScopedReadableValue<'s, Config = Self::ConfigMut> + Send + Sync + Sized
{
    /// The mutable configuration associated with this value.
    type ConfigMut: WritableConfig;

    /// Returns a mutable reference to the byte value, if it is one.
    fn as_byte_mut<'a>(&'a mut self) -> Option<&'a mut i8>
    where
        's: 'a;

    /// Sets the byte value. Returns `true` if successful (i.e., the value was a byte).
    fn set_byte(&mut self, data: i8) -> bool;

    /// Updates the byte value using a function. Returns `true` if successful.
    fn update_byte(&mut self, f: impl FnOnce(i8) -> i8) -> bool;

    /// Returns a mutable reference to the short value, if it is one.
    fn as_short_mut<'a>(
        &'a mut self,
    ) -> Option<&'a mut byteorder::I16<<Self::ConfigMut as ReadableConfig>::ByteOrder>>
    where
        's: 'a;

    /// Sets the short value. Returns `true` if successful.
    fn set_short(&mut self, data: i16) -> bool;

    /// Updates the short value using a function. Returns `true` if successful.
    fn update_short(&mut self, f: impl FnOnce(i16) -> i16) -> bool;

    /// Returns a mutable reference to the int value, if it is one.
    fn as_int_mut<'a>(
        &'a mut self,
    ) -> Option<&'a mut byteorder::I32<<Self::ConfigMut as ReadableConfig>::ByteOrder>>
    where
        's: 'a;

    /// Sets the int value. Returns `true` if successful.
    fn set_int(&mut self, data: i32) -> bool;

    /// Updates the int value using a function. Returns `true` if successful.
    fn update_int(&mut self, f: impl FnOnce(i32) -> i32) -> bool;

    /// Returns a mutable reference to the long value, if it is one.
    fn as_long_mut<'a>(
        &'a mut self,
    ) -> Option<&'a mut byteorder::I64<<Self::ConfigMut as ReadableConfig>::ByteOrder>>
    where
        's: 'a;

    /// Sets the long value. Returns `true` if successful.
    fn set_long(&mut self, data: i64) -> bool;

    /// Updates the long value using a function. Returns `true` if successful.
    fn update_long(&mut self, f: impl FnOnce(i64) -> i64) -> bool;

    /// Returns a mutable reference to the float value, if it is one.
    fn as_float_mut<'a>(
        &'a mut self,
    ) -> Option<&'a mut byteorder::F32<<Self::ConfigMut as ReadableConfig>::ByteOrder>>
    where
        's: 'a;

    /// Sets the float value. Returns `true` if successful.
    fn set_float(&mut self, data: f32) -> bool;

    /// Updates the float value using a function. Returns `true` if successful.
    fn update_float(&mut self, f: impl FnOnce(f32) -> f32) -> bool;

    /// Returns a mutable reference to the double value, if it is one.
    fn as_double_mut<'a>(
        &'a mut self,
    ) -> Option<&'a mut byteorder::F64<<Self::ConfigMut as ReadableConfig>::ByteOrder>>
    where
        's: 'a;

    /// Sets the double value. Returns `true` if successful.
    fn set_double(&mut self, data: f64) -> bool;

    /// Updates the double value using a function. Returns `true` if successful.
    fn update_double(&mut self, f: impl FnOnce(f64) -> f64) -> bool;

    /// Returns a mutable reference to the byte array with a scoped lifetime, if it is one.
    fn as_byte_array_mut_scoped<'a>(&'a mut self) -> Option<VecViewMut<'a, i8>>
    where
        's: 'a;

    /// Returns a mutable reference to the string with a scoped lifetime, if it is one.
    fn as_string_mut_scoped<'a>(&'a mut self) -> Option<StringViewMut<'a>>
    where
        's: 'a;

    /// Returns a mutable reference to the list with a scoped lifetime, if it is one.
    fn as_list_mut_scoped<'a>(
        &'a mut self,
    ) -> Option<<Self::ConfigMut as WritableConfig>::ListMut<'a>>
    where
        's: 'a;

    /// Returns a mutable reference to the compound with a scoped lifetime, if it is one.
    fn as_compound_mut_scoped<'a>(
        &'a mut self,
    ) -> Option<<Self::ConfigMut as WritableConfig>::CompoundMut<'a>>
    where
        's: 'a;

    /// Returns a mutable reference to the int array with a scoped lifetime, if it is one.
    fn as_int_array_mut_scoped<'a>(
        &'a mut self,
    ) -> Option<VecViewMut<'a, byteorder::I32<<Self::ConfigMut as ReadableConfig>::ByteOrder>>>
    where
        's: 'a;

    /// Returns a mutable reference to the long array with a scoped lifetime, if it is one.
    fn as_long_array_mut_scoped<'a>(
        &'a mut self,
    ) -> Option<VecViewMut<'a, byteorder::I64<<Self::ConfigMut as ReadableConfig>::ByteOrder>>>
    where
        's: 'a;

    fn get_mut<'a, I: Index>(
        &'a mut self,
        index: I,
    ) -> Option<<Self::ConfigMut as WritableConfig>::ValueMut<'a>>
    where
        's: 'a;

    fn visit_mut_scoped<'a, R>(
        &'a mut self,
        match_fn: impl FnOnce(ValueMutScoped<'a, Self::ConfigMut>) -> R,
    ) -> R
    where
        's: 'a;
}

pub trait ScopedWritableList<'s>:
    ScopedReadableList<'s, Config = Self::ConfigMut> + Send + Sync + Sized
{
    type ConfigMut: WritableConfig;

    fn get_mut<'a>(
        &'a mut self,
        index: usize,
    ) -> Option<<Self::ConfigMut as WritableConfig>::ValueMut<'a>>
    where
        's: 'a;

    fn iter_mut<'a>(&'a mut self) -> <Self::ConfigMut as WritableConfig>::ListIterMut<'a>
    where
        's: 'a;

    fn push<V: IntoOwnedValue<<Self::ConfigMut as ReadableConfig>::ByteOrder>>(&mut self, value: V);

    /// .
    ///
    /// # Safety
    ///
    /// .
    unsafe fn push_unchecked<V: IntoOwnedValue<<Self::ConfigMut as ReadableConfig>::ByteOrder>>(
        &mut self,
        value: V,
    );

    fn insert<V: IntoOwnedValue<<Self::ConfigMut as ReadableConfig>::ByteOrder>>(
        &mut self,
        index: usize,
        value: V,
    );

    /// .
    ///
    /// # Safety
    ///
    /// .
    unsafe fn insert_unchecked<V: IntoOwnedValue<<Self::ConfigMut as ReadableConfig>::ByteOrder>>(
        &mut self,
        index: usize,
        value: V,
    );

    fn pop(&mut self) -> Option<OwnedValue<<Self::ConfigMut as ReadableConfig>::ByteOrder>>;

    fn remove(
        &mut self,
        index: usize,
    ) -> OwnedValue<<Self::ConfigMut as ReadableConfig>::ByteOrder>;
}

pub trait ScopedWritableCompound<'s>:
    ScopedReadableCompound<'s, Config = Self::ConfigMut> + Send + Sync + Sized
{
    type ConfigMut: WritableConfig;

    fn get_mut<'a>(
        &'a mut self,
        key: &str,
    ) -> Option<<Self::ConfigMut as WritableConfig>::ValueMut<'a>>
    where
        's: 'a;

    fn iter_mut<'a>(&'a mut self) -> <Self::ConfigMut as WritableConfig>::CompoundIterMut<'a>
    where
        's: 'a;

    fn insert<V: IntoOwnedValue<<Self::ConfigMut as ReadableConfig>::ByteOrder>>(
        &mut self,
        key: &str,
        value: V,
    ) -> Option<OwnedValue<<Self::ConfigMut as ReadableConfig>::ByteOrder>>;

    fn remove(
        &mut self,
        key: &str,
    ) -> Option<OwnedValue<<Self::ConfigMut as ReadableConfig>::ByteOrder>>;
}
