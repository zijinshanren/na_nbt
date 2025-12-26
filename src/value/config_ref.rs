use std::ops::Deref;

use zerocopy::byteorder;

use crate::{
    ByteOrder, CompoundRef, GenericNBT, ListRef, NBT, StringRef, TagID, TypedListRef, ValueRef,
    tag::{
        Byte, ByteArray, Compound, Double, End, Float, Int, IntArray, List, Long, LongArray, Short,
        String,
    },
};

pub trait ConfigRef: Send + Sync + Sized + Clone + 'static {
    type ByteOrder: ByteOrder;
    type Value<'doc>: ValueRef<'doc, Config = Self>;
    type ByteArray<'doc>: Deref<Target = [i8]> + Clone + Default;
    type String<'doc>: StringRef<'doc>;
    type List<'doc>: ListRef<'doc, Config = Self>;
    type ListIter<'doc>: Iterator<Item = Self::Value<'doc>> + ExactSizeIterator + Clone + Default;
    type TypedList<'doc, T: NBT>: TypedListRef<'doc, T, Config = Self>;
    type TypedListIter<'doc, T: NBT>: Iterator<Item = T::TypeRef<'doc, Self>>
        + ExactSizeIterator
        + Clone
        + Default;
    type Compound<'doc>: CompoundRef<'doc, Config = Self>;
    type CompoundIter<'doc>: Iterator<Item = (Self::String<'doc>, Self::Value<'doc>)>
        + Clone
        + Default;
    type IntArray<'doc>: Deref<Target = [byteorder::I32<Self::ByteOrder>]> + Clone + Default;
    type LongArray<'doc>: Deref<Target = [byteorder::I64<Self::ByteOrder>]> + Clone + Default;

    type ReadParams<'a>: Sized;

    /// .
    ///
    /// # Safety
    ///
    /// .
    unsafe fn list_get<'a, 'doc, T: GenericNBT>(
        value: Self::ReadParams<'a>,
        index: usize,
    ) -> Self::ReadParams<'a>
    where
        'doc: 'a;

    /// .
    ///
    /// # Safety
    ///
    /// .
    unsafe fn compound_get<'a, 'doc>(
        value: Self::ReadParams<'a>,
        key: &str,
    ) -> Option<(TagID, Self::ReadParams<'a>)>
    where
        'doc: 'a;

    /// .
    ///
    /// # Safety
    ///
    /// .
    unsafe fn read<'a, 'doc, T: GenericNBT>(
        params: Self::ReadParams<'a>,
    ) -> Option<T::TypeRef<'doc, Self>>;

    /// .
    ///
    /// # Safety
    ///
    /// .
    #[inline]
    #[allow(clippy::unit_arg)]
    unsafe fn read_value<'a, 'doc>(
        tag_id: TagID,
        params: Self::ReadParams<'a>,
    ) -> Self::Value<'doc> {
        unsafe {
            match tag_id {
                TagID::End => From::from(Self::read::<End>(params).unwrap_unchecked()),
                TagID::Byte => From::from(Self::read::<Byte>(params).unwrap_unchecked()),
                TagID::Short => From::from(Self::read::<Short>(params).unwrap_unchecked()),
                TagID::Int => From::from(Self::read::<Int>(params).unwrap_unchecked()),
                TagID::Long => From::from(Self::read::<Long>(params).unwrap_unchecked()),
                TagID::Float => From::from(Self::read::<Float>(params).unwrap_unchecked()),
                TagID::Double => From::from(Self::read::<Double>(params).unwrap_unchecked()),
                TagID::ByteArray => From::from(Self::read::<ByteArray>(params).unwrap_unchecked()),
                TagID::String => From::from(Self::read::<String>(params).unwrap_unchecked()),
                TagID::List => From::from(Self::read::<List>(params).unwrap_unchecked()),
                TagID::Compound => From::from(Self::read::<Compound>(params).unwrap_unchecked()),
                TagID::IntArray => From::from(Self::read::<IntArray>(params).unwrap_unchecked()),
                TagID::LongArray => From::from(Self::read::<LongArray>(params).unwrap_unchecked()),
            }
        }
    }
}
