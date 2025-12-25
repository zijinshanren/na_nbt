use std::{marker::PhantomData, ptr, slice};

use zerocopy::byteorder;

use crate::{
    ByteOrder, MutableConfig, MutableGenericImpl, MutableImpl, NBT, RefCompound, RefList,
    RefString, SIZE_USIZE,
    tag::{
        Byte, ByteArray, Compound, Double, End, Float, Int, IntArray, List, Long, LongArray, Short,
        String, TypedList,
    },
};

impl MutableGenericImpl for End {
    unsafe fn read_mutable_impl<'a, 'doc, O: ByteOrder>(
        _data: *const u8,
    ) -> Option<Self::TypeRef<'doc, MutableConfig<O>>> {
        Some(())
    }
}

impl MutableImpl for End {}

impl MutableGenericImpl for Byte {
    unsafe fn read_mutable_impl<'a, 'doc, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeRef<'doc, MutableConfig<O>>> {
        Some(unsafe { *data.cast() })
    }
}

impl MutableImpl for Byte {}

impl MutableGenericImpl for Short {
    unsafe fn read_mutable_impl<'a, 'doc, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeRef<'doc, MutableConfig<O>>> {
        Some(unsafe { byteorder::I16::<O>::from_bytes(*data.cast()).get() })
    }
}

impl MutableImpl for Short {}

impl MutableGenericImpl for Int {
    unsafe fn read_mutable_impl<'a, 'doc, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeRef<'doc, MutableConfig<O>>> {
        Some(unsafe { byteorder::I32::<O>::from_bytes(*data.cast()).get() })
    }
}

impl MutableImpl for Int {}

impl MutableGenericImpl for Long {
    unsafe fn read_mutable_impl<'a, 'doc, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeRef<'doc, MutableConfig<O>>> {
        Some(unsafe { byteorder::I64::<O>::from_bytes(*data.cast()).get() })
    }
}

impl MutableImpl for Long {}

impl MutableGenericImpl for Float {
    unsafe fn read_mutable_impl<'a, 'doc, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeRef<'doc, MutableConfig<O>>> {
        Some(unsafe { byteorder::F32::<O>::from_bytes(*data.cast()).get() })
    }
}

impl MutableImpl for Float {}

impl MutableGenericImpl for Double {
    unsafe fn read_mutable_impl<'a, 'doc, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeRef<'doc, MutableConfig<O>>> {
        Some(unsafe { byteorder::F64::<O>::from_bytes(*data.cast()).get() })
    }
}

impl MutableImpl for Double {}

impl MutableGenericImpl for ByteArray {
    unsafe fn read_mutable_impl<'a, 'doc, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeRef<'doc, MutableConfig<O>>> {
        unsafe {
            let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
            let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
            Some(slice::from_raw_parts(ptr, len))
        }
    }
}

impl MutableImpl for ByteArray {}

impl MutableGenericImpl for String {
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
}

impl MutableImpl for String {}

impl MutableGenericImpl for List {
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
}

impl MutableImpl for List {}

impl MutableGenericImpl for Compound {
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
}

impl MutableImpl for Compound {}

impl MutableGenericImpl for IntArray {
    unsafe fn read_mutable_impl<'a, 'doc, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeRef<'doc, MutableConfig<O>>> {
        unsafe {
            let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
            let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
            Some(slice::from_raw_parts(ptr, len))
        }
    }
}

impl MutableImpl for IntArray {}

impl MutableGenericImpl for LongArray {
    unsafe fn read_mutable_impl<'a, 'doc, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeRef<'doc, MutableConfig<O>>> {
        unsafe {
            let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
            let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
            Some(slice::from_raw_parts(ptr, len))
        }
    }
}

impl MutableImpl for LongArray {}

impl<T: NBT> MutableGenericImpl for TypedList<T> {
    unsafe fn read_mutable_impl<'a, 'doc, O: ByteOrder>(
        data: *const u8,
    ) -> Option<Self::TypeRef<'doc, MutableConfig<O>>> {
        unsafe { List::read_mutable_impl(data)?.typed_::<T>() }
    }
}
