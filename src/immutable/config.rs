use std::marker::PhantomData;

use zerocopy::byteorder;

use crate::{
    ByteOrder, ConfigRef, Document, GenericNBT, Mark, NBT, NBTBase, ReadonlyArray,
    ReadonlyCompound, ReadonlyCompoundIter, ReadonlyList, ReadonlyListIter, ReadonlyString,
    ReadonlyTypedList, ReadonlyTypedListIter, ReadonlyValue,
};

#[derive(Clone)]
pub struct ImmutableConfig<O: ByteOrder, D: Document> {
    _marker: PhantomData<(O, D)>,
}

impl<O: ByteOrder, D: Document> ConfigRef for ImmutableConfig<O, D> {
    type ByteOrder = O;
    type Value<'doc> = ReadonlyValue<'doc, O, D>;
    type ByteArray<'doc> = ReadonlyArray<'doc, i8, D>;
    type String<'doc> = ReadonlyString<'doc, D>;
    type List<'doc> = ReadonlyList<'doc, O, D>;
    type ListIter<'doc> = ReadonlyListIter<'doc, O, D>;
    type TypedList<'doc, T: NBT> = ReadonlyTypedList<'doc, O, D, T>;
    type TypedListIter<'doc, T: NBT> = ReadonlyTypedListIter<'doc, O, D, T>;
    type Compound<'doc> = ReadonlyCompound<'doc, O, D>;
    type CompoundIter<'doc> = ReadonlyCompoundIter<'doc, O, D>;
    type IntArray<'doc> = ReadonlyArray<'doc, byteorder::I32<O>, D>;
    type LongArray<'doc> = ReadonlyArray<'doc, byteorder::I64<O>, D>;

    type ReadParams<'a> = (*const u8, *const Mark, &'a D);

    #[inline]
    unsafe fn read<'a, 'doc, T: GenericNBT>(
        params: Self::ReadParams<'a>,
    ) -> Option<T::TypeRef<'doc, Self>> {
        unsafe { T::read_immutable_impl(params) }
    }
}

pub trait ImmutableGenericImpl: NBTBase {
    /// .
    ///
    /// # Safety
    ///
    /// .
    unsafe fn read_immutable_impl<'a, 'doc, O: ByteOrder, D: Document>(
        params: <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a>,
    ) -> Option<Self::TypeRef<'doc, ImmutableConfig<O, D>>>;

    /// .
    ///
    /// # Safety
    ///
    /// .
    unsafe fn list_get_immutable_impl<'a, O: ByteOrder, D: Document>(
        params: <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a>,
        index: usize,
    ) -> <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a>;
}

pub trait ImmutableImpl: ImmutableGenericImpl {
    /// .
    ///
    /// # Safety
    ///
    /// .
    unsafe fn size_immutable_impl<O: ByteOrder>(
        payload: *const u8,
        mark: *const Mark,
    ) -> (usize, usize);
}
