use std::{marker::PhantomData, ptr};

use zerocopy::byteorder;

use crate::{
    ByteOrder, Document, EMPTY_LIST, GenericNBT, ImmutableConfig, ImmutableGenericNBTImpl, Mark,
    NBT, Never, ReadableList, ReadableTypedList, ReadonlyValue, ScopedReadableList,
    ScopedReadableTypedList, TagID, cold_path,
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
            tag_id: self.tag_id(),
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
    pub fn tag_id(&self) -> TagID {
        unsafe { *self.data.as_ptr().cast() }
    }

    #[inline]
    pub fn is<T: NBT>(&self) -> bool {
        self.tag_id() == T::TAG_ID
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

    /// .
    ///
    /// # Safety
    ///
    /// .
    pub unsafe fn get_unchecked_<T: GenericNBT>(
        &self,
        index: usize,
    ) -> Option<T::Type<'doc, ImmutableConfig<O, D>>> {
        if index >= self.len() {
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

    pub fn get_<T: GenericNBT>(
        &self,
        index: usize,
    ) -> Option<T::Type<'doc, ImmutableConfig<O, D>>> {
        if index >= self.len() {
            cold_path();
            return None;
        }

        if self.tag_id() != T::TAG_ID {
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
                        $( ($tag_id:ident, $tag_type:ident) ),* $(,)?
                    ], $tag_id_val:expr, $ptr:expr, $index:expr, $doc:expr, $mark:expr
                ) => {
                    match $tag_id_val {
                        $(
                            TagID::$tag_id => Some(ReadonlyValue::$tag_id(
                                $tag_type::get_index_unchecked::<O, D>($ptr, $index, $doc, $mark).unwrap_unchecked()
                            )),
                        )*
                    }
                };
            }

            match_tag_id_expand!(match_tag_id, self.tag_id(), ptr, index, &self.doc, mark)
        }
    }

    /// Returns an iterator over the elements of this list.
    #[inline]
    pub fn iter(&self) -> ReadonlyListIter<'doc, O, D> {
        ReadonlyListIter {
            tag_id: self.tag_id(),
            remaining: self.len() as u32,
            data: unsafe { self.data.as_ptr().add(1 + 4) },
            mark: self.mark,
            doc: self.doc.clone(),
            _marker: PhantomData,
        }
    }

    /// .
    ///
    /// # Safety
    ///
    /// .
    #[inline]
    pub unsafe fn extract_typed_list_unchecked<T: NBT>(self) -> ReadonlyTypedList<'doc, O, D, T> {
        unsafe { self.extract_typed_list().unwrap_unchecked() }
    }

    #[inline]
    pub fn extract_typed_list<T: NBT>(self) -> Option<ReadonlyTypedList<'doc, O, D, T>> {
        self.is::<T>().then_some(ReadonlyTypedList {
            length: self.len() as u32,
            data: unsafe { self.data.as_ptr().add(1 + 4) },
            mark: self.mark,
            doc: self.doc,
            _marker: PhantomData,
        })
    }
}

impl<'doc, O: ByteOrder, D: Document> ScopedReadableList<'doc> for ReadonlyList<'doc, O, D> {
    type Config = ImmutableConfig<O, D>;

    #[inline]
    fn tag_id(&self) -> TagID {
        self.tag_id()
    }

    #[inline]
    fn is<T: NBT>(&self) -> bool {
        self.is::<T>()
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
    unsafe fn at_unchecked_<'a, T: GenericNBT>(
        &'a self,
        index: usize,
    ) -> Option<T::Type<'a, Self::Config>>
    where
        'doc: 'a,
    {
        unsafe { self.get_unchecked_::<T>(index) }
    }

    #[inline]
    fn at_<'a, T: GenericNBT>(&'a self, index: usize) -> Option<T::Type<'a, Self::Config>>
    where
        'doc: 'a,
    {
        self.get_::<T>(index)
    }

    #[inline]
    fn at<'a>(
        &'a self,
        index: usize,
    ) -> Option<<Self::Config as crate::ReadableConfig>::Value<'a>>
    where
        'doc: 'a,
    {
        self.get(index)
    }

    #[inline]
    fn iter_scoped<'a>(&'a self) -> <Self::Config as crate::ReadableConfig>::ListIter<'a>
    where
        'doc: 'a,
    {
        self.iter()
    }

    #[inline]
    unsafe fn to_typed_list_unchecked<'a, T: NBT>(
        &'a self,
    ) -> <Self::Config as crate::ReadableConfig>::TypedList<'a, T>
    where
        'doc: 'a,
    {
        unsafe { self.to_typed_().unwrap_unchecked() }
    }

    #[inline]
    fn to_typed_<'a, T: NBT>(
        &'a self,
    ) -> Option<<Self::Config as crate::ReadableConfig>::TypedList<'a, T>>
    where
        'doc: 'a,
    {
        self.clone().extract_typed_list::<T>()
    }
}

