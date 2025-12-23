use std::{marker::PhantomData, slice};

use zerocopy::byteorder;

use crate::{
    ByteOrder, Document, ImmutableConfig, Mark, NBT, NBTBase, ReadonlyArray, ReadonlyCompound,
    ReadonlyList, ReadonlyString, ReadonlyValue,
    tag::{
        Byte, ByteArray, Compound, Double, End, Float, Int, IntArray, List, Long, LongArray, Short,
        String, TypedList,
    },
};

pub trait ImmutableGenericNBTImpl: NBTBase {
    /// .
    ///
    /// # Safety
    ///
    /// .
    unsafe fn read<'doc, O: ByteOrder, D: Document>(
        data: *const u8,
        mark: *const Mark,
        doc: &D,
    ) -> Self::Type<'doc, ImmutableConfig<O, D>>;

    /// .
    ///
    /// # Safety
    ///
    /// .
    unsafe fn get_index_unchecked<'doc, O: ByteOrder, D: Document>(
        ptr: *const u8,
        index: usize,
        doc: &D,
        mark: *const Mark,
    ) -> Option<Self::Type<'doc, ImmutableConfig<O, D>>>;

    fn _from<'doc, O: ByteOrder, D: Document>(
        value: ReadonlyValue<'doc, O, D>,
    ) -> Option<Self::Type<'doc, ImmutableConfig<O, D>>>;
}

pub trait ImmutableNBTImpl: ImmutableGenericNBTImpl {
    fn ref_<'a, 'doc, O: ByteOrder, D: Document>(
        value: &'a ReadonlyValue<'doc, O, D>,
    ) -> Option<&'a Self::Type<'doc, ImmutableConfig<O, D>>>
    where
        'doc: 'a;

    /// .
    ///
    /// # Safety
    ///
    /// .
    unsafe fn size<O: ByteOrder>(payload: *const u8, mark: *const Mark) -> (usize, usize);
}

macro_rules! immutable_generic_nbt_impl {
    ($name:ident) => {
        #[inline]
        fn _from<'doc, O: ByteOrder, D: Document>(
            value: ReadonlyValue<'doc, O, D>,
        ) -> Option<Self::Type<'doc, ImmutableConfig<O, D>>> {
            match value {
                ReadonlyValue::$name(v) => Some(v),
                _ => None,
            }
        }
    };
}

macro_rules! immutable_nbt_impl {
    ($name:ident) => {
        #[inline]
        fn ref_<'a, 'doc, O: ByteOrder, D: Document>(
            value: &'a ReadonlyValue<'doc, O, D>,
        ) -> Option<&'a Self::Type<'doc, ImmutableConfig<O, D>>>
        where
            'doc: 'a,
        {
            match value {
                ReadonlyValue::$name(v) => Some(v),
                _ => None,
            }
        }
    };
}

impl ImmutableGenericNBTImpl for End {
    #[inline]
    unsafe fn read<'doc, O: ByteOrder, D: Document>(
        _data: *const u8,
        _mark: *const Mark,
        _doc: &D,
    ) -> Self::Type<'doc, ImmutableConfig<O, D>> {
    }

    #[inline]
    unsafe fn get_index_unchecked<'doc, O: ByteOrder, D: Document>(
        _ptr: *const u8,
        _index: usize,
        _doc: &D,
        _mark: *const Mark,
    ) -> Option<Self::Type<'doc, ImmutableConfig<O, D>>> {
        Some(())
    }

    immutable_generic_nbt_impl!(End);
}

impl ImmutableNBTImpl for End {
    #[inline]
    unsafe fn size<O: ByteOrder>(_payload: *const u8, _mark: *const Mark) -> (usize, usize) {
        (0, 0)
    }

    immutable_nbt_impl!(End);
}

impl ImmutableGenericNBTImpl for Byte {
    #[inline]
    unsafe fn read<'doc, O: ByteOrder, D: Document>(
        data: *const u8,
        _mark: *const Mark,
        _doc: &D,
    ) -> Self::Type<'doc, ImmutableConfig<O, D>> {
        unsafe { *data.cast() }
    }

    #[inline]
    unsafe fn get_index_unchecked<'doc, O: ByteOrder, D: Document>(
        ptr: *const u8,
        index: usize,
        _doc: &D,
        _mark: *const Mark,
    ) -> Option<Self::Type<'doc, ImmutableConfig<O, D>>> {
        Some(unsafe { *ptr.add(index).cast() })
    }

    immutable_generic_nbt_impl!(Byte);
}

