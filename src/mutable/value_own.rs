use zerocopy::byteorder;

use crate::{
    BinaryReadWrite, ByteOrder, OwnedCompound, OwnedList, StringViewOwn, TagID, VecViewOwn,
};

pub enum OwnedValue<O: ByteOrder> {
    End(()),
    Byte(i8),
    Short(byteorder::I16<O>),
    Int(byteorder::I32<O>),
    Long(byteorder::I64<O>),
    Float(byteorder::F32<O>),
    Double(byteorder::F64<O>),
    ByteArray(VecViewOwn<i8>),
    String(StringViewOwn),
    List(OwnedList<O>),
    Compound(OwnedCompound<O>),
    IntArray(VecViewOwn<byteorder::I32<O>>),
    LongArray(VecViewOwn<byteorder::I64<O>>),
}

impl<O: ByteOrder> Default for OwnedValue<O> {
    #[inline]
    fn default() -> Self {
        Self::End(())
    }
}

// impl<O: ByteOrder> OwnedValue<O> {
//     unsafe fn write(self, dst: *mut u8) {
//         unsafe {
//             match self {
//                 OwnedValue::End(value) => value.write(dst),
//                 OwnedValue::Byte(value) => value.write(dst),
//                 OwnedValue::Short(value) => value.write(dst),
//                 OwnedValue::Int(value) => value.write(dst),
//                 OwnedValue::Long(value) => value.write(dst),
//                 OwnedValue::Float(value) => value.write(dst),
//                 OwnedValue::Double(value) => value.write(dst),
//                 OwnedValue::ByteArray(value) => value.write(dst),
//                 OwnedValue::String(value) => value.write(dst),
//                 OwnedValue::List(value) => value.write(dst),
//                 OwnedValue::Compound(value) => value.write(dst),
//                 OwnedValue::IntArray(value) => value.write(dst),
//                 OwnedValue::LongArray(value) => value.write(dst),
//             };
//         }
//     }

//     #[allow(clippy::unit_arg)]
//     unsafe fn read(tag_id: TagID, src: *mut u8) -> Self {
//         unsafe {
//             match tag_id {
//                 TagID::End => OwnedValue::End(BinaryReadWrite::read(src)),
//                 TagID::Byte => OwnedValue::Byte(BinaryReadWrite::read(src)),
//                 TagID::Short => OwnedValue::Short(BinaryReadWrite::read(src)),
//                 TagID::Int => OwnedValue::Int(BinaryReadWrite::read(src)),
//                 TagID::Long => OwnedValue::Long(BinaryReadWrite::read(src)),
//                 TagID::Float => OwnedValue::Float(BinaryReadWrite::read(src)),
//                 TagID::Double => OwnedValue::Double(BinaryReadWrite::read(src)),
//                 TagID::ByteArray => OwnedValue::ByteArray(VecViewOwn::read(src)),
//                 TagID::String => OwnedValue::String(BinaryReadWrite::read(src)),
//                 TagID::List => OwnedValue::List(BinaryReadWrite::read(src)),
//                 TagID::Compound => OwnedValue::Compound(BinaryReadWrite::read(src)),
//                 TagID::IntArray => OwnedValue::IntArray(BinaryReadWrite::read(src)),
//                 TagID::LongArray => OwnedValue::LongArray(BinaryReadWrite::read(src)),
//             }
//         }
//     }
// }
