use crate::{CompoundMut, ConfigRef, ListMut, NBT, TypedListMut, ValueMut};

pub trait ConfigMut: ConfigRef {
    type ValueMut<'doc>: ValueMut<'doc, Config = Self>;
    type ListMut<'doc>: ListMut<'doc, Config = Self>;
    type ListIterMut<'doc>: Iterator<Item = Self::ValueMut<'doc>>
        + ExactSizeIterator
        + Clone
        + Default;
    type TypedListMut<'doc, T: NBT>: TypedListMut<'doc, T, Config = Self>;
    type TypedListIterMut<'doc, T: NBT>: Iterator<Item = T::TypeMut<'doc, Self>>
        + ExactSizeIterator
        + Clone
        + Default;
    type CompoundMut<'doc>: CompoundMut<'doc, Config = Self>;
    type CompoundIterMut<'doc>: Iterator<Item = (Self::String<'doc>, Self::ValueMut<'doc>)>
        + Clone
        + Default;
}
