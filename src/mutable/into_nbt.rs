use crate::{ByteOrder, NBT, NBTBase};

pub trait IntoNBT<O: ByteOrder>: Into<<Self::Tag as NBTBase>::Type<O>> {
    type Tag: NBT;
}
