use std::{marker::PhantomData, ptr};

use zerocopy::byteorder;

use crate::{
    ByteOrder, NBT, ReadableConfig, ReadableTypedList, ScopedReadableTypedList, TagEnd, TagID,
    cold_path,
    immutable::{mark::Mark, trait_impl::Config, util::ImmutableNBTImpl, value::Document},
};

#[derive(Clone)]
pub struct ReadonlyTypedList<'doc, O: ByteOrder, D: Document, T: NBT> {
    pub(crate) data: *const u8,
    pub(crate) mark: *const Mark,
    pub(crate) length: u32,
    pub(crate) doc: D,
    pub(crate) _marker: PhantomData<(&'doc (), O, T)>,
}

unsafe impl<'doc, O: ByteOrder, D: Document, T: NBT> Send for ReadonlyTypedList<'doc, O, D, T> {}
unsafe impl<'doc, O: ByteOrder, D: Document, T: NBT> Sync for ReadonlyTypedList<'doc, O, D, T> {}

impl<'doc, O: ByteOrder, D: Document, T: NBT> Default for ReadonlyTypedList<'doc, O, D, T> {
    fn default() -> Self {
        Self {
            data: ptr::null(),
            mark: ptr::null(),
            length: 0,
            doc: unsafe { D::never() },
            _marker: PhantomData,
        }
    }
}

impl<'doc, O: ByteOrder, D: Document, T: ImmutableNBTImpl> ReadonlyTypedList<'doc, O, D, T> {
    #[inline]
    pub fn tag_id(&self) -> TagID {
        T::TAG_ID
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.length as usize
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<T::Type<'doc, Config<O, D>>> {
        if index >= self.length as usize {
            cold_path();
            return None;
        }

        todo!()
    }

    #[inline]
    pub fn iter(&self) -> ReadonlyTypedList<'doc, O, D, T> {
        self.clone()
    }
}

impl<'doc, O: ByteOrder, D: Document, T: ImmutableNBTImpl> Iterator
    for ReadonlyTypedList<'doc, O, D, T>
{
    type Item = T::Type<'doc, Config<O, D>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.length == 0 {
            cold_path();
            return None;
        }

        self.length -= 1;

        let value = unsafe { T::read::<O, D>(self.data, self.mark, self.doc.clone()) };

        let (data_advance, mark_advance) = unsafe { T::size::<O>(self.data, self.mark) };

        self.data = unsafe { self.data.add(data_advance) };
        self.mark = unsafe { self.mark.add(mark_advance) };

        Some(value)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.length as usize;
        (len, Some(len))
    }
}

impl<'doc, O: ByteOrder, D: Document, T: ImmutableNBTImpl> ExactSizeIterator
    for ReadonlyTypedList<'doc, O, D, T>
{
}

impl<'doc, O: ByteOrder, D: Document, T: ImmutableNBTImpl> ScopedReadableTypedList<'doc, T>
    for ReadonlyTypedList<'doc, O, D, T>
{
    type Config = Config<O, D>;

    #[inline]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    #[inline]
    fn get_scoped<'a>(&'a self, index: usize) -> Option<<T as NBT>::Type<'a, Self::Config>>
    where
        'doc: 'a,
    {
        self.get(index)
    }

    #[inline]
    fn iter_scoped<'a>(&'a self) -> <Self::Config as ReadableConfig>::TypedListIter<'a, T>
    where
        'doc: 'a,
    {
        self.iter()
    }
}

impl<'doc, O: ByteOrder, D: Document, T: ImmutableNBTImpl> ReadableTypedList<'doc, T>
    for ReadonlyTypedList<'doc, O, D, T>
{
    #[inline]
    fn get(&self, index: usize) -> Option<<T as NBT>::Type<'doc, Self::Config>> {
        self.get(index)
    }

    #[inline]
    fn iter(&self) -> <Self::Config as ReadableConfig>::TypedListIter<'doc, T> {
        self.iter()
    }
}
