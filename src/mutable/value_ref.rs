use zerocopy::byteorder;

use crate::{
    ByteOrder, MutableGenericNBTImpl, RefCompound, RefList, RefString, TagID,
    tag::{
        Byte, ByteArray, Compound, Double, End, Float, Int, IntArray, List, Long, LongArray, Short,
        String,
    },
};

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

impl<'s, O: ByteOrder> RefValue<'s, O> {
    pub(crate) unsafe fn read_ref(tag_id: TagID, data: *const u8) -> Self {
        unsafe {
            match tag_id {
                TagID::End => RefValue::End(End::read_ref::<O>(data).unwrap_unchecked()),
                TagID::Byte => RefValue::Byte(Byte::read_ref::<O>(data).unwrap_unchecked()),
                TagID::Short => RefValue::Short(Short::read_ref::<O>(data).unwrap_unchecked()),
                TagID::Int => RefValue::Int(Int::read_ref::<O>(data).unwrap_unchecked()),
                TagID::Long => RefValue::Long(Long::read_ref::<O>(data).unwrap_unchecked()),
                TagID::Float => RefValue::Float(Float::read_ref::<O>(data).unwrap_unchecked()),
                TagID::Double => RefValue::Double(Double::read_ref::<O>(data).unwrap_unchecked()),
                TagID::ByteArray => {
                    RefValue::ByteArray(ByteArray::read_ref::<O>(data).unwrap_unchecked())
                }
                TagID::String => RefValue::String(String::read_ref::<O>(data).unwrap_unchecked()),
                TagID::List => RefValue::List(List::read_ref::<O>(data).unwrap_unchecked()),
                TagID::Compound => {
                    RefValue::Compound(Compound::read_ref::<O>(data).unwrap_unchecked())
                }
                TagID::IntArray => {
                    RefValue::IntArray(IntArray::read_ref::<O>(data).unwrap_unchecked())
                }
                TagID::LongArray => {
                    RefValue::LongArray(LongArray::read_ref::<O>(data).unwrap_unchecked())
                }
            }
        }
    }
}
