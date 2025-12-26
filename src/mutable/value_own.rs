use std::{marker::PhantomData, ptr};

use zerocopy::byteorder;

use crate::{
    ByteOrder, NBT, NBTBase, OwnCompound, OwnList, OwnString, OwnTypedList, OwnVec, TagID,
};

pub enum OwnValue<O: ByteOrder> {
    End(()),
    Byte(i8),
    Short(byteorder::I16<O>),
    Int(byteorder::I32<O>),
    Long(byteorder::I64<O>),
    Float(byteorder::F32<O>),
    Double(byteorder::F64<O>),
    ByteArray(OwnVec<i8>),
    String(OwnString),
    List(OwnList<O>),
    Compound(OwnCompound<O>),
    IntArray(OwnVec<byteorder::I32<O>>),
    LongArray(OwnVec<byteorder::I64<O>>),
}

impl<O: ByteOrder> Default for OwnValue<O> {
    #[inline]
    fn default() -> Self {
        Self::End(())
    }
}

// todo: impl Drop

impl<O: ByteOrder> OwnValue<O> {
    pub(crate) unsafe fn write(self, dst: *mut u8) {
        unsafe {
            match self {
                OwnValue::End(value) => ptr::write(dst.cast(), value),
                OwnValue::Byte(value) => ptr::write(dst.cast(), value),
                OwnValue::Short(value) => ptr::write(dst.cast(), value),
                OwnValue::Int(value) => ptr::write(dst.cast(), value),
                OwnValue::Long(value) => ptr::write(dst.cast(), value),
                OwnValue::Float(value) => ptr::write(dst.cast(), value),
                OwnValue::Double(value) => ptr::write(dst.cast(), value),
                OwnValue::ByteArray(value) => ptr::write(dst.cast(), value),
                OwnValue::String(value) => ptr::write(dst.cast(), value),
                OwnValue::List(value) => ptr::write(dst.cast(), value),
                OwnValue::Compound(value) => ptr::write(dst.cast(), value),
                OwnValue::IntArray(value) => ptr::write(dst.cast(), value),
                OwnValue::LongArray(value) => ptr::write(dst.cast(), value),
            };
        }
    }

    #[allow(clippy::unit_arg)]
    pub(crate) unsafe fn read(tag_id: TagID, src: *mut u8) -> Self {
        unsafe {
            match tag_id {
                TagID::End => OwnValue::End(ptr::read(src.cast())),
                TagID::Byte => OwnValue::Byte(ptr::read(src.cast())),
                TagID::Short => OwnValue::Short(ptr::read(src.cast())),
                TagID::Int => OwnValue::Int(ptr::read(src.cast())),
                TagID::Long => OwnValue::Long(ptr::read(src.cast())),
                TagID::Float => OwnValue::Float(ptr::read(src.cast())),
                TagID::Double => OwnValue::Double(ptr::read(src.cast())),
                TagID::ByteArray => OwnValue::ByteArray(ptr::read(src.cast())),
                TagID::String => OwnValue::String(ptr::read(src.cast())),
                TagID::List => OwnValue::List(ptr::read(src.cast())),
                TagID::Compound => OwnValue::Compound(ptr::read(src.cast())),
                TagID::IntArray => OwnValue::IntArray(ptr::read(src.cast())),
                TagID::LongArray => OwnValue::LongArray(ptr::read(src.cast())),
            }
        }
    }
}

impl<O: ByteOrder> From<()> for OwnValue<O> {
    #[inline]
    fn from(value: ()) -> Self {
        OwnValue::End(value)
    }
}

impl<O: ByteOrder> From<i8> for OwnValue<O> {
    #[inline]
    fn from(value: i8) -> Self {
        OwnValue::Byte(value)
    }
}

impl<O: ByteOrder> From<byteorder::I16<O>> for OwnValue<O> {
    #[inline]
    fn from(value: byteorder::I16<O>) -> Self {
        OwnValue::Short(value)
    }
}

impl<O: ByteOrder> From<byteorder::I32<O>> for OwnValue<O> {
    #[inline]
    fn from(value: byteorder::I32<O>) -> Self {
        OwnValue::Int(value)
    }
}

impl<O: ByteOrder> From<byteorder::I64<O>> for OwnValue<O> {
    #[inline]
    fn from(value: byteorder::I64<O>) -> Self {
        OwnValue::Long(value)
    }
}

impl<O: ByteOrder> From<byteorder::F32<O>> for OwnValue<O> {
    #[inline]
    fn from(value: byteorder::F32<O>) -> Self {
        OwnValue::Float(value)
    }
}

impl<O: ByteOrder> From<byteorder::F64<O>> for OwnValue<O> {
    #[inline]
    fn from(value: byteorder::F64<O>) -> Self {
        OwnValue::Double(value)
    }
}

impl<O: ByteOrder> From<OwnVec<i8>> for OwnValue<O> {
    #[inline]
    fn from(value: OwnVec<i8>) -> Self {
        OwnValue::ByteArray(value)
    }
}

impl<O: ByteOrder> From<OwnString> for OwnValue<O> {
    #[inline]
    fn from(value: OwnString) -> Self {
        OwnValue::String(value)
    }
}

impl<O: ByteOrder> From<OwnList<O>> for OwnValue<O> {
    #[inline]
    fn from(value: OwnList<O>) -> Self {
        OwnValue::List(value)
    }
}

impl<O: ByteOrder> From<OwnCompound<O>> for OwnValue<O> {
    #[inline]
    fn from(value: OwnCompound<O>) -> Self {
        OwnValue::Compound(value)
    }
}

impl<O: ByteOrder> From<OwnVec<byteorder::I32<O>>> for OwnValue<O> {
    #[inline]
    fn from(value: OwnVec<byteorder::I32<O>>) -> Self {
        OwnValue::IntArray(value)
    }
}

impl<O: ByteOrder> From<OwnVec<byteorder::I64<O>>> for OwnValue<O> {
    #[inline]
    fn from(value: OwnVec<byteorder::I64<O>>) -> Self {
        OwnValue::LongArray(value)
    }
}

impl<O: ByteOrder, T: NBT> From<OwnTypedList<O, T>> for OwnValue<O> {
    #[inline]
    fn from(value: OwnTypedList<O, T>) -> Self {
        OwnValue::List(OwnList {
            data: value.data,
            _marker: PhantomData,
        })
    }
}
