use std::ops::Deref;

use crate::MUTF8Str;

#[derive(Clone)]
pub struct RefString<'s> {
    pub(crate) data: &'s MUTF8Str,
}

impl<'s> Default for RefString<'s> {
    #[inline]
    fn default() -> Self {
        Self {
            data: unsafe { MUTF8Str::from_mutf8_unchecked(&[]) },
        }
    }
}

impl<'s> Deref for RefString<'s> {
    type Target = MUTF8Str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.data
    }
}
