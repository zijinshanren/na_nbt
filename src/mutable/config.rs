use std::{marker::PhantomData, ptr, slice};

use zerocopy::byteorder;

use crate::{
    ByteOrder, ConfigMut, ConfigRef, GenericNBT, MutCompound, MutCompoundIter, MutList,
    MutListIter, MutTypedList, MutTypedListIter, MutValue, MutVec, NBT, NBTBase, OwnList, OwnValue,
    RefCompound, RefCompoundIter, RefList, RefListIter, RefString, RefTypedList, RefTypedListIter,
    RefValue, TagID, cold_path, mutable_tag_size, tag::List,
};

unsafe fn list_decrease<O: ByteOrder>(data: &mut MutVec<'_, u8>) {
    unsafe {
        ptr::write(
            data.as_mut_ptr().add(1).cast(),
            byteorder::U32::<O>::new(
                byteorder::U32::<O>::from_bytes(*data.as_ptr().add(1).cast()).get() - 1,
            ),
        )
    }
}

unsafe fn list_increase<O: ByteOrder>(data: &mut MutVec<'_, u8>) {
    unsafe {
        let len = byteorder::U32::<O>::from_bytes(*data.as_ptr().add(1).cast()).get();
        assert!(len < u32::MAX, "list length too long");
        ptr::write(
            data.as_mut_ptr().add(1).cast(),
            byteorder::U32::<O>::new(len + 1),
        )
    }
}

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
        key: &[u8],
    ) -> Option<(crate::TagID, Self::ReadParams<'a>)>
    where
        'doc: 'a,
    {
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

                if key == name_bytes {
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

    type WriteParams<'a> = MutVec<'a, u8>;

    unsafe fn read_mut<'a, 'doc, T: GenericNBT>(
        params: Self::ReadParams<'a>,
    ) -> Option<T::TypeMut<'doc, Self>> {
        unsafe { T::read_mutable_mut_impl(params) }
    }

    unsafe fn list_push<'a, T: NBT>(
        mut params: Self::WriteParams<'a>,
        value: T::Type<Self::ByteOrder>,
    ) {
        unsafe {
            let tag_size = mutable_tag_size(T::TAG_ID);
            let len_bytes = params.len();
            params.reserve(tag_size);
            ptr::write(params.as_mut_ptr().add(len_bytes).cast(), value);
            params.set_len(len_bytes + tag_size);
            list_increase::<O>(&mut params);
        }
    }

    unsafe fn list_pop<'a, T: GenericNBT>(
        params: Self::WriteParams<'a>,
    ) -> Option<T::Type<Self::ByteOrder>> {
        T::dispatch(
            params,
            |mut params| unsafe {
                list_decrease::<O>(&mut params);
                Some(Default::default())
            },
            |mut params| unsafe {
                let tag_size = mutable_tag_size(T::TAG_ID);
                let len_bytes = params.len();
                let value = ptr::read(params.as_mut_ptr().add(len_bytes - tag_size).cast());
                params.set_len(len_bytes - tag_size);
                list_decrease::<O>(&mut params);
                Some(value)
            },
            |mut params| unsafe {
                let tag_size = mutable_tag_size(T::TAG_ID);
                let len_bytes = params.len();
                let value = ptr::read(
                    params
                        .as_mut_ptr()
                        .add(len_bytes - tag_size)
                        .cast::<OwnList<O>>(),
                )
                .typed_::<T::Element>()?;
                params.set_len(len_bytes - tag_size);
                list_decrease::<O>(&mut params);
                let result = ptr::read(&value as *const _ as *const _);
                std::mem::forget(value);
                Some(result)
            },
        )
    }

    unsafe fn list_insert<'a, T: NBT>(
        mut params: Self::WriteParams<'a>,
        index: usize,
        value: T::Type<Self::ByteOrder>,
    ) {
        unsafe {
            let tag_size = mutable_tag_size(T::TAG_ID);
            let pos_bytes = index * tag_size + 1 + 4;
            let len_bytes = params.len();
            params.reserve(tag_size);
            let start = params.as_mut_ptr().add(pos_bytes);
            ptr::copy(start, start.add(tag_size), len_bytes - pos_bytes);
            ptr::write(start.cast(), value);
            params.set_len(len_bytes + tag_size);
            list_increase::<O>(&mut params);
        }
    }

    unsafe fn list_remove<'a, T: GenericNBT>(
        params: Self::WriteParams<'a>,
        index: usize,
    ) -> Option<T::Type<Self::ByteOrder>> {
        T::dispatch(
            params,
            |mut params| unsafe {
                list_decrease::<O>(&mut params);
                Some(Default::default())
            },
            |mut params| unsafe {
                let tag_size = mutable_tag_size(T::TAG_ID);
                let pos_bytes = index * tag_size + 1 + 4;
                let len_bytes = params.len();
                let value = ptr::read(params.as_mut_ptr().add(pos_bytes).cast());
                let start = params.as_mut_ptr().add(pos_bytes);
                ptr::copy(start.add(tag_size), start, len_bytes - pos_bytes - tag_size);
                params.set_len(len_bytes - tag_size);
                list_decrease::<O>(&mut params);
                Some(value)
            },
            |mut params| unsafe {
                let tag_size = mutable_tag_size(T::TAG_ID);
                let pos_bytes = index * tag_size + 1 + 4;
                let len_bytes = params.len();
                let value = ptr::read(params.as_mut_ptr().add(pos_bytes).cast::<OwnList<O>>())
                    .typed_::<T::Element>()?;
                let start = params.as_mut_ptr().add(pos_bytes);
                ptr::copy(start.add(tag_size), start, len_bytes - pos_bytes - tag_size);
                params.set_len(len_bytes - tag_size);
                list_decrease::<O>(&mut params);
                let result = ptr::read(&value as *const _ as *const _);
                std::mem::forget(value);
                Some(result)
            },
        )
    }

    unsafe fn compound_insert<'a, T: GenericNBT>(
        params: Self::WriteParams<'a>,
        key: &[u8],
        value: T::Type<Self::ByteOrder>,
    ) {
        T::dispatch(
            (params, key, value),
            |_| panic!("End cannot be inserted into a compound"),
            |(mut params, key, value)| unsafe {
                let name_bytes = key;
                let name_len = byteorder::U16::<O>::new(name_bytes.len() as u16).to_bytes();
                let tag_size = mutable_tag_size(T::TAG_ID);
                let old_len = params.len();

                params.reserve(1 + 2 + name_bytes.len() + tag_size);

                let mut write_ptr = params.as_mut_ptr().add(old_len);
                ptr::write(write_ptr.cast(), T::TAG_ID as u8);
                write_ptr = write_ptr.add(1);
                ptr::write(write_ptr.cast(), name_len);
                write_ptr = write_ptr.add(2);
                ptr::copy_nonoverlapping(name_bytes.as_ptr(), write_ptr.cast(), name_bytes.len());
                write_ptr = write_ptr.add(name_bytes.len());
                ptr::write(write_ptr.cast(), value);
                write_ptr = write_ptr.add(tag_size);
                ptr::write(write_ptr.cast(), TagID::End as u8);

                params.set_len(old_len + 1 + 2 + name_bytes.len() + tag_size);
            },
            |(params, key, value)| unsafe {
                let list = ptr::read(&value as *const _ as *const _);
                std::mem::forget(value);
                MutableConfig::<O>::compound_insert::<List>(params, key, list);
            },
        )
    }

    unsafe fn compound_remove<'a>(
        mut params: Self::WriteParams<'a>,
        key: &[u8],
    ) -> Option<super::OwnValue<Self::ByteOrder>> {
        unsafe {
            let mut ptr = params.as_mut_ptr();
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

                if key == name_bytes {
                    let tag_size = mutable_tag_size(tag_id);
                    let pos_bytes = ptr.byte_offset_from_unsigned(params.as_mut_ptr());
                    let value = OwnValue::<O>::read(tag_id, ptr);
                    let len_bytes = params.len();
                    ptr::copy(
                        ptr.add(tag_size),
                        ptr.sub(name_len as usize + 2 + 1),
                        len_bytes - pos_bytes - tag_size,
                    );
                    params.set_len(len_bytes - (tag_size + name_len as usize + 2 + 1));
                    return Some(value);
                }

                ptr = ptr.add(mutable_tag_size(tag_id));
            }
        }
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
