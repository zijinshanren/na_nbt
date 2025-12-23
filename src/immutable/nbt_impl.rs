use std::{marker::PhantomData, slice};

use zerocopy::byteorder;

use crate::{
    ByteOrder, Document, ImmutableConfig, Mark, NBTBase, ReadonlyArray, ReadonlyCompound,
    ReadonlyList, ReadonlyString, ReadonlyValue, TagByte, TagByteArray, TagCompound, TagDouble,
    TagEnd, TagFloat, TagInt, TagIntArray, TagList, TagLong, TagLongArray, TagShort, TagString,
};

pub trait ImmutableNBTImpl: NBTBase {
    fn extract<'doc, O: ByteOrder, D: Document>(
        value: ReadonlyValue<'doc, O, D>,
    ) -> Option<Self::Type<'doc, ImmutableConfig<O, D>>>;

    fn peek<'a, 'doc, O: ByteOrder, D: Document>(
        value: &'a ReadonlyValue<'doc, O, D>,
    ) -> Option<&'a Self::Type<'doc, ImmutableConfig<O, D>>>
    where
        'doc: 'a;

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
    unsafe fn size<O: ByteOrder>(payload: *const u8, mark: *const Mark) -> (usize, usize);

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
    ) -> Self::Type<'doc, ImmutableConfig<O, D>>;
}

macro_rules! immutable_nbt_impl {
    ($name:ident, $value:ident) => {
        #[inline]
        fn extract<'doc, O: ByteOrder, D: Document>(
            value: ReadonlyValue<'doc, O, D>,
        ) -> Option<Self::Type<'doc, ImmutableConfig<O, D>>> {
            match value {
                ReadonlyValue::$value(v) => Some(v),
                _ => None,
            }
        }

        #[inline]
        fn peek<'a, 'doc, O: ByteOrder, D: Document>(
            value: &'a ReadonlyValue<'doc, O, D>,
        ) -> Option<&'a Self::Type<'doc, ImmutableConfig<O, D>>>
        where
            'doc: 'a,
        {
            match value {
                ReadonlyValue::$value(v) => Some(v),
                _ => None,
            }
        }
    };
}

impl ImmutableNBTImpl for TagEnd {
    #[inline]
    unsafe fn read<'doc, O: ByteOrder, D: Document>(
        _data: *const u8,
        _mark: *const Mark,
        _doc: &D,
    ) -> Self::Type<'doc, ImmutableConfig<O, D>> {
    }

    #[inline]
    unsafe fn size<O: ByteOrder>(_payload: *const u8, _mark: *const Mark) -> (usize, usize) {
        (0, 0)
    }

    #[inline]
    unsafe fn get_index_unchecked<'doc, O: ByteOrder, D: Document>(
        _ptr: *const u8,
        _index: usize,
        _doc: &D,
        _mark: *const Mark,
    ) -> Self::Type<'doc, ImmutableConfig<O, D>> {
    }

    immutable_nbt_impl!(TagEnd, End);
}

impl ImmutableNBTImpl for TagByte {
    #[inline]
    unsafe fn read<'doc, O: ByteOrder, D: Document>(
        data: *const u8,
        _mark: *const Mark,
        _doc: &D,
    ) -> Self::Type<'doc, ImmutableConfig<O, D>> {
        unsafe { *data.cast() }
    }

    #[inline]
    unsafe fn size<O: ByteOrder>(_payload: *const u8, _mark: *const Mark) -> (usize, usize) {
        (1, 0)
    }

    #[inline]
    unsafe fn get_index_unchecked<'doc, O: ByteOrder, D: Document>(
        ptr: *const u8,
        index: usize,
        _doc: &D,
        _mark: *const Mark,
    ) -> Self::Type<'doc, ImmutableConfig<O, D>> {
        unsafe { *ptr.add(index).cast() }
    }

    immutable_nbt_impl!(TagByte, Byte);
}

