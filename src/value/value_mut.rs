use crate::{
    CompoundBase, ConfigMut, ConfigRef, Index, IntoNBT, ListBase, NBT, NBTBase, OwnValue,
    TypedListBase, ValueBase, Visit, VisitMut,
};

pub trait ValueMut<'s>: ValueBase {
    type Config: ConfigMut;

    fn mut_<'a, T: NBT>(&'a mut self) -> Option<&'a mut T::TypeMut<'s, Self::Config>>
    where
        's: 'a;

    fn into_<T: NBT>(self) -> Option<T::TypeRef<'s, Self::Config>>;

    fn into_mut_<T: NBT>(self) -> Option<T::TypeMut<'s, Self::Config>>;

    fn get<'a>(&'a self, index: impl Index) -> Option<<Self::Config as ConfigRef>::Value<'a>>
    where
        's: 'a;

    fn get_<'a, T: NBT>(&'a self, index: impl Index) -> Option<T::TypeRef<'a, Self::Config>>
    where
        's: 'a;

    fn get_mut<'a>(
        &'a mut self,
        index: impl Index,
    ) -> Option<<Self::Config as ConfigMut>::ValueMut<'a>>
    where
        's: 'a;

    fn get_mut_<'a, T: NBT>(
        &'a mut self,
        index: impl Index,
    ) -> Option<T::TypeMut<'a, Self::Config>>
    where
        's: 'a;

    fn map<R>(self, match_fn: impl FnOnce(Visit<'s, Self::Config>) -> R) -> R;

    fn map_mut<R>(self, match_fn: impl FnOnce(VisitMut<'s, Self::Config>) -> R) -> R;
}

pub trait ListMut<'s>:
    ListBase + IntoIterator<Item = <Self::Config as ConfigMut>::ValueMut<'s>>
{
    type Config: ConfigMut;

    fn get<'a>(&'a self, index: usize) -> Option<<Self::Config as ConfigRef>::Value<'a>>
    where
        's: 'a;

    fn get_<'a, T: NBT>(&'a self, index: usize) -> Option<T::TypeRef<'a, Self::Config>>
    where
        's: 'a;

    fn get_mut<'a>(&'a mut self, index: usize) -> Option<<Self::Config as ConfigMut>::ValueMut<'a>>
    where
        's: 'a;

    fn get_mut_<'a, T: NBT>(&'a mut self, index: usize) -> Option<T::TypeMut<'a, Self::Config>>
    where
        's: 'a;

    fn push<V: IntoNBT<<Self::Config as ConfigRef>::ByteOrder>>(
        &mut self,
        value: V,
    ) -> Option<<V::Tag as NBTBase>::Type<<Self::Config as ConfigRef>::ByteOrder>>;

    fn pop(&mut self) -> Option<OwnValue<<Self::Config as ConfigRef>::ByteOrder>>;

    fn pop_<T: NBT>(&mut self) -> Option<T::Type<<Self::Config as ConfigRef>::ByteOrder>>;

    fn insert<V: IntoNBT<<Self::Config as ConfigRef>::ByteOrder>>(
        &mut self,
        index: usize,
        value: V,
    ) -> Option<<V::Tag as NBTBase>::Type<<Self::Config as ConfigRef>::ByteOrder>>;

    fn remove(&mut self, index: usize) -> Option<OwnValue<<Self::Config as ConfigRef>::ByteOrder>>;

    fn remove_<T: NBT>(
        &mut self,
        index: usize,
    ) -> Option<T::Type<<Self::Config as ConfigRef>::ByteOrder>>;

    fn typed_<T: NBT>(self) -> Option<<Self::Config as ConfigRef>::TypedList<'s, T>>;

    fn typed_mut_<T: NBT>(self) -> Option<<Self::Config as ConfigMut>::TypedListMut<'s, T>>;

    fn iter<'a>(&'a self) -> <Self::Config as ConfigRef>::ListIter<'a>
    where
        's: 'a;

    fn iter_mut<'a>(&'a mut self) -> <Self::Config as ConfigMut>::ListIterMut<'a>
    where
        's: 'a;
}

pub trait TypedListMut<'s, T: NBT>:
    TypedListBase<T> + IntoIterator<Item = T::TypeMut<'s, Self::Config>>
{
    type Config: ConfigMut;

    fn get<'a>(&'a self, index: usize) -> Option<T::TypeRef<'a, Self::Config>>
    where
        's: 'a;

    fn get_mut<'a>(&'a mut self, index: usize) -> Option<T::TypeMut<'a, Self::Config>>
    where
        's: 'a;

    fn push(
        &mut self,
        value: impl IntoNBT<<Self::Config as ConfigRef>::ByteOrder, Tag = T>,
    ) -> Option<T::Type<<Self::Config as ConfigRef>::ByteOrder>>;

    fn pop(&mut self) -> Option<T::Type<<Self::Config as ConfigRef>::ByteOrder>>;

    fn insert(
        &mut self,
        index: usize,
        value: impl IntoNBT<<Self::Config as ConfigRef>::ByteOrder, Tag = T>,
    ) -> Option<T::Type<<Self::Config as ConfigRef>::ByteOrder>>;

    fn remove(&mut self, index: usize) -> Option<T::Type<<Self::Config as ConfigRef>::ByteOrder>>;

    fn iter<'a>(&'a self) -> <Self::Config as ConfigRef>::TypedListIter<'a, T>
    where
        's: 'a;

    fn iter_mut<'a>(&'a mut self) -> <Self::Config as ConfigMut>::TypedListIterMut<'a, T>
    where
        's: 'a;
}

pub trait CompoundMut<'s>:
    CompoundBase
    + IntoIterator<
        Item = (
            <Self::Config as ConfigRef>::String<'s>,
            <Self::Config as ConfigMut>::ValueMut<'s>,
        ),
    >
{
    type Config: ConfigMut;

    fn get<'a>(&'a self, key: &str) -> Option<<Self::Config as ConfigRef>::Value<'a>>
    where
        's: 'a;

    fn get_<'a, T: NBT>(&'a self, key: &str) -> Option<T::TypeRef<'a, Self::Config>>
    where
        's: 'a;

    fn get_mut<'a>(&'a mut self, key: &str) -> Option<<Self::Config as ConfigMut>::ValueMut<'a>>
    where
        's: 'a;

    fn get_mut_<'a, T: NBT>(&'a mut self, key: &str) -> Option<T::TypeMut<'a, Self::Config>>
    where
        's: 'a;

    fn insert(
        &mut self,
        key: &str,
        value: impl IntoNBT<<Self::Config as ConfigRef>::ByteOrder>,
    ) -> Option<OwnValue<<Self::Config as ConfigRef>::ByteOrder>>;

    fn insert_<T: NBT>(
        &mut self,
        key: &str,
        value: impl IntoNBT<<Self::Config as ConfigRef>::ByteOrder>,
    ) -> Option<T::Type<<Self::Config as ConfigRef>::ByteOrder>>;

    fn remove(&mut self, key: &str) -> Option<OwnValue<<Self::Config as ConfigRef>::ByteOrder>>;

    fn remove_<T: NBT>(
        &mut self,
        key: &str,
    ) -> Option<T::Type<<Self::Config as ConfigRef>::ByteOrder>>;

    fn iter<'a>(&'a self) -> <Self::Config as ConfigRef>::CompoundIter<'a>
    where
        's: 'a;

    fn iter_mut<'a>(&'a mut self) -> <Self::Config as ConfigMut>::CompoundIterMut<'a>
    where
        's: 'a;
}
