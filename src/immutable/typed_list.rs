use std::marker::PhantomData;

use zerocopy::byteorder;

use crate::{
    ByteOrder, ReadableByteList, ReadableDoubleList, ReadableEndList, ReadableFloatList,
    ReadableIntList, ReadableLongList, ReadableShortList, ScopedReadableByteList,
    ScopedReadableDoubleList, ScopedReadableEndList, ScopedReadableFloatList,
    ScopedReadableIntList, ScopedReadableLongList, ScopedReadableShortList, cold_path,
    immutable::{trait_impl::Config, value::Document},
};

#[derive(Clone)]
pub struct ReadonlyPrimitiveList<
    'doc,
    O: ByteOrder,
    D: Document,
    T: Sized + Clone,
    X: Sized + From<T>,
> {
    pub(crate) data: &'doc [T],
    pub(crate) _doc: D,
    pub(crate) _marker: PhantomData<(O, X)>,
}

impl<'doc, O: ByteOrder, D: Document, T: Sized + Clone, X: Sized + From<T>> Default
    for ReadonlyPrimitiveList<'doc, O, D, T, X>
{
    fn default() -> Self {
        Self {
            data: &[],
            _doc: unsafe { D::never() },
            _marker: PhantomData,
        }
    }
}

impl<'doc, O: ByteOrder, D: Document, T: Sized + Clone, X: Sized + From<T>> Iterator
    for ReadonlyPrimitiveList<'doc, O, D, T, X>
{
    type Item = X;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.data.is_empty() {
            cold_path();
            return None;
        }

        let item = self.data[0].clone().into();
        self.data = &self.data[1..];
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.data.len();
        (len, Some(len))
    }
}

impl<'doc, O: ByteOrder, D: Document, T: Sized + Clone, X: Sized + From<T>> ExactSizeIterator
    for ReadonlyPrimitiveList<'doc, O, D, T, X>
{
}

macro_rules! readonly_primitive_list {
    ($scoped:ident, $readable:ident, $type:ty) => {
        impl<'doc, O: ByteOrder, D: Document> $scoped<'doc>
            for ReadonlyPrimitiveList<'doc, O, D, $type, $type>
        {
            type Config = Config<O, D>;

            #[inline]
            fn len(&self) -> usize {
                self.data.len()
            }

            #[inline]
            fn is_empty(&self) -> bool {
                self.data.is_empty()
            }

            #[inline]
            fn get(&self, index: usize) -> Option<$type> {
                self.data.get(index).cloned()
            }

            #[inline]
            fn iter_scoped<'a>(&'a self) -> Self
            where
                'doc: 'a,
            {
                self.clone()
            }
        }

        impl<'doc, O: ByteOrder, D: Document> $readable<'doc>
            for ReadonlyPrimitiveList<'doc, O, D, $type, $type>
        {
            #[inline]
            fn iter(&self) -> Self {
                self.clone()
            }
        }
    };
    ($scoped:ident, $readable:ident, $type:ty, $convert:ty) => {
        impl<'doc, O: ByteOrder, D: Document> $scoped<'doc>
            for ReadonlyPrimitiveList<'doc, O, D, $type, $convert>
        {
            type Config = Config<O, D>;

            #[inline]
            fn len(&self) -> usize {
                self.data.len()
            }

            #[inline]
            fn is_empty(&self) -> bool {
                self.data.is_empty()
            }

            #[inline]
            fn get(&self, index: usize) -> Option<$convert> {
                self.data.get(index).map(|x| x.get())
            }

            #[inline]
            fn iter_scoped<'a>(&'a self) -> Self
            where
                'doc: 'a,
            {
                self.clone()
            }
        }

        impl<'doc, O: ByteOrder, D: Document> $readable<'doc>
            for ReadonlyPrimitiveList<'doc, O, D, $type, $convert>
        {
            #[inline]
            fn iter(&self) -> Self {
                self.clone()
            }
        }
    };
}

readonly_primitive_list!(ScopedReadableEndList, ReadableEndList, ());
readonly_primitive_list!(ScopedReadableByteList, ReadableByteList, i8);
readonly_primitive_list!(
    ScopedReadableShortList,
    ReadableShortList,
    byteorder::I16<O>,
    i16
);
readonly_primitive_list!(
    ScopedReadableIntList,
    ReadableIntList,
    byteorder::I32<O>,
    i32
);
readonly_primitive_list!(
    ScopedReadableLongList,
    ReadableLongList,
    byteorder::I64<O>,
    i64
);
readonly_primitive_list!(
    ScopedReadableFloatList,
    ReadableFloatList,
    byteorder::F32<O>,
    f32
);
readonly_primitive_list!(
    ScopedReadableDoubleList,
    ReadableDoubleList,
    byteorder::F64<O>,
    f64
);
