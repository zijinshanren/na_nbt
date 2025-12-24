use zerocopy::byteorder;

use crate::{
    ByteOrder, MutableConfig, MutableGenericNBTImpl, RefCompound, RefList, RefString, TagID,
    ValueBase, ValueRef,
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

impl<'s, O: ByteOrder> Default for RefValue<'s, O> {
    #[inline]
    fn default() -> Self {
        RefValue::End(())
    }
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

impl<'s, O: ByteOrder> RefValue<'s, O> {
    #[inline]
    pub fn tag_id(&self) -> TagID {
        match self {
            RefValue::End(_) => TagID::End,
            RefValue::Byte(_) => TagID::Byte,
            RefValue::Short(_) => TagID::Short,
            RefValue::Int(_) => TagID::Int,
            RefValue::Long(_) => TagID::Long,
            RefValue::Float(_) => TagID::Float,
            RefValue::Double(_) => TagID::Double,
            RefValue::ByteArray(_) => TagID::ByteArray,
            RefValue::String(_) => TagID::String,
            RefValue::List(_) => TagID::List,
            RefValue::Compound(_) => TagID::Compound,
            RefValue::IntArray(_) => TagID::IntArray,
            RefValue::LongArray(_) => TagID::LongArray,
        }
    }

    #[inline]
    pub fn is_<T: crate::NBT>(&self) -> bool {
        self.tag_id() == T::TAG_ID
    }

    #[inline]
    pub fn ref_<'a, T: crate::NBT>(&'a self) -> Option<&'a T::TypeRef<'s, MutableConfig<O>>>
    where
        's: 'a,
    {
        todo!()
    }
}

impl<'s, O: ByteOrder> ValueBase for RefValue<'s, O> {
    #[inline]
    fn tag_id(&self) -> TagID {
        self.tag_id()
    }

    #[inline]
    fn is_<T: crate::NBT>(&self) -> bool {
        self.is_::<T>()
    }
}

impl<'s, O: ByteOrder> ValueRef<'s> for RefValue<'s, O> {
    type Config = MutableConfig<O>;

    fn ref_<'a, T: crate::NBT>(&'a self) -> Option<&'a T::TypeRef<'s, Self::Config>>
    where
        's: 'a,
    {
        todo!()
    }

    fn into_<T: crate::NBT>(self) -> Option<T::TypeRef<'s, Self::Config>> {
        todo!()
    }

    fn get(
        &self,
        index: impl crate::Index,
    ) -> Option<<Self::Config as crate::ConfigRef>::Value<'s>> {
        todo!()
    }

    fn get_<T: crate::NBT>(
        &self,
        index: impl crate::Index,
    ) -> Option<T::TypeRef<'s, Self::Config>> {
        todo!()
    }

    fn map<R>(self, match_fn: impl FnOnce(crate::MapRef<'s, Self::Config>) -> R) -> R {
        todo!()
    }
}
