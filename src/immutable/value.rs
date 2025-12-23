use zerocopy::byteorder;

use crate::{
    ByteOrder, Document, GenericNBT, ImmutableConfig, ImmutableGenericNBTImpl, ImmutableNBTImpl,
    Index, Mark, NBT, ReadableValue, ReadonlyArray, ReadonlyCompound, ReadonlyList, ReadonlyString,
    ScopedReadableValue, TagID, Value,
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
                        $( ($tag_id:ident, $tag_type:ident) ),* $(,)?
                    ], $tag_id_val:expr, $data:expr, $mark:expr, $doc:expr
                ) => {
                    match $tag_id_val {
                        $(
                            TagID::$tag_id => ReadonlyValue::$tag_id(
                                $tag_type::read::<O, D>($data, $mark, $doc)
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
                        $( ($tag_id:ident, $tag_type:ident) ),* $(,)?
                    ], $tag_id_val:expr, $data:expr, $mark:expr
                ) => {
                    match $tag_id_val {
                        $(
                            TagID::$tag_id => $tag_type::size::<O>(data, mark),
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
    pub fn is<T: NBT>(&self) -> bool {
        self.tag_id() == T::TAG_ID
    }

    /// Returns a reference to the peek unchecked of this [`ReadonlyValue<O, D>`].
    ///
    /// # Safety
    ///
    /// .
    #[inline]
    pub unsafe fn peek_unchecked<'a, T: NBT>(&'a self) -> &'a T::Type<'doc, ImmutableConfig<O, D>>
    where
        'doc: 'a,
    {
        unsafe { self.peek::<T>().unwrap_unchecked() }
    }

    #[inline]
    pub fn peek<'a, T: NBT>(&'a self) -> Option<&'a T::Type<'doc, ImmutableConfig<O, D>>>
    where
        'doc: 'a,
    {
        T::peek(self)
    }

    /// .
    ///
    /// # Safety
    ///
    /// .
    #[inline]
    pub unsafe fn extract_unchecked<T: GenericNBT>(self) -> T::Type<'doc, ImmutableConfig<O, D>> {
        unsafe { self.extract::<T>().unwrap_unchecked() }
    }

    #[inline]
    pub fn extract<T: GenericNBT>(self) -> Option<T::Type<'doc, ImmutableConfig<O, D>>> {
        T::extract(self)
    }

    /// .
    ///
    /// # Safety
    ///
    /// .
    #[inline]
    pub unsafe fn get_typed_unchecked<T: GenericNBT>(
        &self,
        index: impl Index,
    ) -> Option<T::Type<'doc, ImmutableConfig<O, D>>> {
        index.index_dispatch(
            self,
            |value, index| match value {
                ReadonlyValue::List(value) => unsafe { value.get_typed_unchecked::<T>(index) },
                _ => None,
            },
            |value, key| match value {
                ReadonlyValue::Compound(value) => unsafe { value.get_typed_unchecked::<T>(key) },
                _ => None,
            },
        )
    }

    #[inline]
    pub fn get_typed<T: GenericNBT>(
        &self,
        index: impl Index,
    ) -> Option<T::Type<'doc, ImmutableConfig<O, D>>> {
        index.index_dispatch(
            self,
            |value, index| match value {
                ReadonlyValue::List(value) => value.get_typed::<T>(index),
                _ => None,
            },
            |value, key| match value {
                ReadonlyValue::Compound(value) => value.get_typed::<T>(key),
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

impl<'doc, O: ByteOrder, D: Document> ScopedReadableValue<'doc> for ReadonlyValue<'doc, O, D> {
    type Config = ImmutableConfig<O, D>;

    #[inline]
    fn tag_id(&self) -> TagID {
        self.tag_id()
    }

    #[inline]
    unsafe fn to_unchecked<'a, T: GenericNBT>(&'a self) -> T::Type<'a, Self::Config>
    where
        'doc: 'a,
    {
        unsafe { self.clone().extract_unchecked::<T>() }
    }

    #[inline]
    fn to<'a, T: GenericNBT>(&'a self) -> Option<T::Type<'a, Self::Config>>
    where
        'doc: 'a,
    {
        self.clone().extract::<T>()
    }

    #[inline]
    fn is<T: NBT>(&self) -> bool {
        self.is::<T>()
    }

    #[inline]
    fn to_readable<'a>(&'a self) -> <Self::Config as crate::ReadableConfig>::Value<'a>
    where
        'doc: 'a,
    {
        self.clone()
    }

    #[inline]
    fn get_scoped<'a>(
        &'a self,
        index: impl Index,
    ) -> Option<<Self::Config as crate::ReadableConfig>::Value<'a>>
    where
        'doc: 'a,
    {
        self.get(index)
    }

    #[inline]
    unsafe fn get_typed_unchecked_scoped<'a, T: GenericNBT>(
        &'a self,
        index: impl Index,
    ) -> Option<T::Type<'a, Self::Config>>
    where
        'doc: 'a,
    {
        unsafe { self.get_typed_unchecked::<T>(index) }
    }

    #[inline]
    fn get_typed_scoped<'a, T: GenericNBT>(
        &'a self,
        index: impl Index,
    ) -> Option<T::Type<'a, Self::Config>>
    where
        'doc: 'a,
    {
        self.get_typed::<T>(index)
    }

    fn with<'a, R>(&'a self, match_fn: impl FnOnce(crate::Value<'a, Self::Config>) -> R) -> R
    where
        'doc: 'a,
    {
        match self {
            ReadonlyValue::End(()) => match_fn(Value::End(())),
            ReadonlyValue::Byte(v) => match_fn(Value::Byte(*v)),
            ReadonlyValue::Short(v) => match_fn(Value::Short(*v)),
            ReadonlyValue::Int(v) => match_fn(Value::Int(*v)),
            ReadonlyValue::Long(v) => match_fn(Value::Long(*v)),
            ReadonlyValue::Float(v) => match_fn(Value::Float(*v)),
            ReadonlyValue::Double(v) => match_fn(Value::Double(*v)),
            ReadonlyValue::ByteArray(v) => match_fn(Value::ByteArray(v.clone())),
            ReadonlyValue::String(v) => match_fn(Value::String(v.clone())),
            ReadonlyValue::List(v) => match_fn(Value::List(v.clone())),
            ReadonlyValue::Compound(v) => match_fn(Value::Compound(v.clone())),
            ReadonlyValue::IntArray(v) => match_fn(Value::IntArray(v.clone())),
            ReadonlyValue::LongArray(v) => match_fn(Value::LongArray(v.clone())),
        }
    }
}

