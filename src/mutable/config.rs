use std::marker::PhantomData;

use zerocopy::byteorder;

use crate::{
    ByteOrder, ConfigMut, ConfigRef, GenericNBT, MutCompound, MutCompoundIter, MutList,
    MutListIter, MutTypedList, MutTypedListIter, MutValue, NBT, NBTBase, RefCompound,
    RefCompoundIter, RefList, RefListIter, RefString, RefTypedList, RefTypedListIter, RefValue,
};

#[derive(Clone)]
pub struct MutableConfig<O: ByteOrder> {
    _marker: PhantomData<O>,
}

impl<O: ByteOrder> ConfigRef for MutableConfig<O> {
    type ByteOrder = O;
    type Value<'doc> = RefValue<'doc, O>;
    type ByteArray<'doc> = &'doc [i8];
    type String<'doc> = RefString<'doc>;
    type List<'doc> = RefList<'doc, O>;
    type ListIter<'doc> = RefListIter<'doc, O>;
    type TypedList<'doc, T: NBT> = RefTypedList<'doc, O, T>;
    type TypedListIter<'doc, T: NBT> = RefTypedListIter<'doc, O, T>;
    type Compound<'doc> = RefCompound<'doc, O>;
    type CompoundIter<'doc> = RefCompoundIter<'doc, O>;
    type IntArray<'doc> = &'doc [byteorder::I32<O>];
    type LongArray<'doc> = &'doc [byteorder::I64<O>];

    type ReadParams<'a> = *const u8;

    unsafe fn read<'a, 'doc, T: GenericNBT>(
        params: Self::ReadParams<'a>,
    ) -> Option<T::TypeRef<'doc, Self>> {
        unsafe { T::read_mutable_impl(params) }
    }
}

// impl<O: ByteOrder> ConfigMut for MutableConfig<O> {
//     type ValueMut<'doc> = MutValue<'doc, O>;
//     type ListMut<'doc> = MutList<'doc, O>;
//     type ListIterMut<'doc> = MutListIter<'doc, O>;
//     type TypedListMut<'doc, T: NBT> = MutTypedList<'doc, O, T>;
//     type TypedListIterMut<'doc, T: NBT> = MutTypedListIter<'doc, O, T>;
//     type CompoundMut<'doc> = MutCompound<'doc, O>;
//     type CompoundIterMut<'doc> = MutCompoundIter<'doc, O>;

//     unsafe fn read_mut<'a, 'doc, T: GenericNBT>(
//         params: Self::ReadParams<'a>,
//     ) -> Option<T::TypeMut<'doc, Self>> {
//         unsafe { T::read_mutable_mut_impl(params) }
//     }
// }

pub trait MutableGenericImpl: NBTBase {
    unsafe fn read_mutable_impl<'a, 'doc, O: ByteOrder>(
        params: <MutableConfig<O> as ConfigRef>::ReadParams<'a>,
    ) -> Option<Self::TypeRef<'doc, MutableConfig<O>>> {
        todo!()
    }

    // unsafe fn read_mutable_mut_impl<'a, 'doc, O: ByteOrder>(
    //     params: <MutableConfig<O> as ConfigRef>::ReadParams<'a>,
    // ) -> Option<Self::TypeMut<'doc, MutableConfig<O>>>;
}

impl<T: NBTBase> MutableGenericImpl for T {}

pub trait MutableImpl: MutableGenericImpl {}

impl<T: MutableGenericImpl> MutableImpl for T {}
