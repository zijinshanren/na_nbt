use std::{marker::PhantomData, mem::ManuallyDrop, ptr};

use zerocopy::byteorder;

use crate::{
    ByteOrder, GenericNBT, Index, MutValue, MutableConfig, NBT, OwnCompound, OwnList, OwnString,
    OwnTypedList, OwnVec, RefValue, TagID,
};

pub enum OwnValue<O: ByteOrder> {
    End(()),
    Byte(i8),
    Short(byteorder::I16<O>),
    Int(byteorder::I32<O>),
    Long(byteorder::I64<O>),
    Float(byteorder::F32<O>),
    Double(byteorder::F64<O>),
    ByteArray(OwnVec<i8>),
    String(OwnString),
    List(OwnList<O>),
    Compound(OwnCompound<O>),
    IntArray(OwnVec<byteorder::I32<O>>),
    LongArray(OwnVec<byteorder::I64<O>>),
}

impl<O: ByteOrder> Default for OwnValue<O> {
    #[inline]
    fn default() -> Self {
        Self::End(())
    }
}

impl<O: ByteOrder> OwnValue<O> {
    #[allow(clippy::unit_arg)]
    pub(crate) unsafe fn read(tag_id: TagID, src: *mut u8) -> Self {
        unsafe {
            match tag_id {
                TagID::End => OwnValue::End(ptr::read(src.cast())),
                TagID::Byte => OwnValue::Byte(ptr::read(src.cast())),
                TagID::Short => OwnValue::Short(ptr::read(src.cast())),
                TagID::Int => OwnValue::Int(ptr::read(src.cast())),
                TagID::Long => OwnValue::Long(ptr::read(src.cast())),
                TagID::Float => OwnValue::Float(ptr::read(src.cast())),
                TagID::Double => OwnValue::Double(ptr::read(src.cast())),
                TagID::ByteArray => OwnValue::ByteArray(ptr::read(src.cast())),
                TagID::String => OwnValue::String(ptr::read(src.cast())),
                TagID::List => OwnValue::List(ptr::read(src.cast())),
                TagID::Compound => OwnValue::Compound(ptr::read(src.cast())),
                TagID::IntArray => OwnValue::IntArray(ptr::read(src.cast())),
                TagID::LongArray => OwnValue::LongArray(ptr::read(src.cast())),
            }
        }
    }
}

impl<O: ByteOrder> OwnValue<O> {
    #[inline]
    pub fn tag_id(&self) -> TagID {
        match self {
            OwnValue::End(_) => TagID::End,
            OwnValue::Byte(_) => TagID::Byte,
            OwnValue::Short(_) => TagID::Short,
            OwnValue::Int(_) => TagID::Int,
            OwnValue::Long(_) => TagID::Long,
            OwnValue::Float(_) => TagID::Float,
            OwnValue::Double(_) => TagID::Double,
            OwnValue::ByteArray(_) => TagID::ByteArray,
            OwnValue::String(_) => TagID::String,
            OwnValue::List(_) => TagID::List,
            OwnValue::Compound(_) => TagID::Compound,
            OwnValue::IntArray(_) => TagID::IntArray,
            OwnValue::LongArray(_) => TagID::LongArray,
        }
    }

    #[inline]
    pub fn is_<T: NBT>(&self) -> bool {
        self.tag_id() == T::TAG_ID
    }

