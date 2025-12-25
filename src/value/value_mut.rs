use crate::{
    CompoundBase, ConfigMut, ConfigRef, GenericNBT, Index, IntoNBT, ListBase, MapMut, NBT, NBTBase,
    OwnValue, TagID, TypedListBase, ValueBase, VisitMut, cold_path,
    tag::{
        Byte, ByteArray, Compound, Double, End, Float, Int, IntArray, List, Long, LongArray, Short,
        String, TypedList,
    },
};

pub trait ValueMut<'s>:
    ValueBase<ConfigRef = Self::ConfigMut>
    + From<<End as NBTBase>::TypeMut<'s, Self::ConfigMut>>
    + From<<Byte as NBTBase>::TypeMut<'s, Self::ConfigMut>>
    + From<<Short as NBTBase>::TypeMut<'s, Self::ConfigMut>>
    + From<<Int as NBTBase>::TypeMut<'s, Self::ConfigMut>>
    + From<<Long as NBTBase>::TypeMut<'s, Self::ConfigMut>>
    + From<<Float as NBTBase>::TypeMut<'s, Self::ConfigMut>>
    + From<<Double as NBTBase>::TypeMut<'s, Self::ConfigMut>>
    + From<<ByteArray as NBTBase>::TypeMut<'s, Self::ConfigMut>>
    + From<<String as NBTBase>::TypeMut<'s, Self::ConfigMut>>
    + From<<List as NBTBase>::TypeMut<'s, Self::ConfigMut>>
    + From<<Compound as NBTBase>::TypeMut<'s, Self::ConfigMut>>
    + From<<IntArray as NBTBase>::TypeMut<'s, Self::ConfigMut>>
    + From<<LongArray as NBTBase>::TypeMut<'s, Self::ConfigMut>>
    + From<<TypedList<End> as NBTBase>::TypeMut<'s, Self::ConfigMut>>
    + From<<TypedList<Byte> as NBTBase>::TypeMut<'s, Self::ConfigMut>>
    + From<<TypedList<Short> as NBTBase>::TypeMut<'s, Self::ConfigMut>>
    + From<<TypedList<Int> as NBTBase>::TypeMut<'s, Self::ConfigMut>>
    + From<<TypedList<Long> as NBTBase>::TypeMut<'s, Self::ConfigMut>>
    + From<<TypedList<Float> as NBTBase>::TypeMut<'s, Self::ConfigMut>>
    + From<<TypedList<Double> as NBTBase>::TypeMut<'s, Self::ConfigMut>>
    + From<<TypedList<ByteArray> as NBTBase>::TypeMut<'s, Self::ConfigMut>>
    + From<<TypedList<String> as NBTBase>::TypeMut<'s, Self::ConfigMut>>
    + From<<TypedList<List> as NBTBase>::TypeMut<'s, Self::ConfigMut>>
    + From<<TypedList<Compound> as NBTBase>::TypeMut<'s, Self::ConfigMut>>
    + From<<TypedList<IntArray> as NBTBase>::TypeMut<'s, Self::ConfigMut>>
    + From<<TypedList<LongArray> as NBTBase>::TypeMut<'s, Self::ConfigMut>>
{
    type ConfigMut: ConfigMut;

    #[inline]
    fn mut_<'a, T: NBT>(&'a mut self) -> Option<&'a mut T::TypeMut<'s, Self::ConfigMut>>
    where
        's: 'a,
    {
        T::mut_(self)
    }

    #[inline]
    fn into_<T: GenericNBT>(self) -> Option<T::TypeMut<'s, Self::ConfigMut>> {
        T::mut_into_(self)
    }

    #[inline]
    fn get<'a>(&'a self, index: impl Index) -> Option<<Self::ConfigMut as ConfigRef>::Value<'a>>
    where
        's: 'a,
    {
        index.index_dispatch(
            self,
            |value, index| value.get(index),
            |value, key| value.get(key),
        )
    }

    #[inline]
    fn get_<'a, T: GenericNBT>(
        &'a self,
        index: impl Index,
    ) -> Option<T::TypeRef<'a, Self::ConfigMut>>
    where
        's: 'a,
    {
        index.index_dispatch(
            self,
            |value, index| value.get_::<T>(index),
            |value, key| value.get_::<T>(key),
        )
    }

    #[inline]
    fn get_mut<'a>(
        &'a mut self,
        index: impl Index,
    ) -> Option<<Self::ConfigMut as ConfigMut>::ValueMut<'a>>
    where
        's: 'a,
    {
        index.index_dispatch_mut(
            self,
            |value, index| value.get_mut(index),
            |value, key| value.get_mut(key),
        )
    }

    #[inline]
    fn get_mut_<'a, T: GenericNBT>(
        &'a mut self,
        index: impl Index,
    ) -> Option<T::TypeMut<'a, Self::ConfigMut>>
    where
        's: 'a,
    {
        index.index_dispatch_mut(
            self,
            |value, index| value.get_mut_::<T>(index),
            |value, key| value.get_mut_::<T>(key),
        )
    }

    fn visit<'a, R>(
        &'a mut self,
        match_fn: impl FnOnce(VisitMut<'a, 's, Self::ConfigMut>) -> R,
    ) -> R
    where
        's: 'a;

    fn map<R>(self, match_fn: impl FnOnce(MapMut<'s, Self::ConfigMut>) -> R) -> R;
}

pub trait ListMut<'s>:
    ListBase<ConfigRef = Self::ConfigMut>
    + IntoIterator<Item = <Self::ConfigMut as ConfigMut>::ValueMut<'s>>
{
    type ConfigMut: ConfigMut;

    #[inline]
    #[allow(clippy::unit_arg)]
    fn get<'a>(&'a self, index: usize) -> Option<<Self::ConfigMut as ConfigRef>::Value<'a>>
    where
        's: 'a,
    {
        if index >= self.len() {
            cold_path();
            return None;
        }

        unsafe {
            Some(match self.element_tag_id() {
                TagID::End => From::from(
                    Self::ConfigRef::read::<End>(self.list_get_impl::<End>(index))
                        .unwrap_unchecked(),
                ),
                TagID::Byte => From::from(
                    Self::ConfigRef::read::<Byte>(self.list_get_impl::<Byte>(index))
                        .unwrap_unchecked(),
                ),
                TagID::Short => From::from(
                    Self::ConfigRef::read::<Short>(self.list_get_impl::<Short>(index))
                        .unwrap_unchecked(),
                ),
                TagID::Int => From::from(
                    Self::ConfigRef::read::<Int>(self.list_get_impl::<Int>(index))
                        .unwrap_unchecked(),
                ),
                TagID::Long => From::from(
                    Self::ConfigRef::read::<Long>(self.list_get_impl::<Long>(index))
                        .unwrap_unchecked(),
                ),
                TagID::Float => From::from(
                    Self::ConfigRef::read::<Float>(self.list_get_impl::<Float>(index))
                        .unwrap_unchecked(),
                ),
                TagID::Double => From::from(
                    Self::ConfigRef::read::<Double>(self.list_get_impl::<Double>(index))
                        .unwrap_unchecked(),
                ),
                TagID::ByteArray => From::from(
                    Self::ConfigRef::read::<ByteArray>(self.list_get_impl::<ByteArray>(index))
                        .unwrap_unchecked(),
                ),
                TagID::String => From::from(
                    Self::ConfigRef::read::<String>(self.list_get_impl::<String>(index))
                        .unwrap_unchecked(),
                ),
                TagID::List => From::from(
                    Self::ConfigRef::read::<List>(self.list_get_impl::<List>(index))
                        .unwrap_unchecked(),
                ),
                TagID::Compound => From::from(
                    Self::ConfigRef::read::<Compound>(self.list_get_impl::<Compound>(index))
                        .unwrap_unchecked(),
                ),
                TagID::IntArray => From::from(
                    Self::ConfigRef::read::<IntArray>(self.list_get_impl::<IntArray>(index))
                        .unwrap_unchecked(),
                ),
                TagID::LongArray => From::from(
                    Self::ConfigRef::read::<LongArray>(self.list_get_impl::<LongArray>(index))
                        .unwrap_unchecked(),
                ),
            })
        }
    }

    #[inline]
    fn get_<'a, T: GenericNBT>(&'a self, index: usize) -> Option<T::TypeRef<'a, Self::ConfigMut>>
    where
        's: 'a,
    {
        if index >= self.len() {
            cold_path();
            return None;
        }

        if !(self.element_tag_id() == <T>::TAG_ID
            || (self.element_tag_id() == TagID::End && self.is_empty()))
        {
            cold_path();
            return None;
        }

        unsafe { Self::ConfigRef::read::<T>(self.list_get_impl::<T>(index)) }
    }

    #[inline]
    fn get_mut<'a>(
        &'a mut self,
        index: usize,
    ) -> Option<<Self::ConfigMut as ConfigMut>::ValueMut<'a>>
    where
        's: 'a,
    {
        if index >= self.len() {
            cold_path();
            return None;
        }

        unsafe {
            Some(match self.element_tag_id() {
                TagID::End => From::from(
                    Self::ConfigMut::read_mut::<End>(self.list_get_impl::<End>(index))
                        .unwrap_unchecked(),
                ),
                TagID::Byte => From::from(
                    Self::ConfigMut::read_mut::<Byte>(self.list_get_impl::<Byte>(index))
                        .unwrap_unchecked(),
                ),
                TagID::Short => From::from(
                    Self::ConfigMut::read_mut::<Short>(self.list_get_impl::<Short>(index))
                        .unwrap_unchecked(),
                ),
                TagID::Int => From::from(
                    Self::ConfigMut::read_mut::<Int>(self.list_get_impl::<Int>(index))
                        .unwrap_unchecked(),
                ),
                TagID::Long => From::from(
                    Self::ConfigMut::read_mut::<Long>(self.list_get_impl::<Long>(index))
                        .unwrap_unchecked(),
                ),
                TagID::Float => From::from(
                    Self::ConfigMut::read_mut::<Float>(self.list_get_impl::<Float>(index))
                        .unwrap_unchecked(),
                ),
                TagID::Double => From::from(
                    Self::ConfigMut::read_mut::<Double>(self.list_get_impl::<Double>(index))
                        .unwrap_unchecked(),
                ),
                TagID::ByteArray => From::from(
                    Self::ConfigMut::read_mut::<ByteArray>(self.list_get_impl::<ByteArray>(index))
                        .unwrap_unchecked(),
                ),
                TagID::String => From::from(
                    Self::ConfigMut::read_mut::<String>(self.list_get_impl::<String>(index))
                        .unwrap_unchecked(),
                ),
                TagID::List => From::from(
                    Self::ConfigMut::read_mut::<List>(self.list_get_impl::<List>(index))
                        .unwrap_unchecked(),
                ),
                TagID::Compound => From::from(
                    Self::ConfigMut::read_mut::<Compound>(self.list_get_impl::<Compound>(index))
                        .unwrap_unchecked(),
                ),
                TagID::IntArray => From::from(
                    Self::ConfigMut::read_mut::<IntArray>(self.list_get_impl::<IntArray>(index))
                        .unwrap_unchecked(),
                ),
                TagID::LongArray => From::from(
                    Self::ConfigMut::read_mut::<LongArray>(self.list_get_impl::<LongArray>(index))
                        .unwrap_unchecked(),
                ),
            })
        }
    }

    #[inline]
    fn get_mut_<'a, T: GenericNBT>(
        &'a mut self,
        index: usize,
    ) -> Option<T::TypeMut<'a, Self::ConfigMut>>
    where
        's: 'a,
    {
        if index >= self.len() {
            cold_path();
            return None;
        }

        if !(self.element_tag_id() == <T>::TAG_ID
            || (self.element_tag_id() == TagID::End && self.is_empty()))
        {
            cold_path();
            return None;
        }

        unsafe { Self::ConfigMut::read_mut::<T>(self.list_get_impl::<T>(index)) }
    }

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

    fn typed_<T: NBT>(self) -> Option<<Self::ConfigMut as ConfigMut>::TypedListMut<'s, T>>;

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

    #[inline]
    fn get<'a>(&'a self, index: usize) -> Option<T::TypeRef<'a, Self::ConfigMut>>
    where
        's: 'a,
    {
        if index >= self.len() {
            cold_path();
            return None;
        }

        unsafe { Self::ConfigRef::read::<T>(self.typed_list_get_impl(index)) }
    }

    #[inline]
    fn get_mut<'a>(&'a mut self, index: usize) -> Option<T::TypeMut<'a, Self::ConfigMut>>
    where
        's: 'a,
    {
        if index >= self.len() {
            cold_path();
            return None;
        }

        unsafe { Self::ConfigMut::read_mut::<T>(self.typed_list_get_impl(index)) }
    }

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

    #[inline]
    fn get<'a>(&'a self, key: &str) -> Option<<Self::ConfigMut as ConfigRef>::Value<'a>>
    where
        's: 'a,
    {
        let (tag_id, params) = self.compound_get_impl(key)?;
        unsafe { Some(Self::ConfigRef::read_value(tag_id, params)) }
    }

    #[inline]
    fn get_<'a, T: GenericNBT>(&'a self, key: &str) -> Option<T::TypeRef<'a, Self::ConfigMut>>
    where
        's: 'a,
    {
        let (tag_id, params) = self.compound_get_impl(key)?;
        if tag_id != T::TAG_ID {
            cold_path();
            return None;
        }
        unsafe { Self::ConfigRef::read::<T>(params) }
    }

    #[inline]
    fn get_mut<'a>(&'a mut self, key: &str) -> Option<<Self::ConfigMut as ConfigMut>::ValueMut<'a>>
    where
        's: 'a,
    {
        let (tag_id, params) = self.compound_get_impl(key)?;
        unsafe { Some(Self::ConfigMut::read_value_mut(tag_id, params)) }
    }

    #[inline]
    fn get_mut_<'a, T: GenericNBT>(
        &'a mut self,
        key: &str,
    ) -> Option<T::TypeMut<'a, Self::ConfigMut>>
    where
        's: 'a,
    {
        let (tag_id, params) = self.compound_get_impl(key)?;
        if tag_id != T::TAG_ID {
            cold_path();
            return None;
        }
        unsafe { Self::ConfigMut::read_mut::<T>(params) }
    }

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
