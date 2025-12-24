use crate::{
    ByteOrder, ConfigMut, ConfigRef, ImmutableGenericNBTImpl, ImmutableNBTImpl, ValueDispatch,
};

pub mod tag;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum TagID {
    End = 0,
    Byte = 1,
    Short = 2,
    Int = 3,
    Long = 4,
    Float = 5,
    Double = 6,
    ByteArray = 7,
    String = 8,
    List = 9,
    Compound = 10,
    IntArray = 11,
    LongArray = 12,
}

impl TagID {
    /// Creates a `Tag` from a raw byte value without validation.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `value` is a valid tag type (0-12).
    /// Passing an invalid value results in undefined behavior.
    pub(crate) const unsafe fn from_u8_unchecked(value: u8) -> Self {
        unsafe { std::mem::transmute(value) }
    }

    /// Returns `true` if this is a primitive tag type.
    ///
    /// Primitive tags are: End, Byte, Short, Int, Long, Float, Double.
    /// These tags store their values directly without additional structure.
    ///
    /// # Example
    ///
    /// ```
    /// use na_nbt::Tag;
    ///
    /// assert!(Tag::Int.is_primitive());
    /// assert!(Tag::Double.is_primitive());
    /// assert!(!Tag::List.is_primitive());
    /// assert!(!Tag::ByteArray.is_primitive());
    /// ```
    pub const fn is_primitive(self) -> bool {
        matches!(
            self,
            Self::End
                | Self::Byte
                | Self::Short
                | Self::Int
                | Self::Long
                | Self::Float
                | Self::Double
        )
    }

    /// Returns `true` if this is an array tag type.
    ///
    /// Array tags are: ByteArray, IntArray, LongArray.
    /// These store contiguous sequences of primitive values.
    ///
    /// # Example
    ///
    /// ```
    /// use na_nbt::Tag;
    ///
    /// assert!(Tag::ByteArray.is_array());
    /// assert!(Tag::IntArray.is_array());
    /// assert!(Tag::LongArray.is_array());
    /// assert!(!Tag::List.is_array());
    /// ```
    pub const fn is_array(self) -> bool {
        matches!(self, Self::ByteArray | Self::IntArray | Self::LongArray)
    }

    /// Returns `true` if this is a composite tag type.
    ///
    /// Composite tags are: List, Compound.
    /// These contain other NBT values as children.
    ///
    /// # Example
    ///
    /// ```
    /// use na_nbt::Tag;
    ///
    /// assert!(Tag::List.is_composite());
    /// assert!(Tag::Compound.is_composite());
    /// assert!(!Tag::Int.is_composite());
    /// assert!(!Tag::ByteArray.is_composite());
    /// ```
    pub const fn is_composite(self) -> bool {
        matches!(self, Self::List | Self::Compound)
    }
}

mod private {
    use crate::{NBT, tag::*};
    pub trait Sealed {}
    impl Sealed for End {}
    impl Sealed for Byte {}
    impl Sealed for Short {}
    impl Sealed for Int {}
    impl Sealed for Long {}
    impl Sealed for Float {}
    impl Sealed for Double {}
    impl Sealed for ByteArray {}
    impl Sealed for String {}
    impl Sealed for List {}
    impl Sealed for Compound {}
    impl Sealed for IntArray {}
    impl Sealed for LongArray {}
    impl<T: NBT> Sealed for TypedList<T> {}
}

pub trait NBTBase: private::Sealed + Send + Sync + Sized + Clone + Copy + 'static {
    const TAG_ID: TagID;
    type TypeRef<'a, Config: ConfigRef>: Clone;
    type TypeMut<'a, Config: ConfigMut>;
    type Type<O: ByteOrder>: Default;
}

pub trait PrimitiveNBTBase: NBTBase {}

pub trait NBTImpl: ImmutableNBTImpl + ValueDispatch {}

impl<T: ImmutableNBTImpl + ValueDispatch> NBTImpl for T {}

pub trait NBT: NBTBase + NBTImpl {}

impl<T: NBTBase + NBTImpl> NBT for T {}
pub trait PrimitiveNBT: NBT + PrimitiveNBTBase {}

impl<T: NBT + PrimitiveNBTBase> PrimitiveNBT for T {}

pub trait GenericNBTImpl: ImmutableGenericNBTImpl {}

impl<T: ImmutableGenericNBTImpl> GenericNBTImpl for T {}

pub trait GenericNBT: NBTBase + GenericNBTImpl {}

impl<T: NBTBase + GenericNBTImpl> GenericNBT for T {}