impl ImmutableNBTImpl for Byte {
    #[inline]
    unsafe fn size<O: ByteOrder>(_payload: *const u8, _mark: *const Mark) -> (usize, usize) {
        (1, 0)
    }

    immutable_nbt_impl!(Byte);
}

impl ImmutableGenericNBTImpl for Short {
    #[inline]
    unsafe fn read<'doc, O: ByteOrder, D: Document>(
        data: *const u8,
        _mark: *const Mark,
        _doc: &D,
    ) -> Self::Type<'doc, ImmutableConfig<O, D>> {
        unsafe { byteorder::I16::<O>::from_bytes(*data.cast()).get() }
    }

    #[inline]
    unsafe fn get_index_unchecked<'doc, O: ByteOrder, D: Document>(
        ptr: *const u8,
        index: usize,
        _doc: &D,
        _mark: *const Mark,
    ) -> Option<Self::Type<'doc, ImmutableConfig<O, D>>> {
        Some(unsafe { byteorder::I16::<O>::from_bytes(*ptr.add(index * 2).cast()).get() })
    }

    immutable_generic_nbt_impl!(Short);
}

impl ImmutableNBTImpl for Short {
    #[inline]
    unsafe fn size<O: ByteOrder>(_payload: *const u8, _mark: *const Mark) -> (usize, usize) {
        (2, 0)
    }

    immutable_nbt_impl!(Short);
}

impl ImmutableGenericNBTImpl for Int {
    #[inline]
    unsafe fn read<'doc, O: ByteOrder, D: Document>(
        data: *const u8,
        _mark: *const Mark,
        _doc: &D,
    ) -> Self::Type<'doc, ImmutableConfig<O, D>> {
        unsafe { byteorder::I32::<O>::from_bytes(*data.cast()).get() }
    }

    #[inline]
    unsafe fn get_index_unchecked<'doc, O: ByteOrder, D: Document>(
        ptr: *const u8,
        index: usize,
        _doc: &D,
        _mark: *const Mark,
    ) -> Option<Self::Type<'doc, ImmutableConfig<O, D>>> {
        Some(unsafe { byteorder::I32::<O>::from_bytes(*ptr.add(index * 4).cast()).get() })
    }

    immutable_generic_nbt_impl!(Int);
}

impl ImmutableNBTImpl for Int {
    #[inline]
    unsafe fn size<O: ByteOrder>(_payload: *const u8, _mark: *const Mark) -> (usize, usize) {
        (4, 0)
    }

    immutable_nbt_impl!(Int);
}

impl ImmutableGenericNBTImpl for Long {
    #[inline]
    unsafe fn read<'doc, O: ByteOrder, D: Document>(
        data: *const u8,
        _mark: *const Mark,
        _doc: &D,
    ) -> Self::Type<'doc, ImmutableConfig<O, D>> {
        unsafe { byteorder::I64::<O>::from_bytes(*data.cast()).get() }
    }

    #[inline]
    unsafe fn get_index_unchecked<'doc, O: ByteOrder, D: Document>(
        ptr: *const u8,
        index: usize,
        _doc: &D,
        _mark: *const Mark,
    ) -> Option<Self::Type<'doc, ImmutableConfig<O, D>>> {
        Some(unsafe { byteorder::I64::<O>::from_bytes(*ptr.add(index * 8).cast()).get() })
    }

    immutable_generic_nbt_impl!(Long);
}

impl ImmutableNBTImpl for Long {
    #[inline]
    unsafe fn size<O: ByteOrder>(_payload: *const u8, _mark: *const Mark) -> (usize, usize) {
        (8, 0)
    }

    immutable_nbt_impl!(Long);
}

impl ImmutableGenericNBTImpl for Float {
    #[inline]
    unsafe fn read<'doc, O: ByteOrder, D: Document>(
        data: *const u8,
        _mark: *const Mark,
        _doc: &D,
    ) -> Self::Type<'doc, ImmutableConfig<O, D>> {
        unsafe { byteorder::F32::<O>::from_bytes(*data.cast()).get() }
    }

    #[inline]
    unsafe fn get_index_unchecked<'doc, O: ByteOrder, D: Document>(
        ptr: *const u8,
        index: usize,
        _doc: &D,
        _mark: *const Mark,
    ) -> Option<Self::Type<'doc, ImmutableConfig<O, D>>> {
        Some(unsafe { byteorder::F32::<O>::from_bytes(*ptr.add(index * 4).cast()).get() })
    }

    immutable_generic_nbt_impl!(Float);
}

