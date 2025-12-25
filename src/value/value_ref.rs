use crate::{
    CompoundBase, ConfigRef, FromAnyNBTRefl, GenericNBT, Index, ListBase, MapRef, NBT, NBTBase,
    TagID, TypedListBase, ValueBase, VisitRef, cold_path,
    tag::{
        Byte, ByteArray, Compound, Double, End, Float, Int, IntArray, List, Long, LongArray, Short,
        String,
    },
};

pub trait ValueRef<'s>: ValueBase + Clone + Default + FromAnyNBTRefl<'s, Self::ConfigRef> {
    #[inline]
    fn ref_<'a, T: NBT>(&'a self) -> Option<&'a T::TypeRef<'s, Self::ConfigRef>>
    where
        's: 'a,
    {
        T::ref_(self)
    }

    #[inline]
    fn into_<T: GenericNBT>(self) -> Option<T::TypeRef<'s, Self::ConfigRef>> {
        T::into_(self)
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
    fn get(&self, index: usize) -> Option<<Self::ConfigRef as ConfigRef>::Value<'s>>;

    fn get_<T: GenericNBT>(&self, index: usize) -> Option<T::TypeRef<'s, Self::ConfigRef>>;

    fn typed_<T: NBT>(self) -> Option<<Self::ConfigRef as ConfigRef>::TypedList<'s, T>>;

    fn iter(&self) -> <Self::ConfigRef as ConfigRef>::ListIter<'s>;
}

pub trait TypedListRef<'s, T: NBTBase>:
    TypedListBase<T> + IntoIterator<Item = T::TypeRef<'s, Self::ConfigRef>> + Clone + Default
{
    fn get(&self, index: usize) -> Option<T::TypeRef<'s, Self::ConfigRef>>;

    fn iter(&self) -> <Self::ConfigRef as ConfigRef>::TypedListIter<'s, T>;
}

pub trait CompoundRef<'s>:
    CompoundBase
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
        match tag_id {
            TagID::End => Some(<Self::ConfigRef as ConfigRef>::Value::from(unsafe {
                <Self::ConfigRef as ConfigRef>::read::<End>(params).unwrap_unchecked()
            })),
            TagID::Byte => Some(<Self::ConfigRef as ConfigRef>::Value::from(unsafe {
                <Self::ConfigRef as ConfigRef>::read::<Byte>(params).unwrap_unchecked()
            })),
            TagID::Short => Some(<Self::ConfigRef as ConfigRef>::Value::from(unsafe {
                <Self::ConfigRef as ConfigRef>::read::<Short>(params).unwrap_unchecked()
            })),
            TagID::Int => Some(<Self::ConfigRef as ConfigRef>::Value::from(unsafe {
                <Self::ConfigRef as ConfigRef>::read::<Int>(params).unwrap_unchecked()
            })),
            TagID::Long => Some(<Self::ConfigRef as ConfigRef>::Value::from(unsafe {
                <Self::ConfigRef as ConfigRef>::read::<Long>(params).unwrap_unchecked()
            })),
            TagID::Float => Some(<Self::ConfigRef as ConfigRef>::Value::from(unsafe {
                <Self::ConfigRef as ConfigRef>::read::<Float>(params).unwrap_unchecked()
            })),
            TagID::Double => Some(<Self::ConfigRef as ConfigRef>::Value::from(unsafe {
                <Self::ConfigRef as ConfigRef>::read::<Double>(params).unwrap_unchecked()
            })),
            TagID::ByteArray => Some(<Self::ConfigRef as ConfigRef>::Value::from(unsafe {
                <Self::ConfigRef as ConfigRef>::read::<ByteArray>(params).unwrap_unchecked()
            })),
            TagID::String => Some(<Self::ConfigRef as ConfigRef>::Value::from(unsafe {
                <Self::ConfigRef as ConfigRef>::read::<String>(params).unwrap_unchecked()
            })),
            TagID::List => Some(<Self::ConfigRef as ConfigRef>::Value::from(unsafe {
                <Self::ConfigRef as ConfigRef>::read::<List>(params).unwrap_unchecked()
            })),
            TagID::Compound => Some(<Self::ConfigRef as ConfigRef>::Value::from(unsafe {
                <Self::ConfigRef as ConfigRef>::read::<Compound>(params).unwrap_unchecked()
            })),
            TagID::IntArray => Some(<Self::ConfigRef as ConfigRef>::Value::from(unsafe {
                <Self::ConfigRef as ConfigRef>::read::<IntArray>(params).unwrap_unchecked()
            })),
            TagID::LongArray => Some(<Self::ConfigRef as ConfigRef>::Value::from(unsafe {
                <Self::ConfigRef as ConfigRef>::read::<LongArray>(params).unwrap_unchecked()
            })),
        }
    }

    #[inline]
    fn get_<T: GenericNBT>(&self, key: &str) -> Option<T::TypeRef<'s, Self::ConfigRef>> {
        let (tag_id, params) = self.compound_get_impl(key)?;
        if tag_id != T::TAG_ID {
            cold_path();
            return None;
        }
        unsafe { <Self::ConfigRef as ConfigRef>::read::<T>(params) }
    }

    fn iter(&self) -> <Self::ConfigRef as ConfigRef>::CompoundIter<'s>;
}
