use std::marker::PhantomData;

use zerocopy::byteorder;

use crate::{
    ByteOrder, GenericNBT, Index, MapMut, MutCompound, MutList, MutString, MutTypedList, MutVec,
    MutableConfig, NBT, RefValue, TagID, ValueBase, ValueMut, VisitMut, VisitMutShared,
};

pub enum MutValue<'s, O: ByteOrder> {
    End(&'s mut ()),
    Byte(&'s mut i8),
    Short(&'s mut byteorder::I16<O>),
    Int(&'s mut byteorder::I32<O>),
    Long(&'s mut byteorder::I64<O>),
    Float(&'s mut byteorder::F32<O>),
    Double(&'s mut byteorder::F64<O>),
    ByteArray(MutVec<'s, i8>),
    String(MutString<'s>),
    List(MutList<'s, O>),
    Compound(MutCompound<'s, O>),
    IntArray(MutVec<'s, byteorder::I32<O>>),
    LongArray(MutVec<'s, byteorder::I64<O>>),
}

impl<'s, O: ByteOrder> MutValue<'s, O> {
    #[inline]
    pub fn tag_id(&self) -> TagID {
        match self {
            MutValue::End(_) => TagID::End,
            MutValue::Byte(_) => TagID::Byte,
            MutValue::Short(_) => TagID::Short,
            MutValue::Int(_) => TagID::Int,
            MutValue::Long(_) => TagID::Long,
            MutValue::Float(_) => TagID::Float,
            MutValue::Double(_) => TagID::Double,
            MutValue::ByteArray(_) => TagID::ByteArray,
            MutValue::String(_) => TagID::String,
            MutValue::List(_) => TagID::List,
            MutValue::Compound(_) => TagID::Compound,
            MutValue::IntArray(_) => TagID::IntArray,
            MutValue::LongArray(_) => TagID::LongArray,
        }
    }

    #[inline]
    pub fn is_<T: NBT>(&self) -> bool {
        ValueBase::is_::<T>(self)
    }

    #[inline]
    pub fn get<'a>(&'a self, index: impl Index) -> Option<RefValue<'a, O>> {
        ValueMut::get(self, index)
    }

    #[inline]
    pub fn get_<'a, T: GenericNBT>(
        &'a self,
        index: impl Index,
    ) -> Option<T::TypeRef<'a, MutableConfig<O>>> {
        ValueMut::get_::<T>(self, index)
    }

    #[inline]
    pub fn get_mut<'a>(&'a mut self, index: impl Index) -> Option<MutValue<'a, O>> {
        ValueMut::get_mut(self, index)
    }

    #[inline]
    pub fn get_mut_<'a, T: GenericNBT>(
        &'a mut self,
        index: impl Index,
    ) -> Option<T::TypeMut<'a, MutableConfig<O>>> {
        ValueMut::get_mut_::<T>(self, index)
    }
}

impl<'s, O: ByteOrder> ValueBase for MutValue<'s, O> {
    #[inline]
    fn tag_id(&self) -> TagID {
        self.tag_id()
    }
}

impl<'s, O: ByteOrder> ValueMut<'s> for MutValue<'s, O> {
    type Config = MutableConfig<O>;

    fn visit_shared<'a, R>(
        &'a self,
        match_fn: impl FnOnce(VisitMutShared<'a, 's, Self::Config>) -> R,
    ) -> R
    where
        's: 'a,
    {
        match self {
            MutValue::End(value) => match_fn(VisitMutShared::End(value)),
            MutValue::Byte(value) => match_fn(VisitMutShared::Byte(value)),
            MutValue::Short(value) => match_fn(VisitMutShared::Short(value)),
            MutValue::Int(value) => match_fn(VisitMutShared::Int(value)),
            MutValue::Long(value) => match_fn(VisitMutShared::Long(value)),
            MutValue::Float(value) => match_fn(VisitMutShared::Float(value)),
            MutValue::Double(value) => match_fn(VisitMutShared::Double(value)),
            MutValue::ByteArray(value) => match_fn(VisitMutShared::ByteArray(value)),
            MutValue::String(value) => match_fn(VisitMutShared::String(value)),
            MutValue::List(value) => match_fn(VisitMutShared::List(value)),
            MutValue::Compound(value) => match_fn(VisitMutShared::Compound(value)),
            MutValue::IntArray(value) => match_fn(VisitMutShared::IntArray(value)),
            MutValue::LongArray(value) => match_fn(VisitMutShared::LongArray(value)),
        }
    }

    fn visit<'a, R>(&'a mut self, match_fn: impl FnOnce(VisitMut<'a, 's, Self::Config>) -> R) -> R
    where
        's: 'a,
    {
        match self {
            MutValue::End(value) => match_fn(VisitMut::End(value)),
            MutValue::Byte(value) => match_fn(VisitMut::Byte(value)),
            MutValue::Short(value) => match_fn(VisitMut::Short(value)),
            MutValue::Int(value) => match_fn(VisitMut::Int(value)),
            MutValue::Long(value) => match_fn(VisitMut::Long(value)),
            MutValue::Float(value) => match_fn(VisitMut::Float(value)),
            MutValue::Double(value) => match_fn(VisitMut::Double(value)),
            MutValue::ByteArray(value) => match_fn(VisitMut::ByteArray(value)),
            MutValue::String(value) => match_fn(VisitMut::String(value)),
            MutValue::List(value) => match_fn(VisitMut::List(value)),
            MutValue::Compound(value) => match_fn(VisitMut::Compound(value)),
            MutValue::IntArray(value) => match_fn(VisitMut::IntArray(value)),
            MutValue::LongArray(value) => match_fn(VisitMut::LongArray(value)),
        }
    }

    fn map<R>(self, match_fn: impl FnOnce(MapMut<'s, Self::Config>) -> R) -> R {
        match self {
            MutValue::End(value) => match_fn(MapMut::End(value)),
            MutValue::Byte(value) => match_fn(MapMut::Byte(value)),
            MutValue::Short(value) => match_fn(MapMut::Short(value)),
            MutValue::Int(value) => match_fn(MapMut::Int(value)),
            MutValue::Long(value) => match_fn(MapMut::Long(value)),
            MutValue::Float(value) => match_fn(MapMut::Float(value)),
            MutValue::Double(value) => match_fn(MapMut::Double(value)),
            MutValue::ByteArray(value) => match_fn(MapMut::ByteArray(value)),
            MutValue::String(value) => match_fn(MapMut::String(value)),
            MutValue::List(value) => match_fn(MapMut::List(value)),
            MutValue::Compound(value) => match_fn(MapMut::Compound(value)),
            MutValue::IntArray(value) => match_fn(MapMut::IntArray(value)),
            MutValue::LongArray(value) => match_fn(MapMut::LongArray(value)),
        }
    }
}

impl<'s, O: ByteOrder> From<&'s mut ()> for MutValue<'s, O> {
    #[inline]
    fn from(value: &'s mut ()) -> Self {
        MutValue::End(value)
    }
}

impl<'s, O: ByteOrder> From<&'s mut i8> for MutValue<'s, O> {
    #[inline]
    fn from(value: &'s mut i8) -> Self {
        MutValue::Byte(value)
    }
}