impl<'doc, O: ByteOrder, D: Document> ReadableList<'doc> for ReadonlyList<'doc, O, D> {
    #[inline]
    unsafe fn get_unchecked_<T: GenericNBT>(
        &self,
        index: usize,
    ) -> Option<T::Type<'doc, Self::Config>> {
        unsafe { self.get_unchecked_::<T>(index) }
    }

    #[inline]
    fn get_<T: GenericNBT>(&self, index: usize) -> Option<T::Type<'doc, Self::Config>> {
        self.get_::<T>(index)
    }

    #[inline]
    fn get(&self, index: usize) -> Option<<Self::Config as crate::ReadableConfig>::Value<'doc>> {
        self.get(index)
    }

    #[inline]
    fn iter(&self) -> <Self::Config as crate::ReadableConfig>::ListIter<'doc> {
        self.iter()
    }

    #[inline]
    unsafe fn extract_typed_list_unchecked<T: NBT>(
        self,
    ) -> <Self::Config as crate::ReadableConfig>::TypedList<'doc, T> {
        unsafe { self.extract_typed_list_unchecked::<T>() }
    }

    #[inline]
    fn into_typed_<T: NBT>(
        self,
    ) -> Option<<Self::Config as crate::ReadableConfig>::TypedList<'doc, T>> {
        self.extract_typed_list::<T>()
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

#[derive(Clone)]
pub struct ReadonlyTypedList<'doc, O: ByteOrder, D: Document, T: NBT> {
    pub(crate) length: u32,
    pub(crate) data: *const u8,
    pub(crate) mark: *const Mark,
    pub(crate) doc: D,
    pub(crate) _marker: PhantomData<(&'doc (), O, T)>,
}

impl<'doc, O: ByteOrder, D: Document, T: NBT> Default for ReadonlyTypedList<'doc, O, D, T> {
    fn default() -> Self {
        Self {
            length: 0,
            data: ptr::null(),
            mark: ptr::null(),
            doc: unsafe { Never::never() },
            _marker: PhantomData,
        }
    }
}

unsafe impl<'doc, O: ByteOrder, D: Document, T: NBT> Send for ReadonlyTypedList<'doc, O, D, T> {}
unsafe impl<'doc, O: ByteOrder, D: Document, T: NBT> Sync for ReadonlyTypedList<'doc, O, D, T> {}

impl<'doc, O: ByteOrder, D: Document, T: NBT> ReadonlyTypedList<'doc, O, D, T> {
    #[inline]
    pub fn len(&self) -> usize {
        self.length as usize
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<T::Type<'doc, ImmutableConfig<O, D>>> {
        if index >= self.len() {
            cold_path();
            return None;
        }

        unsafe { T::get_index_unchecked::<O, D>(self.data, index, &self.doc, self.mark) }
    }

    #[inline]
    pub fn iter(&self) -> ReadonlyTypedList<'doc, O, D, T> {
        self.clone()
    }
}

impl<'doc, O: ByteOrder, D: Document, T: NBT> ScopedReadableTypedList<'doc, T>
    for ReadonlyTypedList<'doc, O, D, T>
{
    type Config = ImmutableConfig<O, D>;

    #[inline]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    #[inline]
    fn at<'a>(&'a self, index: usize) -> Option<<T>::Type<'a, Self::Config>>
    where
        'doc: 'a,
    {
        self.get(index)
    }

    #[inline]
    fn iter_scoped<'a>(&'a self) -> <Self::Config as crate::ReadableConfig>::TypedListIter<'a, T>
    where
        'doc: 'a,
    {
        self.iter()
    }
}

impl<'doc, O: ByteOrder, D: Document, T: NBT> ReadableTypedList<'doc, T>
    for ReadonlyTypedList<'doc, O, D, T>
{
    #[inline]
    fn get(&self, index: usize) -> Option<<T>::Type<'doc, Self::Config>> {
        self.get(index)
    }

    #[inline]
    fn iter(&self) -> <Self::Config as crate::ReadableConfig>::TypedListIter<'doc, T> {
        self.iter()
    }
}

impl<'doc, O: ByteOrder, D: Document, T: NBT> Iterator for ReadonlyTypedList<'doc, O, D, T> {
    type Item = T::Type<'doc, ImmutableConfig<O, D>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.length == 0 {
            cold_path();
            return None;
        }

        self.length -= 1;

        let value = unsafe { T::read(self.data, self.mark, &self.doc) };

        let (data_advance, mark_advance) = unsafe { T::size::<O>(self.data, self.mark) };
        self.data = unsafe { self.data.add(data_advance) };
        self.mark = unsafe { self.mark.add(mark_advance) };

        Some(value)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.length as usize;
        (remaining, Some(remaining))
    }
}

impl<'doc, O: ByteOrder, D: Document, T: NBT> ExactSizeIterator
    for ReadonlyTypedList<'doc, O, D, T>
{
}
