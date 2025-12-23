use std::io::Write;

use crate::{ByteOrder, ConfigRef, NBT, Result, TagID};

pub trait Writable {
    fn write_to_vec<TARGET: ByteOrder>(&self, buf: &mut Vec<u8>);

    fn write_to_writer<TARGET: ByteOrder>(&self, writer: impl Write) -> Result<()>;
}

pub trait ValueBase: Writable + Send + Sync + Sized {
    type Config: ConfigRef;

    fn tag_id(&self) -> TagID;

    fn is_<T: NBT>(&self) -> bool;
}

pub trait ListBase: Writable + Send + Sync + Sized {
    type Config: ConfigRef;

    fn element_tag_id(&self) -> TagID;

    fn element_is_<T: NBT>(&self) -> bool;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool;
}

pub trait TypedListBase<T: NBT>: Writable + Send + Sync + Sized {
    type Config: ConfigRef;

    const ELEMENT_TAG_ID: TagID = T::TAG_ID;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool;
}

pub trait CompoundBase: Writable + Send + Sync + Sized {
    type Config: ConfigRef;
}
