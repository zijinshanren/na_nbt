use crate::{
    CompoundBase, ConfigRef, Index, ListBase, MapRef, NBT, NBTBase, TypedListBase, ValueBase,
    VisitRef,
    tag::{
        Byte, ByteArray, Compound, Double, End, Float, Int, IntArray, List, Long, LongArray, Short,
        String,
    },
};

pub trait ValueDispatch: NBTBase {
    fn ref_<'a, 's: 'a, V: ValueRef<'s>>(value: &'a V) -> Option<&'a Self::TypeRef<'s, V::Config>>;

    fn into_<'s, V: ValueRef<'s>>(value: V) -> Option<Self::TypeRef<'s, V::Config>>;
}

macro_rules! impl_value_ref_dispatch {
    ($($t:ident),*) => {
        $(
            impl ValueDispatch for $t {
                #[inline]
                fn ref_<'a, 's: 'a, V:ValueRef<'s>>(value: &'a V) -> Option<&'a Self::TypeRef<'s, V::Config>> {
                    value.visit(|value| match value {
                        VisitRef::$t(v) => Some(v),
                        _ => None,
                    })
                }

                #[inline]
                fn into_<'s, V: ValueRef<'s>>(value: V) -> Option<Self::TypeRef<'s, V::Config>> {
                    value.map(|value| match value {
                        MapRef::$t(v) => Some(v),
                        _ => None,
                    })
                }
            }
        )*
    };
}

impl_value_ref_dispatch!(
    End, Byte, Short, Int, Long, Float, Double, ByteArray, String, List, Compound, IntArray,
    LongArray
);

pub trait ValueRef<'s>: ValueBase + Clone + Default {
    type Config: ConfigRef;

    #[inline]
    fn ref_<'a, T: NBT>(&'a self) -> Option<&'a T::TypeRef<'s, Self::Config>>
    where
        's: 'a,
    {
        T::ref_(self)
    }

    #[inline]
    fn into_<T: NBT>(self) -> Option<T::TypeRef<'s, Self::Config>> {
        T::into_(self)
    }

    fn get(&self, index: impl Index) -> Option<<Self::Config as ConfigRef>::Value<'s>>;

    fn get_<T: NBT>(&self, index: impl Index) -> Option<T::TypeRef<'s, Self::Config>>;

    fn visit<'a, R>(&'a self, match_fn: impl FnOnce(VisitRef<'a, 's, Self::Config>) -> R) -> R
    where
        's: 'a;

    fn map<R>(self, match_fn: impl FnOnce(MapRef<'s, Self::Config>) -> R) -> R;
}

pub trait ListRef<'s>:
    ListBase + IntoIterator<Item = <Self::Config as ConfigRef>::Value<'s>> + Clone + Default
{
    type Config: ConfigRef;

    fn get(&self, index: usize) -> Option<<Self::Config as ConfigRef>::Value<'s>>;

    fn get_<T: NBT>(&self, index: usize) -> Option<T::TypeRef<'s, Self::Config>>;

    fn typed_<T: NBT>(self) -> Option<<Self::Config as ConfigRef>::TypedList<'s, T>>;

    fn iter(&self) -> <Self::Config as ConfigRef>::ListIter<'s>;
}

pub trait TypedListRef<'s, T: NBT>:
    TypedListBase<T> + IntoIterator<Item = T::TypeRef<'s, Self::Config>> + Clone + Default
{
    type Config: ConfigRef;

    fn get(&self, index: usize) -> Option<T::TypeRef<'s, Self::Config>>;

    fn iter(&self) -> <Self::Config as ConfigRef>::TypedListIter<'s, T>;
}

pub trait CompoundRef<'s>:
    CompoundBase
    + IntoIterator<
        Item = (
            <Self::Config as ConfigRef>::String<'s>,
            <Self::Config as ConfigRef>::Value<'s>,
        ),
    > + Clone
    + Default
{
    type Config: ConfigRef;

    fn get(&self, key: &str) -> Option<<Self::Config as ConfigRef>::Value<'s>>;

    fn get_<T: NBT>(&self, key: &str) -> Option<T::TypeRef<'s, Self::Config>>;

    fn iter(&self) -> <Self::Config as ConfigRef>::CompoundIter<'s>;
}
