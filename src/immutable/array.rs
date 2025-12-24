use std::ops::Deref;

use crate::Document;

#[derive(Clone)]
pub struct ReadonlyArray<'doc, T, D: Document> {
    pub(crate) data: &'doc [T],
    pub(crate) _doc: D,
}

impl<'doc, T, D: Document> Default for ReadonlyArray<'doc, T, D> {
    fn default() -> Self {
        Self {
            data: &[],
            _doc: unsafe { D::never() },
        }
    }
}

impl<'doc, T, D: Document> Deref for ReadonlyArray<'doc, T, D> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'doc, T, D: Document> ReadonlyArray<'doc, T, D> {
    #[inline]
    pub fn as_slice<'a>(&'a self) -> &'a [T]
    where
        'doc: 'a,
    {
        self.data
    }
}
