use std::marker::PhantomData;

use zerocopy::byteorder;

use crate::{
    ByteOrder, Index, MapRef, MutableConfig, NBT, RefCompound, RefList, RefString, RefTypedList,
    TagID, ValueBase, ValueRef, VisitRef,
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
    pub fn is_<T: NBT>(&self) -> bool {
        ValueBase::is_::<T>(self)
    }

    #[inline]
    pub fn ref_<T: NBT>(&self) -> Option<&T::TypeRef<'s, MutableConfig<O>>> {
        ValueRef::ref_::<T>(self)
    }

    #[inline]
    pub fn into_<T: NBT>(self) -> Option<T::TypeRef<'s, MutableConfig<O>>> {
        ValueRef::into_::<T>(self)
    }

    #[inline]
    pub fn get(&self, index: impl Index) -> Option<RefValue<'s, O>> {
        ValueRef::get(self, index)
    }

    #[inline]
    pub fn get_<T: NBT>(&self, index: impl Index) -> Option<T::TypeRef<'s, MutableConfig<O>>> {
        ValueRef::get_::<T>(self, index)
    }
}

impl<'s, O: ByteOrder> ValueBase for RefValue<'s, O> {
    type ConfigRef = MutableConfig<O>;

    #[inline]
    fn tag_id(&self) -> TagID {
        self.tag_id()
    }
}

impl<'s, O: ByteOrder> ValueRef<'s> for RefValue<'s, O> {
    fn visit<'a, R>(
        &'a self,
        match_fn: impl FnOnce(crate::VisitRef<'a, 's, Self::ConfigRef>) -> R,
    ) -> R
    where
        's: 'a,
    {
        match self {
            RefValue::End(value) => match_fn(VisitRef::End(value)),
            RefValue::Byte(value) => match_fn(VisitRef::Byte(value)),
            RefValue::Short(value) => match_fn(VisitRef::Short(value)),
            RefValue::Int(value) => match_fn(VisitRef::Int(value)),
            RefValue::Long(value) => match_fn(VisitRef::Long(value)),
            RefValue::Float(value) => match_fn(VisitRef::Float(value)),
            RefValue::Double(value) => match_fn(VisitRef::Double(value)),
            RefValue::ByteArray(value) => match_fn(VisitRef::ByteArray(value)),
            RefValue::String(value) => match_fn(VisitRef::String(value)),
            RefValue::List(value) => match_fn(VisitRef::List(value)),
            RefValue::Compound(value) => match_fn(VisitRef::Compound(value)),
            RefValue::IntArray(value) => match_fn(VisitRef::IntArray(value)),
            RefValue::LongArray(value) => match_fn(VisitRef::LongArray(value)),
        }
    }

    fn map<R>(self, match_fn: impl FnOnce(crate::MapRef<'s, Self::ConfigRef>) -> R) -> R {
        match self {
            RefValue::End(value) => match_fn(MapRef::End(value)),
            RefValue::Byte(value) => match_fn(MapRef::Byte(value)),
            RefValue::Short(value) => match_fn(MapRef::Short(value)),
            RefValue::Int(value) => match_fn(MapRef::Int(value)),
            RefValue::Long(value) => match_fn(MapRef::Long(value)),
            RefValue::Float(value) => match_fn(MapRef::Float(value)),
            RefValue::Double(value) => match_fn(MapRef::Double(value)),
            RefValue::ByteArray(value) => match_fn(MapRef::ByteArray(value)),
            RefValue::String(value) => match_fn(MapRef::String(value)),
            RefValue::List(value) => match_fn(MapRef::List(value)),
            RefValue::Compound(value) => match_fn(MapRef::Compound(value)),
            RefValue::IntArray(value) => match_fn(MapRef::IntArray(value)),
            RefValue::LongArray(value) => match_fn(MapRef::LongArray(value)),
        }
    }
}

impl<'s, O: ByteOrder> From<()> for RefValue<'s, O> {
    #[inline]
    fn from(_: ()) -> Self {
        RefValue::End(())
    }
}

impl<'s, O: ByteOrder> From<i8> for RefValue<'s, O> {
    #[inline]
    fn from(value: i8) -> Self {
        RefValue::Byte(value)
    }
}

impl<'s, O: ByteOrder> From<i16> for RefValue<'s, O> {
    #[inline]
    fn from(value: i16) -> Self {
        RefValue::Short(value)
    }
}

impl<'s, O: ByteOrder> From<i32> for RefValue<'s, O> {
    #[inline]
    fn from(value: i32) -> Self {
        RefValue::Int(value)
    }
}

impl<'s, O: ByteOrder> From<i64> for RefValue<'s, O> {
    #[inline]
    fn from(value: i64) -> Self {
        RefValue::Long(value)
    }
}

impl<'s, O: ByteOrder> From<f32> for RefValue<'s, O> {
    #[inline]
    fn from(value: f32) -> Self {
        RefValue::Float(value)
    }
}

impl<'s, O: ByteOrder> From<f64> for RefValue<'s, O> {
    #[inline]
    fn from(value: f64) -> Self {
        RefValue::Double(value)
    }
}

impl<'s, O: ByteOrder> From<&'s [i8]> for RefValue<'s, O> {
    #[inline]
    fn from(value: &'s [i8]) -> Self {
        RefValue::ByteArray(value)
    }
}

impl<'s, O: ByteOrder> From<RefString<'s>> for RefValue<'s, O> {
    #[inline]
    fn from(value: RefString<'s>) -> Self {
        RefValue::String(value)
    }
}

impl<'s, O: ByteOrder> From<RefList<'s, O>> for RefValue<'s, O> {
    #[inline]
    fn from(value: RefList<'s, O>) -> Self {
        RefValue::List(value)
    }
}

impl<'s, O: ByteOrder> From<RefCompound<'s, O>> for RefValue<'s, O> {
    #[inline]
    fn from(value: RefCompound<'s, O>) -> Self {
        RefValue::Compound(value)
    }
}

impl<'s, O: ByteOrder> From<&'s [byteorder::I32<O>]> for RefValue<'s, O> {
    #[inline]
    fn from(value: &'s [byteorder::I32<O>]) -> Self {
        RefValue::IntArray(value)
    }
}

impl<'s, O: ByteOrder> From<&'s [byteorder::I64<O>]> for RefValue<'s, O> {
    #[inline]
    fn from(value: &'s [byteorder::I64<O>]) -> Self {
        RefValue::LongArray(value)
    }
}

impl<'s, O: ByteOrder, T: NBT> From<RefTypedList<'s, O, T>> for RefValue<'s, O> {
    #[inline]
    fn from(value: RefTypedList<'s, O, T>) -> Self {
        RefValue::List(RefList {
            data: value.data,
            _marker: PhantomData,
        })
    }
}
