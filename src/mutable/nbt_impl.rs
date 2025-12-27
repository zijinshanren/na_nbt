use std::{marker::PhantomData, ptr, slice};

use zerocopy::byteorder;

use crate::{
    ByteOrder, ListMut, ListRef, MUTF8Str, MutCompound, MutList, MutString, MutVec, MutableConfig,
    MutableGenericImpl, NBT, RefCompound, RefList, RefString, SIZE_USIZE,
    tag::{
        Byte, ByteArray, Compound, Double, End, Float, Int, IntArray, List, Long, LongArray, Short,
        String, TypedList,
    },
};

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
}

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
}

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
}

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
}

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
}

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
}

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
}

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
}

impl MutableGenericImpl for String {
    #[inline]
    unsafe fn read_mutable_impl<'a, 'doc, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeRef<'doc, MutableConfig<O>>> {
        unsafe {
            let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
            let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
            Some(RefString {
                data: MUTF8Str::from_mutf8_unchecked(slice::from_raw_parts(ptr, len)),
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
}

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
}

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
}

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
}

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
}

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
}
