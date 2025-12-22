use std::{io::Write, marker::PhantomData};

use zerocopy::byteorder;

use crate::{
    ByteOrder, NBT, ReadableCompound, ReadableConfig, ReadableList, ReadableString, ReadableValue, ReadonlyArray, Result, ScopedReadableCompound, ScopedReadableList, ScopedReadableValue, TagID, Value, immutable::{
        typed_list::{ReadonlyPrimitiveList, ReadonlyTypedList},
        value::{
            Document, ReadonlyCompound, ReadonlyCompoundIter, ReadonlyList, ReadonlyListIter,
            ReadonlyString, ReadonlyValue,
        },
    }, index::Index
};

impl<'doc, D: Document> ReadableString<'doc> for ReadonlyString<'doc, D> {
    #[inline]
    fn raw_bytes(&self) -> &[u8] {
        self.raw_bytes()
    }

    #[inline]
    fn decode(&self) -> std::borrow::Cow<'_, str> {
        self.decode()
    }
}

pub struct Config<O: ByteOrder, D: Document> {
    _marker: PhantomData<(O, D)>,
}

impl<O: ByteOrder, D: Document> ReadableConfig for Config<O, D> {
    type ByteOrder = O;
    type Value<'doc> = ReadonlyValue<'doc, O, D>;
    type ByteArray<'doc> = ReadonlyArray<'doc, i8, D>;
    type String<'doc> = ReadonlyString<'doc, D>;
    type List<'doc> = ReadonlyList<'doc, O, D>;
    type ListIter<'doc> = ReadonlyListIter<'doc, O, D>;
    type TypedList<'doc, T: NBT> = ReadonlyTypedList<'doc, O, D, T>;
    type TypedListIter<'doc, T: NBT> = ReadonlyTypedList<'doc, O, D, T>;
    type Compound<'doc> = ReadonlyCompound<'doc, O, D>;
    type CompoundIter<'doc> = ReadonlyCompoundIter<'doc, O, D>;
    type IntArray<'doc> = ReadonlyArray<'doc, byteorder::I32<O>, D>;
    type LongArray<'doc> = ReadonlyArray<'doc, byteorder::I64<O>, D>;
}

impl<'doc, O: ByteOrder, D: Document> ScopedReadableValue<'doc> for ReadonlyValue<'doc, O, D> {
    type Config = Config<O, D>;

    #[inline]
    fn tag_id(&self) -> TagID {
        self.tag_id()
    }

    unsafe fn to_unchecked<'a, T: crate::NBT>(&'a self) -> T::Type<'a, Self::Config>
    where
        'doc: 'a,
    {
        todo!()
    }

    fn to<'a, T: crate::NBT>(&'a self) -> Option<T::Type<'a, Self::Config>>
    where
        'doc: 'a,
    {
        todo!()
    }

    fn to_readable<'a>(&'a self) -> <Self::Config as ReadableConfig>::Value<'a>
    where
        'doc: 'a,
    {
        todo!()
    }

    #[inline]
    fn get_scoped<'a, I: Index>(
        &'a self,
        index: I,
    ) -> Option<<Self::Config as ReadableConfig>::Value<'a>>
    where
        'doc: 'a,
    {
        self.get(index)
    }

    fn with<'a, R>(&'a self, match_fn: impl FnOnce(Value<'a, Self::Config>) -> R) -> R
    where
        'doc: 'a,
    {
        match self {
            ReadonlyValue::End => match_fn(Value::End),
            ReadonlyValue::Byte(value) => match_fn(Value::Byte(*value)),
            ReadonlyValue::Short(value) => match_fn(Value::Short(*value)),
            ReadonlyValue::Int(value) => match_fn(Value::Int(*value)),
            ReadonlyValue::Long(value) => match_fn(Value::Long(*value)),
            ReadonlyValue::Float(value) => match_fn(Value::Float(*value)),
            ReadonlyValue::Double(value) => match_fn(Value::Double(*value)),
            ReadonlyValue::ByteArray(value) => match_fn(Value::ByteArray(value.clone())),
            ReadonlyValue::String(value) => match_fn(Value::String(value.clone())),
            ReadonlyValue::List(value) => match_fn(Value::List(value.clone())),
            ReadonlyValue::Compound(value) => match_fn(Value::Compound(value.clone())),
            ReadonlyValue::IntArray(value) => match_fn(Value::IntArray(value.clone())),
            ReadonlyValue::LongArray(value) => match_fn(Value::LongArray(value.clone())),
        }
    }

    #[inline]
    fn write_to_vec<TARGET: ByteOrder>(&self) -> Result<Vec<u8>> {
        self.write_to_vec::<TARGET>()
    }

    #[inline]
    fn write_to_writer<TARGET: ByteOrder>(&self, writer: impl Write) -> Result<()> {
        self.write_to_writer::<TARGET>(writer)
    }
}

