use std::{marker::PhantomData, mem::ManuallyDrop, ptr};

use zerocopy::byteorder;

use crate::{
    ByteOrder, GenericNBT, NBT, CompoundOwn, ListOwn, StringOwn, TypedListOwn, VecOwn, TagID,
    index::Index,
    mutable::{
        compound_mut::MutCompound, compound_ref::RefCompound, config::MutableConfig,
        list_mut::MutList, list_ref::RefList, string_ref::RefString, value_mut::MutValue,
        value_ref::RefValue,
    },
};

pub enum ValueOwn<O: ByteOrder> {
    End(()),
    Byte(i8),
    Short(byteorder::I16<O>),
    Int(byteorder::I32<O>),
    Long(byteorder::I64<O>),
    Float(byteorder::F32<O>),
    Double(byteorder::F64<O>),
    ByteArray(VecOwn<i8>),
    String(StringOwn),
    List(ListOwn<O>),
    Compound(CompoundOwn<O>),
    IntArray(VecOwn<byteorder::I32<O>>),
    LongArray(VecOwn<byteorder::I64<O>>),
}

impl<O: ByteOrder> Default for ValueOwn<O> {
    #[inline]
    fn default() -> Self {
        Self::End(())
    }
}

impl<O: ByteOrder> ValueOwn<O> {
    #[allow(clippy::unit_arg)]
    pub(crate) unsafe fn read(tag_id: TagID, src: *mut u8) -> Self {
        unsafe {
            match tag_id {
                TagID::End => ValueOwn::End(ptr::read(src.cast())),
                TagID::Byte => ValueOwn::Byte(ptr::read(src.cast())),
                TagID::Short => ValueOwn::Short(ptr::read(src.cast())),
                TagID::Int => ValueOwn::Int(ptr::read(src.cast())),
                TagID::Long => ValueOwn::Long(ptr::read(src.cast())),
                TagID::Float => ValueOwn::Float(ptr::read(src.cast())),
                TagID::Double => ValueOwn::Double(ptr::read(src.cast())),
                TagID::ByteArray => ValueOwn::ByteArray(ptr::read(src.cast())),
                TagID::String => ValueOwn::String(ptr::read(src.cast())),
                TagID::List => ValueOwn::List(ptr::read(src.cast())),
                TagID::Compound => ValueOwn::Compound(ptr::read(src.cast())),
                TagID::IntArray => ValueOwn::IntArray(ptr::read(src.cast())),
                TagID::LongArray => ValueOwn::LongArray(ptr::read(src.cast())),
            }
        }
    }
}

impl<O: ByteOrder> ValueOwn<O> {
    #[inline]
    pub fn tag_id(&self) -> TagID {
        match self {
            ValueOwn::End(_) => TagID::End,
            ValueOwn::Byte(_) => TagID::Byte,
            ValueOwn::Short(_) => TagID::Short,
            ValueOwn::Int(_) => TagID::Int,
            ValueOwn::Long(_) => TagID::Long,
            ValueOwn::Float(_) => TagID::Float,
            ValueOwn::Double(_) => TagID::Double,
            ValueOwn::ByteArray(_) => TagID::ByteArray,
            ValueOwn::String(_) => TagID::String,
            ValueOwn::List(_) => TagID::List,
            ValueOwn::Compound(_) => TagID::Compound,
            ValueOwn::IntArray(_) => TagID::IntArray,
            ValueOwn::LongArray(_) => TagID::LongArray,
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
                ValueOwn::List(value) => value.get(index),
                _ => None,
            },
            |value, key| match value {
                ValueOwn::Compound(value) => value.get(key),
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
                ValueOwn::List(value) => value.get_::<T>(index),
                _ => None,
            },
            |value, key| match value {
                ValueOwn::Compound(value) => value.get_::<T>(key),
                _ => None,
            },
        )
    }

    #[inline]
    pub fn get_mut<'a>(&'a mut self, index: impl Index) -> Option<MutValue<'a, O>> {
        index.index_dispatch_mut(
            self,
            |value, index| match value {
                ValueOwn::List(value) => value.get_mut(index),
                _ => None,
            },
            |value, key| match value {
                ValueOwn::Compound(value) => value.get_mut(key),
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
                ValueOwn::List(value) => value.get_mut_::<T>(index),
                _ => None,
            },
            |value, key| match value {
                ValueOwn::Compound(value) => value.get_mut_::<T>(key),
                _ => None,
            },
        )
    }

    #[inline]
    #[allow(clippy::unit_arg)]
    pub fn to_ref<'a>(&'a self) -> RefValue<'a, O> {
        match self {
            ValueOwn::End(value) => RefValue::End(*value),
            ValueOwn::Byte(value) => RefValue::Byte(*value),
            ValueOwn::Short(value) => RefValue::Short(value.get()),
            ValueOwn::Int(value) => RefValue::Int(value.get()),
            ValueOwn::Long(value) => RefValue::Long(value.get()),
            ValueOwn::Float(value) => RefValue::Float(value.get()),
            ValueOwn::Double(value) => RefValue::Double(value.get()),
            ValueOwn::ByteArray(value) => RefValue::ByteArray(value),
            ValueOwn::String(value) => RefValue::String(RefString {
                data: value.as_mutf8_str(),
            }),
            ValueOwn::List(value) => RefValue::List(RefList {
                data: value.data.as_ptr(),
                _marker: PhantomData,
            }),
            ValueOwn::Compound(value) => RefValue::Compound(RefCompound {
                data: value.data.as_ptr(),
                _marker: PhantomData,
            }),
            ValueOwn::IntArray(value) => RefValue::IntArray(value),
            ValueOwn::LongArray(value) => RefValue::LongArray(value),
        }
    }

    #[inline]
    pub fn to_mut<'a>(&'a mut self) -> MutValue<'a, O> {
        match self {
            ValueOwn::End(value) => MutValue::End(&mut *value),
            ValueOwn::Byte(value) => MutValue::Byte(&mut *value),
            ValueOwn::Short(value) => MutValue::Short(&mut *value),
            ValueOwn::Int(value) => MutValue::Int(&mut *value),
            ValueOwn::Long(value) => MutValue::Long(&mut *value),
            ValueOwn::Float(value) => MutValue::Float(&mut *value),
            ValueOwn::Double(value) => MutValue::Double(&mut *value),
            ValueOwn::ByteArray(value) => MutValue::ByteArray(value.to_mut()),
            ValueOwn::String(value) => MutValue::String(value.to_mut()),
            ValueOwn::List(value) => MutValue::List(MutList {
                data: value.data.to_mut(),
                _marker: PhantomData,
            }),
            ValueOwn::Compound(value) => MutValue::Compound(MutCompound {
                data: value.data.to_mut(),
                _marker: PhantomData,
            }),
            ValueOwn::IntArray(value) => MutValue::IntArray(value.to_mut()),
            ValueOwn::LongArray(value) => MutValue::LongArray(value.to_mut()),
        }
    }
}

