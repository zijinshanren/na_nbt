use zerocopy::byteorder;

use crate::{
    WritableConfig,
    value_trait::config::ReadableConfig,
    view::{StringViewMut, VecViewMut},
};

pub enum Value<'a, C: ReadableConfig> {
    End,
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(C::ByteArray<'a>),
    String(C::String<'a>),
    List(C::List<'a>),
    Compound(C::Compound<'a>),
    IntArray(C::IntArray<'a>),
    LongArray(C::LongArray<'a>),
}

pub enum ValueMut<'a, C: WritableConfig> {
    End,
    Byte(&'a mut i8),
    Short(&'a mut byteorder::I16<C::ByteOrder>),
    Int(&'a mut byteorder::I32<C::ByteOrder>),
    Long(&'a mut byteorder::I64<C::ByteOrder>),
    Float(&'a mut byteorder::F32<C::ByteOrder>),
    Double(&'a mut byteorder::F64<C::ByteOrder>),
    ByteArray(VecViewMut<'a, i8>),
    String(StringViewMut<'a>),
    List(C::ListMut<'a>),
    Compound(C::CompoundMut<'a>),
    IntArray(VecViewMut<'a, byteorder::I32<C::ByteOrder>>),
    LongArray(VecViewMut<'a, byteorder::I64<C::ByteOrder>>),
}
