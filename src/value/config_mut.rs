use crate::{
    CompoundMut, ConfigRef, GenericNBT, ListMut, NBT, TagID, TypedListMut, ValueMut,
    tag::{
        Byte, ByteArray, Compound, Double, End, Float, Int, IntArray, List, Long, LongArray, Short,
        String,
    },
};

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

    unsafe fn list_get_mut<'a, 'doc, T: GenericNBT>(
        value: &'a Self::ListMut<'doc>,
        index: usize,
    ) -> Self::ReadParams<'a>
    where
        'doc: 'a;

    unsafe fn typed_list_get_mut<'a, 'doc, T: NBT>(
        value: &'a Self::TypedListMut<'doc, T>,
        index: usize,
    ) -> Self::ReadParams<'a>
    where
        'doc: 'a;

    unsafe fn compound_get_mut<'a, 'doc>(
        value: &'a Self::CompoundMut<'doc>,
        key: &str,
    ) -> Option<(TagID, Self::ReadParams<'a>)>
    where
        'doc: 'a;

    /// .
    ///
    /// # Safety
    ///
    /// .
    unsafe fn read_mut<'a, 'doc, T: GenericNBT>(
        params: Self::ReadParams<'a>,
    ) -> Option<T::TypeMut<'doc, Self>>;

    /// .
    ///
    /// # Safety
    ///
    /// .
    #[inline]
    #[allow(clippy::unit_arg)]
    unsafe fn read_value_mut<'a, 'doc>(
        tag_id: TagID,
        params: Self::ReadParams<'a>,
    ) -> Self::ValueMut<'doc> {
        unsafe {
            match tag_id {
                TagID::End => From::from(Self::read_mut::<End>(params).unwrap_unchecked()),
                TagID::Byte => From::from(Self::read_mut::<Byte>(params).unwrap_unchecked()),
                TagID::Short => From::from(Self::read_mut::<Short>(params).unwrap_unchecked()),
                TagID::Int => From::from(Self::read_mut::<Int>(params).unwrap_unchecked()),
                TagID::Long => From::from(Self::read_mut::<Long>(params).unwrap_unchecked()),
                TagID::Float => From::from(Self::read_mut::<Float>(params).unwrap_unchecked()),
                TagID::Double => From::from(Self::read_mut::<Double>(params).unwrap_unchecked()),
                TagID::ByteArray => {
                    From::from(Self::read_mut::<ByteArray>(params).unwrap_unchecked())
                }
                TagID::String => From::from(Self::read_mut::<String>(params).unwrap_unchecked()),
                TagID::List => From::from(Self::read_mut::<List>(params).unwrap_unchecked()),
                TagID::Compound => {
                    From::from(Self::read_mut::<Compound>(params).unwrap_unchecked())
                }
                TagID::IntArray => {
                    From::from(Self::read_mut::<IntArray>(params).unwrap_unchecked())
                }
                TagID::LongArray => {
                    From::from(Self::read_mut::<LongArray>(params).unwrap_unchecked())
                }
            }
        }
    }
}
