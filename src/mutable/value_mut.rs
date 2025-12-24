use zerocopy::byteorder;

use crate::{ByteOrder, MutCompound, MutList, MutString, MutVec};

pub enum MutValue<'s, O: ByteOrder> {
    End(()),
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(MutVec<'s, i8>),
    String(MutString<'s>),
    List(MutList<'s, O>),
    Compound(MutCompound<'s, O>),
    IntArray(MutVec<'s, byteorder::I32<O>>),
    LongArray(MutVec<'s, byteorder::I64<O>>),
}
