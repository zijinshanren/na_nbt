use std::{marker::PhantomData, ptr};

use zerocopy::byteorder;

use crate::{
    ByteOrder, ConfigRef, Document, EMPTY_LIST, GenericNBT, ImmutableConfig,
    ImmutableGenericNBTImpl, ListBase, ListRef, Mark, NBT, Never, ReadonlyTypedList, ReadonlyValue,
    TagID, cold_path,
    tag::{
        Byte, ByteArray, Compound, Double, End, Float, Int, IntArray, List, Long, LongArray, Short,
        String,
    },
};

#[derive(Clone)]
pub struct ReadonlyList<'doc, O: ByteOrder, D: Document> {
    pub(crate) data: &'doc [u8],
    pub(crate) mark: *const Mark,
    pub(crate) doc: D,
    pub(crate) _marker: PhantomData<O>,
}

impl<'doc, O: ByteOrder, D: Document> Default for ReadonlyList<'doc, O, D> {
    #[inline]
    fn default() -> Self {
        Self {
            data: &EMPTY_LIST,
            mark: ptr::null(),
            doc: unsafe { Never::never() },
            _marker: PhantomData,
        }
    }
}

unsafe impl<'doc, O: ByteOrder, D: Document> Send for ReadonlyList<'doc, O, D> {}
unsafe impl<'doc, O: ByteOrder, D: Document> Sync for ReadonlyList<'doc, O, D> {}

impl<'doc, O: ByteOrder, D: Document> IntoIterator for ReadonlyList<'doc, O, D> {
    type Item = ReadonlyValue<'doc, O, D>;
    type IntoIter = ReadonlyListIter<'doc, O, D>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        ReadonlyListIter {
            tag_id: self.element_tag_id(),
            remaining: self.len() as u32,
            data: unsafe { self.data.as_ptr().add(1 + 4) },
            mark: self.mark,
            doc: self.doc,
            _marker: PhantomData,
        }
    }
}

impl<'doc, O: ByteOrder, D: Document> ReadonlyList<'doc, O, D> {
    #[inline]
    pub fn element_tag_id(&self) -> TagID {
        unsafe { *self.data.as_ptr().cast() }
    }

    #[inline]
    pub fn element_is_<T: NBT>(&self) -> bool {
        self.element_tag_id() == T::TAG_ID
            || (self.element_tag_id() == TagID::End && self.is_empty())
    }

    /// Returns the number of elements in this list.
    #[inline]
    pub fn len(&self) -> usize {
        unsafe { byteorder::U32::<O>::from_bytes(*self.data.as_ptr().add(1).cast()).get() as usize }
    }

    /// Returns `true` if this list contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get_<T: GenericNBT>(
        &self,
        index: usize,
    ) -> Option<T::TypeRef<'doc, ImmutableConfig<O, D>>> {
        if index >= self.len() {
            cold_path();
            return None;
        }

        if self.element_tag_id() != T::TAG_ID {
            cold_path();
            return None;
        }

        unsafe {
            T::get_index_unchecked::<O, D>(
                self.data.as_ptr().add(1 + 4),
                index,
                &self.doc,
                self.mark,
            )
        }
    }

    /// Returns the element at the given index, or `None` if out of bounds.
    pub fn get(&self, index: usize) -> Option<ReadonlyValue<'doc, O, D>> {
        if index >= self.len() {
            cold_path();
            return None;
        }

        unsafe {
            let ptr = self.data.as_ptr().add(1 + 4);
            let mark = self.mark;

            macro_rules! match_tag_id {
                (
                    [
                        $( $tag:ident ),* $(,)?
                    ], $tag_id_val:expr, $ptr:expr, $index:expr, $doc:expr, $mark:expr
                ) => {
                    match $tag_id_val {
                        $(
                            TagID::$tag => Some(ReadonlyValue::$tag(
                                $tag::get_index_unchecked::<O, D>($ptr, $index, $doc, $mark).unwrap_unchecked()
                            )),
                        )*
                    }
                };
            }

            match_tag_id!(
                [
                    End, Byte, Short, Int, Long, Float, Double, ByteArray, String, List, Compound,
                    IntArray, LongArray
                ],
                self.element_tag_id(),
                ptr,
                index,
                &self.doc,
                mark
            )
        }
    }

    /// Returns an iterator over the elements of this list.
    #[inline]
    pub fn iter(&self) -> ReadonlyListIter<'doc, O, D> {
        ReadonlyListIter {
            tag_id: self.element_tag_id(),
            remaining: self.len() as u32,
            data: unsafe { self.data.as_ptr().add(1 + 4) },
            mark: self.mark,
            doc: self.doc.clone(),
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn typed_<T: NBT>(self) -> Option<ReadonlyTypedList<'doc, O, D, T>> {
        self.element_is_::<T>().then_some(ReadonlyTypedList {
            data: self.data,
            mark: self.mark,
            doc: self.doc,
            _marker: PhantomData,
        })
    }
}

