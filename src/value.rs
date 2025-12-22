mod config;
mod readable;
mod scoped_readable;
mod string;

pub use config::*;
pub use readable::*;
pub use scoped_readable::*;
pub use string::*;

pub enum Value<'a, C: ReadableConfig> {
    End(()),
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
