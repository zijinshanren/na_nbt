use std::ptr;

use zerocopy::byteorder;

use crate::{ByteOrder, NBT, OwnCompound, OwnList, OwnString, OwnVec, TagID};

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

// impl<O: ByteOrder> OwnedValue<O> {
//     unsafe fn write(self, dst: *mut u8) {
//         unsafe {
//             match self {
//                 OwnedValue::End(value) => ptr::write(dst.cast(), value),
//                 OwnedValue::Byte(value) => ptr::write(dst.cast(), value),
//                 OwnedValue::Short(value) => ptr::write(dst.cast(), value),
//                 OwnedValue::Int(value) => ptr::write(dst.cast(), value),
//                 OwnedValue::Long(value) => ptr::write(dst.cast(), value),
//                 OwnedValue::Float(value) => ptr::write(dst.cast(), value),
//                 OwnedValue::Double(value) => ptr::write(dst.cast(), value),
//                 OwnedValue::ByteArray(value) => ptr::write(dst.cast(), value),
//                 OwnedValue::String(value) => ptr::write(dst.cast(), value),
//                 OwnedValue::List(value) => ptr::write(dst.cast(), value),
//                 OwnedValue::Compound(value) => ptr::write(dst.cast(), value),
//                 OwnedValue::IntArray(value) => ptr::write(dst.cast(), value),
//                 OwnedValue::LongArray(value) => ptr::write(dst.cast(), value),
//             };
//         }
//     }

//     #[allow(clippy::unit_arg)]
//     unsafe fn read(tag_id: TagID, src: *mut u8) -> Self {
//         unsafe {
//             match tag_id {
//                 TagID::End => OwnedValue::End(ptr::read(src.cast())),
//                 TagID::Byte => OwnedValue::Byte(ptr::read(src.cast())),
//                 TagID::Short => OwnedValue::Short(ptr::read(src.cast())),
//                 TagID::Int => OwnedValue::Int(ptr::read(src.cast())),
//                 TagID::Long => OwnedValue::Long(ptr::read(src.cast())),
//                 TagID::Float => OwnedValue::Float(ptr::read(src.cast())),
//                 TagID::Double => OwnedValue::Double(ptr::read(src.cast())),
//                 TagID::ByteArray => OwnedValue::ByteArray(ptr::read(src.cast())),
//                 TagID::String => OwnedValue::String(ptr::read(src.cast())),
//                 TagID::List => OwnedValue::List(ptr::read(src.cast())),
//                 TagID::Compound => OwnedValue::Compound(ptr::read(src.cast())),
//                 TagID::IntArray => OwnedValue::IntArray(ptr::read(src.cast())),
//                 TagID::LongArray => OwnedValue::LongArray(ptr::read(src.cast())),
//             }
//         }
//     }
// }
