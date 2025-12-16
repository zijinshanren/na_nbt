use std::{io::Write, marker::PhantomData};

use crate::{
    ByteOrder, ReadableCompound, ReadableConfig, ReadableList, ReadableString, ReadableValue,
    Result, ScopedReadableCompound, ScopedReadableList, ScopedReadableValue, Tag, Value,
    ValueScoped,
    immutable::value::{
        Document, ReadonlyCompoundIter, ReadonlyListIter, ReadonlyCompound, ReadonlyList,
        ReadonlyString, ReadonlyValue,
    },
    index::Index,
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
    type String<'doc> = ReadonlyString<'doc, D>;
    type List<'doc> = ReadonlyList<'doc, O, D>;
    type ListIter<'doc> = ReadonlyListIter<'doc, O, D>;
    type Compound<'doc> = ReadonlyCompound<'doc, O, D>;
    type CompoundIter<'doc> = ReadonlyCompoundIter<'doc, O, D>;
}

impl<'doc, O: ByteOrder, D: Document> ScopedReadableValue<'doc> for ReadonlyValue<'doc, O, D> {
    type Config = Config<O, D>;

    #[inline]
    fn tag_id(&self) -> Tag {
        self.tag_id()
    }

    #[inline]
    fn as_end(&self) -> Option<()> {
        self.as_end()
    }

    #[inline]
    fn is_end(&self) -> bool {
        self.is_end()
    }

    #[inline]
    fn as_byte(&self) -> Option<i8> {
        self.as_byte()
    }

    #[inline]
    fn is_byte(&self) -> bool {
        self.is_byte()
    }

    #[inline]
    fn as_short(&self) -> Option<i16> {
        self.as_short()
    }

    #[inline]
    fn is_short(&self) -> bool {
        self.is_short()
    }

    #[inline]
    fn as_int(&self) -> Option<i32> {
        self.as_int()
    }

    #[inline]
    fn is_int(&self) -> bool {
        self.is_int()
    }

    #[inline]
    fn as_long(&self) -> Option<i64> {
        self.as_long()
    }

    #[inline]
    fn is_long(&self) -> bool {
        self.is_long()
    }

    #[inline]
    fn as_float(&self) -> Option<f32> {
        self.as_float()
    }

    #[inline]
    fn is_float(&self) -> bool {
        self.is_float()
    }

    #[inline]
    fn as_double(&self) -> Option<f64> {
        self.as_double()
    }

    #[inline]
    fn is_double(&self) -> bool {
        self.is_double()
    }

    #[inline]
    fn as_byte_array<'a>(&'a self) -> Option<&'a [i8]>
    where
        'doc: 'a,
    {
        self.as_byte_array()
    }

    #[inline]
    fn is_byte_array(&self) -> bool {
        self.is_byte_array()
    }

    #[inline]
    fn as_string_scoped<'a>(&'a self) -> Option<<Self::Config as ReadableConfig>::String<'a>>
    where
        'doc: 'a,
    {
        self.as_string().cloned()
    }

    #[inline]
    fn is_string(&self) -> bool {
        self.is_string()
    }

    #[inline]
    fn as_list_scoped<'a>(&'a self) -> Option<<Self::Config as ReadableConfig>::List<'a>>
    where
        'doc: 'a,
    {
        self.as_list().cloned()
    }

    #[inline]
    fn is_list(&self) -> bool {
        self.is_list()
    }

    #[inline]
    fn as_compound_scoped<'a>(&'a self) -> Option<<Self::Config as ReadableConfig>::Compound<'a>>
    where
        'doc: 'a,
    {
        self.as_compound().cloned()
    }

    #[inline]
    fn is_compound(&self) -> bool {
        self.is_compound()
    }

    #[inline]
    fn as_int_array<'a>(
        &'a self,
    ) -> Option<&'a [zerocopy::byteorder::I32<<Self::Config as ReadableConfig>::ByteOrder>]>
    where
        'doc: 'a,
    {
        self.as_int_array()
    }

    #[inline]
    fn is_int_array(&self) -> bool {
        self.is_int_array()
    }

    #[inline]
    fn as_long_array<'a>(
        &'a self,
    ) -> Option<&'a [zerocopy::byteorder::I64<<Self::Config as ReadableConfig>::ByteOrder>]>
    where
        'doc: 'a,
    {
        self.as_long_array()
    }

    #[inline]
    fn is_long_array(&self) -> bool {
        self.is_long_array()
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

    fn visit_scoped<'a, R>(&'a self, match_fn: impl FnOnce(ValueScoped<'a, Self::Config>) -> R) -> R
    where
        'doc: 'a,
    {
        match self {
            ReadonlyValue::End => match_fn(ValueScoped::End),
            ReadonlyValue::Byte(value) => match_fn(ValueScoped::Byte(*value)),
            ReadonlyValue::Short(value) => match_fn(ValueScoped::Short(*value)),
            ReadonlyValue::Int(value) => match_fn(ValueScoped::Int(*value)),
            ReadonlyValue::Long(value) => match_fn(ValueScoped::Long(*value)),
            ReadonlyValue::Float(value) => match_fn(ValueScoped::Float(*value)),
            ReadonlyValue::Double(value) => match_fn(ValueScoped::Double(*value)),
            ReadonlyValue::ByteArray(value) => match_fn(ValueScoped::ByteArray(value.as_slice())),
            ReadonlyValue::String(value) => match_fn(ValueScoped::String(value.clone())),
            ReadonlyValue::List(value) => match_fn(ValueScoped::List(value.clone())),
            ReadonlyValue::Compound(value) => match_fn(ValueScoped::Compound(value.clone())),
            ReadonlyValue::IntArray(value) => match_fn(ValueScoped::IntArray(value.as_slice())),
            ReadonlyValue::LongArray(value) => match_fn(ValueScoped::LongArray(value.as_slice())),
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
    #[inline]
    fn as_string<'a>(&'a self) -> Option<&'a <Self::Config as ReadableConfig>::String<'doc>>
    where
        'doc: 'a,
    {
        self.as_string()
    }

    #[inline]
    fn as_list<'a>(&'a self) -> Option<&'a <Self::Config as ReadableConfig>::List<'doc>>
    where
        'doc: 'a,
    {
        self.as_list()
    }

    #[inline]
    fn as_compound<'a>(&'a self) -> Option<&'a <Self::Config as ReadableConfig>::Compound<'doc>>
    where
        'doc: 'a,
    {
        self.as_compound()
    }

    #[inline]
    fn get<I: Index>(&self, index: I) -> Option<<Self::Config as ReadableConfig>::Value<'doc>> {
        self.get(index)
    }

    fn visit<'a, R>(
        &'a self,
        match_fn: impl FnOnce(crate::value_trait::Value<'a, 'doc, Self::Config>) -> R,
    ) -> R
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
            ReadonlyValue::ByteArray(value) => match_fn(Value::ByteArray(value.as_slice())),
            ReadonlyValue::String(value) => match_fn(Value::String(value)),
            ReadonlyValue::List(value) => match_fn(Value::List(value)),
            ReadonlyValue::Compound(value) => match_fn(Value::Compound(value)),
            ReadonlyValue::IntArray(value) => match_fn(Value::IntArray(value.as_slice())),
            ReadonlyValue::LongArray(value) => match_fn(Value::LongArray(value.as_slice())),
        }
    }
}

impl<'doc, O: ByteOrder, D: Document> ScopedReadableList<'doc> for ReadonlyList<'doc, O, D> {
    type Config = Config<O, D>;

    #[inline]
    fn tag_id(&self) -> Tag {
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