impl<'doc, O: ByteOrder, D: Document> ReadableValue<'doc> for ReadonlyValue<'doc, O, D> {

    unsafe fn peek_unchecked<'a, T: crate::NBT>(&'a self) -> &'a T::Type<'doc, Self::Config>
    where
        'doc: 'a {
        todo!()
    }
    
    fn peek<'a, T: crate::NBT>(&'a self) -> Option<&'a T::Type<'doc, Self::Config>>
    where
        'doc: 'a {
        todo!()
    }
    
    unsafe fn extract_unchecked<T: crate::NBT>(self) -> T::Type<'doc, Self::Config> {
        todo!()
    }
    
    fn extract<T: crate::NBT>(self) -> Option<T::Type<'doc, Self::Config>> {
        todo!()
    }
    
    #[inline]
    fn get<I: Index>(&self, index: I) -> Option<<Self::Config as ReadableConfig>::Value<'doc>> {
        self.get(index)
    }
    
    fn visit<R>(self, match_fn: impl FnOnce(Value<'doc, Self::Config>) -> R) -> R {
        todo!()
    }

}

impl<'doc, O: ByteOrder, D: Document> ScopedReadableList<'doc> for ReadonlyList<'doc, O, D> {
    type Config = Config<O, D>;

    #[inline]
    fn tag_id(&self) -> TagID {
        self.tag_id()
    }

    #[inline]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    #[inline]
    fn get_scoped<'a>(&'a self, index: usize) -> Option<<Self::Config as ReadableConfig>::Value<'a>>
    where
        'doc: 'a,
    {
        self.get(index)
    }

    #[inline]
    fn iter_scoped<'a>(&'a self) -> <Self::Config as ReadableConfig>::ListIter<'a>
    where
        'doc: 'a,
    {
        self.iter()
    }
    
    unsafe fn to_typed_list_unchecked<'a, T: crate::NBT>(
        &'a self,
    ) -> <Self::Config as ReadableConfig>::TypedList<'a, T>
    where
        'doc: 'a {
        todo!()
    }
    
    fn to_typed_list<'a, T: crate::NBT>(
        &'a self,
    ) -> Option<<Self::Config as ReadableConfig>::TypedList<'a, T>>
    where
        'doc: 'a {
        todo!()
    }

}

impl<'doc, O: ByteOrder, D: Document> ReadableList<'doc> for ReadonlyList<'doc, O, D> {
    #[inline]
    fn get(&self, index: usize) -> Option<<Self::Config as ReadableConfig>::Value<'doc>> {
        self.get(index)
    }

    #[inline]
    fn iter(&self) -> <Self::Config as ReadableConfig>::ListIter<'doc> {
        self.iter()
    }
    
    fn extract_typed_list<T: crate::NBT>(
        self,
    ) -> Option<<Self::Config as ReadableConfig>::TypedList<'doc, T>> {
        todo!()
    }

}

impl<'doc, O: ByteOrder, D: Document> ScopedReadableCompound<'doc>
    for ReadonlyCompound<'doc, O, D>
{
    type Config = Config<O, D>;

    #[inline]
    fn get_scoped<'a>(&'a self, key: &str) -> Option<<Self::Config as ReadableConfig>::Value<'a>>
    where
        'doc: 'a,
    {
        self.get(key)
    }

    #[inline]
    fn iter_scoped<'a>(&'a self) -> <Self::Config as ReadableConfig>::CompoundIter<'a>
    where
        'doc: 'a,
    {
        self.iter()
    }
}

impl<'doc, O: ByteOrder, D: Document> ReadableCompound<'doc> for ReadonlyCompound<'doc, O, D> {
    #[inline]
    fn get(&self, key: &str) -> Option<<Self::Config as ReadableConfig>::Value<'doc>> {
        self.get(key)
    }

    #[inline]
    fn iter(&self) -> <Self::Config as ReadableConfig>::CompoundIter<'doc> {
        self.iter()
    }
}
