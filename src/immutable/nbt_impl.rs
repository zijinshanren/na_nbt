use std::{marker::PhantomData, slice};

use zerocopy::byteorder;

use crate::{
    ByteOrder, ConfigRef, Document, ImmutableConfig, ImmutableGenericImpl, MUTF8Str, NBT,
    ReadonlyArray, ReadonlyCompound, ReadonlyList, ReadonlyString,
    tag::{
        Byte, ByteArray, Compound, Double, End, Float, Int, IntArray, List, Long, LongArray, Short,
        String, TypedList,
    },
};

impl ImmutableGenericImpl for End {
    #[inline]
    unsafe fn read_immutable_impl<'a, 'doc, O: ByteOrder, D: Document>(
        _params: <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a>,
    ) -> Option<Self::TypeRef<'doc, ImmutableConfig<O, D>>> {
        Some(())
    }

    #[inline]
    unsafe fn list_get_immutable_impl<'a, O: ByteOrder, D: Document>(
        params: <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a>,
        _index: usize,
    ) -> <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a> {
        params
    }
}

impl ImmutableGenericImpl for Byte {
    #[inline]
    unsafe fn read_immutable_impl<'a, 'doc, O: ByteOrder, D: Document>(
        params: <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a>,
    ) -> Option<Self::TypeRef<'doc, ImmutableConfig<O, D>>> {
        Some(unsafe { *params.0.cast() })
    }

    #[inline]
    unsafe fn list_get_immutable_impl<'a, O: ByteOrder, D: Document>(
        params: <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a>,
        index: usize,
    ) -> <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a> {
        (unsafe { params.0.add(index) }, params.1, params.2)
    }
}

impl ImmutableGenericImpl for Short {
    #[inline]
    unsafe fn read_immutable_impl<'a, 'doc, O: ByteOrder, D: Document>(
        params: <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a>,
    ) -> Option<Self::TypeRef<'doc, ImmutableConfig<O, D>>> {
        Some(unsafe { byteorder::I16::<O>::from_bytes(*params.0.cast()).get() })
    }

    #[inline]
    unsafe fn list_get_immutable_impl<'a, O: ByteOrder, D: Document>(
        params: <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a>,
        index: usize,
    ) -> <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a> {
        (unsafe { params.0.add(index * 2) }, params.1, params.2)
    }
}

impl ImmutableGenericImpl for Int {
    #[inline]
    unsafe fn read_immutable_impl<'a, 'doc, O: ByteOrder, D: Document>(
        params: <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a>,
    ) -> Option<Self::TypeRef<'doc, ImmutableConfig<O, D>>> {
        Some(unsafe { byteorder::I32::<O>::from_bytes(*params.0.cast()).get() })
    }

    #[inline]
    unsafe fn list_get_immutable_impl<'a, O: ByteOrder, D: Document>(
        params: <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a>,
        index: usize,
    ) -> <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a> {
        (unsafe { params.0.add(index * 4) }, params.1, params.2)
    }
}

impl ImmutableGenericImpl for Long {
    #[inline]
    unsafe fn read_immutable_impl<'a, 'doc, O: ByteOrder, D: Document>(
        params: <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a>,
    ) -> Option<Self::TypeRef<'doc, ImmutableConfig<O, D>>> {
        Some(unsafe { byteorder::I64::<O>::from_bytes(*params.0.cast()).get() })
    }

    #[inline]
    unsafe fn list_get_immutable_impl<'a, O: ByteOrder, D: Document>(
        params: <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a>,
        index: usize,
    ) -> <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a> {
        (unsafe { params.0.add(index * 8) }, params.1, params.2)
    }
}

impl ImmutableGenericImpl for Float {
    #[inline]
    unsafe fn read_immutable_impl<'a, 'doc, O: ByteOrder, D: Document>(
        params: <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a>,
    ) -> Option<Self::TypeRef<'doc, ImmutableConfig<O, D>>> {
        Some(unsafe { byteorder::F32::<O>::from_bytes(*params.0.cast()).get() })
    }

    #[inline]
    unsafe fn list_get_immutable_impl<'a, O: ByteOrder, D: Document>(
        params: <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a>,
        index: usize,
    ) -> <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a> {
        (unsafe { params.0.add(index * 4) }, params.1, params.2)
    }
}

