use std::{marker::PhantomData, ptr};

use zerocopy::byteorder;

use crate::{
    ByteOrder, ConfigRef, Document, EMPTY_LIST, ImmutableConfig, Mark, NBT, Never, ReadonlyValue,
    TypedListBase, TypedListRef, cold_path,
};

#[derive(Clone)]
pub struct ReadonlyTypedList<'doc, O: ByteOrder, D: Document, T: NBT> {
    pub(crate) data: &'doc [u8],
    pub(crate) mark: *const Mark,
    pub(crate) doc: D,
    pub(crate) _marker: PhantomData<(O, T)>,
}

impl<'doc, O: ByteOrder, D: Document, T: NBT> Default for ReadonlyTypedList<'doc, O, D, T> {
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

unsafe impl<'doc, O: ByteOrder, D: Document, T: NBT> Send for ReadonlyTypedList<'doc, O, D, T> {}
unsafe impl<'doc, O: ByteOrder, D: Document, T: NBT> Sync for ReadonlyTypedList<'doc, O, D, T> {}

impl<'doc, O: ByteOrder, D: Document, T: NBT> IntoIterator for ReadonlyTypedList<'doc, O, D, T> {
    type Item = T::TypeRef<'doc, ImmutableConfig<O, D>>;
    type IntoIter = ReadonlyTypedListIter<'doc, O, D, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        ReadonlyTypedListIter {
            remaining: self.len() as u32,
            data: unsafe { self.data.as_ptr().add(1 + 4) },
            mark: self.mark,
            doc: self.doc,
            _marker: PhantomData,
        }
    }
}

impl<'doc, O: ByteOrder, D: Document, T: NBT> ReadonlyTypedList<'doc, O, D, T> {
    #[inline]
    pub fn len(&self) -> usize {
        unsafe { byteorder::U32::<O>::from_bytes(*self.data.as_ptr().add(1).cast()).get() as usize }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        TypedListBase::is_empty(self)
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<T::TypeRef<'doc, ImmutableConfig<O, D>>> {
        TypedListRef::get(self, index)
    }

    #[inline]
    pub fn iter(&self) -> ReadonlyTypedListIter<'doc, O, D, T> {
        ReadonlyTypedListIter {
            remaining: self.len() as u32,
            data: unsafe { self.data.as_ptr().add(1 + 4) },
            mark: self.mark,
            doc: self.doc.clone(),
            _marker: PhantomData,
        }
    }
}

impl<'doc, O: ByteOrder, D: Document, T: NBT> TypedListBase<T>
    for ReadonlyTypedList<'doc, O, D, T>
{
    #[inline]
    fn len(&self) -> usize {
        self.len()
    }
}

impl<'doc, O: ByteOrder, D: Document, T: NBT> TypedListRef<'doc, T>
    for ReadonlyTypedList<'doc, O, D, T>
{
    type Config = ImmutableConfig<O, D>;

    fn _to_read_params<'a>(&'a self) -> <Self::Config as ConfigRef>::ReadParams<'a>
    where
        'doc: 'a,
    {
        (
            unsafe { self.data.as_ptr().add(1 + 4) },
            self.mark,
            &self.doc,
        )
    }

    #[inline]
    fn iter(&self) -> <Self::Config as ConfigRef>::TypedListIter<'doc, T> {
        self.iter()
    }
}

#[derive(Clone)]
pub struct ReadonlyTypedListIter<'doc, O: ByteOrder, D: Document, T: NBT> {
    remaining: u32,
    data: *const u8,
    mark: *const Mark,
    doc: D,
    _marker: PhantomData<(&'doc (), O, T)>,
}

impl<'doc, O: ByteOrder, D: Document, T: NBT> Default for ReadonlyTypedListIter<'doc, O, D, T> {
    #[inline]
    fn default() -> Self {
        Self {
            remaining: 0,
            data: ptr::null(),
            mark: ptr::null(),
            doc: unsafe { Never::never() },
            _marker: PhantomData,
        }
    }
}

unsafe impl<'doc, O: ByteOrder, D: Document, T: NBT> Send for ReadonlyTypedListIter<'doc, O, D, T> {}
unsafe impl<'doc, O: ByteOrder, D: Document, T: NBT> Sync for ReadonlyTypedListIter<'doc, O, D, T> {}

impl<'doc, O: ByteOrder, D: Document, T: NBT> Iterator for ReadonlyTypedListIter<'doc, O, D, T> {
    type Item = T::TypeRef<'doc, ImmutableConfig<O, D>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            cold_path();
            return None;
        }

        self.remaining -= 1;

        let value = unsafe { T::read_immutable_impl((self.data, self.mark, &self.doc)) };

        let (data_advance, mark_advance) =
            unsafe { ReadonlyValue::<O, D>::size(T::TAG_ID, self.data, self.mark) };
        self.data = unsafe { self.data.add(data_advance) };
        self.mark = unsafe { self.mark.add(mark_advance) };

        value
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.remaining as usize;
        (remaining, Some(remaining))
    }
}

impl<'doc, O: ByteOrder, D: Document, T: NBT> ExactSizeIterator
    for ReadonlyTypedListIter<'doc, O, D, T>
{
}