    #[inline]
    pub fn get<'a>(&'a self, index: impl Index) -> Option<RefValue<'a, O>> {
        index.index_dispatch(
            self,
            |value, index| match value {
                OwnValue::List(value) => value.get(index),
                _ => None,
            },
            |value, key| match value {
                OwnValue::Compound(value) => value.get(key),
                _ => None,
            },
        )
    }

    #[inline]
    pub fn get_<'a, T: GenericNBT>(
        &'a self,
        index: impl Index,
    ) -> Option<T::TypeRef<'a, MutableConfig<O>>> {
        index.index_dispatch(
            self,
            |value, index| match value {
                OwnValue::List(value) => value.get_::<T>(index),
                _ => None,
            },
            |value, key| match value {
                OwnValue::Compound(value) => value.get_::<T>(key),
                _ => None,
            },
        )
    }

    #[inline]
    pub fn get_mut<'a>(&'a mut self, index: impl Index) -> Option<MutValue<'a, O>> {
        index.index_dispatch_mut(
            self,
            |value, index| match value {
                OwnValue::List(value) => value.get_mut(index),
                _ => None,
            },
            |value, key| match value {
                OwnValue::Compound(value) => value.get_mut(key),
                _ => None,
            },
        )
    }

    #[inline]
    pub fn get_mut_<'a, T: GenericNBT>(
        &'a mut self,
        index: impl Index,
    ) -> Option<T::TypeMut<'a, MutableConfig<O>>> {
        index.index_dispatch_mut(
            self,
            |value, index| match value {
                OwnValue::List(value) => value.get_mut_::<T>(index),
                _ => None,
            },
            |value, key| match value {
                OwnValue::Compound(value) => value.get_mut_::<T>(key),
                _ => None,
            },
        )
    }
}

impl<O: ByteOrder> From<()> for OwnValue<O> {
    #[inline]
    fn from(value: ()) -> Self {
        OwnValue::End(value)
    }
}

impl<O: ByteOrder> From<i8> for OwnValue<O> {
    #[inline]
    fn from(value: i8) -> Self {
        OwnValue::Byte(value)
    }
}

impl<O: ByteOrder> From<byteorder::I16<O>> for OwnValue<O> {
    #[inline]
    fn from(value: byteorder::I16<O>) -> Self {
        OwnValue::Short(value)
    }
}

impl<O: ByteOrder> From<byteorder::I32<O>> for OwnValue<O> {
    #[inline]
    fn from(value: byteorder::I32<O>) -> Self {
        OwnValue::Int(value)
    }
}

impl<O: ByteOrder> From<byteorder::I64<O>> for OwnValue<O> {
    #[inline]
    fn from(value: byteorder::I64<O>) -> Self {
        OwnValue::Long(value)
    }
}

impl<O: ByteOrder> From<byteorder::F32<O>> for OwnValue<O> {
    #[inline]
    fn from(value: byteorder::F32<O>) -> Self {
        OwnValue::Float(value)
    }
}

impl<O: ByteOrder> From<byteorder::F64<O>> for OwnValue<O> {
    #[inline]
    fn from(value: byteorder::F64<O>) -> Self {
        OwnValue::Double(value)
    }
}

impl<O: ByteOrder> From<OwnVec<i8>> for OwnValue<O> {
    #[inline]
    fn from(value: OwnVec<i8>) -> Self {
        OwnValue::ByteArray(value)
    }
}

impl<O: ByteOrder> From<OwnString> for OwnValue<O> {
    #[inline]
    fn from(value: OwnString) -> Self {
        OwnValue::String(value)
    }
}

impl<O: ByteOrder> From<OwnList<O>> for OwnValue<O> {
    #[inline]
    fn from(value: OwnList<O>) -> Self {
        OwnValue::List(value)
    }
}

impl<O: ByteOrder> From<OwnCompound<O>> for OwnValue<O> {
    #[inline]
    fn from(value: OwnCompound<O>) -> Self {
        OwnValue::Compound(value)
    }
}

impl<O: ByteOrder> From<OwnVec<byteorder::I32<O>>> for OwnValue<O> {
    #[inline]
    fn from(value: OwnVec<byteorder::I32<O>>) -> Self {
        OwnValue::IntArray(value)
    }
}

impl<O: ByteOrder> From<OwnVec<byteorder::I64<O>>> for OwnValue<O> {
    #[inline]
    fn from(value: OwnVec<byteorder::I64<O>>) -> Self {
        OwnValue::LongArray(value)
    }
}

impl<O: ByteOrder, T: NBT> From<OwnTypedList<O, T>> for OwnValue<O> {
    #[inline]
    fn from(value: OwnTypedList<O, T>) -> Self {
        let me = ManuallyDrop::new(value);
        OwnValue::List(OwnList {
            data: unsafe { ptr::read(&me.data) },
            _marker: PhantomData,
        })
    }
}