impl ImmutableGenericImpl for Double {
    #[inline]
    unsafe fn read_immutable_impl<'a, 'doc, O: ByteOrder, D: Document>(
        params: <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a>,
    ) -> Option<Self::TypeRef<'doc, ImmutableConfig<O, D>>> {
        Some(unsafe { byteorder::F64::<O>::from_bytes(*params.0.cast()).get() })
    }

    #[inline]
    unsafe fn list_get_immutable_impl<'a, O: ByteOrder, D: Document>(
        params: <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a>,
        index: usize,
    ) -> <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a> {
        (unsafe { params.0.add(index * 8) }, params.1, params.2)
    }
}

impl ImmutableGenericImpl for ByteArray {
    #[inline]
    unsafe fn read_immutable_impl<'a, 'doc, O: ByteOrder, D: Document>(
        params: <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a>,
    ) -> Option<Self::TypeRef<'doc, ImmutableConfig<O, D>>> {
        Some(ReadonlyArray {
            data: unsafe {
                slice::from_raw_parts(
                    params.0.add(4).cast(),
                    byteorder::U32::<O>::from_bytes(*params.0.cast()).get() as usize,
                )
            },
            _doc: params.2.clone(),
        })
    }

    #[inline]
    unsafe fn list_get_immutable_impl<'a, O: ByteOrder, D: Document>(
        params: <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a>,
        index: usize,
    ) -> <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a> {
        unsafe {
            let mut p = params.0;
            for _ in 0..index {
                let len = byteorder::U32::<O>::from_bytes(*p.cast()).get();
                p = p.add(4 + len as usize);
            }
            (p, params.1, params.2)
        }
    }
}

impl ImmutableGenericImpl for String {
    #[inline]
    unsafe fn read_immutable_impl<'a, 'doc, O: ByteOrder, D: Document>(
        params: <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a>,
    ) -> Option<Self::TypeRef<'doc, ImmutableConfig<O, D>>> {
        Some(ReadonlyString {
            data: unsafe {
                MUTF8Str::from_mutf8_unchecked(slice::from_raw_parts(
                    params.0.add(2).cast(),
                    byteorder::U16::<O>::from_bytes(*params.0.cast()).get() as usize,
                ))
            },
            _doc: params.2.clone(),
        })
    }
    #[inline]
    unsafe fn list_get_immutable_impl<'a, O: ByteOrder, D: Document>(
        params: <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a>,
        index: usize,
    ) -> <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a> {
        unsafe {
            let mut p = params.0;
            for _ in 0..index {
                let len = byteorder::U16::<O>::from_bytes(*p.cast()).get();
                p = p.add(2 + len as usize);
            }
            (p, params.1, params.2)
        }
    }
}

impl ImmutableGenericImpl for List {
    #[inline]
    unsafe fn read_immutable_impl<'a, 'doc, O: ByteOrder, D: Document>(
        params: <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a>,
    ) -> Option<Self::TypeRef<'doc, ImmutableConfig<O, D>>> {
        Some(ReadonlyList {
            data: unsafe {
                slice::from_raw_parts(
                    params.0,
                    (*params.1)
                        .store
                        .end_pointer
                        .byte_offset_from_unsigned(params.0),
                )
            },
            mark: unsafe { params.1.add(1) },
            doc: params.2.clone(),
            _marker: PhantomData,
        })
    }

    #[inline]
    unsafe fn list_get_immutable_impl<'a, O: ByteOrder, D: Document>(
        params: <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a>,
        index: usize,
    ) -> <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a> {
        unsafe {
            let mut p = params.0;
            let mut m = params.1;
            for _ in 0..index {
                p = (*m).store.end_pointer;
                m = m.add((*m).store.flat_next_mark as usize);
            }
            (p, m, params.2)
        }
    }
}

