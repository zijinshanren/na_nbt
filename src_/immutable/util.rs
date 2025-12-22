use std::{marker::PhantomData, slice};

use zerocopy::byteorder;

use crate::{
    NBTBase, ReadonlyArray, ReadonlyCompound, ReadonlyList, ReadonlyString, TagByte, TagByteArray,
    TagCompound, TagDouble, TagEnd, TagFloat, TagID, TagInt, TagIntArray, TagList, TagLong,
    TagLongArray, TagShort, TagString,
    immutable::{mark::Mark, trait_impl::Config, value::Document},
    util::ByteOrder,
};

pub unsafe fn tag_size<O: ByteOrder>(
    tag_id: TagID,
    payload: *const u8,
    mark: *const Mark,
) -> (usize, usize) {
    unsafe {
        match tag_id {
            TagID::End => TagEnd::size::<O>(payload, mark),
            TagID::Byte => TagByte::size::<O>(payload, mark),
            TagID::Short => TagShort::size::<O>(payload, mark),
            TagID::Int => TagInt::size::<O>(payload, mark),
            TagID::Long => TagLong::size::<O>(payload, mark),
            TagID::Float => TagFloat::size::<O>(payload, mark),
            TagID::Double => TagDouble::size::<O>(payload, mark),
            TagID::ByteArray => TagByteArray::size::<O>(payload, mark),
            TagID::String => TagString::size::<O>(payload, mark),
            TagID::List => TagList::size::<O>(payload, mark),
            TagID::Compound => TagCompound::size::<O>(payload, mark),
            TagID::IntArray => TagIntArray::size::<O>(payload, mark),
            TagID::LongArray => TagLongArray::size::<O>(payload, mark),
        }
    }
}

pub trait ImmutableNBTImpl: NBTBase {
    unsafe fn read<'doc, O: ByteOrder, D: Document>(
        data: *const u8,
        mark: *const Mark,
        doc: D,
    ) -> Self::Type<'doc, Config<O, D>>;

    unsafe fn size<O: ByteOrder>(payload: *const u8, mark: *const Mark) -> (usize, usize);
}

impl ImmutableNBTImpl for TagEnd {
    #[inline]
    unsafe fn read<'doc, O: ByteOrder, D: Document>(
        _data: *const u8,
        _mark: *const Mark,
        _doc: D,
    ) -> Self::Type<'doc, Config<O, D>> {
    }

    #[inline]
    unsafe fn size<O: ByteOrder>(_payload: *const u8, _mark: *const Mark) -> (usize, usize) {
        (0, 0)
    }
}

impl ImmutableNBTImpl for TagByte {
    #[inline]
    unsafe fn read<'doc, O: ByteOrder, D: Document>(
        data: *const u8,
        _mark: *const Mark,
        _doc: D,
    ) -> Self::Type<'doc, Config<O, D>> {
        unsafe { *data.cast() }
    }

    #[inline]
    unsafe fn size<O: ByteOrder>(_payload: *const u8, _mark: *const Mark) -> (usize, usize) {
        (1, 0)
    }
}

impl ImmutableNBTImpl for TagShort {
    #[inline]
    unsafe fn read<'doc, O: ByteOrder, D: Document>(
        data: *const u8,
        _mark: *const Mark,
        _doc: D,
    ) -> Self::Type<'doc, Config<O, D>> {
        unsafe { byteorder::I16::<O>::from_bytes(*data.cast()).get() }
    }

    #[inline]
    unsafe fn size<O: ByteOrder>(_payload: *const u8, _mark: *const Mark) -> (usize, usize) {
        (2, 0)
    }
}

impl ImmutableNBTImpl for TagInt {
    #[inline]
    unsafe fn read<'doc, O: ByteOrder, D: Document>(
        data: *const u8,
        _mark: *const Mark,
        _doc: D,
    ) -> Self::Type<'doc, Config<O, D>> {
        unsafe { byteorder::I32::<O>::from_bytes(*data.cast()).get() }
    }

    #[inline]
    unsafe fn size<O: ByteOrder>(_payload: *const u8, _mark: *const Mark) -> (usize, usize) {
        (4, 0)
    }
}

impl ImmutableNBTImpl for TagLong {
    #[inline]
    unsafe fn read<'doc, O: ByteOrder, D: Document>(
        data: *const u8,
        _mark: *const Mark,
        _doc: D,
    ) -> Self::Type<'doc, Config<O, D>> {
        unsafe { byteorder::I64::<O>::from_bytes(*data.cast()).get() }
    }

    #[inline]
    unsafe fn size<O: ByteOrder>(_payload: *const u8, _mark: *const Mark) -> (usize, usize) {
        (8, 0)
    }
}

impl ImmutableNBTImpl for TagFloat {
    #[inline]
    unsafe fn read<'doc, O: ByteOrder, D: Document>(
        data: *const u8,
        _mark: *const Mark,
        _doc: D,
    ) -> Self::Type<'doc, Config<O, D>> {
        unsafe { byteorder::F32::<O>::from_bytes(*data.cast()).get() }
    }

    #[inline]
    unsafe fn size<O: ByteOrder>(_payload: *const u8, _mark: *const Mark) -> (usize, usize) {
        (4, 0)
    }
}

