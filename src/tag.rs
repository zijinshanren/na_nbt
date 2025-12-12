#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Tag {
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

impl Tag {
    pub(crate) unsafe fn from_u8_unchecked(value: u8) -> Self {
        unsafe { std::mem::transmute(value) }
    }

    pub const fn is_primitive(self) -> bool {
        matches!(
            self,
            Self::Byte | Self::Short | Self::Int | Self::Long | Self::Float | Self::Double
        )
    }

    pub const fn is_array(self) -> bool {
        matches!(self, Self::ByteArray | Self::IntArray | Self::LongArray)
    }

    pub const fn is_composite(self) -> bool {
        matches!(self, Self::List | Self::Compound)
    }
}
