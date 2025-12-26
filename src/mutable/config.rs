use std::{marker::PhantomData, slice};

use zerocopy::byteorder;

use crate::{
    ByteOrder, ConfigMut, ConfigRef, GenericNBT, MutCompound, MutCompoundIter, MutList,
    MutListIter, MutTypedList, MutTypedListIter, MutValue, NBT, NBTBase, RefCompound,
    RefCompoundIter, RefList, RefListIter, RefString, RefTypedList, RefTypedListIter, RefValue,
    TagID, cold_path, mutable_tag_size,
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

    unsafe fn list_get<'a, 'doc, T: GenericNBT>(
        params: Self::ReadParams<'a>,
        index: usize,
    ) -> Self::ReadParams<'a>
    where
        'doc: 'a,
    {
        unsafe { params.add(index * mutable_tag_size(T::TAG_ID)) }
    }

    unsafe fn compound_get<'a, 'doc>(
        params: Self::ReadParams<'a>,
        key: &str,
    ) -> Option<(crate::TagID, Self::ReadParams<'a>)>
    where
        'doc: 'a,
    {
        let name = simd_cesu8::mutf8::encode(key);

        unsafe {
            let mut ptr = params;
            loop {
                let tag_id = *ptr.cast();
                ptr = ptr.add(1);

                if tag_id == TagID::End {
                    cold_path();
                    return None;
                }

                let name_len = byteorder::U16::<O>::from_bytes(*ptr.cast()).get();
                ptr = ptr.add(2);

                let name_bytes = slice::from_raw_parts(ptr, name_len as usize);
                ptr = ptr.add(name_len as usize);

                if name == name_bytes {
                    return Some((tag_id, ptr));
                }

                ptr = ptr.add(mutable_tag_size(tag_id));
            }
        }
    }

    unsafe fn read<'a, 'doc, T: GenericNBT>(
        params: Self::ReadParams<'a>,
    ) -> Option<T::TypeRef<'doc, Self>> {
        unsafe { T::read_mutable_impl(params) }
    }
}

impl<O: ByteOrder> ConfigMut for MutableConfig<O> {
    type ValueMut<'doc> = MutValue<'doc, O>;
    type ListMut<'doc> = MutList<'doc, O>;
    type ListIterMut<'doc> = MutListIter<'doc, O>;
    type TypedListMut<'doc, T: NBT> = MutTypedList<'doc, O, T>;
    type TypedListIterMut<'doc, T: NBT> = MutTypedListIter<'doc, O, T>;
    type CompoundMut<'doc> = MutCompound<'doc, O>;
    type CompoundIterMut<'doc> = MutCompoundIter<'doc, O>;

    unsafe fn read_mut<'a, 'doc, T: GenericNBT>(
        params: Self::ReadParams<'a>,
    ) -> Option<T::TypeMut<'doc, Self>> {
        unsafe { T::read_mutable_mut_impl(params) }
    }
}

pub trait MutableGenericImpl: NBTBase {
    /// .
    ///
    /// # Safety
    ///
    /// .
    unsafe fn read_mutable_impl<'a, 'doc, O: ByteOrder>(
        params: <MutableConfig<O> as ConfigRef>::ReadParams<'a>,
    ) -> Option<Self::TypeRef<'doc, MutableConfig<O>>>;

    /// .
    ///
    /// # Safety
    ///
    /// .
    unsafe fn read_mutable_mut_impl<'a, 'doc, O: ByteOrder>(
        params: <MutableConfig<O> as ConfigRef>::ReadParams<'a>,
    ) -> Option<Self::TypeMut<'doc, MutableConfig<O>>>;
}

pub trait MutableImpl: MutableGenericImpl {}
