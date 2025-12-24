use std::{marker::PhantomData, ptr, slice};

use zerocopy::byteorder;

use crate::{
    ByteOrder, MutableConfig, NBT, NBTBase, RefCompound, RefList, RefString, SIZE_USIZE,
    tag::{
        Byte, ByteArray, Compound, Double, End, Float, Int, IntArray, List, Long, LongArray, Short,
        String, TypedList,
    },
};

pub trait MutableGenericNBTImpl: NBTBase {
    /// .
    ///
    /// # Safety
    ///
    /// .
    unsafe fn read_ref<'s, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeRef<'s, MutableConfig<O>>>;

    // unsafe fn read_mut<'s, O: ByteOrder>(data: *mut u8) -> Self::TypeMut<'s, MutableConfig<O>>;
}

pub trait MutableNBTImpl: MutableGenericNBTImpl {}

impl MutableGenericNBTImpl for End {
    unsafe fn read_ref<'s, O: ByteOrder>(
        _data: *const u8,
    ) -> Option<Self::TypeRef<'s, MutableConfig<O>>> {
        Some(())
    }
}

impl MutableNBTImpl for End {}

impl MutableGenericNBTImpl for Byte {
    unsafe fn read_ref<'s, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeRef<'s, MutableConfig<O>>> {
        Some(unsafe { *data.cast() })
    }
}

impl MutableNBTImpl for Byte {}

impl MutableGenericNBTImpl for Short {
    unsafe fn read_ref<'s, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeRef<'s, MutableConfig<O>>> {
        Some(unsafe { byteorder::I16::<O>::from_bytes(*data.cast()).get() })
    }
}

impl MutableNBTImpl for Short {}

impl MutableGenericNBTImpl for Int {
    unsafe fn read_ref<'s, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeRef<'s, MutableConfig<O>>> {
        Some(unsafe { byteorder::I32::<O>::from_bytes(*data.cast()).get() })
    }
}

impl MutableNBTImpl for Int {}

impl MutableGenericNBTImpl for Long {
    unsafe fn read_ref<'s, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeRef<'s, MutableConfig<O>>> {
        Some(unsafe { byteorder::I64::<O>::from_bytes(*data.cast()).get() })
    }
}

impl MutableNBTImpl for Long {}

impl MutableGenericNBTImpl for Float {
    unsafe fn read_ref<'s, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeRef<'s, MutableConfig<O>>> {
        Some(unsafe { byteorder::F32::<O>::from_bytes(*data.cast()).get() })
    }
}

impl MutableNBTImpl for Float {}

impl MutableGenericNBTImpl for Double {
    unsafe fn read_ref<'s, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeRef<'s, MutableConfig<O>>> {
        Some(unsafe { byteorder::F64::<O>::from_bytes(*data.cast()).get() })
    }
}

impl MutableNBTImpl for Double {}

impl MutableGenericNBTImpl for ByteArray {
    unsafe fn read_ref<'s, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeRef<'s, MutableConfig<O>>> {
        unsafe {
            let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
            let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
            Some(slice::from_raw_parts(ptr, len))
        }
    }
}

impl MutableNBTImpl for ByteArray {}

impl MutableGenericNBTImpl for String {
    unsafe fn read_ref<'s, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeRef<'s, MutableConfig<O>>> {
        unsafe {
            let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
            let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
            Some(RefString {
                data: slice::from_raw_parts(ptr, len),
            })
        }
    }
}

impl MutableNBTImpl for String {}

impl MutableGenericNBTImpl for List {
    unsafe fn read_ref<'s, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeRef<'s, MutableConfig<O>>> {
        unsafe {
            let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
            Some(RefList {
                data: ptr,
                _marker: PhantomData,
            })
        }
    }
}

impl MutableNBTImpl for List {}

impl MutableGenericNBTImpl for Compound {
    unsafe fn read_ref<'s, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeRef<'s, MutableConfig<O>>> {
        unsafe {
            let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
            Some(RefCompound {
                data: ptr,
                _marker: PhantomData,
            })
        }
    }
}

impl MutableNBTImpl for Compound {}

impl MutableGenericNBTImpl for IntArray {
    unsafe fn read_ref<'s, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeRef<'s, MutableConfig<O>>> {
        unsafe {
            let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
            let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
            Some(slice::from_raw_parts(ptr, len))
        }
    }
}

impl MutableNBTImpl for IntArray {}

impl MutableGenericNBTImpl for LongArray {
    unsafe fn read_ref<'s, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeRef<'s, MutableConfig<O>>> {
        unsafe {
            let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
            let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
            Some(slice::from_raw_parts(ptr, len))
        }
    }
}

impl MutableNBTImpl for LongArray {}

impl<T: NBT> MutableGenericNBTImpl for TypedList<T> {
    unsafe fn read_ref<'s, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeRef<'s, MutableConfig<O>>> {
        unsafe { List::read_ref(data).and_then(|list| list.typed_::<T>()) }
    }
}
