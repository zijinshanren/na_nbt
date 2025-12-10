use zerocopy::byteorder;

use crate::{
    IntoOwnedValue, OwnedValue, ReadableConfig, ScopedReadableCompound, ScopedReadableList,
    ScopedReadableValue,
    index::Index,
    value_trait::{config::WritableConfig, value::ValueMutScoped},
    view::{StringViewMut, VecViewMut},
};

pub trait ScopedWritableValue<'s>:
    ScopedReadableValue<'s, Config = Self::ConfigMut> + Send + Sync + Sized
{
    type ConfigMut: WritableConfig;

    fn as_byte_mut<'a>(&'a mut self) -> Option<&'a mut i8>
    where
        's: 'a;

    fn set_byte(&mut self, data: i8) -> bool;

    fn update_byte(&mut self, f: impl FnOnce(i8) -> i8) -> bool;

    fn as_short_mut<'a>(
        &'a mut self,
    ) -> Option<&'a mut byteorder::I16<<Self::ConfigMut as ReadableConfig>::ByteOrder>>
    where
        's: 'a;

    fn set_short(&mut self, data: i16) -> bool;

    fn update_short(&mut self, f: impl FnOnce(i16) -> i16) -> bool;

    fn as_int_mut<'a>(
        &'a mut self,
    ) -> Option<&'a mut byteorder::I32<<Self::ConfigMut as ReadableConfig>::ByteOrder>>
    where
        's: 'a;

    fn set_int(&mut self, data: i32) -> bool;

    fn update_int(&mut self, f: impl FnOnce(i32) -> i32) -> bool;

    fn as_long_mut<'a>(
        &'a mut self,
    ) -> Option<&'a mut byteorder::I64<<Self::ConfigMut as ReadableConfig>::ByteOrder>>
    where
        's: 'a;

    fn set_long(&mut self, data: i64) -> bool;

    fn update_long(&mut self, f: impl FnOnce(i64) -> i64) -> bool;

    fn as_float_mut<'a>(
        &'a mut self,
    ) -> Option<&'a mut byteorder::F32<<Self::ConfigMut as ReadableConfig>::ByteOrder>>
    where
        's: 'a;

    fn set_float(&mut self, data: f32) -> bool;

    fn update_float(&mut self, f: impl FnOnce(f32) -> f32) -> bool;

    fn as_double_mut<'a>(
        &'a mut self,
    ) -> Option<&'a mut byteorder::F64<<Self::ConfigMut as ReadableConfig>::ByteOrder>>
    where
        's: 'a;

    fn set_double(&mut self, data: f64) -> bool;

    fn update_double(&mut self, f: impl FnOnce(f64) -> f64) -> bool;

    fn as_byte_array_mut_scoped<'a>(&'a mut self) -> Option<VecViewMut<'a, i8>>
    where
        's: 'a;

    fn as_string_mut_scoped<'a>(&'a mut self) -> Option<StringViewMut<'a>>
    where
        's: 'a;

    fn as_list_mut_scoped<'a>(
        &'a mut self,
    ) -> Option<<Self::ConfigMut as WritableConfig>::ListMut<'a>>
    where
        's: 'a;

    fn as_compound_mut_scoped<'a>(
        &'a mut self,
    ) -> Option<<Self::ConfigMut as WritableConfig>::CompoundMut<'a>>
    where
        's: 'a;

    fn as_int_array_mut_scoped<'a>(
        &'a mut self,
    ) -> Option<VecViewMut<'a, byteorder::I32<<Self::ConfigMut as ReadableConfig>::ByteOrder>>>
    where
        's: 'a;

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
