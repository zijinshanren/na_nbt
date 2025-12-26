use crate::{
    CompoundMut, ConfigRef, GenericNBT, ListMut, NBT, OwnValue, TagID, TypedListMut, ValueMut,
    tag::{
        Byte, ByteArray, Compound, Double, End, Float, Int, IntArray, List, Long, LongArray, Short,
        String,
    },
};

pub trait ConfigMut: ConfigRef {
    type ValueMut<'doc>: ValueMut<'doc, Config = Self>;
    type ListMut<'doc>: ListMut<'doc, Config = Self>;
    type ListIterMut<'doc>: Iterator<Item = Self::ValueMut<'doc>> + ExactSizeIterator + Default;
    type TypedListMut<'doc, T: NBT>: TypedListMut<'doc, T, Config = Self>;
    type TypedListIterMut<'doc, T: NBT>: Iterator<Item = T::TypeMut<'doc, Self>>
        + ExactSizeIterator
        + Default;
    type CompoundMut<'doc>: CompoundMut<'doc, Config = Self>;
    type CompoundIterMut<'doc>: Iterator<Item = (Self::String<'doc>, Self::ValueMut<'doc>)>
        + Default;

    type WriteParams<'a>: Sized;

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

    /// .
    ///
    /// # Safety
    ///
    /// Will NOT check if the value is of the correct type.
    unsafe fn list_push<'a, T: NBT>(params: Self::WriteParams<'a>, value: T::Type<Self::ByteOrder>);

    /// .
    ///
    /// # Safety
    ///
    /// Will NOT check bounds nor if the value is of the correct type.
    /// But will NOT pop and return None if T is TypedList and the type of the elements is not TypedList<N>
    unsafe fn list_pop<'a, T: GenericNBT>(
        params: Self::WriteParams<'a>,
    ) -> Option<T::Type<Self::ByteOrder>>;

    /// .
    ///
    /// # Safety
    ///
    /// Will NOT check bounds nor if the value is of the correct type.
    unsafe fn list_insert<'a, T: NBT>(
        params: Self::WriteParams<'a>,
        index: usize,
        value: T::Type<Self::ByteOrder>,
    );

    /// .
    ///
    /// # Safety
    ///
    /// Will NOT check bounds nor if the value is of the correct type.
    unsafe fn list_remove<'a, T: GenericNBT>(
        params: Self::WriteParams<'a>,
        index: usize,
    ) -> Option<T::Type<Self::ByteOrder>>;

    /// .
    ///
    /// # Safety
    ///
    /// Will NOT check if the key is already in the compound.
    /// NEED to call compound_remove first to check if the key is already in the compound.
    unsafe fn compound_insert<'a, T: GenericNBT>(
        params: Self::WriteParams<'a>,
        key: &[u8],
        value: T::Type<Self::ByteOrder>,
    );

    /// .
    ///
    /// # Safety
    ///
    /// .
    unsafe fn compound_remove<'a>(
        params: Self::WriteParams<'a>,
        key: &[u8],
    ) -> Option<OwnValue<Self::ByteOrder>>;
}
