use zerocopy::byteorder;

use crate::value_trait::config::ReadableConfig;

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
