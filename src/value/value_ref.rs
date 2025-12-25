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
    + From<<End as NBTBase>::TypeRef<'s, Self::ConfigRef>>
    + From<<Byte as NBTBase>::TypeRef<'s, Self::ConfigRef>>
    + From<<Short as NBTBase>::TypeRef<'s, Self::ConfigRef>>
    + From<<Int as NBTBase>::TypeRef<'s, Self::ConfigRef>>
    + From<<Long as NBTBase>::TypeRef<'s, Self::ConfigRef>>
    + From<<Float as NBTBase>::TypeRef<'s, Self::ConfigRef>>
    + From<<Double as NBTBase>::TypeRef<'s, Self::ConfigRef>>
    + From<<ByteArray as NBTBase>::TypeRef<'s, Self::ConfigRef>>
    + From<<String as NBTBase>::TypeRef<'s, Self::ConfigRef>>
    + From<<List as NBTBase>::TypeRef<'s, Self::ConfigRef>>
    + From<<Compound as NBTBase>::TypeRef<'s, Self::ConfigRef>>
    + From<<IntArray as NBTBase>::TypeRef<'s, Self::ConfigRef>>
    + From<<LongArray as NBTBase>::TypeRef<'s, Self::ConfigRef>>
    + From<<TypedList<End> as NBTBase>::TypeRef<'s, Self::ConfigRef>>
    + From<<TypedList<Byte> as NBTBase>::TypeRef<'s, Self::ConfigRef>>
    + From<<TypedList<Short> as NBTBase>::TypeRef<'s, Self::ConfigRef>>
    + From<<TypedList<Int> as NBTBase>::TypeRef<'s, Self::ConfigRef>>
    + From<<TypedList<Long> as NBTBase>::TypeRef<'s, Self::ConfigRef>>
    + From<<TypedList<Float> as NBTBase>::TypeRef<'s, Self::ConfigRef>>
    + From<<TypedList<Double> as NBTBase>::TypeRef<'s, Self::ConfigRef>>
    + From<<TypedList<ByteArray> as NBTBase>::TypeRef<'s, Self::ConfigRef>>
    + From<<TypedList<String> as NBTBase>::TypeRef<'s, Self::ConfigRef>>
    + From<<TypedList<List> as NBTBase>::TypeRef<'s, Self::ConfigRef>>
    + From<<TypedList<Compound> as NBTBase>::TypeRef<'s, Self::ConfigRef>>
    + From<<TypedList<IntArray> as NBTBase>::TypeRef<'s, Self::ConfigRef>>
    + From<<TypedList<LongArray> as NBTBase>::TypeRef<'s, Self::ConfigRef>>
{
    #[inline]
    fn ref_<'a, T: NBT>(&'a self) -> Option<&'a T::TypeRef<'s, Self::ConfigRef>>
    where
        's: 'a,
    {
        T::ref_(self)
    }

    #[inline]
    fn into_<T: GenericNBT>(self) -> Option<T::TypeRef<'s, Self::ConfigRef>> {
        T::ref_into_(self)
    }

    #[inline]
    fn get(&self, index: impl Index) -> Option<<Self::ConfigRef as ConfigRef>::Value<'s>> {
        index.index_dispatch(
            self,
            |value, index| value.ref_::<List>()?.get(index),
            |value, key| value.ref_::<Compound>()?.get(key),
        )
    }

    #[inline]
    fn get_<T: GenericNBT>(&self, index: impl Index) -> Option<T::TypeRef<'s, Self::ConfigRef>> {
        index.index_dispatch(
            self,
            |value, index| value.ref_::<List>()?.get_::<T>(index),
            |value, key| value.ref_::<Compound>()?.get_::<T>(key),
        )
    }

    fn visit<'a, R>(&'a self, match_fn: impl FnOnce(VisitRef<'a, 's, Self::ConfigRef>) -> R) -> R
    where
        's: 'a;

    fn map<R>(self, match_fn: impl FnOnce(MapRef<'s, Self::ConfigRef>) -> R) -> R;
}

pub trait ListRef<'s>:
    ListBase + IntoIterator<Item = <Self::ConfigRef as ConfigRef>::Value<'s>> + Clone + Default
{
    #[inline]
    #[allow(clippy::unit_arg)]
    fn get(&self, index: usize) -> Option<<Self::ConfigRef as ConfigRef>::Value<'s>> {
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
    fn get_<T: GenericNBT>(&self, index: usize) -> Option<T::TypeRef<'s, Self::ConfigRef>> {
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

    fn typed_<T: NBT>(self) -> Option<<Self::ConfigRef as ConfigRef>::TypedList<'s, T>>;

    fn iter(&self) -> <Self::ConfigRef as ConfigRef>::ListIter<'s>;
}

pub trait TypedListRef<'s, T: NBT>:
    TypedListBase<T> + IntoIterator<Item = T::TypeRef<'s, Self::ConfigRef>> + Clone + Default
{
    #[inline]
    fn get(&self, index: usize) -> Option<T::TypeRef<'s, Self::ConfigRef>> {
        if index >= self.len() {
            cold_path();
            return None;
        }

        unsafe { Self::ConfigRef::read::<T>(self.typed_list_get_impl(index)) }
    }

    fn iter(&self) -> <Self::ConfigRef as ConfigRef>::TypedListIter<'s, T>;
}

pub trait CompoundRef<'s>:
    CompoundBase<'s>
    + IntoIterator<
        Item = (
            <Self::ConfigRef as ConfigRef>::String<'s>,
            <Self::ConfigRef as ConfigRef>::Value<'s>,
        ),
    > + Clone
    + Default
{
    #[inline]
    fn get(&self, key: &str) -> Option<<Self::ConfigRef as ConfigRef>::Value<'s>> {
        let (tag_id, params) = self.compound_get_impl(key)?;
        unsafe { Some(Self::ConfigRef::read_value(tag_id, params)) }
    }

    #[inline]
    fn get_<T: GenericNBT>(&self, key: &str) -> Option<T::TypeRef<'s, Self::ConfigRef>> {
        let (tag_id, params) = self.compound_get_impl(key)?;
        if tag_id != T::TAG_ID {
            cold_path();
            return None;
        }
        unsafe { Self::ConfigRef::read::<T>(params) }
    }

    fn iter(&self) -> <Self::ConfigRef as ConfigRef>::CompoundIter<'s>;
}
