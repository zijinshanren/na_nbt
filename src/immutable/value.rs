use zerocopy::byteorder;

use crate::{
    ByteOrder, ConfigRef, Document, GenericNBT, ImmutableConfig, ImmutableGenericNBTImpl,
    ImmutableNBTImpl, Index, Mark, NBT, ReadonlyArray, ReadonlyCompound, ReadonlyList,
    ReadonlyString, TagID, Value, ValueBase, ValueRef,
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
    fn default() -> Self {
        ReadonlyValue::End(())
    }
}

impl<'doc, O: ByteOrder, D: Document> ReadonlyValue<'doc, O, D> {
    pub(crate) unsafe fn read(tag_id: TagID, data: *const u8, mark: *const Mark, doc: &D) -> Self {
        unsafe {
            macro_rules! match_tag_id {
                (
                    [
                        $( $tag:ident ),* $(,)?
                    ], $tag_id_val:expr, $data:expr, $mark:expr, $doc:expr
                ) => {
                    match $tag_id_val {
                        $(
                            TagID::$tag => ReadonlyValue::$tag(
                                $tag::read::<O, D>($data, $mark, $doc)
                            ),
                        )*
                    }
                };
            }

            match_tag_id_expand!(match_tag_id, tag_id, data, mark, doc)
        }
    }

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

            match_tag_id_expand!(match_tag_id, tag_id, data, mark)
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
        self.tag_id() == T::TAG_ID
    }

    #[inline]
    pub fn ref_<'a, T: NBT>(&'a self) -> Option<&'a T::Type<'doc, ImmutableConfig<O, D>>>
    where
        'doc: 'a,
    {
        T::ref_(self)
    }

    #[inline]
    pub fn into_<T: GenericNBT>(self) -> Option<T::Type<'doc, ImmutableConfig<O, D>>> {
        T::_from(self)
    }

    #[inline]
    pub fn get_<T: GenericNBT>(
        &self,
        index: impl Index,
    ) -> Option<T::Type<'doc, ImmutableConfig<O, D>>> {
        index.index_dispatch(
            self,
            |value, index| match value {
                ReadonlyValue::List(value) => value.get_::<T>(index),
                _ => None,
            },
            |value, key| match value {
                ReadonlyValue::Compound(value) => value.get_::<T>(key),
                _ => None,
            },
        )
    }

    #[inline]
    pub fn get(&self, index: impl Index) -> Option<ReadonlyValue<'doc, O, D>> {
        index.index_dispatch(
            self,
            |value, index| match value {
                ReadonlyValue::List(value) => value.get(index),
                _ => None,
            },
            |value, key| match value {
                ReadonlyValue::Compound(value) => value.get(key),
                _ => None,
            },
        )
    }
}

impl<'doc, O: ByteOrder, D: Document> ValueBase for ReadonlyValue<'doc, O, D> {
    type ConfigRef = ImmutableConfig<O, D>;

    #[inline]
    fn tag_id(&self) -> TagID {
        self.tag_id()
    }

    #[inline]
    fn is_<T: NBT>(&self) -> bool {
        self.is_::<T>()
    }
}

impl<'doc, O: ByteOrder, D: Document> ValueRef<'doc> for ReadonlyValue<'doc, O, D> {
    #[inline]
    fn ref_<'a, T: NBT>(&'a self) -> Option<&'a T::Type<'doc, Self::ConfigRef>>
    where
        'doc: 'a,
    {
        self.ref_::<T>()
    }

    #[inline]
    fn into_<T: NBT>(self) -> Option<T::Type<'doc, Self::ConfigRef>> {
        self.into_::<T>()
    }

    #[inline]
    fn get(&self, index: impl Index) -> Option<<Self::ConfigRef as ConfigRef>::Value<'doc>> {
        self.get(index)
    }

    #[inline]
    fn get_<T: NBT>(&self, index: impl Index) -> Option<T::Type<'doc, Self::ConfigRef>> {
        self.get_::<T>(index)
    }

    fn map<R>(self, match_fn: impl FnOnce(Value<'doc, Self::ConfigRef>) -> R) -> R {
        match self {
            ReadonlyValue::End(()) => match_fn(Value::End(())),
            ReadonlyValue::Byte(value) => match_fn(Value::Byte(value)),
            ReadonlyValue::Short(value) => match_fn(Value::Short(value)),
            ReadonlyValue::Int(value) => match_fn(Value::Int(value)),
            ReadonlyValue::Long(value) => match_fn(Value::Long(value)),
            ReadonlyValue::Float(value) => match_fn(Value::Float(value)),
            ReadonlyValue::Double(value) => match_fn(Value::Double(value)),
            ReadonlyValue::ByteArray(value) => match_fn(Value::ByteArray(value)),
            ReadonlyValue::String(value) => match_fn(Value::String(value)),
            ReadonlyValue::List(value) => match_fn(Value::List(value)),
            ReadonlyValue::Compound(value) => match_fn(Value::Compound(value)),
            ReadonlyValue::IntArray(value) => match_fn(Value::IntArray(value)),
            ReadonlyValue::LongArray(value) => match_fn(Value::LongArray(value)),
        }
    }
}
