use std::marker::PhantomData;

use zerocopy::byteorder;

use crate::{
    ByteOrder, Document, GenericNBT, ImmutableConfig, ImmutableImpl, Index, MapRef, Mark, NBT,
    ReadonlyArray, ReadonlyCompound, ReadonlyList, ReadonlyString, ReadonlyTypedList, TagID,
    ValueBase, ValueRef, VisitRef,
    tag::{
        Byte, ByteArray, Compound, Double, End, Float, Int, IntArray, List, Long, LongArray, Short,
        String,
    },
};

#[derive(Clone)]
pub enum ReadonlyValue<'doc, O: ByteOrder, D: Document> {
    End(()),
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(ReadonlyArray<'doc, i8, D>),
    String(ReadonlyString<'doc, D>),
    List(ReadonlyList<'doc, O, D>),
    Compound(ReadonlyCompound<'doc, O, D>),
    IntArray(ReadonlyArray<'doc, byteorder::I32<O>, D>),
    LongArray(ReadonlyArray<'doc, byteorder::I64<O>, D>),
}

impl<'doc, O: ByteOrder, D: Document> Default for ReadonlyValue<'doc, O, D> {
    #[inline]
    fn default() -> Self {
        ReadonlyValue::End(())
    }
}

impl<'doc, O: ByteOrder, D: Document> ReadonlyValue<'doc, O, D> {
    pub(crate) unsafe fn size(tag_id: TagID, data: *const u8, mark: *const Mark) -> (usize, usize) {
        unsafe {
            match tag_id {
                TagID::End => End::size_immutable_impl::<O>(data, mark),
                TagID::Byte => Byte::size_immutable_impl::<O>(data, mark),
                TagID::Short => Short::size_immutable_impl::<O>(data, mark),
                TagID::Int => Int::size_immutable_impl::<O>(data, mark),
                TagID::Long => Long::size_immutable_impl::<O>(data, mark),
                TagID::Float => Float::size_immutable_impl::<O>(data, mark),
                TagID::Double => Double::size_immutable_impl::<O>(data, mark),
                TagID::ByteArray => ByteArray::size_immutable_impl::<O>(data, mark),
                TagID::String => String::size_immutable_impl::<O>(data, mark),
                TagID::List => List::size_immutable_impl::<O>(data, mark),
                TagID::Compound => Compound::size_immutable_impl::<O>(data, mark),
                TagID::IntArray => IntArray::size_immutable_impl::<O>(data, mark),
                TagID::LongArray => LongArray::size_immutable_impl::<O>(data, mark),
            }
        }
    }
}

impl<'doc, O: ByteOrder, D: Document> ReadonlyValue<'doc, O, D> {
    #[inline]
    pub fn tag_id(&self) -> TagID {
        match self {
            ReadonlyValue::End(_) => TagID::End,
            ReadonlyValue::Byte(_) => TagID::Byte,
            ReadonlyValue::Short(_) => TagID::Short,
            ReadonlyValue::Int(_) => TagID::Int,
            ReadonlyValue::Long(_) => TagID::Long,
            ReadonlyValue::Float(_) => TagID::Float,
            ReadonlyValue::Double(_) => TagID::Double,
            ReadonlyValue::ByteArray(_) => TagID::ByteArray,
            ReadonlyValue::String(_) => TagID::String,
            ReadonlyValue::List(_) => TagID::List,
            ReadonlyValue::Compound(_) => TagID::Compound,
            ReadonlyValue::IntArray(_) => TagID::IntArray,
            ReadonlyValue::LongArray(_) => TagID::LongArray,
        }
    }

    #[inline]
    pub fn is_<T: NBT>(&self) -> bool {
        ValueBase::is_::<T>(self)
    }

    #[inline]
    pub fn ref_<T: NBT>(&self) -> Option<&T::TypeRef<'doc, ImmutableConfig<O, D>>> {
        ValueRef::ref_::<T>(self)
    }

    #[inline]
    pub fn into_<T: GenericNBT>(self) -> Option<T::TypeRef<'doc, ImmutableConfig<O, D>>> {
        ValueRef::into_::<T>(self)
    }

    #[inline]
    pub fn get(&self, index: impl Index) -> Option<ReadonlyValue<'doc, O, D>> {
        ValueRef::get(self, index)
    }

    #[inline]
    pub fn get_<T: GenericNBT>(
        &self,
        index: impl Index,
    ) -> Option<T::TypeRef<'doc, ImmutableConfig<O, D>>> {
        ValueRef::get_::<T>(self, index)
    }
}

impl<'doc, O: ByteOrder, D: Document> ValueBase for ReadonlyValue<'doc, O, D> {
    #[inline]
    fn tag_id(&self) -> TagID {
        self.tag_id()
    }
}

impl<'doc, O: ByteOrder, D: Document> ValueRef<'doc> for ReadonlyValue<'doc, O, D> {
    type Config = ImmutableConfig<O, D>;

    fn visit<'a, R>(&'a self, match_fn: impl FnOnce(VisitRef<'a, 'doc, Self::Config>) -> R) -> R
    where
        'doc: 'a,
    {
        match self {
            ReadonlyValue::End(value) => match_fn(VisitRef::End(value)),
            ReadonlyValue::Byte(value) => match_fn(VisitRef::Byte(value)),
            ReadonlyValue::Short(value) => match_fn(VisitRef::Short(value)),
            ReadonlyValue::Int(value) => match_fn(VisitRef::Int(value)),
            ReadonlyValue::Long(value) => match_fn(VisitRef::Long(value)),
            ReadonlyValue::Float(value) => match_fn(VisitRef::Float(value)),
            ReadonlyValue::Double(value) => match_fn(VisitRef::Double(value)),
            ReadonlyValue::ByteArray(value) => match_fn(VisitRef::ByteArray(value)),
            ReadonlyValue::String(value) => match_fn(VisitRef::String(value)),
            ReadonlyValue::List(value) => match_fn(VisitRef::List(value)),
            ReadonlyValue::Compound(value) => match_fn(VisitRef::Compound(value)),
            ReadonlyValue::IntArray(value) => match_fn(VisitRef::IntArray(value)),
            ReadonlyValue::LongArray(value) => match_fn(VisitRef::LongArray(value)),
        }
    }

    fn map<R>(self, match_fn: impl FnOnce(MapRef<'doc, Self::Config>) -> R) -> R {
        match self {
            ReadonlyValue::End(()) => match_fn(MapRef::End(())),
            ReadonlyValue::Byte(value) => match_fn(MapRef::Byte(value)),
            ReadonlyValue::Short(value) => match_fn(MapRef::Short(value)),
            ReadonlyValue::Int(value) => match_fn(MapRef::Int(value)),
            ReadonlyValue::Long(value) => match_fn(MapRef::Long(value)),
            ReadonlyValue::Float(value) => match_fn(MapRef::Float(value)),
            ReadonlyValue::Double(value) => match_fn(MapRef::Double(value)),
            ReadonlyValue::ByteArray(value) => match_fn(MapRef::ByteArray(value)),
            ReadonlyValue::String(value) => match_fn(MapRef::String(value)),
            ReadonlyValue::List(value) => match_fn(MapRef::List(value)),
            ReadonlyValue::Compound(value) => match_fn(MapRef::Compound(value)),
            ReadonlyValue::IntArray(value) => match_fn(MapRef::IntArray(value)),
            ReadonlyValue::LongArray(value) => match_fn(MapRef::LongArray(value)),
        }
    }
}