impl<O: ByteOrder> From<()> for ValueOwn<O> {
    #[inline]
    fn from(value: ()) -> Self {
        ValueOwn::End(value)
    }
}

impl<O: ByteOrder> From<i8> for ValueOwn<O> {
    #[inline]
    fn from(value: i8) -> Self {
        ValueOwn::Byte(value)
    }
}

impl<O: ByteOrder> From<byteorder::I16<O>> for ValueOwn<O> {
    #[inline]
    fn from(value: byteorder::I16<O>) -> Self {
        ValueOwn::Short(value)
    }
}

impl<O: ByteOrder> From<byteorder::I32<O>> for ValueOwn<O> {
    #[inline]
    fn from(value: byteorder::I32<O>) -> Self {
        ValueOwn::Int(value)
    }
}

impl<O: ByteOrder> From<byteorder::I64<O>> for ValueOwn<O> {
    #[inline]
    fn from(value: byteorder::I64<O>) -> Self {
        ValueOwn::Long(value)
    }
}

impl<O: ByteOrder> From<byteorder::F32<O>> for ValueOwn<O> {
    #[inline]
    fn from(value: byteorder::F32<O>) -> Self {
        ValueOwn::Float(value)
    }
}

impl<O: ByteOrder> From<byteorder::F64<O>> for ValueOwn<O> {
    #[inline]
    fn from(value: byteorder::F64<O>) -> Self {
        ValueOwn::Double(value)
    }
}

impl<O: ByteOrder> From<VecOwn<i8>> for ValueOwn<O> {
    #[inline]
    fn from(value: VecOwn<i8>) -> Self {
        ValueOwn::ByteArray(value)
    }
}

impl<O: ByteOrder> From<StringOwn> for ValueOwn<O> {
    #[inline]
    fn from(value: StringOwn) -> Self {
        ValueOwn::String(value)
    }
}

impl<O: ByteOrder> From<ListOwn<O>> for ValueOwn<O> {
    #[inline]
    fn from(value: ListOwn<O>) -> Self {
        ValueOwn::List(value)
    }
}

impl<O: ByteOrder> From<CompoundOwn<O>> for ValueOwn<O> {
    #[inline]
    fn from(value: CompoundOwn<O>) -> Self {
        ValueOwn::Compound(value)
    }
}

impl<O: ByteOrder> From<VecOwn<byteorder::I32<O>>> for ValueOwn<O> {
    #[inline]
    fn from(value: VecOwn<byteorder::I32<O>>) -> Self {
        ValueOwn::IntArray(value)
    }
}

impl<O: ByteOrder> From<VecOwn<byteorder::I64<O>>> for ValueOwn<O> {
    #[inline]
    fn from(value: VecOwn<byteorder::I64<O>>) -> Self {
        ValueOwn::LongArray(value)
    }
}

impl<O: ByteOrder, T: NBT> From<TypedListOwn<O, T>> for ValueOwn<O> {
    #[inline]
    fn from(value: TypedListOwn<O, T>) -> Self {
        let me = ManuallyDrop::new(value);
        ValueOwn::List(ListOwn {
            data: unsafe { ptr::read(&me.data) },
            _marker: PhantomData,
        })
    }
}