impl<'doc, O: ByteOrder, D: Document> ListBase for ReadonlyList<'doc, O, D> {
    #[inline]
    fn element_tag_id(&self) -> TagID {
        self.element_tag_id()
    }

    #[inline]
    fn element_is_<T: NBT>(&self) -> bool {
        self.element_is_::<T>()
    }

    #[inline]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl<'doc, O: ByteOrder, D: Document> ListRef<'doc> for ReadonlyList<'doc, O, D> {
    type Config = ImmutableConfig<O, D>;

    #[inline]
    fn get(&self, index: usize) -> Option<<Self::Config as ConfigRef>::Value<'doc>> {
        self.get(index)
    }

    #[inline]
    fn get_<T: NBT>(&self, index: usize) -> Option<T::TypeRef<'doc, Self::Config>> {
        self.get_::<T>(index)
    }

    #[inline]
    fn typed_<T: NBT>(self) -> Option<<Self::Config as ConfigRef>::TypedList<'doc, T>> {
        self.typed_::<T>()
    }

    #[inline]
    fn iter(&self) -> <Self::Config as ConfigRef>::ListIter<'doc> {
        self.iter()
    }
}

#[derive(Clone)]
pub struct ReadonlyListIter<'doc, O: ByteOrder, D: Document> {
    tag_id: TagID,
    remaining: u32,
    data: *const u8,
    mark: *const Mark,
    doc: D,
    _marker: PhantomData<(&'doc (), O)>,
}

impl<'doc, O: ByteOrder, D: Document> Default for ReadonlyListIter<'doc, O, D> {
    #[inline]
    fn default() -> Self {
        Self {
            tag_id: TagID::End,
            remaining: 0,
            data: ptr::null(),
            mark: ptr::null(),
            doc: unsafe { Never::never() },
            _marker: PhantomData,
        }
    }
}

unsafe impl<'doc, O: ByteOrder, D: Document> Send for ReadonlyListIter<'doc, O, D> {}
unsafe impl<'doc, O: ByteOrder, D: Document> Sync for ReadonlyListIter<'doc, O, D> {}

impl<'doc, O: ByteOrder, D: Document> Iterator for ReadonlyListIter<'doc, O, D> {
    type Item = ReadonlyValue<'doc, O, D>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            cold_path();
            return None;
        }

        self.remaining -= 1;

        let value = unsafe { ReadonlyValue::read(self.tag_id, self.data, self.mark, &self.doc) };

        let (data_advance, mark_advance) =
            unsafe { ReadonlyValue::<O, D>::size(self.tag_id, self.data, self.mark) };
        self.data = unsafe { self.data.add(data_advance) };
        self.mark = unsafe { self.mark.add(mark_advance) };

        Some(value)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.remaining as usize;
        (remaining, Some(remaining))
    }
}

impl<'doc, O: ByteOrder, D: Document> ExactSizeIterator for ReadonlyListIter<'doc, O, D> {}
