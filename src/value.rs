mod config_mut;
mod config_ref;
mod string_ref;
mod value_base;
mod value_mut;
mod value_ref;

pub use config_mut::*;
pub use config_ref::*;
pub use string_ref::*;
pub use value_base::*;
pub use value_mut::*;
pub use value_ref::*;
use zerocopy::byteorder;

use crate::{StringViewMut, VecViewMut};

pub enum Visit<'s, C: ConfigRef> {
    End(()),
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(C::ByteArray<'s>),
    String(C::String<'s>),
    List(C::List<'s>),
    Compound(C::Compound<'s>),
    IntArray(C::IntArray<'s>),
    LongArray(C::LongArray<'s>),
}

pub enum VisitMut<'s, C: ConfigMut> {
    End(&'s mut ()),
    Byte(&'s mut i8),
    Short(&'s mut byteorder::I16<C::ByteOrder>),
    Int(&'s mut byteorder::I32<C::ByteOrder>),
    Long(&'s mut byteorder::I64<C::ByteOrder>),
    Float(&'s mut byteorder::F32<C::ByteOrder>),
    Double(&'s mut byteorder::F64<C::ByteOrder>),
    ByteArray(VecViewMut<'s, i8>),
    String(StringViewMut<'s>),
    List(C::ListMut<'s>),
    Compound(C::CompoundMut<'s>),
    IntArray(VecViewMut<'s, byteorder::I32<C::ByteOrder>>),
    LongArray(VecViewMut<'s, byteorder::I64<C::ByteOrder>>),
}
