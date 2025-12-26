use crate::{
    CompoundBase, ConfigMut, ConfigRef, GenericNBT, Index, IntoNBT, ListBase, MapMut, NBT, NBTBase,
    OwnValue, TagID, TypedListBase, ValueBase, VisitMut, VisitMutShared, cold_path,
    tag::{
        Byte, ByteArray, Compound, Double, End, Float, Int, IntArray, List, Long, LongArray, Short,
        String, TypedList,
    },
};

pub trait ValueMut<'s>:
    ValueBase
    + From<<End as NBTBase>::TypeMut<'s, Self::Config>>
    + From<<Byte as NBTBase>::TypeMut<'s, Self::Config>>
    + From<<Short as NBTBase>::TypeMut<'s, Self::Config>>
    + From<<Int as NBTBase>::TypeMut<'s, Self::Config>>
    + From<<Long as NBTBase>::TypeMut<'s, Self::Config>>
    + From<<Float as NBTBase>::TypeMut<'s, Self::Config>>
    + From<<Double as NBTBase>::TypeMut<'s, Self::Config>>
    + From<<ByteArray as NBTBase>::TypeMut<'s, Self::Config>>
    + From<<String as NBTBase>::TypeMut<'s, Self::Config>>
    + From<<List as NBTBase>::TypeMut<'s, Self::Config>>
    + From<<Compound as NBTBase>::TypeMut<'s, Self::Config>>
    + From<<IntArray as NBTBase>::TypeMut<'s, Self::Config>>
    + From<<LongArray as NBTBase>::TypeMut<'s, Self::Config>>
    + From<<TypedList<End> as NBTBase>::TypeMut<'s, Self::Config>>
    + From<<TypedList<Byte> as NBTBase>::TypeMut<'s, Self::Config>>
    + From<<TypedList<Short> as NBTBase>::TypeMut<'s, Self::Config>>
    + From<<TypedList<Int> as NBTBase>::TypeMut<'s, Self::Config>>
    + From<<TypedList<Long> as NBTBase>::TypeMut<'s, Self::Config>>
    + From<<TypedList<Float> as NBTBase>::TypeMut<'s, Self::Config>>
    + From<<TypedList<Double> as NBTBase>::TypeMut<'s, Self::Config>>
    + From<<TypedList<ByteArray> as NBTBase>::TypeMut<'s, Self::Config>>
    + From<<TypedList<String> as NBTBase>::TypeMut<'s, Self::Config>>
    + From<<TypedList<List> as NBTBase>::TypeMut<'s, Self::Config>>
    + From<<TypedList<Compound> as NBTBase>::TypeMut<'s, Self::Config>>
    + From<<TypedList<IntArray> as NBTBase>::TypeMut<'s, Self::Config>>
    + From<<TypedList<LongArray> as NBTBase>::TypeMut<'s, Self::Config>>
{
    type Config: ConfigMut;

    #[inline]
    fn ref_<'a, T: NBT>(&'a self) -> Option<&'a T::TypeMut<'s, Self::Config>>
    where
        's: 'a,
    {
        T::mut_shared_(self)
    }

    #[inline]
    fn mut_<'a, T: NBT>(&'a mut self) -> Option<&'a mut T::TypeMut<'s, Self::Config>>
    where
        's: 'a,
    {
        T::mut_(self)
    }

    #[inline]
    fn into_<T: GenericNBT>(self) -> Option<T::TypeMut<'s, Self::Config>> {
        T::mut_into_(self)
    }

    #[inline]
    fn get<'a>(&'a self, index: impl Index) -> Option<<Self::Config as ConfigRef>::Value<'a>>
    where
        's: 'a,
    {
        index.index_dispatch(
            self,
            |value, index| value.ref_::<List>()?.get(index),
            |value, key| value.ref_::<Compound>()?.get(key),
        )
    }

    #[inline]
    fn get_<'a, T: GenericNBT>(&'a self, index: impl Index) -> Option<T::TypeRef<'a, Self::Config>>
    where
        's: 'a,
    {
        index.index_dispatch(
            self,
            |value, index| value.ref_::<List>()?.get_::<T>(index),
            |value, key| value.ref_::<Compound>()?.get_::<T>(key),
        )
    }

    #[inline]
    fn get_mut<'a>(
        &'a mut self,
        index: impl Index,
    ) -> Option<<Self::Config as ConfigMut>::ValueMut<'a>>
    where
        's: 'a,
    {
        index.index_dispatch_mut(
            self,
            |value, index| value.mut_::<List>()?.get_mut(index),
            |value, key| value.mut_::<Compound>()?.get_mut(key),
        )
    }

    #[inline]
    fn get_mut_<'a, T: GenericNBT>(
        &'a mut self,
        index: impl Index,
    ) -> Option<T::TypeMut<'a, Self::Config>>
    where
        's: 'a,
    {
        index.index_dispatch_mut(
            self,
            |value, index| value.mut_::<List>()?.get_mut_::<T>(index),
            |value, key| value.mut_::<Compound>()?.get_mut_::<T>(key),
        )
    }

    fn visit_shared<'a, R>(
        &'a self,
        match_fn: impl FnOnce(VisitMutShared<'a, 's, Self::Config>) -> R,
    ) -> R
    where
        's: 'a;

    fn visit<'a, R>(&'a mut self, match_fn: impl FnOnce(VisitMut<'a, 's, Self::Config>) -> R) -> R
    where
        's: 'a;

    fn map<R>(self, match_fn: impl FnOnce(MapMut<'s, Self::Config>) -> R) -> R;
}

pub trait ListMut<'s>:
    ListBase
    + IntoIterator<
        IntoIter = <Self::Config as ConfigMut>::ListIterMut<'s>,
        Item = <Self::Config as ConfigMut>::ValueMut<'s>,
    >
{
    type Config: ConfigMut;

    fn _to_read_params<'a>(&'a self) -> <Self::Config as ConfigRef>::ReadParams<'a>
    where
        's: 'a;

    #[inline]
    #[allow(clippy::unit_arg)]
    fn get<'a>(&'a self, index: usize) -> Option<<Self::Config as ConfigRef>::Value<'a>>
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
                    Self::Config::read::<End>(Self::Config::list_get::<End>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::Byte => From::from(
                    Self::Config::read::<Byte>(Self::Config::list_get::<Byte>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::Short => From::from(
                    Self::Config::read::<Short>(Self::Config::list_get::<Short>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::Int => From::from(
                    Self::Config::read::<Int>(Self::Config::list_get::<Int>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::Long => From::from(
                    Self::Config::read::<Long>(Self::Config::list_get::<Long>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::Float => From::from(
                    Self::Config::read::<Float>(Self::Config::list_get::<Float>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::Double => From::from(
                    Self::Config::read::<Double>(Self::Config::list_get::<Double>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::ByteArray => From::from(
                    Self::Config::read::<ByteArray>(Self::Config::list_get::<ByteArray>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::String => From::from(
                    Self::Config::read::<String>(Self::Config::list_get::<String>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::List => From::from(
                    Self::Config::read::<List>(Self::Config::list_get::<List>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::Compound => From::from(
                    Self::Config::read::<Compound>(Self::Config::list_get::<Compound>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::IntArray => From::from(
                    Self::Config::read::<IntArray>(Self::Config::list_get::<IntArray>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::LongArray => From::from(
                    Self::Config::read::<LongArray>(Self::Config::list_get::<LongArray>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
            })
        }
    }

    #[inline]
    fn get_<'a, T: GenericNBT>(&'a self, index: usize) -> Option<T::TypeRef<'a, Self::Config>>
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

        unsafe {
            Self::Config::read::<T>(Self::Config::list_get::<T>(self._to_read_params(), index))
        }
    }

    #[inline]
    fn get_mut<'a>(&'a mut self, index: usize) -> Option<<Self::Config as ConfigMut>::ValueMut<'a>>
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
                    Self::Config::read_mut::<End>(Self::Config::list_get::<End>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::Byte => From::from(
                    Self::Config::read_mut::<Byte>(Self::Config::list_get::<Byte>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::Short => From::from(
                    Self::Config::read_mut::<Short>(Self::Config::list_get::<Short>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::Int => From::from(
                    Self::Config::read_mut::<Int>(Self::Config::list_get::<Int>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::Long => From::from(
                    Self::Config::read_mut::<Long>(Self::Config::list_get::<Long>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::Float => From::from(
                    Self::Config::read_mut::<Float>(Self::Config::list_get::<Float>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::Double => From::from(
                    Self::Config::read_mut::<Double>(Self::Config::list_get::<Double>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::ByteArray => From::from(
                    Self::Config::read_mut::<ByteArray>(Self::Config::list_get::<ByteArray>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::String => From::from(
                    Self::Config::read_mut::<String>(Self::Config::list_get::<String>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::List => From::from(
                    Self::Config::read_mut::<List>(Self::Config::list_get::<List>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::Compound => From::from(
                    Self::Config::read_mut::<Compound>(Self::Config::list_get::<Compound>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::IntArray => From::from(
                    Self::Config::read_mut::<IntArray>(Self::Config::list_get::<IntArray>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
                TagID::LongArray => From::from(
                    Self::Config::read_mut::<LongArray>(Self::Config::list_get::<LongArray>(
                        self._to_read_params(),
                        index,
                    ))
                    .unwrap_unchecked(),
                ),
            })
        }
    }

    #[inline]
    fn get_mut_<'a, T: GenericNBT>(
        &'a mut self,
        index: usize,
    ) -> Option<T::TypeMut<'a, Self::Config>>
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

        unsafe {
            Self::Config::read_mut::<T>(Self::Config::list_get::<T>(self._to_read_params(), index))
        }
    }

    #[inline]
    fn push<V: IntoNBT<<Self::Config as ConfigRef>::ByteOrder>>(
        &mut self,
        value: V,
    ) -> Option<<V::Tag as NBTBase>::Type<<Self::Config as ConfigRef>::ByteOrder>> {
        todo!()
    }

    #[inline]
    fn pop(&mut self) -> Option<OwnValue<<Self::Config as ConfigRef>::ByteOrder>> {
        todo!()
    }

    #[inline]
    fn pop_<T: GenericNBT>(&mut self) -> Option<T::Type<<Self::Config as ConfigRef>::ByteOrder>> {
        todo!()
    }

    #[inline]
    fn insert<V: IntoNBT<<Self::Config as ConfigRef>::ByteOrder>>(
        &mut self,
        index: usize,
        value: V,
    ) -> Option<<V::Tag as NBTBase>::Type<<Self::Config as ConfigRef>::ByteOrder>> {
        todo!()
    }

    #[inline]
    fn remove(&mut self, index: usize) -> Option<OwnValue<<Self::Config as ConfigRef>::ByteOrder>> {
        todo!()
    }

    #[inline]
    fn remove_<T: GenericNBT>(
        &mut self,
        index: usize,
    ) -> Option<T::Type<<Self::Config as ConfigRef>::ByteOrder>> {
        todo!()
    }

    fn typed_<T: NBT>(self) -> Option<<Self::Config as ConfigMut>::TypedListMut<'s, T>>;

    fn iter<'a>(&'a self) -> <Self::Config as ConfigRef>::ListIter<'a>
    where
        's: 'a;

    fn iter_mut<'a>(&'a mut self) -> <Self::Config as ConfigMut>::ListIterMut<'a>
    where
        's: 'a;
}

pub trait TypedListMut<'s, T: NBT>:
    TypedListBase<T>
    + IntoIterator<
        IntoIter = <Self::Config as ConfigMut>::TypedListIterMut<'s, T>,
        Item = T::TypeMut<'s, Self::Config>,
    >
{
    type Config: ConfigMut;

    fn _to_read_params<'a>(&'a self) -> <Self::Config as ConfigRef>::ReadParams<'a>
    where
        's: 'a;

    #[inline]
    fn get<'a>(&'a self, index: usize) -> Option<T::TypeRef<'a, Self::Config>>
    where
        's: 'a,
    {
        if index >= self.len() {
            cold_path();
            return None;
        }

        unsafe {
            Self::Config::read::<T>(Self::Config::list_get::<T>(self._to_read_params(), index))
        }
    }

    #[inline]
    fn get_mut<'a>(&'a mut self, index: usize) -> Option<T::TypeMut<'a, Self::Config>>
    where
        's: 'a,
    {
        if index >= self.len() {
            cold_path();
            return None;
        }

        unsafe {
            Self::Config::read_mut::<T>(Self::Config::list_get::<T>(self._to_read_params(), index))
        }
    }

    #[inline]
    fn push(
        &mut self,
        value: impl IntoNBT<<Self::Config as ConfigRef>::ByteOrder, Tag = T>,
    ) -> Option<T::Type<<Self::Config as ConfigRef>::ByteOrder>> {
        todo!()
    }

    #[inline]
    fn pop(&mut self) -> Option<T::Type<<Self::Config as ConfigRef>::ByteOrder>> {
        todo!()
    }

    #[inline]
    fn insert(
        &mut self,
        index: usize,
        value: impl IntoNBT<<Self::Config as ConfigRef>::ByteOrder, Tag = T>,
    ) -> Option<T::Type<<Self::Config as ConfigRef>::ByteOrder>> {
        todo!()
    }

    #[inline]
    fn remove(&mut self, index: usize) -> Option<T::Type<<Self::Config as ConfigRef>::ByteOrder>> {
        todo!()
    }

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
        IntoIter = <Self::Config as ConfigMut>::CompoundIterMut<'s>,
        Item = (
            <Self::Config as ConfigRef>::String<'s>,
            <Self::Config as ConfigMut>::ValueMut<'s>,
        ),
    >
{
    type Config: ConfigMut;

    fn _to_read_params<'a>(&'a self) -> <Self::Config as ConfigRef>::ReadParams<'a>
    where
        's: 'a;

    #[inline]
    fn get<'a>(&'a self, key: &str) -> Option<<Self::Config as ConfigRef>::Value<'a>>
    where
        's: 'a,
    {
        unsafe {
            let (tag_id, params) = Self::Config::compound_get(self._to_read_params(), key)?;
            Some(Self::Config::read_value(tag_id, params))
        }
    }

    #[inline]
    fn get_<'a, T: GenericNBT>(&'a self, key: &str) -> Option<T::TypeRef<'a, Self::Config>>
    where
        's: 'a,
    {
        unsafe {
            let (tag_id, params) = Self::Config::compound_get(self._to_read_params(), key)?;
            if tag_id != T::TAG_ID {
                cold_path();
                return None;
            }
            Self::Config::read::<T>(params)
        }
    }

    #[inline]
    fn get_mut<'a>(&'a mut self, key: &str) -> Option<<Self::Config as ConfigMut>::ValueMut<'a>>
    where
        's: 'a,
    {
        unsafe {
            let (tag_id, params) = Self::Config::compound_get(self._to_read_params(), key)?;
            Some(Self::Config::read_value_mut(tag_id, params))
        }
    }

    #[inline]
    fn get_mut_<'a, T: GenericNBT>(&'a mut self, key: &str) -> Option<T::TypeMut<'a, Self::Config>>
    where
        's: 'a,
    {
        unsafe {
            let (tag_id, params) = Self::Config::compound_get(self._to_read_params(), key)?;
            if tag_id != T::TAG_ID {
                cold_path();
                return None;
            }
            Self::Config::read_mut::<T>(params)
        }
    }

    #[inline]
    fn insert(
        &mut self,
        key: &str,
        value: impl IntoNBT<<Self::Config as ConfigRef>::ByteOrder>,
    ) -> Option<OwnValue<<Self::Config as ConfigRef>::ByteOrder>> {
        todo!()
    }

    #[inline]
    fn insert_<T: GenericNBT>(
        &mut self,
        key: &str,
        value: impl IntoNBT<<Self::Config as ConfigRef>::ByteOrder, Tag = T>,
    ) -> Option<T::Type<<Self::Config as ConfigRef>::ByteOrder>> {
        todo!()
    }

    #[inline]
    fn remove(&mut self, key: &str) -> Option<OwnValue<<Self::Config as ConfigRef>::ByteOrder>> {
        todo!()
    }

    fn remove_<T: GenericNBT>(
        &mut self,
        key: &str,
    ) -> Option<T::Type<<Self::Config as ConfigRef>::ByteOrder>> {
        todo!()
    }

    fn iter<'a>(&'a self) -> <Self::Config as ConfigRef>::CompoundIter<'a>
    where
        's: 'a;

    fn iter_mut<'a>(&'a mut self) -> <Self::Config as ConfigMut>::CompoundIterMut<'a>
    where
        's: 'a;
}