impl ImmutableNBTImpl for TagShort {
    #[inline]
    unsafe fn read<'doc, O: ByteOrder, D: Document>(
        data: *const u8,
        _mark: *const Mark,
        _doc: &D,
    ) -> Self::Type<'doc, ImmutableConfig<O, D>> {
        unsafe { byteorder::I16::<O>::from_bytes(*data.cast()).get() }
    }

    #[inline]
    unsafe fn size<O: ByteOrder>(_payload: *const u8, _mark: *const Mark) -> (usize, usize) {
        (2, 0)
    }

    #[inline]
    unsafe fn get_index_unchecked<'doc, O: ByteOrder, D: Document>(
        ptr: *const u8,
        index: usize,
        _doc: &D,
        _mark: *const Mark,
    ) -> Self::Type<'doc, ImmutableConfig<O, D>> {
        unsafe { byteorder::I16::<O>::from_bytes(*ptr.add(index * 2).cast()).get() }
    }

    immutable_nbt_impl!(TagShort, Short);
}

impl ImmutableNBTImpl for TagInt {
    #[inline]
    unsafe fn read<'doc, O: ByteOrder, D: Document>(
        data: *const u8,
        _mark: *const Mark,
        _doc: &D,
    ) -> Self::Type<'doc, ImmutableConfig<O, D>> {
        unsafe { byteorder::I32::<O>::from_bytes(*data.cast()).get() }
    }

    #[inline]
    unsafe fn size<O: ByteOrder>(_payload: *const u8, _mark: *const Mark) -> (usize, usize) {
        (4, 0)
    }

    #[inline]
    unsafe fn get_index_unchecked<'doc, O: ByteOrder, D: Document>(
        ptr: *const u8,
        index: usize,
        _doc: &D,
        _mark: *const Mark,
    ) -> Self::Type<'doc, ImmutableConfig<O, D>> {
        unsafe { byteorder::I32::<O>::from_bytes(*ptr.add(index * 4).cast()).get() }
    }

    immutable_nbt_impl!(TagInt, Int);
}

impl ImmutableNBTImpl for TagLong {
    #[inline]
    unsafe fn read<'doc, O: ByteOrder, D: Document>(
        data: *const u8,
        _mark: *const Mark,
        _doc: &D,
    ) -> Self::Type<'doc, ImmutableConfig<O, D>> {
        unsafe { byteorder::I64::<O>::from_bytes(*data.cast()).get() }
    }

    #[inline]
    unsafe fn size<O: ByteOrder>(_payload: *const u8, _mark: *const Mark) -> (usize, usize) {
        (8, 0)
    }

    #[inline]
    unsafe fn get_index_unchecked<'doc, O: ByteOrder, D: Document>(
        ptr: *const u8,
        index: usize,
        _doc: &D,
        _mark: *const Mark,
    ) -> Self::Type<'doc, ImmutableConfig<O, D>> {
        unsafe { byteorder::I64::<O>::from_bytes(*ptr.add(index * 8).cast()).get() }
    }

    immutable_nbt_impl!(TagLong, Long);
}

impl ImmutableNBTImpl for TagFloat {
    #[inline]
    unsafe fn read<'doc, O: ByteOrder, D: Document>(
        data: *const u8,
        _mark: *const Mark,
        _doc: &D,
    ) -> Self::Type<'doc, ImmutableConfig<O, D>> {
        unsafe { byteorder::F32::<O>::from_bytes(*data.cast()).get() }
    }

    #[inline]
    unsafe fn size<O: ByteOrder>(_payload: *const u8, _mark: *const Mark) -> (usize, usize) {
        (4, 0)
    }

    #[inline]
    unsafe fn get_index_unchecked<'doc, O: ByteOrder, D: Document>(
        ptr: *const u8,
        index: usize,
        _doc: &D,
        _mark: *const Mark,
    ) -> Self::Type<'doc, ImmutableConfig<O, D>> {
        unsafe { byteorder::F32::<O>::from_bytes(*ptr.add(index * 4).cast()).get() }
    }

    immutable_nbt_impl!(TagFloat, Float);
}

