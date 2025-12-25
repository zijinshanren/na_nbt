use crate::{
    CompoundBase, ConfigMut, ConfigRef, GenericNBT, Index, IntoNBT, ListBase, MapMut, NBT, NBTBase,
    OwnValue, TypedListBase, ValueBase, VisitMut,
};

pub trait ValueMut<'s>: ValueBase<ConfigRef = Self::ConfigMut> {
    type ConfigMut: ConfigMut;

    fn mut_<'a, T: NBT>(&'a mut self) -> Option<&'a mut T::TypeMut<'s, Self::ConfigMut>>
    where
        's: 'a;

    fn into_<T: GenericNBT>(self) -> Option<T::TypeRef<'s, Self::ConfigMut>>;

    fn into_mut_<T: GenericNBT>(self) -> Option<T::TypeMut<'s, Self::ConfigMut>>;

    fn get<'a>(&'a self, index: impl Index) -> Option<<Self::ConfigMut as ConfigRef>::Value<'a>>
    where
        's: 'a;

    fn get_<'a, T: GenericNBT>(
        &'a self,
        index: impl Index,
    ) -> Option<T::TypeRef<'a, Self::ConfigMut>>
    where
        's: 'a;

    fn get_mut<'a>(
        &'a mut self,
        index: impl Index,
    ) -> Option<<Self::ConfigMut as ConfigMut>::ValueMut<'a>>
    where
        's: 'a;

    fn get_mut_<'a, T: GenericNBT>(
        &'a mut self,
        index: impl Index,
    ) -> Option<T::TypeMut<'a, Self::ConfigMut>>
    where
        's: 'a;

    fn visit_mut<'a, R>(
        &'a mut self,
        match_fn: impl FnOnce(VisitMut<'a, 's, Self::ConfigMut>) -> R,
    ) -> R
    where
        's: 'a;

    fn map_mut<R>(self, match_fn: impl FnOnce(MapMut<'s, Self::ConfigMut>) -> R) -> R;
}

pub trait ListMut<'s>:
    ListBase<ConfigRef = Self::ConfigMut>
    + IntoIterator<Item = <Self::ConfigMut as ConfigMut>::ValueMut<'s>>
{
    type ConfigMut: ConfigMut;

    fn get<'a>(&'a self, index: usize) -> Option<<Self::ConfigMut as ConfigRef>::Value<'a>>
    where
        's: 'a;

    fn get_<'a, T: GenericNBT>(&'a self, index: usize) -> Option<T::TypeRef<'a, Self::ConfigMut>>
    where
        's: 'a;

    fn get_mut<'a>(
        &'a mut self,
        index: usize,
    ) -> Option<<Self::ConfigMut as ConfigMut>::ValueMut<'a>>
    where
        's: 'a;

    fn get_mut_<'a, T: GenericNBT>(
        &'a mut self,
        index: usize,
    ) -> Option<T::TypeMut<'a, Self::ConfigMut>>
    where
        's: 'a;

    fn push<V: IntoNBT<<Self::ConfigMut as ConfigRef>::ByteOrder>>(
        &mut self,
        value: V,
    ) -> Option<<V::Tag as NBTBase>::Type<<Self::ConfigMut as ConfigRef>::ByteOrder>>;

    fn pop(&mut self) -> Option<OwnValue<<Self::ConfigMut as ConfigRef>::ByteOrder>>;

    fn pop_<T: GenericNBT>(&mut self)
    -> Option<T::Type<<Self::ConfigMut as ConfigRef>::ByteOrder>>;

    fn insert<V: IntoNBT<<Self::ConfigMut as ConfigRef>::ByteOrder>>(
        &mut self,
        index: usize,
        value: V,
    ) -> Option<<V::Tag as NBTBase>::Type<<Self::ConfigMut as ConfigRef>::ByteOrder>>;

    fn remove(
        &mut self,
        index: usize,
    ) -> Option<OwnValue<<Self::ConfigMut as ConfigRef>::ByteOrder>>;

    fn remove_<T: GenericNBT>(
        &mut self,
        index: usize,
    ) -> Option<T::Type<<Self::ConfigMut as ConfigRef>::ByteOrder>>;

    fn typed_<T: NBT>(self) -> Option<<Self::ConfigMut as ConfigRef>::TypedList<'s, T>>;

    fn typed_mut_<T: NBT>(self) -> Option<<Self::ConfigMut as ConfigMut>::TypedListMut<'s, T>>;

    fn iter<'a>(&'a self) -> <Self::ConfigMut as ConfigRef>::ListIter<'a>
    where
        's: 'a;

    fn iter_mut<'a>(&'a mut self) -> <Self::ConfigMut as ConfigMut>::ListIterMut<'a>
    where
        's: 'a;
}

