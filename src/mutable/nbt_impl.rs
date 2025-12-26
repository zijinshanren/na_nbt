use std::{marker::PhantomData, ptr, slice};

use zerocopy::byteorder;

use crate::{
    ByteOrder, ConfigMut, MutCompound, MutList, MutString, MutVec, MutableConfig,
    MutableGenericImpl, MutableImpl, NBT, NBTBase, OwnList, RefCompound, RefList, RefString,
    SIZE_USIZE, TagID, mutable_tag_size,
    tag::{
        Byte, ByteArray, Compound, Double, End, Float, Int, IntArray, List, Long, LongArray, Short,
        String, TypedList,
    },
};

pub(crate) unsafe fn list_decrease<O: ByteOrder>(data: &mut MutVec<'_, u8>) {
    unsafe {
        ptr::write(
            data.as_mut_ptr().add(1).cast(),
            byteorder::U32::<O>::new(
                byteorder::U32::<O>::from_bytes(*data.as_ptr().add(1).cast()).get() - 1,
            ),
        )
    }
}

pub(crate) unsafe fn list_increase<O: ByteOrder>(data: &mut MutVec<'_, u8>) {
    unsafe {
        let len = byteorder::U32::<O>::from_bytes(*data.as_ptr().add(1).cast()).get();
        assert!(len < u32::MAX, "list length too long");
        ptr::write(
            data.as_mut_ptr().add(1).cast(),
            byteorder::U32::<O>::new(len + 1),
        )
    }
}

impl MutableGenericImpl for End {
    #[inline]
    unsafe fn read_mutable_impl<'a, 'doc, O: ByteOrder>(
        _data: *const u8,
    ) -> Option<Self::TypeRef<'doc, MutableConfig<O>>> {
        Some(())
    }

    #[inline]
    unsafe fn read_mutable_mut_impl<'a, 'doc, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeMut<'doc, MutableConfig<O>>> {
        Some(unsafe { &mut *data.cast_mut().cast() })
    }

    #[inline]
    unsafe fn list_pop_impl<'a, O: ByteOrder>(
        mut params: <MutableConfig<O> as ConfigMut>::WriteParams<'a>,
    ) -> Option<Self::Type<O>> {
        unsafe { list_decrease::<O>(&mut params) };
        Some(())
    }

    #[inline]
    unsafe fn list_remove_impl<'a, O: ByteOrder>(
        mut params: <MutableConfig<O> as ConfigMut>::WriteParams<'a>,
        _: usize,
    ) -> Option<Self::Type<O>> {
        unsafe { list_decrease::<O>(&mut params) };
        Some(())
    }

    #[inline]
    unsafe fn compound_insert_impl<'a, O: ByteOrder>(
        _: <MutableConfig<O> as ConfigMut>::WriteParams<'a>,
        _: &[u8],
        _: Self::Type<O>,
    ) {
        panic!("End cannot be inserted into a compound");
    }
}