impl ImmutableNBTImpl for TagDouble {
    #[inline]
    unsafe fn read<'doc, O: ByteOrder, D: Document>(
        data: *const u8,
        _mark: *const Mark,
        _doc: &D,
    ) -> Self::Type<'doc, ImmutableConfig<O, D>> {
        unsafe { byteorder::F64::<O>::from_bytes(*data.cast()).get() }
    }

    #[inline]
    unsafe fn size<O: ByteOrder>(_payload: *const u8, _mark: *const Mark) -> (usize, usize) {
        (8, 0)
    }

    #[inline]
    unsafe fn get_index_unchecked<'doc, O: ByteOrder, D: Document>(
        ptr: *const u8,
        index: usize,
        _doc: &D,
        _mark: *const Mark,
    ) -> Self::Type<'doc, ImmutableConfig<O, D>> {
        unsafe { byteorder::F64::<O>::from_bytes(*ptr.add(index * 8).cast()).get() }
    }

    immutable_nbt_impl!(TagDouble, Double);
}

impl ImmutableNBTImpl for TagByteArray {
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
    unsafe fn size<O: ByteOrder>(payload: *const u8, _mark: *const Mark) -> (usize, usize) {
        (
            4 + byteorder::U32::<O>::from_bytes(unsafe { *payload.cast() }).get() as usize,
            0,
        )
    }

    #[inline]
    unsafe fn get_index_unchecked<'doc, O: ByteOrder, D: Document>(
        ptr: *const u8,
        index: usize,
        doc: &D,
        _mark: *const Mark,
    ) -> Self::Type<'doc, ImmutableConfig<O, D>> {
        unsafe {
            let mut p = ptr;
            for _ in 0..index {
                let len = byteorder::U32::<O>::from_bytes(*p.cast()).get();
                p = p.add(4 + len as usize);
            }
            let len = byteorder::U32::<O>::from_bytes(*p.cast()).get();
            ReadonlyArray {
                data: slice::from_raw_parts(p.add(4).cast(), len as usize),
                _doc: doc.clone(),
            }
        }
    }

    immutable_nbt_impl!(TagByteArray, ByteArray);
}

impl ImmutableNBTImpl for TagString {
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
    unsafe fn size<O: ByteOrder>(payload: *const u8, _mark: *const Mark) -> (usize, usize) {
        (
            2 + byteorder::U16::<O>::from_bytes(unsafe { *payload.cast() }).get() as usize,
            0,
        )
    }

    #[inline]
    unsafe fn get_index_unchecked<'doc, O: ByteOrder, D: Document>(
        ptr: *const u8,
        index: usize,
        doc: &D,
        _mark: *const Mark,
    ) -> Self::Type<'doc, ImmutableConfig<O, D>> {
        unsafe {
            let mut p = ptr;
            for _ in 0..index {
                let len = byteorder::U16::<O>::from_bytes(*p.cast()).get();
                p = p.add(2 + len as usize);
            }
            let len = byteorder::U16::<O>::from_bytes(*p.cast()).get();
            ReadonlyString {
                data: slice::from_raw_parts(p.add(2), len as usize),
                _doc: doc.clone(),
            }
        }
    }

    immutable_nbt_impl!(TagString, String);
}

impl ImmutableNBTImpl for TagList {
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
    unsafe fn size<O: ByteOrder>(payload: *const u8, mark: *const Mark) -> (usize, usize) {
        unsafe {
            (
                (*mark).store.end_pointer.byte_offset_from_unsigned(payload),
                (*mark).store.flat_next_mark as usize,
            )
        }
    }

