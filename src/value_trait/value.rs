use zerocopy::byteorder;

use crate::{
    value_trait::config::{ReadableConfig, WritableConfig},
    view::{StringViewMut, VecViewMut},
};

pub enum Value<'a, 'doc, C: ReadableConfig> {
    End,
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(&'a [i8]),
    String(&'a C::String<'doc>),
    List(&'a C::List<'doc>),
    Compound(&'a C::Compound<'doc>),
    IntArray(&'a [byteorder::I32<C::ByteOrder>]),
    LongArray(&'a [byteorder::I64<C::ByteOrder>]),
}

pub enum ValueScoped<'a, C: ReadableConfig> {
    End,
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(&'a [i8]),
    String(C::String<'a>),
    List(C::List<'a>),
    Compound(C::Compound<'a>),
    IntArray(&'a [byteorder::I32<C::ByteOrder>]),
    LongArray(&'a [byteorder::I64<C::ByteOrder>]),
}

pub enum ValueMut<'a, 's: 'a, C: WritableConfig> {
    End,
    Byte(&'a mut i8),
    Short(&'a mut byteorder::I16<C::ByteOrder>),
    Int(&'a mut byteorder::I32<C::ByteOrder>),
    Long(&'a mut byteorder::I64<C::ByteOrder>),
    Float(&'a mut byteorder::F32<C::ByteOrder>),
    Double(&'a mut byteorder::F64<C::ByteOrder>),
    ByteArray(&'a mut VecViewMut<'s, i8>),
    String(&'a mut StringViewMut<'s>),
    List(&'a mut C::ListMut<'s>),
    Compound(&'a mut C::CompoundMut<'s>),
    IntArray(&'a mut VecViewMut<'s, byteorder::I32<C::ByteOrder>>),
    LongArray(&'a mut VecViewMut<'s, byteorder::I64<C::ByteOrder>>),
}

pub enum ValueMutScoped<'a, C: WritableConfig> {
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
