use std::ops::Deref;

use crate::{Document, MUTF8Str};

#[derive(Clone)]
pub struct ReadonlyString<'doc, D> {
    pub(crate) data: &'doc MUTF8Str,
    pub(crate) _doc: D,
}

impl<'doc, D: Document> Default for ReadonlyString<'doc, D> {
    #[inline]
    fn default() -> Self {
        Self {
            data: Default::default(),
            _doc: D::empty(),
        }
    }
}

impl<'doc, D: Document> Deref for ReadonlyString<'doc, D> {
    type Target = MUTF8Str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.data
    }
}