impl<'s, O: ByteOrder> From<&'s mut byteorder::I16<O>> for MutValue<'s, O> {
    #[inline]
    fn from(value: &'s mut byteorder::I16<O>) -> Self {
        MutValue::Short(value)
    }
}

impl<'s, O: ByteOrder> From<&'s mut byteorder::I32<O>> for MutValue<'s, O> {
    #[inline]
    fn from(value: &'s mut byteorder::I32<O>) -> Self {
        MutValue::Int(value)
    }
}

impl<'s, O: ByteOrder> From<&'s mut byteorder::I64<O>> for MutValue<'s, O> {
    #[inline]
    fn from(value: &'s mut byteorder::I64<O>) -> Self {
        MutValue::Long(value)
    }
}

impl<'s, O: ByteOrder> From<&'s mut byteorder::F32<O>> for MutValue<'s, O> {
    #[inline]
    fn from(value: &'s mut byteorder::F32<O>) -> Self {
        MutValue::Float(value)
    }
}

impl<'s, O: ByteOrder> From<&'s mut byteorder::F64<O>> for MutValue<'s, O> {
    #[inline]
    fn from(value: &'s mut byteorder::F64<O>) -> Self {
        MutValue::Double(value)
    }
}

impl<'s, O: ByteOrder> From<MutVec<'s, i8>> for MutValue<'s, O> {
    #[inline]
    fn from(value: MutVec<'s, i8>) -> Self {
        MutValue::ByteArray(value)
    }
}

impl<'s, O: ByteOrder> From<MutString<'s>> for MutValue<'s, O> {
    #[inline]
    fn from(value: MutString<'s>) -> Self {
        MutValue::String(value)
    }
}

impl<'s, O: ByteOrder> From<MutList<'s, O>> for MutValue<'s, O> {
    #[inline]
    fn from(value: MutList<'s, O>) -> Self {
        MutValue::List(value)
    }
}

impl<'s, O: ByteOrder> From<MutCompound<'s, O>> for MutValue<'s, O> {
    #[inline]
    fn from(value: MutCompound<'s, O>) -> Self {
        MutValue::Compound(value)
    }
}

impl<'s, O: ByteOrder> From<MutVec<'s, byteorder::I32<O>>> for MutValue<'s, O> {
    #[inline]
    fn from(value: MutVec<'s, byteorder::I32<O>>) -> Self {
        MutValue::IntArray(value)
    }
}

impl<'s, O: ByteOrder> From<MutVec<'s, byteorder::I64<O>>> for MutValue<'s, O> {
    #[inline]
    fn from(value: MutVec<'s, byteorder::I64<O>>) -> Self {
        MutValue::LongArray(value)
    }
}

impl<'s, O: ByteOrder, T: NBT> From<MutTypedList<'s, O, T>> for MutValue<'s, O> {
    #[inline]
    fn from(value: MutTypedList<'s, O, T>) -> Self {
        MutValue::List(MutList {
            data: value.data,
            _marker: PhantomData,
        })
    }
}
