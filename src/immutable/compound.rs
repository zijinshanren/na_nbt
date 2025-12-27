use std::{marker::PhantomData, ptr, slice};

use zerocopy::byteorder;

use crate::{
    ByteOrder, CompoundBase, CompoundRef, ConfigRef, Document, EMPTY_COMPOUND, ImmutableConfig,
    MUTF8Str, Mark, ReadonlyString, ReadonlyValue, TagID, cold_path, immutable_tag_size,
};

#[derive(Clone)]
pub struct ReadonlyCompound<'doc, O: ByteOrder, D: Document> {
    pub(crate) data: &'doc [u8],
    pub(crate) mark: *const Mark,
    pub(crate) doc: D,
    pub(crate) _marker: PhantomData<O>,
}

impl<'doc, O: ByteOrder, D: Document> Default for ReadonlyCompound<'doc, O, D> {
    #[inline]
    fn default() -> Self {
        Self {
            data: &EMPTY_COMPOUND,
            mark: ptr::null(),
            doc: D::empty(),
            _marker: PhantomData,
        }
    }
}

unsafe impl<'doc, O: ByteOrder, D: Document> Send for ReadonlyCompound<'doc, O, D> {}
unsafe impl<'doc, O: ByteOrder, D: Document> Sync for ReadonlyCompound<'doc, O, D> {}

impl<'doc, O: ByteOrder, D: Document> IntoIterator for ReadonlyCompound<'doc, O, D> {
    type Item = (ReadonlyString<'doc, D>, ReadonlyValue<'doc, O, D>);
    type IntoIter = ReadonlyCompoundIter<'doc, O, D>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        ReadonlyCompoundIter {
            data: self.data.as_ptr(),
            mark: self.mark,
            doc: self.doc,
            _marker: PhantomData,
        }
    }
}

impl<'doc, O: ByteOrder, D: Document> CompoundBase for ReadonlyCompound<'doc, O, D> {}

impl<'doc, O: ByteOrder, D: Document> CompoundRef<'doc> for ReadonlyCompound<'doc, O, D> {
    type Config = ImmutableConfig<O, D>;

    #[inline]
    fn _to_read_params<'a>(&'a self) -> <Self::Config as ConfigRef>::ReadParams<'a>
    where
        'doc: 'a,
    {
        (self.data.as_ptr(), self.mark, &self.doc)
    }

    #[inline]
    fn iter(&self) -> <Self::Config as ConfigRef>::CompoundIter<'doc> {
        ReadonlyCompoundIter {
            data: self.data.as_ptr(),
            mark: self.mark,
            doc: self.doc.clone(),
            _marker: PhantomData,
        }
    }
}

#[derive(Clone)]
pub struct ReadonlyCompoundIter<'doc, O: ByteOrder, D: Document> {
    data: *const u8,
    mark: *const Mark,
    doc: D,
    _marker: PhantomData<(&'doc (), O)>,
}

impl<'doc, O: ByteOrder, D: Document> Default for ReadonlyCompoundIter<'doc, O, D> {
    #[inline]
    fn default() -> Self {
        Self {
            data: EMPTY_COMPOUND.as_ptr(),
            mark: ptr::null(),
            doc: D::empty(),
            _marker: PhantomData,
        }
    }
}

unsafe impl<'doc, O: ByteOrder, D: Document> Send for ReadonlyCompoundIter<'doc, O, D> {}
unsafe impl<'doc, O: ByteOrder, D: Document> Sync for ReadonlyCompoundIter<'doc, O, D> {}

impl<'doc, O: ByteOrder, D: Document> Iterator for ReadonlyCompoundIter<'doc, O, D> {
    type Item = (ReadonlyString<'doc, D>, ReadonlyValue<'doc, O, D>);

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let tag_id = *self.data.cast();

            if tag_id == TagID::End {
                cold_path();
                return None;
            }

            let name_len = byteorder::U16::<O>::from_bytes(*self.data.add(1).cast()).get();
            let name = ReadonlyString {
                data: MUTF8Str::from_mutf8_unchecked(slice::from_raw_parts(
                    self.data.add(3),
                    name_len as usize,
                )),
                _doc: self.doc.clone(),
            };

            let value = <ImmutableConfig<O, D> as ConfigRef>::read_value(
                tag_id,
                (self.data.add(3 + name_len as usize), self.mark, &self.doc),
            );

            self.data = self.data.add(1 + 2 + name_len as usize);

            let (data_advance, mark_advance) =
                immutable_tag_size::<O>(tag_id, self.data, self.mark);
            self.data = self.data.add(data_advance);
            self.mark = self.mark.add(mark_advance);

            Some((name, value))
        }
    }
}