macro_rules! common_impl {
    () => {
        #[inline]
        unsafe fn list_pop_impl<'a, O: ByteOrder>(
            mut params: <MutableConfig<O> as ConfigMut>::WriteParams<'a>,
        ) -> Option<Self::Type<O>> {
            unsafe {
                let tag_size = mutable_tag_size(Self::TAG_ID);
                let len_bytes = params.len();
                let value = ptr::read(params.as_mut_ptr().add(len_bytes - tag_size).cast());
                params.set_len(len_bytes - tag_size);
                list_decrease::<O>(&mut params);
                Some(value)
            }
        }

        #[inline]
        unsafe fn list_remove_impl<'a, O: ByteOrder>(
            mut params: <MutableConfig<O> as ConfigMut>::WriteParams<'a>,
            index: usize,
        ) -> Option<Self::Type<O>> {
            unsafe {
                let tag_size = mutable_tag_size(Self::TAG_ID);
                let pos_bytes = index * tag_size + 1 + 4;
                let len_bytes = params.len();
                let value = ptr::read(params.as_mut_ptr().add(pos_bytes).cast());
                let start = params.as_mut_ptr().add(pos_bytes);
                ptr::copy(start.add(tag_size), start, len_bytes - pos_bytes - tag_size);
                params.set_len(len_bytes - tag_size);
                list_decrease::<O>(&mut params);
                Some(value)
            }
        }

        #[inline]
        unsafe fn compound_insert_impl<'a, O: ByteOrder>(
            mut params: <MutableConfig<O> as ConfigMut>::WriteParams<'a>,
            key: &[u8],
            value: Self::Type<O>,
        ) {
            unsafe {
                let name_bytes = key;
                let name_len = byteorder::U16::<O>::new(name_bytes.len() as u16).to_bytes();
                let tag_size = mutable_tag_size(Self::TAG_ID);
                let old_len = params.len();

                params.reserve(1 + 2 + name_bytes.len() + tag_size);

                let mut write_ptr = params.as_mut_ptr().add(old_len);
                ptr::write(write_ptr.cast(), Self::TAG_ID as u8);
                write_ptr = write_ptr.add(1);
                ptr::write(write_ptr.cast(), name_len);
                write_ptr = write_ptr.add(2);
                ptr::copy_nonoverlapping(name_bytes.as_ptr(), write_ptr.cast(), name_bytes.len());
                write_ptr = write_ptr.add(name_bytes.len());
                ptr::write(write_ptr.cast(), value);
                write_ptr = write_ptr.add(tag_size);
                ptr::write(write_ptr.cast(), TagID::End as u8);

                params.set_len(old_len + 1 + 2 + name_bytes.len() + tag_size);
            }
        }
    };
}

impl MutableImpl for End {}

impl MutableGenericImpl for Byte {
    #[inline]
    unsafe fn read_mutable_impl<'a, 'doc, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeRef<'doc, MutableConfig<O>>> {
        Some(unsafe { *data.cast() })
    }

    #[inline]
    unsafe fn read_mutable_mut_impl<'a, 'doc, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeMut<'doc, MutableConfig<O>>> {
        Some(unsafe { &mut *data.cast_mut().cast() })
    }

    common_impl!();
}

impl MutableImpl for Byte {}

impl MutableGenericImpl for Short {
    #[inline]
    unsafe fn read_mutable_impl<'a, 'doc, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeRef<'doc, MutableConfig<O>>> {
        Some(unsafe { byteorder::I16::<O>::from_bytes(*data.cast()).get() })
    }

    #[inline]
    unsafe fn read_mutable_mut_impl<'a, 'doc, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeMut<'doc, MutableConfig<O>>> {
        Some(unsafe { &mut *data.cast_mut().cast() })
    }

    common_impl!();
}

impl MutableImpl for Short {}

impl MutableGenericImpl for Int {
    #[inline]
    unsafe fn read_mutable_impl<'a, 'doc, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeRef<'doc, MutableConfig<O>>> {
        Some(unsafe { byteorder::I32::<O>::from_bytes(*data.cast()).get() })
    }

    #[inline]
    unsafe fn read_mutable_mut_impl<'a, 'doc, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeMut<'doc, MutableConfig<O>>> {
        Some(unsafe { &mut *data.cast_mut().cast() })
    }

    common_impl!();
}

impl MutableImpl for Int {}

impl MutableGenericImpl for Long {
    #[inline]
    unsafe fn read_mutable_impl<'a, 'doc, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeRef<'doc, MutableConfig<O>>> {
        Some(unsafe { byteorder::I64::<O>::from_bytes(*data.cast()).get() })
    }

    #[inline]
    unsafe fn read_mutable_mut_impl<'a, 'doc, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeMut<'doc, MutableConfig<O>>> {
        Some(unsafe { &mut *data.cast_mut().cast() })
    }

    common_impl!();
}

impl MutableImpl for Long {}

impl MutableGenericImpl for Float {
    #[inline]
    unsafe fn read_mutable_impl<'a, 'doc, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeRef<'doc, MutableConfig<O>>> {
        Some(unsafe { byteorder::F32::<O>::from_bytes(*data.cast()).get() })
    }

    #[inline]
    unsafe fn read_mutable_mut_impl<'a, 'doc, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeMut<'doc, MutableConfig<O>>> {
        Some(unsafe { &mut *data.cast_mut().cast() })
    }

    common_impl!();
}