pub trait TypedListMut<'s, T: NBT>:
    TypedListBase<T, ConfigRef = Self::ConfigMut> + IntoIterator<Item = T::TypeMut<'s, Self::ConfigMut>>
{
    type ConfigMut: ConfigMut;

    fn get<'a>(&'a self, index: usize) -> Option<T::TypeRef<'a, Self::ConfigMut>>
    where
        's: 'a;

    fn get_mut<'a>(&'a mut self, index: usize) -> Option<T::TypeMut<'a, Self::ConfigMut>>
    where
        's: 'a;

    fn push(
        &mut self,
        value: impl IntoNBT<<Self::ConfigMut as ConfigRef>::ByteOrder, Tag = T>,
    ) -> Option<T::Type<<Self::ConfigMut as ConfigRef>::ByteOrder>>;

    fn pop(&mut self) -> Option<T::Type<<Self::ConfigMut as ConfigRef>::ByteOrder>>;

    fn insert(
        &mut self,
        index: usize,
        value: impl IntoNBT<<Self::ConfigMut as ConfigRef>::ByteOrder, Tag = T>,
    ) -> Option<T::Type<<Self::ConfigMut as ConfigRef>::ByteOrder>>;

    fn remove(
        &mut self,
        index: usize,
    ) -> Option<T::Type<<Self::ConfigMut as ConfigRef>::ByteOrder>>;

    fn iter<'a>(&'a self) -> <Self::ConfigMut as ConfigRef>::TypedListIter<'a, T>
    where
        's: 'a;

    fn iter_mut<'a>(&'a mut self) -> <Self::ConfigMut as ConfigMut>::TypedListIterMut<'a, T>
    where
        's: 'a;
}

pub trait CompoundMut<'s>:
    CompoundBase<ConfigRef = Self::ConfigMut>
    + IntoIterator<
        Item = (
            <Self::ConfigMut as ConfigRef>::String<'s>,
            <Self::ConfigMut as ConfigMut>::ValueMut<'s>,
        ),
    >
{
    type ConfigMut: ConfigMut;

    fn get<'a>(&'a self, key: &str) -> Option<<Self::ConfigMut as ConfigRef>::Value<'a>>
    where
        's: 'a;

    fn get_<'a, T: GenericNBT>(&'a self, key: &str) -> Option<T::TypeRef<'a, Self::ConfigMut>>
    where
        's: 'a;

    fn get_mut<'a>(&'a mut self, key: &str) -> Option<<Self::ConfigMut as ConfigMut>::ValueMut<'a>>
    where
        's: 'a;

    fn get_mut_<'a, T: GenericNBT>(
        &'a mut self,
        key: &str,
    ) -> Option<T::TypeMut<'a, Self::ConfigMut>>
    where
        's: 'a;

    fn insert(
        &mut self,
        key: &str,
        value: impl IntoNBT<<Self::ConfigMut as ConfigRef>::ByteOrder>,
    ) -> Option<OwnValue<<Self::ConfigMut as ConfigRef>::ByteOrder>>;

    fn insert_<T: GenericNBT>(
        &mut self,
        key: &str,
        value: impl IntoNBT<<Self::ConfigMut as ConfigRef>::ByteOrder>,
    ) -> Option<T::Type<<Self::ConfigMut as ConfigRef>::ByteOrder>>;

    fn remove(&mut self, key: &str) -> Option<OwnValue<<Self::ConfigMut as ConfigRef>::ByteOrder>>;

    fn remove_<T: GenericNBT>(
        &mut self,
        key: &str,
    ) -> Option<T::Type<<Self::ConfigMut as ConfigRef>::ByteOrder>>;

    fn iter<'a>(&'a self) -> <Self::ConfigMut as ConfigRef>::CompoundIter<'a>
    where
        's: 'a;

    fn iter_mut<'a>(&'a mut self) -> <Self::ConfigMut as ConfigMut>::CompoundIterMut<'a>
    where
        's: 'a;
}
