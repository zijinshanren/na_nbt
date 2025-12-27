use std::{marker::PhantomData, ptr};

use zerocopy::byteorder;

use crate::{
    ByteOrder, ConfigRef, Document, EMPTY_LIST, ImmutableConfig, ListBase, ListRef, Mark, NBT,
    ReadonlyTypedList, ReadonlyValue, TagID, cold_path, immutable_tag_size,
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
            doc: D::empty(),
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

impl<'doc, O: ByteOrder, D: Document> ListBase for ReadonlyList<'doc, O, D> {
    #[inline]
    fn element_tag_id(&self) -> TagID {
        unsafe { *self.data.as_ptr().cast() }
    }

    #[inline]
    fn len(&self) -> usize {
        unsafe { byteorder::U32::<O>::from_bytes(*self.data.as_ptr().add(1).cast()).get() as usize }
    }
}

impl<'doc, O: ByteOrder, D: Document> ListRef<'doc> for ReadonlyList<'doc, O, D> {
    type Config = ImmutableConfig<O, D>;

    #[inline]
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
    fn typed_<T: NBT>(self) -> Option<<Self::Config as ConfigRef>::TypedList<'doc, T>> {
        self.element_is_::<T>().then_some(ReadonlyTypedList {
            data: self.data,
            mark: self.mark,
            doc: self.doc,
            _marker: PhantomData,
        })
    }

    #[inline]
    fn iter(&self) -> <Self::Config as ConfigRef>::ListIter<'doc> {
        ReadonlyListIter {
            tag_id: self.element_tag_id(),
            remaining: self.len() as u32,
            data: unsafe { self.data.as_ptr().add(1 + 4) },
            mark: self.mark,
            doc: self.doc.clone(),
            _marker: PhantomData,
        }
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
            doc: D::empty(),
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

        let value = unsafe {
            ImmutableConfig::<O, D>::read_value(self.tag_id, (self.data, self.mark, &self.doc))
        };

        let (data_advance, mark_advance) =
            unsafe { immutable_tag_size::<O>(self.tag_id, self.data, self.mark) };
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
