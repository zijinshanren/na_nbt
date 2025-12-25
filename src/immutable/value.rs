use zerocopy::byteorder;

use crate::{
    ByteOrder, Document, ImmutableConfig, ImmutableGenericImpl, ImmutableNBTImpl, MapRef, Mark,
    ReadonlyArray, ReadonlyCompound, ReadonlyList, ReadonlyString, TagID, ValueBase, ValueRef,
    VisitRef,
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
            macro_rules! match_tag_id {
                (
                    [
                        $( $tag:ident ),* $(,)?
                    ], $tag_id_val:expr, $data:expr, $mark:expr
                ) => {
                    match $tag_id_val {
                        $(
                            TagID::$tag => $tag::size::<O>(data, mark),
                        )*
                    }
                };
            }

            match_tag_id!(
                [
                    End, Byte, Short, Int, Long, Float, Double, ByteArray, String, List, Compound,
                    IntArray, LongArray
                ],
                tag_id,
                data,
                mark
            )
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
}

impl<'s, O: ByteOrder, D: Document> ValueBase for ReadonlyValue<'s, O, D> {
    type ConfigRef = ImmutableConfig<O, D>;

    #[inline]
    fn tag_id(&self) -> TagID {
        self.tag_id()
    }
}

impl<'doc, O: ByteOrder, D: Document> ValueRef<'doc> for ReadonlyValue<'doc, O, D> {
    fn visit<'a, R>(&'a self, match_fn: impl FnOnce(VisitRef<'a, 'doc, Self::ConfigRef>) -> R) -> R
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

    fn map<R>(self, match_fn: impl FnOnce(MapRef<'doc, Self::ConfigRef>) -> R) -> R {
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