    #[inline]
    unsafe fn get_index_unchecked<'doc, O: ByteOrder, D: Document>(
        ptr: *const u8,
        index: usize,
        doc: &D,
        mark: *const Mark,
    ) -> Self::Type<'doc, ImmutableConfig<O, D>> {
        unsafe {
            let mut p = ptr;
            let mut m = mark;
            for _ in 0..index {
                p = (*m).store.end_pointer;
                m = m.add((*m).store.flat_next_mark as usize);
            }
            ReadonlyList {
                data: slice::from_raw_parts(p, (*m).store.end_pointer.byte_offset_from_unsigned(p)),
                mark: m.add(1),
                doc: doc.clone(),
                _marker: PhantomData,
            }
        }
    }

    immutable_nbt_impl!(TagList, List);
}

impl ImmutableNBTImpl for TagCompound {
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
    unsafe fn size<O: ByteOrder>(payload: *const u8, mark: *const Mark) -> (usize, usize) {
        unsafe {
            (
                (*mark).store.end_pointer.byte_offset_from_unsigned(payload),
                (*mark).store.flat_next_mark as usize,
            )
        }
    }

    #[inline]
    unsafe fn get_index_unchecked<'doc, O: ByteOrder, D: Document>(
        ptr: *const u8,
        index: usize,
        doc: &D,
        mark: *const Mark,
    ) -> Self::Type<'doc, ImmutableConfig<O, D>> {
        unsafe {
            let mut p = ptr;
            let mut m = mark;
            for _ in 0..index {
                p = (*m).store.end_pointer;
                m = m.add((*m).store.flat_next_mark as usize);
            }
            ReadonlyCompound {
                data: slice::from_raw_parts(p, (*m).store.end_pointer.byte_offset_from_unsigned(p)),
                mark: m.add(1),
                doc: doc.clone(),
                _marker: PhantomData,
            }
        }
    }

    immutable_nbt_impl!(TagCompound, Compound);
}

impl ImmutableNBTImpl for TagIntArray {
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
    unsafe fn size<O: ByteOrder>(payload: *const u8, _mark: *const Mark) -> (usize, usize) {
        (
            4 + byteorder::U32::<O>::from_bytes(unsafe { *payload.cast() }).get() as usize * 4,
            0,
        )
    }

    #[inline]
    unsafe fn get_index_unchecked<'doc, O: ByteOrder, D: Document>(
        ptr: *const u8,
        index: usize,
        doc: &D,
        _mark: *const Mark,
    ) -> Self::Type<'doc, ImmutableConfig<O, D>> {
        unsafe {
            let mut p = ptr;
            for _ in 0..index {
                let len = byteorder::U32::<O>::from_bytes(*p.cast()).get();
                p = p.add(4 + len as usize * 4);
            }
            let len = byteorder::U32::<O>::from_bytes(*p.cast()).get();
            ReadonlyArray {
                data: slice::from_raw_parts(p.add(4).cast(), len as usize),
                _doc: doc.clone(),
            }
        }
    }

    immutable_nbt_impl!(TagIntArray, IntArray);
}

impl ImmutableNBTImpl for TagLongArray {
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
    unsafe fn size<O: ByteOrder>(payload: *const u8, _mark: *const Mark) -> (usize, usize) {
        (
            4 + byteorder::U32::<O>::from_bytes(unsafe { *payload.cast() }).get() as usize * 8,
            0,
        )
    }

    #[inline]
    unsafe fn get_index_unchecked<'doc, O: ByteOrder, D: Document>(
        ptr: *const u8,
        index: usize,
        doc: &D,
        _mark: *const Mark,
    ) -> Self::Type<'doc, ImmutableConfig<O, D>> {
        unsafe {
            let mut p = ptr;
            for _ in 0..index {
                let len = byteorder::U32::<O>::from_bytes(*p.cast()).get();
                p = p.add(4 + len as usize * 8);
            }
            let len = byteorder::U32::<O>::from_bytes(*p.cast()).get();
            ReadonlyArray {
                data: slice::from_raw_parts(p.add(4).cast(), len as usize),
                _doc: doc.clone(),
            }
        }
    }

    immutable_nbt_impl!(TagLongArray, LongArray);
}
