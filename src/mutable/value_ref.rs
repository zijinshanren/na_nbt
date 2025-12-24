use zerocopy::byteorder;

use crate::{ByteOrder, RefCompound, RefList, RefString};

#[derive(Clone)]
pub enum RefValue<'s, O: ByteOrder> {
    End(()),
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(&'s [i8]),
    String(RefString<'s>),
    List(RefList<'s, O>),
    Compound(RefCompound<'s, O>),
    IntArray(&'s [byteorder::I32<O>]),
    LongArray(&'s [byteorder::I64<O>]),
}