impl ImmutableGenericImpl for Compound {
    #[inline]
    unsafe fn read_immutable_impl<'a, 'doc, O: ByteOrder, D: Document>(
        params: <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a>,
    ) -> Option<Self::TypeRef<'doc, ImmutableConfig<O, D>>> {
        Some(ReadonlyCompound {
            data: unsafe {
                slice::from_raw_parts(
                    params.0,
                    (*params.1)
                        .store
                        .end_pointer
                        .byte_offset_from_unsigned(params.0),
                )
            },
            mark: unsafe { params.1.add(1) },
            doc: params.2.clone(),
            _marker: PhantomData,
        })
    }

    #[inline]
    unsafe fn list_get_immutable_impl<'a, O: ByteOrder, D: Document>(
        params: <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a>,
        index: usize,
    ) -> <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a> {
        unsafe {
            let mut p = params.0;
            let mut m = params.1;
            for _ in 0..index {
                p = (*m).store.end_pointer;
                m = m.add((*m).store.flat_next_mark as usize);
            }
            (p, m, params.2)
        }
    }
}

impl ImmutableGenericImpl for IntArray {
    #[inline]
    unsafe fn read_immutable_impl<'a, 'doc, O: ByteOrder, D: Document>(
        params: <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a>,
    ) -> Option<Self::TypeRef<'doc, ImmutableConfig<O, D>>> {
        Some(ReadonlyArray {
            data: unsafe {
                slice::from_raw_parts(
                    params.0.add(4).cast(),
                    byteorder::U32::<O>::from_bytes(*params.0.cast()).get() as usize,
                )
            },
            _doc: params.2.clone(),
        })
    }

    #[inline]
    unsafe fn list_get_immutable_impl<'a, O: ByteOrder, D: Document>(
        params: <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a>,
        index: usize,
    ) -> <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a> {
        unsafe {
            let mut p = params.0;
            for _ in 0..index {
                let len = byteorder::U32::<O>::from_bytes(*p.cast()).get();
                p = p.add(4 + len as usize);
            }
            (p, params.1, params.2)
        }
    }
}

impl ImmutableGenericImpl for LongArray {
    #[inline]
    unsafe fn read_immutable_impl<'a, 'doc, O: ByteOrder, D: Document>(
        params: <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a>,
    ) -> Option<Self::TypeRef<'doc, ImmutableConfig<O, D>>> {
        Some(ReadonlyArray {
            data: unsafe {
                slice::from_raw_parts(
                    params.0.add(4).cast(),
                    byteorder::U32::<O>::from_bytes(*params.0.cast()).get() as usize,
                )
            },
            _doc: params.2.clone(),
        })
    }

    #[inline]
    unsafe fn list_get_immutable_impl<'a, O: ByteOrder, D: Document>(
        params: <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a>,
        index: usize,
    ) -> <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a> {
        unsafe {
            let mut p = params.0;
            for _ in 0..index {
                let len = byteorder::U32::<O>::from_bytes(*p.cast()).get();
                p = p.add(4 + len as usize);
            }
            (p, params.1, params.2)
        }
    }
}

impl<T: NBT> ImmutableGenericImpl for TypedList<T> {
    #[inline]
    unsafe fn read_immutable_impl<'a, 'doc, O: ByteOrder, D: Document>(
        params: <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a>,
    ) -> Option<Self::TypeRef<'doc, ImmutableConfig<O, D>>> {
        (unsafe { List::read_immutable_impl(params) })?.typed_::<T>()
    }

    #[inline]
    unsafe fn list_get_immutable_impl<'a, O: ByteOrder, D: Document>(
        params: <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a>,
        index: usize,
    ) -> <ImmutableConfig<O, D> as ConfigRef>::ReadParams<'a> {
        unsafe { List::list_get_immutable_impl::<O, D>(params, index) }
    }
}