impl MutableImpl for Float {}

impl MutableGenericImpl for Double {
    #[inline]
    unsafe fn read_mutable_impl<'a, 'doc, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeRef<'doc, MutableConfig<O>>> {
        Some(unsafe { byteorder::F64::<O>::from_bytes(*data.cast()).get() })
    }

    #[inline]
    unsafe fn read_mutable_mut_impl<'a, 'doc, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeMut<'doc, MutableConfig<O>>> {
        Some(unsafe { &mut *data.cast_mut().cast() })
    }

    common_impl!();
}

impl MutableImpl for Double {}

impl MutableGenericImpl for ByteArray {
    #[inline]
    unsafe fn read_mutable_impl<'a, 'doc, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeRef<'doc, MutableConfig<O>>> {
        unsafe {
            let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
            let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
            Some(slice::from_raw_parts(ptr, len))
        }
    }

    #[inline]
    unsafe fn read_mutable_mut_impl<'a, 'doc, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeMut<'doc, MutableConfig<O>>> {
        unsafe {
            let data = data.cast_mut();
            let ptr_ref = &mut *(data.cast());
            let len_ref = &mut *(data.add(SIZE_USIZE).cast());
            let cap_ref = &mut *(data.add(SIZE_USIZE * 2).cast());
            Some(MutVec::new(ptr_ref, len_ref, cap_ref))
        }
    }

    common_impl!();
}

impl MutableImpl for ByteArray {}

impl MutableGenericImpl for String {
    #[inline]
    unsafe fn read_mutable_impl<'a, 'doc, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeRef<'doc, MutableConfig<O>>> {
        unsafe {
            let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
            let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
            Some(RefString {
                data: slice::from_raw_parts(ptr, len),
            })
        }
    }

    #[inline]
    unsafe fn read_mutable_mut_impl<'a, 'doc, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeMut<'doc, MutableConfig<O>>> {
        unsafe {
            let data = data.cast_mut();
            let ptr_ref = &mut *(data.cast());
            let len_ref = &mut *(data.add(SIZE_USIZE).cast());
            let cap_ref = &mut *(data.add(SIZE_USIZE * 2).cast());
            Some(MutString::new(ptr_ref, len_ref, cap_ref))
        }
    }

    common_impl!();
}

impl MutableImpl for String {}

impl MutableGenericImpl for List {
    #[inline]
    unsafe fn read_mutable_impl<'a, 'doc, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeRef<'doc, MutableConfig<O>>> {
        unsafe {
            let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
            Some(RefList {
                data: ptr,
                _marker: PhantomData,
            })
        }
    }

    #[inline]
    unsafe fn read_mutable_mut_impl<'a, 'doc, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeMut<'doc, MutableConfig<O>>> {
        unsafe {
            let data = data.cast_mut();
            let ptr_ref = &mut *(data.cast());
            let len_ref = &mut *(data.add(SIZE_USIZE).cast());
            let cap_ref = &mut *(data.add(SIZE_USIZE * 2).cast());
            Some(MutList {
                data: MutVec::new(ptr_ref, len_ref, cap_ref),
                _marker: PhantomData,
            })
        }
    }

    common_impl!();
}

impl MutableImpl for List {}

impl MutableGenericImpl for Compound {
    #[inline]
    unsafe fn read_mutable_impl<'a, 'doc, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeRef<'doc, MutableConfig<O>>> {
        unsafe {
            let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
            Some(RefCompound {
                data: ptr,
                _marker: PhantomData,
            })
        }
    }

    #[inline]
    unsafe fn read_mutable_mut_impl<'a, 'doc, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeMut<'doc, MutableConfig<O>>> {
        unsafe {
            let data = data.cast_mut();
            let ptr_ref = &mut *(data.cast());
            let len_ref = &mut *(data.add(SIZE_USIZE).cast());
            let cap_ref = &mut *(data.add(SIZE_USIZE * 2).cast());
            Some(MutCompound {
                data: MutVec::new(ptr_ref, len_ref, cap_ref),
                _marker: PhantomData,
            })
        }
    }

    common_impl!();
}

impl MutableImpl for Compound {}

