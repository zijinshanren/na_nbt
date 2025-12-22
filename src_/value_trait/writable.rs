use zerocopy::byteorder;

use crate::{
    NBT, ReadableConfig, ScopedWritableCompound, ScopedWritableList, ScopedWritableTypedList,
    ScopedWritableValue, ValueMut, WritableConfig,
    view::{StringViewMut, VecViewMut},
};

/// A trait for values that can be written to or modified as NBT data.
///
/// This trait extends [`ScopedWritableValue`] and allows for more extensive modifications,
/// including accessing mutable references to complex types like lists and compounds.
pub trait WritableValue<'s>: ScopedWritableValue<'s> + Send + Sync + Sized {
    /// .
    ///
    /// # Safety
    ///
    /// .
    unsafe fn peek_mut_unchecked<'a, T: NBT>(
        &'a mut self,
    ) -> &'a mut T::TypeMut<'s, Self::ConfigMut>
    where
        's: 'a;

    fn peek_mut<'a, T: NBT>(&'a mut self) -> Option<&'a mut T::TypeMut<'s, Self::ConfigMut>>
    where
        's: 'a;

    /// .
    ///
    /// # Safety
    ///
    /// .
    unsafe fn extract_mut_unchecked<T: NBT>(self) -> T::TypeMut<'s, Self::ConfigMut>;

    fn extract_mut<T: NBT>(self) -> Option<T::TypeMut<'s, Self::ConfigMut>>;

    fn visit_mut<R>(self, match_fn: impl FnOnce(ValueMut<'s, Self::ConfigMut>) -> R) -> R;
}

/// A trait for writable NBT lists.
pub trait WritableList<'s>: ScopedWritableList<'s> + Send + Sync + Sized {}

pub trait WritableTypedList<'s, T: NBT>:
    ScopedWritableTypedList<'s, T> + Send + Sync + Sized
{
}

/// A trait for writable NBT compounds.
pub trait WritableCompound<'s>: ScopedWritableCompound<'s> + Send + Sync + Sized {}
