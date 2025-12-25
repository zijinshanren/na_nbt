use std::{marker::PhantomData, ptr, slice};

use zerocopy::byteorder;

use crate::{
    ByteOrder, CompoundBase, CompoundRef, ConfigRef, Document, EMPTY_COMPOUND, GenericNBT,
    ImmutableConfig, Mark, Never, ReadonlyString, ReadonlyValue, TagID, cold_path,
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
            doc: unsafe { Never::never() },
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

impl<'doc, O: ByteOrder, D: Document> ReadonlyCompound<'doc, O, D> {
    #[inline]
    pub fn get(&self, key: &str) -> Option<ReadonlyValue<'doc, O, D>> {
        CompoundRef::get(self, key)
    }

    #[inline]
    pub fn get_<T: GenericNBT>(
        &self,
        key: &str,
    ) -> Option<T::TypeRef<'doc, ImmutableConfig<O, D>>> {
        CompoundRef::get_::<T>(self, key)
    }

    #[inline]
    pub fn iter(&self) -> ReadonlyCompoundIter<'doc, O, D> {
        ReadonlyCompoundIter {
            data: self.data.as_ptr(),
            mark: self.mark,
            doc: self.doc.clone(),
            _marker: PhantomData,
        }
    }
}

impl<'doc, O: ByteOrder, D: Document> CompoundBase<'doc> for ReadonlyCompound<'doc, O, D> {
    type ConfigRef = ImmutableConfig<O, D>;

    fn compound_get_impl<'a>(
        &'a self,
        key: &str,
    ) -> Option<(TagID, <Self::ConfigRef as ConfigRef>::ReadParams<'a>)> {
        let name = simd_cesu8::mutf8::encode(key);
        unsafe {
            let mut ptr = self.data.as_ptr();
            let mut mark = self.mark;
            loop {
                let tag_id = *ptr.cast();
                ptr = ptr.add(1);

                if tag_id == TagID::End {
                    cold_path();
                    return None;
                }

                let name_len = byteorder::U16::<O>::from_bytes(*ptr.cast()).get();
                ptr = ptr.add(2);

                let name_bytes = core::slice::from_raw_parts(ptr, name_len as usize);
                ptr = ptr.add(name_len as usize);

                if name == name_bytes {
                    return Some((tag_id, (ptr, mark, &self.doc)));
                }

                let (data_advance, mark_advance) = ReadonlyValue::<O, D>::size(tag_id, ptr, mark);
                ptr = ptr.add(data_advance);
                mark = mark.add(mark_advance);
            }
        }
    }
}

impl<'doc, O: ByteOrder, D: Document> CompoundRef<'doc> for ReadonlyCompound<'doc, O, D> {
    #[inline]
    fn iter(&self) -> <Self::ConfigRef as ConfigRef>::CompoundIter<'doc> {
        self.iter()
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
            doc: unsafe { Never::never() },
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
                data: slice::from_raw_parts(self.data.add(3), name_len as usize),
                _doc: self.doc.clone(),
            };

            let value = <ImmutableConfig<O, D> as ConfigRef>::read_value(
                tag_id,
                (self.data.add(3 + name_len as usize), self.mark, &self.doc),
            );

            self.data = self.data.add(1 + 2 + name_len as usize);

            let (data_advance, mark_advance) =
                ReadonlyValue::<O, D>::size(tag_id, self.data, self.mark);
            self.data = self.data.add(data_advance);
            self.mark = self.mark.add(mark_advance);

            Some((name, value))
        }
    }
}