impl MutableGenericImpl for IntArray {
    #[inline]
    unsafe fn read_mutable_impl<'a, 'doc, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeRef<'doc, MutableConfig<O>>> {
        unsafe {
            let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
            let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
            Some(slice::from_raw_parts(ptr, len))
        }
    }

    #[inline]
    unsafe fn read_mutable_mut_impl<'a, 'doc, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeMut<'doc, MutableConfig<O>>> {
        unsafe {
            let data = data.cast_mut();
            let ptr_ref = &mut *(data.cast());
            let len_ref = &mut *(data.add(SIZE_USIZE).cast());
            let cap_ref = &mut *(data.add(SIZE_USIZE * 2).cast());
            Some(MutVec::new(ptr_ref, len_ref, cap_ref))
        }
    }

    common_impl!();
}

impl MutableImpl for IntArray {}

impl MutableGenericImpl for LongArray {
    #[inline]
    unsafe fn read_mutable_impl<'a, 'doc, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeRef<'doc, MutableConfig<O>>> {
        unsafe {
            let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
            let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
            Some(slice::from_raw_parts(ptr, len))
        }
    }

    #[inline]
    unsafe fn read_mutable_mut_impl<'a, 'doc, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeMut<'doc, MutableConfig<O>>> {
        unsafe {
            let data = data.cast_mut();
            let ptr_ref = &mut *(data.cast());
            let len_ref = &mut *(data.add(SIZE_USIZE).cast());
            let cap_ref = &mut *(data.add(SIZE_USIZE * 2).cast());
            Some(MutVec::new(ptr_ref, len_ref, cap_ref))
        }
    }

    common_impl!();
}

impl MutableImpl for LongArray {}

impl<T: NBT> MutableGenericImpl for TypedList<T> {
    #[inline]
    unsafe fn read_mutable_impl<'a, 'doc, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeRef<'doc, MutableConfig<O>>> {
        unsafe { List::read_mutable_impl(data)?.typed_::<T>() }
    }

    #[inline]
    unsafe fn read_mutable_mut_impl<'a, 'doc, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeMut<'doc, MutableConfig<O>>> {
        unsafe { List::read_mutable_mut_impl(data)?.typed_::<T>() }
    }

    #[inline]
    unsafe fn list_pop_impl<'a, O: ByteOrder>(
        mut params: <MutableConfig<O> as ConfigMut>::WriteParams<'a>,
    ) -> Option<Self::Type<O>> {
        unsafe {
            let tag_size = mutable_tag_size(Self::TAG_ID);
            let len_bytes = params.len();
            let value = ptr::read(
                params
                    .as_mut_ptr()
                    .add(len_bytes - tag_size)
                    .cast::<OwnList<O>>(),
            )
            .typed_::<T>()?;
            params.set_len(len_bytes - tag_size);
            list_decrease::<O>(&mut params);
            Some(value)
        }
    }

    #[inline]
    unsafe fn list_remove_impl<'a, O: ByteOrder>(
        mut params: <MutableConfig<O> as ConfigMut>::WriteParams<'a>,
        index: usize,
    ) -> Option<Self::Type<O>> {
        unsafe {
            let tag_size = mutable_tag_size(Self::TAG_ID);
            let pos_bytes = index * tag_size + 1 + 4;
            let len_bytes = params.len();
            let value =
                ptr::read(params.as_mut_ptr().add(pos_bytes).cast::<OwnList<O>>()).typed_::<T>()?;
            let start = params.as_mut_ptr().add(pos_bytes);
            ptr::copy(start.add(tag_size), start, len_bytes - pos_bytes - tag_size);
            params.set_len(len_bytes - tag_size);
            list_decrease::<O>(&mut params);
            Some(value)
        }
    }

    #[inline]
    unsafe fn compound_insert_impl<'a, O: ByteOrder>(
        params: <MutableConfig<O> as ConfigMut>::WriteParams<'a>,
        key: &[u8],
        value: Self::Type<O>,
    ) {
        unsafe {
            List::compound_insert_impl(
                params,
                key,
                OwnList {
                    data: value.data,
                    _marker: PhantomData::<O>,
                },
            )
        };
    }
}
