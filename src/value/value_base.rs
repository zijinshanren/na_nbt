use std::io::Write;

use crate::{ByteOrder, NBT, Result, TagID};

pub trait Writable {
    fn write_to_vec<TARGET: ByteOrder>(&self, buf: &mut Vec<u8>);

    fn write_to_writer<TARGET: ByteOrder>(&self, writer: impl Write) -> Result<()>;
}

pub trait ValueBase: Send + Sync + Sized {
    fn tag_id(&self) -> TagID;

    #[inline]
    fn is_<T: NBT>(&self) -> bool {
        self.tag_id() == T::TAG_ID
    }
}

pub trait ListBase: Send + Sync + Sized {
    fn element_tag_id(&self) -> TagID;

    #[inline]
    fn element_is_<T: NBT>(&self) -> bool {
        self.element_tag_id() == T::TAG_ID
            || (self.element_tag_id() == TagID::End && self.is_empty())
    }

    fn len(&self) -> usize;

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait TypedListBase<T: NBT>: Send + Sync + Sized {
    const ELEMENT_TAG_ID: TagID = T::TAG_ID;

    fn len(&self) -> usize;

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait CompoundBase: Send + Sync + Sized {}