impl ImmutableNBTImpl for TagDouble {
    #[inline]
    unsafe fn read<'doc, O: ByteOrder, D: Document>(
        data: *const u8,
        _mark: *const Mark,
        _doc: D,
    ) -> Self::Type<'doc, Config<O, D>> {
        unsafe { byteorder::F64::<O>::from_bytes(*data.cast()).get() }
    }

    #[inline]
    unsafe fn size<O: ByteOrder>(_payload: *const u8, _mark: *const Mark) -> (usize, usize) {
        (8, 0)
    }
}

impl ImmutableNBTImpl for TagByteArray {
    #[inline]
    unsafe fn read<'doc, O: ByteOrder, D: Document>(
        data: *const u8,
        _mark: *const Mark,
        doc: D,
    ) -> Self::Type<'doc, Config<O, D>> {
        ReadonlyArray {
            data: unsafe {
                slice::from_raw_parts(
                    data.add(4).cast(),
                    byteorder::U32::<O>::from_bytes(*data.cast()).get() as usize,
                )
            },
            _doc: doc,
        }
    }

    #[inline]
    unsafe fn size<O: ByteOrder>(payload: *const u8, _mark: *const Mark) -> (usize, usize) {
        (
            4 + byteorder::U32::<O>::from_bytes(unsafe { *payload.cast() }).get() as usize,
            0,
        )
    }
}

impl ImmutableNBTImpl for TagString {
    #[inline]
    unsafe fn read<'doc, O: ByteOrder, D: Document>(
        data: *const u8,
        _mark: *const Mark,
        doc: D,
    ) -> Self::Type<'doc, Config<O, D>> {
        ReadonlyString {
            data: unsafe {
                slice::from_raw_parts(
                    data.add(2).cast(),
                    byteorder::U16::<O>::from_bytes(*data.cast()).get() as usize,
                )
            },
            _doc: doc,
        }
    }

    #[inline]
    unsafe fn size<O: ByteOrder>(payload: *const u8, _mark: *const Mark) -> (usize, usize) {
        (
            2 + byteorder::U16::<O>::from_bytes(unsafe { *payload.cast() }).get() as usize,
            0,
        )
    }
}

impl ImmutableNBTImpl for TagList {
    #[inline]
    unsafe fn read<'doc, O: ByteOrder, D: Document>(
        data: *const u8,
        mark: *const Mark,
        doc: D,
    ) -> Self::Type<'doc, Config<O, D>> {
        unsafe {
            ReadonlyList {
                data: slice::from_raw_parts(
                    data,
                    (*mark).store.end_pointer.byte_offset_from_unsigned(data),
                ),
                mark: mark.add(1),
                doc,
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
}

impl ImmutableNBTImpl for TagCompound {
    #[inline]
    unsafe fn read<'doc, O: ByteOrder, D: Document>(
        data: *const u8,
        mark: *const Mark,
        doc: D,
    ) -> Self::Type<'doc, Config<O, D>> {
        unsafe {
            ReadonlyCompound {
                data: slice::from_raw_parts(
                    data,
                    (*mark).store.end_pointer.byte_offset_from_unsigned(data),
                ),
                mark: mark.add(1),
                doc,
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
}

impl ImmutableNBTImpl for TagIntArray {
    #[inline]
    unsafe fn read<'doc, O: ByteOrder, D: Document>(
        data: *const u8,
        _mark: *const Mark,
        doc: D,
    ) -> Self::Type<'doc, Config<O, D>> {
        ReadonlyArray {
            data: unsafe {
                slice::from_raw_parts(
                    data.add(4).cast(),
                    byteorder::U32::<O>::from_bytes(*data.cast()).get() as usize,
                )
            },
            _doc: doc,
        }
    }

    #[inline]
    unsafe fn size<O: ByteOrder>(payload: *const u8, _mark: *const Mark) -> (usize, usize) {
        (
            4 + byteorder::U32::<O>::from_bytes(unsafe { *payload.cast() }).get() as usize * 4,
            0,
        )
    }
}

impl ImmutableNBTImpl for TagLongArray {
    #[inline]
    unsafe fn read<'doc, O: ByteOrder, D: Document>(
        data: *const u8,
        _mark: *const Mark,
        doc: D,
    ) -> Self::Type<'doc, Config<O, D>> {
        ReadonlyArray {
            data: unsafe {
                slice::from_raw_parts(
                    data.add(4).cast(),
                    byteorder::U32::<O>::from_bytes(*data.cast()).get() as usize,
                )
            },
            _doc: doc,
        }
    }

    #[inline]
    unsafe fn size<O: ByteOrder>(payload: *const u8, _mark: *const Mark) -> (usize, usize) {
        (
            4 + byteorder::U32::<O>::from_bytes(unsafe { *payload.cast() }).get() as usize * 8,
            0,
        )
    }
}
