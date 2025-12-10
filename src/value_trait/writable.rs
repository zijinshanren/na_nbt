use zerocopy::byteorder;

use crate::{
    ReadableConfig, ScopedWritableCompound, ScopedWritableList, ScopedWritableValue, ValueMut,
    WritableConfig,
    view::{StringViewMut, VecViewMut},
};

pub trait WritableValue<'s>: ScopedWritableValue<'s> + Send + Sync + Sized {
    fn as_byte_array_mut<'a>(&'a mut self) -> Option<&'a mut VecViewMut<'s, i8>>
    where
        's: 'a;

    fn as_string_mut<'a>(&'a mut self) -> Option<&'a mut StringViewMut<'s>>
    where
        's: 'a;

    fn as_list_mut<'a>(
        &'a mut self,
    ) -> Option<&'a mut <Self::ConfigMut as WritableConfig>::ListMut<'s>>
    where
        's: 'a;

    fn as_compound_mut<'a>(
        &'a mut self,
    ) -> Option<&'a mut <Self::ConfigMut as WritableConfig>::CompoundMut<'s>>
    where
        's: 'a;

    fn as_int_array_mut<'a>(
        &'a mut self,
    ) -> Option<
        &'a mut VecViewMut<'s, byteorder::I32<<Self::ConfigMut as ReadableConfig>::ByteOrder>>,
    >
    where
        's: 'a;

    fn as_long_array_mut<'a>(
        &'a mut self,
    ) -> Option<
        &'a mut VecViewMut<'s, byteorder::I64<<Self::ConfigMut as ReadableConfig>::ByteOrder>>,
    >
    where
        's: 'a;

    fn visit_mut<'a, R>(
        &'a mut self,
        match_fn: impl FnOnce(ValueMut<'a, 's, Self::ConfigMut>) -> R,
    ) -> R
    where
        's: 'a;
}
pub trait WritableList<'s>: ScopedWritableList<'s> + Send + Sync + Sized {}
pub trait WritableCompound<'s>: ScopedWritableCompound<'s> + Send + Sync + Sized {}