impl ImmutableNBTImpl for Float {
    #[inline]
    unsafe fn size<O: ByteOrder>(_payload: *const u8, _mark: *const Mark) -> (usize, usize) {
        (4, 0)
    }

    immutable_nbt_impl!(Float);
}

impl ImmutableGenericNBTImpl for Double {
    #[inline]
    unsafe fn read<'doc, O: ByteOrder, D: Document>(
        data: *const u8,
        _mark: *const Mark,
        _doc: &D,
    ) -> Self::Type<'doc, ImmutableConfig<O, D>> {
        unsafe { byteorder::F64::<O>::from_bytes(*data.cast()).get() }
    }
    #[inline]
    unsafe fn get_index_unchecked<'doc, O: ByteOrder, D: Document>(
        ptr: *const u8,
        index: usize,
        _doc: &D,
        _mark: *const Mark,
    ) -> Option<Self::Type<'doc, ImmutableConfig<O, D>>> {
        Some(unsafe { byteorder::F64::<O>::from_bytes(*ptr.add(index * 8).cast()).get() })
    }

    immutable_generic_nbt_impl!(Double);
}

impl ImmutableNBTImpl for Double {
    #[inline]
    unsafe fn size<O: ByteOrder>(_payload: *const u8, _mark: *const Mark) -> (usize, usize) {
        (8, 0)
    }

    immutable_nbt_impl!(Double);
}

impl ImmutableGenericNBTImpl for ByteArray {
    #[inline]
    unsafe fn read<'doc, O: ByteOrder, D: Document>(
        data: *const u8,
        _mark: *const Mark,
        doc: &D,
    ) -> Self::Type<'doc, ImmutableConfig<O, D>> {
        ReadonlyArray {
            data: unsafe {
                slice::from_raw_parts(
                    data.add(4).cast(),
                    byteorder::U32::<O>::from_bytes(*data.cast()).get() as usize,
                )
            },
            _doc: doc.clone(),
        }
    }

    #[inline]
    unsafe fn get_index_unchecked<'doc, O: ByteOrder, D: Document>(
        ptr: *const u8,
        index: usize,
        doc: &D,
        _mark: *const Mark,
    ) -> Option<Self::Type<'doc, ImmutableConfig<O, D>>> {
        unsafe {
            let mut p = ptr;
            for _ in 0..index {
                let len = byteorder::U32::<O>::from_bytes(*p.cast()).get();
                p = p.add(4 + len as usize);
            }
            let len = byteorder::U32::<O>::from_bytes(*p.cast()).get();
            Some(ReadonlyArray {
                data: slice::from_raw_parts(p.add(4).cast(), len as usize),
                _doc: doc.clone(),
            })
        }
    }

    immutable_generic_nbt_impl!(ByteArray);
}

impl ImmutableNBTImpl for ByteArray {
    #[inline]
    unsafe fn size<O: ByteOrder>(payload: *const u8, _mark: *const Mark) -> (usize, usize) {
        (
            4 + byteorder::U32::<O>::from_bytes(unsafe { *payload.cast() }).get() as usize,
            0,
        )
    }

    immutable_nbt_impl!(ByteArray);
}

impl ImmutableGenericNBTImpl for String {
    #[inline]
    unsafe fn read<'doc, O: ByteOrder, D: Document>(
        data: *const u8,
        _mark: *const Mark,
        doc: &D,
    ) -> Self::Type<'doc, ImmutableConfig<O, D>> {
        ReadonlyString {
            data: unsafe {
                slice::from_raw_parts(
                    data.add(2).cast(),
                    byteorder::U16::<O>::from_bytes(*data.cast()).get() as usize,
                )
            },
            _doc: doc.clone(),
        }
    }

    #[inline]
    unsafe fn get_index_unchecked<'doc, O: ByteOrder, D: Document>(
        ptr: *const u8,
        index: usize,
        doc: &D,
        _mark: *const Mark,
    ) -> Option<Self::Type<'doc, ImmutableConfig<O, D>>> {
        unsafe {
            let mut p = ptr;
            for _ in 0..index {
                let len = byteorder::U16::<O>::from_bytes(*p.cast()).get();
                p = p.add(2 + len as usize);
            }
            let len = byteorder::U16::<O>::from_bytes(*p.cast()).get();
            Some(ReadonlyString {
                data: slice::from_raw_parts(p.add(2), len as usize),
                _doc: doc.clone(),
            })
        }
    }

    immutable_generic_nbt_impl!(String);
}

