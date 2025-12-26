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

use crate::{MutString, MutVec};

pub enum VisitRef<'a, 's: 'a, C: ConfigRef> {
    End(&'a ()),
    Byte(&'a i8),
    Short(&'a i16),
    Int(&'a i32),
    Long(&'a i64),
    Float(&'a f32),
    Double(&'a f64),
    ByteArray(&'a C::ByteArray<'s>),
    String(&'a C::String<'s>),
    List(&'a C::List<'s>),
    Compound(&'a C::Compound<'s>),
    IntArray(&'a C::IntArray<'s>),
    LongArray(&'a C::LongArray<'s>),
}

pub enum MapRef<'s, C: ConfigRef> {
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

pub enum VisitMutShared<'a, 's: 'a, C: ConfigMut> {
    End(&'a &'s mut ()),
    Byte(&'a &'s mut i8),
    Short(&'a &'s mut byteorder::I16<C::ByteOrder>),
    Int(&'a &'s mut byteorder::I32<C::ByteOrder>),
    Long(&'a &'s mut byteorder::I64<C::ByteOrder>),
    Float(&'a &'s mut byteorder::F32<C::ByteOrder>),
    Double(&'a &'s mut byteorder::F64<C::ByteOrder>),
    ByteArray(&'a MutVec<'s, i8>),
    String(&'a MutString<'s>),
    List(&'a C::ListMut<'s>),
    Compound(&'a C::CompoundMut<'s>),
    IntArray(&'a MutVec<'s, byteorder::I32<C::ByteOrder>>),
    LongArray(&'a MutVec<'s, byteorder::I64<C::ByteOrder>>),
}

pub enum VisitMut<'a, 's: 'a, C: ConfigMut> {
    End(&'a mut &'s mut ()),
    Byte(&'a mut &'s mut i8),
    Short(&'a mut &'s mut byteorder::I16<C::ByteOrder>),
    Int(&'a mut &'s mut byteorder::I32<C::ByteOrder>),
    Long(&'a mut &'s mut byteorder::I64<C::ByteOrder>),
    Float(&'a mut &'s mut byteorder::F32<C::ByteOrder>),
    Double(&'a mut &'s mut byteorder::F64<C::ByteOrder>),
    ByteArray(&'a mut MutVec<'s, i8>),
    String(&'a mut MutString<'s>),
    List(&'a mut C::ListMut<'s>),
    Compound(&'a mut C::CompoundMut<'s>),
    IntArray(&'a mut MutVec<'s, byteorder::I32<C::ByteOrder>>),
    LongArray(&'a mut MutVec<'s, byteorder::I64<C::ByteOrder>>),
}

pub enum MapMut<'s, C: ConfigMut> {
    End(&'s mut ()),
    Byte(&'s mut i8),
    Short(&'s mut byteorder::I16<C::ByteOrder>),
    Int(&'s mut byteorder::I32<C::ByteOrder>),
    Long(&'s mut byteorder::I64<C::ByteOrder>),
    Float(&'s mut byteorder::F32<C::ByteOrder>),
    Double(&'s mut byteorder::F64<C::ByteOrder>),
    ByteArray(MutVec<'s, i8>),
    String(MutString<'s>),
    List(C::ListMut<'s>),
    Compound(C::CompoundMut<'s>),
    IntArray(MutVec<'s, byteorder::I32<C::ByteOrder>>),
    LongArray(MutVec<'s, byteorder::I64<C::ByteOrder>>),
}
