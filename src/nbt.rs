use crate::{ImmutableNBTImpl, ReadableConfig};

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

pub trait NBTBase: Send + Sync + Sized + Clone + Copy + 'static {
    const TAG_ID: TagID;
    type Type<'a, Config: ReadableConfig>: Clone;
}

pub trait PrimitiveNBTBase: NBTBase {}

pub trait NBT: NBTBase + ImmutableNBTImpl {}

impl<T: NBTBase + ImmutableNBTImpl> NBT for T {}
pub trait PrimitiveNBT: NBT + PrimitiveNBTBase {}

impl<T: NBT + PrimitiveNBTBase> PrimitiveNBT for T {}