impl ImmutableNBTImpl for String {
    #[inline]
    unsafe fn size<O: ByteOrder>(payload: *const u8, _mark: *const Mark) -> (usize, usize) {
        (
            2 + byteorder::U16::<O>::from_bytes(unsafe { *payload.cast() }).get() as usize,
            0,
        )
    }

    immutable_nbt_impl!(String);
}

impl ImmutableGenericNBTImpl for List {
    #[inline]
    unsafe fn read<'doc, O: ByteOrder, D: Document>(
        data: *const u8,
        mark: *const Mark,
        doc: &D,
    ) -> Self::Type<'doc, ImmutableConfig<O, D>> {
        unsafe {
            ReadonlyList {
                data: slice::from_raw_parts(
                    data,
                    (*mark).store.end_pointer.byte_offset_from_unsigned(data),
                ),
                mark: mark.add(1),
                doc: doc.clone(),
                _marker: PhantomData,
            }
        }
    }

    #[inline]
    unsafe fn get_index_unchecked<'doc, O: ByteOrder, D: Document>(
        ptr: *const u8,
        index: usize,
        doc: &D,
        mark: *const Mark,
    ) -> Option<Self::Type<'doc, ImmutableConfig<O, D>>> {
        unsafe {
            let mut p = ptr;
            let mut m = mark;
            for _ in 0..index {
                p = (*m).store.end_pointer;
                m = m.add((*m).store.flat_next_mark as usize);
            }
            Some(ReadonlyList {
                data: slice::from_raw_parts(p, (*m).store.end_pointer.byte_offset_from_unsigned(p)),
                mark: m.add(1),
                doc: doc.clone(),
                _marker: PhantomData,
            })
        }
    }

    immutable_generic_nbt_impl!(List);
}

impl ImmutableNBTImpl for List {
    #[inline]
    unsafe fn size<O: ByteOrder>(payload: *const u8, mark: *const Mark) -> (usize, usize) {
        unsafe {
            (
                (*mark).store.end_pointer.byte_offset_from_unsigned(payload),
                (*mark).store.flat_next_mark as usize,
            )
        }
    }

    immutable_nbt_impl!(List);
}

impl ImmutableGenericNBTImpl for Compound {
    #[inline]
    unsafe fn read<'doc, O: ByteOrder, D: Document>(
        data: *const u8,
        mark: *const Mark,
        doc: &D,
    ) -> Self::Type<'doc, ImmutableConfig<O, D>> {
        unsafe {
            ReadonlyCompound {
                data: slice::from_raw_parts(
                    data,
                    (*mark).store.end_pointer.byte_offset_from_unsigned(data),
                ),
                mark: mark.add(1),
                doc: doc.clone(),
                _marker: PhantomData,
            }
        }
    }

    #[inline]
    unsafe fn get_index_unchecked<'doc, O: ByteOrder, D: Document>(
        ptr: *const u8,
        index: usize,
        doc: &D,
        mark: *const Mark,
    ) -> Option<Self::Type<'doc, ImmutableConfig<O, D>>> {
        unsafe {
            let mut p = ptr;
            let mut m = mark;
            for _ in 0..index {
                p = (*m).store.end_pointer;
                m = m.add((*m).store.flat_next_mark as usize);
            }
            Some(ReadonlyCompound {
                data: slice::from_raw_parts(p, (*m).store.end_pointer.byte_offset_from_unsigned(p)),
                mark: m.add(1),
                doc: doc.clone(),
                _marker: PhantomData,
            })
        }
    }

    immutable_generic_nbt_impl!(Compound);
}

impl ImmutableNBTImpl for Compound {
    #[inline]
    unsafe fn size<O: ByteOrder>(payload: *const u8, mark: *const Mark) -> (usize, usize) {
        unsafe {
            (
                (*mark).store.end_pointer.byte_offset_from_unsigned(payload),
                (*mark).store.flat_next_mark as usize,
            )
        }
    }

    immutable_nbt_impl!(Compound);
}

