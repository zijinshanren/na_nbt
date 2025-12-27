use crate::{
    ByteOrder, ConfigMut, ConfigRef, Error, ImmutableGenericImpl, MutableGenericImpl, NBTInto,
    NBTRef, cold_path,
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
    #[inline]
    pub(crate) const unsafe fn from_u8_unchecked(value: u8) -> Self {
        unsafe { std::mem::transmute(value) }
    }

    #[inline]
    pub const fn from_u8(value: u8) -> Result<Self, Error> {
        match value {
            0 => Ok(Self::End),
            1 => Ok(Self::Byte),
            2 => Ok(Self::Short),
            3 => Ok(Self::Int),
            4 => Ok(Self::Long),
            5 => Ok(Self::Float),
            6 => Ok(Self::Double),
            7 => Ok(Self::ByteArray),
            8 => Ok(Self::String),
            9 => Ok(Self::List),
            10 => Ok(Self::Compound),
            11 => Ok(Self::IntArray),
            12 => Ok(Self::LongArray),
            _ => {
                cold_path();
                Err(Error::INVALID(value))
            }
        }
    }

    /// Returns `true` if this is a primitive tag type.
    ///
    /// Primitive tags are: End, Byte, Short, Int, Long, Float, Double.
    /// These tags store their values directly without additional structure.
    ///
    /// # Example
    ///
    /// ```
    /// use na_nbt::TagID;
    ///
    /// assert!(TagID::Int.is_primitive());
    /// assert!(TagID::Double.is_primitive());
    /// assert!(!TagID::List.is_primitive());
    /// assert!(!TagID::ByteArray.is_primitive());
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
    /// use na_nbt::TagID;
    ///
    /// assert!(TagID::ByteArray.is_array());
    /// assert!(TagID::IntArray.is_array());
    /// assert!(TagID::LongArray.is_array());
    /// assert!(!TagID::List.is_array());
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
    /// use na_nbt::TagID;
    ///
    /// assert!(TagID::List.is_composite());
    /// assert!(TagID::Compound.is_composite());
    /// assert!(!TagID::Int.is_composite());
    /// assert!(!TagID::ByteArray.is_composite());
    /// ```
    pub const fn is_composite(self) -> bool {
        matches!(self, Self::List | Self::Compound)
    }
}

mod private {
    use crate::{NBTBase, tag::*};
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
    impl<T: NBTBase> Sealed for TypedList<T> {}
}

pub trait NBTBase: private::Sealed + Send + Sync + Sized + Clone + Copy + 'static {
    const TAG_ID: TagID;
    type Element: NBT;
    type TypeRef<'a, Config: ConfigRef>: Clone;
    type TypeMut<'a, Config: ConfigMut>;
    type Type<O: ByteOrder>: Default;

    fn dispatch<A, R>(
        a: A,
        end: impl FnOnce(A) -> R,
        normal: impl FnOnce(A) -> R,
        typed_list: impl FnOnce(A) -> R,
    ) -> R;
}

pub trait PrimitiveNBTBase: NBTBase {}

macro_rules! define_trait {
    ($name:ident: $first:path $(, $rest:path)*) => {
        pub trait $name: $first $(+ $rest)* {}

        impl<T: $first $(+ $rest)*> $name for T {}
    };
}

// todo: NBTBase -> GenericNBTBase
// todo: add NBTBase

define_trait!(GenericNBT: NBTBase, NBTInto, ImmutableGenericImpl, MutableGenericImpl);

define_trait!(NBT: GenericNBT, NBTRef);

define_trait!(PrimitiveNBT: NBTBase, PrimitiveNBTBase);
