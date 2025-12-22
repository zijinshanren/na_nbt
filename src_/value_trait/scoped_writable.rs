use zerocopy::byteorder;

use crate::{
    NBT, PrimitiveNBT, ReadableConfig, ScopedReadableCompound, ScopedReadableList,
    ScopedReadableTypedList, ScopedReadableValue, ValueMut, WritableConfig,
    index::Index,
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

    /// .
    ///
    /// # Safety
    ///
    /// .
    unsafe fn to_mut_unchecked<'a, T: NBT>(&'a mut self) -> T::TypeMut<'a, Self::ConfigMut>
    where
        's: 'a;

    fn to_mut<'a, T: NBT>(&'a mut self) -> Option<T::TypeMut<'a, Self::ConfigMut>>
    where
        's: 'a;

    /// .
    ///
    /// # Safety
    ///
    /// .
    unsafe fn set_unchecked<T: PrimitiveNBT>(&mut self, value: T::Type<'s, Self::Config>);

    fn set<T: PrimitiveNBT>(&mut self, value: T::Type<'s, Self::Config>) -> bool;

    /// .
    ///
    /// # Safety
    ///
    /// .
    unsafe fn update_unchecked<T: PrimitiveNBT>(
        &mut self,
        f: impl FnOnce(T::Type<'s, Self::Config>) -> T::Type<'s, Self::Config>,
    );

    fn update<T: PrimitiveNBT>(
        &mut self,
        f: impl FnOnce(T::Type<'s, Self::Config>) -> T::Type<'s, Self::Config>,
    ) -> bool;

    fn to_writable<'a>(&'a mut self) -> <Self::ConfigMut as WritableConfig>::ValueMut<'a>
    where
        's: 'a;

    fn get_mut<'a, I: Index>(
        &'a mut self,
        index: I,
    ) -> Option<<Self::ConfigMut as WritableConfig>::ValueMut<'a>>
    where
        's: 'a;

    fn with_mut<'a, R>(
        &'a mut self,
        match_fn: impl FnOnce(ValueMut<'a, Self::ConfigMut>) -> R,
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

    // fn push<V: IntoOwnedValue<<Self::ConfigMut as ReadableConfig>::ByteOrder>>(&mut self, value: V);

    // unsafe fn push_unchecked<V: IntoOwnedValue<<Self::ConfigMut as ReadableConfig>::ByteOrder>>(
    //     &mut self,
    //     value: V,
    // );

    // fn insert<V: IntoOwnedValue<<Self::ConfigMut as ReadableConfig>::ByteOrder>>(
    //     &mut self,
    //     index: usize,
    //     value: V,
    // );

    // unsafe fn insert_unchecked<V: IntoOwnedValue<<Self::ConfigMut as ReadableConfig>::ByteOrder>>(
    //     &mut self,
    //     index: usize,
    //     value: V,
    // );

    // fn pop(&mut self) -> Option<OwnedValue<<Self::ConfigMut as ReadableConfig>::ByteOrder>>;

    // fn remove(
    //     &mut self,
    //     index: usize,
    // ) -> OwnedValue<<Self::ConfigMut as ReadableConfig>::ByteOrder>;
}

pub trait ScopedWritableTypedList<'s, T: NBT>:
    ScopedReadableTypedList<'s, T, Config = Self::ConfigMut> + Send + Sync + Sized
{
    type ConfigMut: WritableConfig;

    fn get_mut<'a>(&'a mut self, index: usize) -> Option<T::TypeMut<'a, Self::ConfigMut>>
    where
        's: 'a;

    fn iter_mut<'a>(&'a mut self) -> <Self::ConfigMut as WritableConfig>::TypedListIterMut<'a, T>
    where
        's: 'a;

    fn push<V: Into<T::TypeOwn<<Self::ConfigMut as ReadableConfig>::ByteOrder>>>(
        &mut self,
        value: V,
    );

    fn insert<V: Into<T::TypeOwn<<Self::ConfigMut as ReadableConfig>::ByteOrder>>>(
        &mut self,
        index: usize,
        value: V,
    );

    fn pop(&mut self) -> Option<T::TypeOwn<<Self::ConfigMut as ReadableConfig>::ByteOrder>>;

    fn remove(
        &mut self,
        index: usize,
    ) -> T::TypeOwn<<Self::ConfigMut as ReadableConfig>::ByteOrder>;
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

    // fn insert<V: IntoOwnedValue<<Self::ConfigMut as ReadableConfig>::ByteOrder>>(
    //     &mut self,
    //     key: &str,
    //     value: V,
    // ) -> Option<OwnedValue<<Self::ConfigMut as ReadableConfig>::ByteOrder>>;

    // fn remove(
    //     &mut self,
    //     key: &str,
    // ) -> Option<OwnedValue<<Self::ConfigMut as ReadableConfig>::ByteOrder>>;
}