impl ImmutableGenericNBTImpl for IntArray {
    #[inline]
    unsafe fn read<'doc, O: ByteOrder, D: Document>(
        data: *const u8,
        _mark: *const Mark,
        doc: &D,
    ) -> Self::Type<'doc, ImmutableConfig<O, D>> {
        ReadonlyArray {
            data: unsafe {
                slice::from_raw_parts(
                    data.add(4).cast(),
                    byteorder::U32::<O>::from_bytes(*data.cast()).get() as usize,
                )
            },
            _doc: doc.clone(),
        }
    }

    #[inline]
    unsafe fn get_index_unchecked<'doc, O: ByteOrder, D: Document>(
        ptr: *const u8,
        index: usize,
        doc: &D,
        _mark: *const Mark,
    ) -> Option<Self::Type<'doc, ImmutableConfig<O, D>>> {
        unsafe {
            let mut p = ptr;
            for _ in 0..index {
                let len = byteorder::U32::<O>::from_bytes(*p.cast()).get();
                p = p.add(4 + len as usize * 4);
            }
            let len = byteorder::U32::<O>::from_bytes(*p.cast()).get();
            Some(ReadonlyArray {
                data: slice::from_raw_parts(p.add(4).cast(), len as usize),
                _doc: doc.clone(),
            })
        }
    }

    immutable_generic_nbt_impl!(IntArray);
}

impl ImmutableNBTImpl for IntArray {
    #[inline]
    unsafe fn size<O: ByteOrder>(payload: *const u8, _mark: *const Mark) -> (usize, usize) {
        (
            4 + byteorder::U32::<O>::from_bytes(unsafe { *payload.cast() }).get() as usize * 4,
            0,
        )
    }

    immutable_nbt_impl!(IntArray);
}

impl ImmutableGenericNBTImpl for LongArray {
    #[inline]
    unsafe fn read<'doc, O: ByteOrder, D: Document>(
        data: *const u8,
        _mark: *const Mark,
        doc: &D,
    ) -> Self::Type<'doc, ImmutableConfig<O, D>> {
        ReadonlyArray {
            data: unsafe {
                slice::from_raw_parts(
                    data.add(4).cast(),
                    byteorder::U32::<O>::from_bytes(*data.cast()).get() as usize,
                )
            },
            _doc: doc.clone(),
        }
    }

    #[inline]
    unsafe fn get_index_unchecked<'doc, O: ByteOrder, D: Document>(
        ptr: *const u8,
        index: usize,
        doc: &D,
        _mark: *const Mark,
    ) -> Option<Self::Type<'doc, ImmutableConfig<O, D>>> {
        unsafe {
            let mut p = ptr;
            for _ in 0..index {
                let len = byteorder::U32::<O>::from_bytes(*p.cast()).get();
                p = p.add(4 + len as usize * 8);
            }
            let len = byteorder::U32::<O>::from_bytes(*p.cast()).get();
            Some(ReadonlyArray {
                data: slice::from_raw_parts(p.add(4).cast(), len as usize),
                _doc: doc.clone(),
            })
        }
    }

    immutable_generic_nbt_impl!(LongArray);
}

impl ImmutableNBTImpl for LongArray {
    #[inline]
    unsafe fn size<O: ByteOrder>(payload: *const u8, _mark: *const Mark) -> (usize, usize) {
        (
            4 + byteorder::U32::<O>::from_bytes(unsafe { *payload.cast() }).get() as usize * 8,
            0,
        )
    }

    immutable_nbt_impl!(LongArray);
}

impl<T: NBT> ImmutableGenericNBTImpl for TypedList<T> {
    #[inline]
    unsafe fn read<'doc, O: ByteOrder, D: Document>(
        data: *const u8,
        mark: *const Mark,
        doc: &D,
    ) -> Self::Type<'doc, ImmutableConfig<O, D>> {
        unsafe { List::read(data, mark, doc).typed_::<T>().unwrap_unchecked() }
    }

    #[inline]
    unsafe fn get_index_unchecked<'doc, O: ByteOrder, D: Document>(
        ptr: *const u8,
        index: usize,
        doc: &D,
        mark: *const Mark,
    ) -> Option<Self::Type<'doc, ImmutableConfig<O, D>>> {
        unsafe {
            List::get_index_unchecked(ptr, index, doc, mark)
                .unwrap_unchecked()
                .typed_::<T>()
        }
    }

    #[inline]
    fn _from<'doc, O: ByteOrder, D: Document>(
        value: ReadonlyValue<'doc, O, D>,
    ) -> Option<Self::Type<'doc, ImmutableConfig<O, D>>> {
        value.into_::<List>().and_then(|list| list.typed_::<T>())
    }
}
