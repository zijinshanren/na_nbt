use crate::{CompoundMut, ConfigRef, ListMut, NBTBase, TypedListMut, ValueMut};

pub trait ConfigMut: ConfigRef {
    type ValueMut<'doc>: ValueMut<'doc, ConfigMut = Self>;
    type ListMut<'doc>: ListMut<'doc, ConfigMut = Self>;
    type ListIterMut<'doc>: Iterator<Item = Self::ValueMut<'doc>>
        + ExactSizeIterator
        + Clone
        + Default;
    type TypedListMut<'doc, T: NBTBase>: TypedListMut<'doc, T, ConfigMut = Self>;
    type TypedListIterMut<'doc, T: NBTBase>: Iterator<Item = T::TypeMut<'doc, Self>>
        + ExactSizeIterator
        + Clone
        + Default;
    type CompoundMut<'doc>: CompoundMut<'doc, ConfigMut = Self>;
    type CompoundIterMut<'doc>: Iterator<Item = (Self::String<'doc>, Self::ValueMut<'doc>)>
        + Clone
        + Default;
}
