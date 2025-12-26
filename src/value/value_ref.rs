use crate::{
    CompoundBase, ConfigRef, GenericNBT, Index, ListBase, MapRef, NBT, NBTBase, TagID,
    TypedListBase, ValueBase, VisitRef, cold_path,
    tag::{
        Byte, ByteArray, Compound, Double, End, Float, Int, IntArray, List, Long, LongArray, Short,
        String, TypedList,
    },
};

pub trait ValueRef<'s>:
    ValueBase
    + Clone
    + Default
    + From<<End as NBTBase>::TypeRef<'s, Self::Config>>
    + From<<Byte as NBTBase>::TypeRef<'s, Self::Config>>
    + From<<Short as NBTBase>::TypeRef<'s, Self::Config>>
    + From<<Int as NBTBase>::TypeRef<'s, Self::Config>>
    + From<<Long as NBTBase>::TypeRef<'s, Self::Config>>
    + From<<Float as NBTBase>::TypeRef<'s, Self::Config>>
    + From<<Double as NBTBase>::TypeRef<'s, Self::Config>>
    + From<<ByteArray as NBTBase>::TypeRef<'s, Self::Config>>
    + From<<String as NBTBase>::TypeRef<'s, Self::Config>>
    + From<<List as NBTBase>::TypeRef<'s, Self::Config>>
    + From<<Compound as NBTBase>::TypeRef<'s, Self::Config>>
    + From<<IntArray as NBTBase>::TypeRef<'s, Self::Config>>
    + From<<LongArray as NBTBase>::TypeRef<'s, Self::Config>>
    + From<<TypedList<End> as NBTBase>::TypeRef<'s, Self::Config>>
    + From<<TypedList<Byte> as NBTBase>::TypeRef<'s, Self::Config>>
    + From<<TypedList<Short> as NBTBase>::TypeRef<'s, Self::Config>>
    + From<<TypedList<Int> as NBTBase>::TypeRef<'s, Self::Config>>
    + From<<TypedList<Long> as NBTBase>::TypeRef<'s, Self::Config>>
    + From<<TypedList<Float> as NBTBase>::TypeRef<'s, Self::Config>>
    + From<<TypedList<Double> as NBTBase>::TypeRef<'s, Self::Config>>
    + From<<TypedList<ByteArray> as NBTBase>::TypeRef<'s, Self::Config>>
    + From<<TypedList<String> as NBTBase>::TypeRef<'s, Self::Config>>
    + From<<TypedList<List> as NBTBase>::TypeRef<'s, Self::Config>>
    + From<<TypedList<Compound> as NBTBase>::TypeRef<'s, Self::Config>>
    + From<<TypedList<IntArray> as NBTBase>::TypeRef<'s, Self::Config>>
    + From<<TypedList<LongArray> as NBTBase>::TypeRef<'s, Self::Config>>
{
    type Config: ConfigRef;

    #[inline]
    fn ref_<'a, T: NBT>(&'a self) -> Option<&'a T::TypeRef<'s, Self::Config>>
    where
        's: 'a,
    {
        T::ref_(self)
    }

    #[inline]
    fn into_<T: GenericNBT>(self) -> Option<T::TypeRef<'s, Self::Config>> {
        T::ref_into_(self)
    }

    #[inline]
    fn get(&self, index: impl Index) -> Option<<Self::Config as ConfigRef>::Value<'s>> {
        index.index_dispatch(
            self,
            |value, index| value.ref_::<List>()?.get(index),
            |value, key| value.ref_::<Compound>()?.get(key),
        )
    }

    #[inline]
    fn get_<T: GenericNBT>(&self, index: impl Index) -> Option<T::TypeRef<'s, Self::Config>> {
        index.index_dispatch(
            self,
            |value, index| value.ref_::<List>()?.get_::<T>(index),
            |value, key| value.ref_::<Compound>()?.get_::<T>(key),
        )
    }

    fn visit<'a, R>(&'a self, match_fn: impl FnOnce(VisitRef<'a, 's, Self::Config>) -> R) -> R
    where
        's: 'a;

    fn map<R>(self, match_fn: impl FnOnce(MapRef<'s, Self::Config>) -> R) -> R;
}

pub trait ListRef<'s>:
    ListBase
    + IntoIterator<
        IntoIter = <Self::Config as ConfigRef>::ListIter<'s>,
        Item = <Self::Config as ConfigRef>::Value<'s>,
    > + Clone
    + Default
{
    type Config: ConfigRef;

    fn _to_read_params<'a>(&'a self) -> <Self::Config as ConfigRef>::ReadParams<'a>
    where
        's: 'a;

    #[inline]
    #[allow(clippy::unit_arg)]
    fn get(&self, index: usize) -> Option<<Self::Config as ConfigRef>::Value<'s>> {
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
    fn get_<T: GenericNBT>(&self, index: usize) -> Option<T::TypeRef<'s, Self::Config>> {
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

    fn typed_<T: NBT>(self) -> Option<<Self::Config as ConfigRef>::TypedList<'s, T>>;

    fn iter(&self) -> <Self::Config as ConfigRef>::ListIter<'s>;
}

pub trait TypedListRef<'s, T: NBT>:
    TypedListBase<T>
    + IntoIterator<
        IntoIter = <Self::Config as ConfigRef>::TypedListIter<'s, T>,
        Item = T::TypeRef<'s, Self::Config>,
    > + Clone
    + Default
{
    type Config: ConfigRef;

    fn _to_read_params<'a>(&'a self) -> <Self::Config as ConfigRef>::ReadParams<'a>
    where
        's: 'a;

    #[inline]
    fn get(&self, index: usize) -> Option<T::TypeRef<'s, Self::Config>> {
        if index >= self.len() {
            cold_path();
            return None;
        }

        unsafe {
            Self::Config::read::<T>(Self::Config::list_get::<T>(self._to_read_params(), index))
        }
    }

    fn iter(&self) -> <Self::Config as ConfigRef>::TypedListIter<'s, T>;
}

pub trait CompoundRef<'s>:
    CompoundBase
    + IntoIterator<
        IntoIter = <Self::Config as ConfigRef>::CompoundIter<'s>,
        Item = (
            <Self::Config as ConfigRef>::String<'s>,
            <Self::Config as ConfigRef>::Value<'s>,
        ),
    > + Clone
    + Default
{
    type Config: ConfigRef;

    fn _to_read_params<'a>(&'a self) -> <Self::Config as ConfigRef>::ReadParams<'a>
    where
        's: 'a;

    #[inline]
    fn get(&self, key: &str) -> Option<<Self::Config as ConfigRef>::Value<'s>> {
        unsafe {
            let key = simd_cesu8::mutf8::encode(key);
            let (tag_id, params) = Self::Config::compound_get(self._to_read_params(), &key)?;
            Some(Self::Config::read_value(tag_id, params))
        }
    }

    #[inline]
    fn get_<T: GenericNBT>(&self, key: &str) -> Option<T::TypeRef<'s, Self::Config>> {
        unsafe {
            let key = simd_cesu8::mutf8::encode(key);
            let (tag_id, params) = Self::Config::compound_get(self._to_read_params(), &key)?;
            if tag_id != T::TAG_ID {
                cold_path();
                return None;
            }
            Self::Config::read::<T>(params)
        }
    }

    fn iter(&self) -> <Self::Config as ConfigRef>::CompoundIter<'s>;
}