impl<'doc, O: ByteOrder, D: Document> From<()> for ReadonlyValue<'doc, O, D> {
    #[inline]
    fn from(_: ()) -> Self {
        ReadonlyValue::End(())
    }
}

impl<'doc, O: ByteOrder, D: Document> From<i8> for ReadonlyValue<'doc, O, D> {
    #[inline]
    fn from(value: i8) -> Self {
        ReadonlyValue::Byte(value)
    }
}

impl<'doc, O: ByteOrder, D: Document> From<i16> for ReadonlyValue<'doc, O, D> {
    #[inline]
    fn from(value: i16) -> Self {
        ReadonlyValue::Short(value)
    }
}

impl<'doc, O: ByteOrder, D: Document> From<i32> for ReadonlyValue<'doc, O, D> {
    #[inline]
    fn from(value: i32) -> Self {
        ReadonlyValue::Int(value)
    }
}

impl<'doc, O: ByteOrder, D: Document> From<i64> for ReadonlyValue<'doc, O, D> {
    #[inline]
    fn from(value: i64) -> Self {
        ReadonlyValue::Long(value)
    }
}

impl<'doc, O: ByteOrder, D: Document> From<f32> for ReadonlyValue<'doc, O, D> {
    #[inline]
    fn from(value: f32) -> Self {
        ReadonlyValue::Float(value)
    }
}

impl<'doc, O: ByteOrder, D: Document> From<f64> for ReadonlyValue<'doc, O, D> {
    #[inline]
    fn from(value: f64) -> Self {
        ReadonlyValue::Double(value)
    }
}

impl<'doc, O: ByteOrder, D: Document> From<ReadonlyArray<'doc, i8, D>>
    for ReadonlyValue<'doc, O, D>
{
    #[inline]
    fn from(value: ReadonlyArray<'doc, i8, D>) -> Self {
        ReadonlyValue::ByteArray(value)
    }
}

impl<'doc, O: ByteOrder, D: Document> From<ReadonlyString<'doc, D>> for ReadonlyValue<'doc, O, D> {
    #[inline]
    fn from(value: ReadonlyString<'doc, D>) -> Self {
        ReadonlyValue::String(value)
    }
}

impl<'doc, O: ByteOrder, D: Document> From<ReadonlyList<'doc, O, D>> for ReadonlyValue<'doc, O, D> {
    #[inline]
    fn from(value: ReadonlyList<'doc, O, D>) -> Self {
        ReadonlyValue::List(value)
    }
}

impl<'doc, O: ByteOrder, D: Document> From<ReadonlyCompound<'doc, O, D>>
    for ReadonlyValue<'doc, O, D>
{
    #[inline]
    fn from(value: ReadonlyCompound<'doc, O, D>) -> Self {
        ReadonlyValue::Compound(value)
    }
}

impl<'doc, O: ByteOrder, D: Document> From<ReadonlyArray<'doc, byteorder::I32<O>, D>>
    for ReadonlyValue<'doc, O, D>
{
    #[inline]
    fn from(value: ReadonlyArray<'doc, byteorder::I32<O>, D>) -> Self {
        ReadonlyValue::IntArray(value)
    }
}

impl<'doc, O: ByteOrder, D: Document> From<ReadonlyArray<'doc, byteorder::I64<O>, D>>
    for ReadonlyValue<'doc, O, D>
{
    #[inline]
    fn from(value: ReadonlyArray<'doc, byteorder::I64<O>, D>) -> Self {
        ReadonlyValue::LongArray(value)
    }
}

impl<'doc, O: ByteOrder, D: Document, T: NBT> From<ReadonlyTypedList<'doc, O, D, T>>
    for ReadonlyValue<'doc, O, D>
{
    #[inline]
    fn from(value: ReadonlyTypedList<'doc, O, D, T>) -> Self {
        ReadonlyValue::List(ReadonlyList {
            data: value.data,
            mark: value.mark,
            doc: value.doc,
            _marker: PhantomData,
        })
    }
}