impl<'doc, O: ByteOrder, D: Document> ReadableValue<'doc> for ReadonlyValue<'doc, O, D> {
    #[inline]
    unsafe fn peek_unchecked<'a, T: NBT>(&'a self) -> &'a T::Type<'doc, Self::Config>
    where
        'doc: 'a,
    {
        unsafe { self.peek_unchecked::<T>() }
    }

    #[inline]
    fn peek<'a, T: NBT>(&'a self) -> Option<&'a T::Type<'doc, Self::Config>>
    where
        'doc: 'a,
    {
        self.peek::<T>()
    }

    #[inline]
    unsafe fn extract_unchecked<T: GenericNBT>(self) -> T::Type<'doc, Self::Config> {
        unsafe { self.extract_unchecked::<T>() }
    }

    #[inline]
    fn extract<T: GenericNBT>(self) -> Option<T::Type<'doc, Self::Config>> {
        self.extract::<T>()
    }

    #[inline]
    fn get(
        &self,
        index: impl Index,
    ) -> Option<<Self::Config as crate::ReadableConfig>::Value<'doc>> {
        self.get(index)
    }

    #[inline]
    unsafe fn get_typed_unchecked<T: GenericNBT>(
        &self,
        index: impl Index,
    ) -> Option<T::Type<'doc, Self::Config>> {
        unsafe { self.get_typed_unchecked::<T>(index) }
    }

    #[inline]
    fn get_typed<T: GenericNBT>(&self, index: impl Index) -> Option<T::Type<'doc, Self::Config>> {
        self.get_typed::<T>(index)
    }

    fn visit<R>(self, match_fn: impl FnOnce(Value<'doc, Self::Config>) -> R) -> R {
        match self {
            ReadonlyValue::End(v) => match_fn(Value::End(v)),
            ReadonlyValue::Byte(v) => match_fn(Value::Byte(v)),
            ReadonlyValue::Short(v) => match_fn(Value::Short(v)),
            ReadonlyValue::Int(v) => match_fn(Value::Int(v)),
            ReadonlyValue::Long(v) => match_fn(Value::Long(v)),
            ReadonlyValue::Float(v) => match_fn(Value::Float(v)),
            ReadonlyValue::Double(v) => match_fn(Value::Double(v)),
            ReadonlyValue::ByteArray(v) => match_fn(Value::ByteArray(v)),
            ReadonlyValue::String(v) => match_fn(Value::String(v)),
            ReadonlyValue::List(v) => match_fn(Value::List(v)),
            ReadonlyValue::Compound(v) => match_fn(Value::Compound(v)),
            ReadonlyValue::IntArray(v) => match_fn(Value::IntArray(v)),
            ReadonlyValue::LongArray(v) => match_fn(Value::LongArray(v)),
        }
    }
}
