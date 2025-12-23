use std::{hint::unreachable_unchecked, io::Write, marker::PhantomData, mem::ManuallyDrop, ptr};

use zerocopy::byteorder;

use crate::{
    ByteOrder, ImmutableCompound, ImmutableList, ImmutableString, ImmutableValue, IntoOwnedValue,
    MutableCompound, MutableList, MutableValue, Result, ScopedReadableValue as _, TagID, cold_path,
    index::Index,
    mutable::{
        iter::{
            ImmutableCompoundIter, ImmutableListIter, MutableCompoundIter, MutableListIter,
            OwnedCompoundIter, OwnedListIter,
        },
        util::{
            compound_get, compound_get_mut, compound_iter, compound_iter_mut, compound_remove,
            list_get, list_get_mut, list_is_empty, list_iter, list_iter_mut, list_len, list_pop,
            list_remove, list_tag_id, tag_size,
        },
    },
    view::{StringViewMut, StringViewOwn, VecViewMut, VecViewOwn},
    write_owned_to_vec, write_owned_to_writer,
};

impl<T> VecViewOwn<T> {
    pub(crate) unsafe fn write(self, dst: *mut u8) {
        unsafe { ptr::write(dst.cast(), self) };
    }

    pub(crate) unsafe fn read(src: *mut u8) -> Self {
        unsafe { ptr::read(src.cast()) }
    }
}

impl StringViewOwn {
    pub(crate) unsafe fn write(self, dst: *mut u8) {
        unsafe { ptr::write(dst.cast(), self) };
    }

    pub(crate) unsafe fn read(src: *mut u8) -> Self {
        unsafe { ptr::read(src.cast()) }
    }
}

/// An owned, mutable NBT value.
///
/// Unlike [`BorrowedValue`](crate::BorrowedValue), this type owns its data and can be
/// modified. Use this when you need to:
/// - Modify NBT data after parsing
/// - Keep values after the source bytes are dropped
/// - Build NBT structures from scratch
/// - Convert between endianness formats
///
/// # Variants
///
/// Each variant corresponds to an NBT tag type:
///
/// | Variant | Tag ID | Description |
/// |---------|--------|-------------|
/// | `End` | 0 | Marks the end of a compound |
/// | `Byte` | 1 | Signed 8-bit integer |
/// | `Short` | 2 | Signed 16-bit integer |
/// | `Int` | 3 | Signed 32-bit integer |
/// | `Long` | 4 | Signed 64-bit integer |
/// | `Float` | 5 | 32-bit floating point |
/// | `Double` | 6 | 64-bit floating point |
/// | `ByteArray` | 7 | Array of bytes |
/// | `String` | 8 | UTF-8 string |
/// | `List` | 9 | Homogeneous list |
/// | `Compound` | 10 | Key-value map |
/// | `IntArray` | 11 | Array of ints |
/// | `LongArray` | 12 | Array of longs |
///
/// # Creating Values
///
/// `OwnedValue` implements `From` for many Rust types, making construction easy:
///
/// ```
/// use na_nbt::OwnedValue;
/// use zerocopy::byteorder::BigEndian;
///
/// let byte: OwnedValue<BigEndian> = 42i8.into();
/// let int: OwnedValue<BigEndian> = 12345i32.into();
/// let string: OwnedValue<BigEndian> = "Hello".into();
/// let array: OwnedValue<BigEndian> = vec![1i8, 2, 3].into();
/// ```
///
/// # Parsing NBT
///
/// ```
/// use na_nbt::{read_owned, OwnedValue};
/// use zerocopy::byteorder::BigEndian;
///
/// let data = [0x0a, 0x00, 0x00, 0x00]; // Empty compound
/// let root: OwnedValue<BigEndian> = read_owned::<BigEndian, BigEndian>(&data).unwrap();
/// ```
///
/// # Modifying Values
///
/// ```
/// use na_nbt::{read_owned, OwnedValue};
/// use zerocopy::byteorder::BigEndian;
///
/// let data = [0x0a, 0x00, 0x00, 0x00];
/// let mut root: OwnedValue<BigEndian> = read_owned::<BigEndian, BigEndian>(&data).unwrap();
///
/// if let OwnedValue::Compound(ref mut c) = root {
///     c.insert("name", "Steve");
///     c.insert("health", 20i32);
/// }
/// ```
///
/// # Writing to Bytes
///
/// ```
/// use na_nbt::{read_owned, OwnedValue, ScopedReadableValue};
/// use zerocopy::byteorder::BigEndian;
///
/// let data = [0x0a, 0x00, 0x00, 0x00];
/// let root: OwnedValue<BigEndian> = read_owned::<BigEndian, BigEndian>(&data).unwrap();
///
/// // Serialize to bytes
/// let bytes = root.write_to_vec::<BigEndian>().unwrap();
/// ```
///
/// # Generic Parameter
///
/// `O` specifies the byte order for multi-byte values stored in memory.
/// Choose based on your target format to minimize conversion overhead:
///
/// - Use [`BigEndian`](zerocopy::byteorder::BigEndian) for Java Edition
/// - Use [`LittleEndian`](zerocopy::byteorder::LittleEndian) for Bedrock Edition
///
/// # See Also
///
/// - [`MutableValue`] - Mutable borrowed view into an `OwnedValue`
/// - [`ImmutableValue`] - Immutable borrowed view into an `OwnedValue`
/// - [`BorrowedValue`](crate::BorrowedValue) - Zero-copy alternative for read-only access
pub enum OwnedValue<O: ByteOrder> {
    /// End tag (0) - marks the end of a compound.
    End(()),
    /// Byte tag (1) - a signed 8-bit integer.
    Byte(i8),
    /// Short tag (2) - a signed 16-bit integer.
    Short(byteorder::I16<O>),
    /// Int tag (3) - a signed 32-bit integer.
    Int(byteorder::I32<O>),
    /// Long tag (4) - a signed 64-bit integer.
    Long(byteorder::I64<O>),
    /// Float tag (5) - a 32-bit IEEE 754 floating point number.
    Float(byteorder::F32<O>),
    /// Double tag (6) - a 64-bit IEEE 754 floating point number.
    Double(byteorder::F64<O>),
    /// Byte array tag (7) - an array of signed bytes.
    ByteArray(VecViewOwn<i8>),
    /// String tag (8) - a UTF-8 encoded string.
    String(StringViewOwn),
    /// List tag (9) - a list of values, all of the same type.
    List(OwnedList<O>),
    /// Compound tag (10) - a map of string keys to NBT values.
    Compound(OwnedCompound<O>),
    /// Int array tag (11) - an array of signed 32-bit integers.
    IntArray(VecViewOwn<byteorder::I32<O>>),
    /// Long array tag (12) - an array of signed 64-bit integers.
    LongArray(VecViewOwn<byteorder::I64<O>>),
}

impl<O: ByteOrder> From<()> for OwnedValue<O> {
    fn from(_: ()) -> Self {
        OwnedValue::End
    }
}

impl<O: ByteOrder> From<i8> for OwnedValue<O> {
    fn from(value: i8) -> Self {
        OwnedValue::Byte(value)
    }
}

impl<O: ByteOrder> From<i16> for OwnedValue<O> {
    fn from(value: i16) -> Self {
        OwnedValue::Short(value.into())
    }
}

impl<O: ByteOrder> From<byteorder::I16<O>> for OwnedValue<O> {
    fn from(value: byteorder::I16<O>) -> Self {
        OwnedValue::Short(value)
    }
}

impl<O: ByteOrder> From<i32> for OwnedValue<O> {
    fn from(value: i32) -> Self {
        OwnedValue::Int(value.into())
    }
}

impl<O: ByteOrder> From<byteorder::I32<O>> for OwnedValue<O> {
    fn from(value: byteorder::I32<O>) -> Self {
        OwnedValue::Int(value)
    }
}

impl<O: ByteOrder> From<i64> for OwnedValue<O> {
    fn from(value: i64) -> Self {
        OwnedValue::Long(value.into())
    }
}

impl<O: ByteOrder> From<byteorder::I64<O>> for OwnedValue<O> {
    fn from(value: byteorder::I64<O>) -> Self {
        OwnedValue::Long(value)
    }
}

impl<O: ByteOrder> From<f32> for OwnedValue<O> {
    fn from(value: f32) -> Self {
        OwnedValue::Float(value.into())
    }
}

impl<O: ByteOrder> From<byteorder::F32<O>> for OwnedValue<O> {
    fn from(value: byteorder::F32<O>) -> Self {
        OwnedValue::Float(value)
    }
}

impl<O: ByteOrder> From<f64> for OwnedValue<O> {
    fn from(value: f64) -> Self {
        OwnedValue::Double(value.into())
    }
}

impl<O: ByteOrder> From<byteorder::F64<O>> for OwnedValue<O> {
    fn from(value: byteorder::F64<O>) -> Self {
        OwnedValue::Double(value)
    }
}

impl<O: ByteOrder> From<&[i8]> for OwnedValue<O> {
    fn from(value: &[i8]) -> Self {
        OwnedValue::ByteArray(value.into())
    }
}

impl<O: ByteOrder, const N: usize> From<[i8; N]> for OwnedValue<O> {
    fn from(value: [i8; N]) -> Self {
        OwnedValue::ByteArray(value.to_vec().into())
    }
}

impl<O: ByteOrder> From<Vec<i8>> for OwnedValue<O> {
    fn from(value: Vec<i8>) -> Self {
        OwnedValue::ByteArray(value.into())
    }
}

impl<O: ByteOrder> From<&str> for OwnedValue<O> {
    fn from(value: &str) -> Self {
        OwnedValue::String(value.into())
    }
}

impl<O: ByteOrder> From<String> for OwnedValue<O> {
    fn from(value: String) -> Self {
        OwnedValue::String(value.into())
    }
}

impl<O: ByteOrder> From<OwnedList<O>> for OwnedValue<O> {
    fn from(value: OwnedList<O>) -> Self {
        OwnedValue::List(value)
    }
}

impl<O: ByteOrder> From<OwnedCompound<O>> for OwnedValue<O> {
    fn from(value: OwnedCompound<O>) -> Self {
        OwnedValue::Compound(value)
    }
}

impl<O: ByteOrder> From<&[byteorder::I32<O>]> for OwnedValue<O> {
    fn from(value: &[byteorder::I32<O>]) -> Self {
        OwnedValue::IntArray(value.into())
    }
}

impl<O: ByteOrder, const N: usize> From<[byteorder::I32<O>; N]> for OwnedValue<O> {
    fn from(value: [byteorder::I32<O>; N]) -> Self {
        OwnedValue::IntArray(value.to_vec().into())
    }
}

impl<O: ByteOrder> From<Vec<byteorder::I32<O>>> for OwnedValue<O> {
    fn from(value: Vec<byteorder::I32<O>>) -> Self {
        OwnedValue::IntArray(value.into())
    }
}

impl<O: ByteOrder> From<&[byteorder::I64<O>]> for OwnedValue<O> {
    fn from(value: &[byteorder::I64<O>]) -> Self {
        OwnedValue::LongArray(value.into())
    }
}

impl<O: ByteOrder, const N: usize> From<[byteorder::I64<O>; N]> for OwnedValue<O> {
    fn from(value: [byteorder::I64<O>; N]) -> Self {
        OwnedValue::LongArray(value.to_vec().into())
    }
}

impl<O: ByteOrder> From<Vec<byteorder::I64<O>>> for OwnedValue<O> {
    fn from(value: Vec<byteorder::I64<O>>) -> Self {
        OwnedValue::LongArray(value.into())
    }
}

impl<O: ByteOrder> OwnedValue<O> {
    pub(crate) unsafe fn write(self, dst: *mut u8) {
        unsafe {
            match self {
                OwnedValue::End => {}
                OwnedValue::Byte(value) => {
                    ptr::write(dst.cast(), value.to_ne_bytes());
                }
                OwnedValue::Short(value) => {
                    ptr::write(dst.cast(), value.to_bytes());
                }
                OwnedValue::Int(value) => {
                    ptr::write(dst.cast(), value.to_bytes());
                }
                OwnedValue::Long(value) => {
                    ptr::write(dst.cast(), value.to_bytes());
                }
                OwnedValue::Float(value) => {
                    ptr::write(dst.cast(), value.to_bytes());
                }
                OwnedValue::Double(value) => {
                    ptr::write(dst.cast(), value.to_bytes());
                }
                OwnedValue::ByteArray(value) => {
                    value.write(dst);
                }
                OwnedValue::String(value) => {
                    value.write(dst);
                }
                OwnedValue::List(value) => {
                    value.write(dst);
                }
                OwnedValue::Compound(value) => {
                    value.write(dst);
                }
                OwnedValue::IntArray(value) => {
                    value.write(dst);
                }
                OwnedValue::LongArray(value) => {
                    value.write(dst);
                }
            }
        }
    }

    pub(crate) unsafe fn read(tag_id: TagID, src: *mut u8) -> Self {
        unsafe {
            match tag_id {
                TagID::End => OwnedValue::End,
                TagID::Byte => OwnedValue::Byte(ptr::read(src.cast())),
                TagID::Short => OwnedValue::Short(ptr::read(src.cast())),
                TagID::Int => OwnedValue::Int(ptr::read(src.cast())),
                TagID::Long => OwnedValue::Long(ptr::read(src.cast())),
                TagID::Float => OwnedValue::Float(ptr::read(src.cast())),
                TagID::Double => OwnedValue::Double(ptr::read(src.cast())),
                TagID::ByteArray => OwnedValue::ByteArray(VecViewOwn::read(src.cast())),
                TagID::String => OwnedValue::String(StringViewOwn::read(src.cast())),
                TagID::List => OwnedValue::List(OwnedList::read(src.cast())),
                TagID::Compound => OwnedValue::Compound(OwnedCompound::read(src.cast())),
                TagID::IntArray => OwnedValue::IntArray(VecViewOwn::read(src.cast())),
                TagID::LongArray => OwnedValue::LongArray(VecViewOwn::read(src.cast())),
            }
        }
    }
}

impl<O: ByteOrder> OwnedValue<O> {
    #[inline]
    pub fn tag_id(&self) -> TagID {
        unsafe { *(self as *const Self as *const TagID) }
    }

    #[inline]
    pub fn as_end(&self) -> Option<()> {
        match self {
            OwnedValue::End => Some(()),
            _ => None,
        }
    }

    #[inline]
    pub fn is_end(&self) -> bool {
        matches!(self, OwnedValue::End)
    }

    #[inline]
    pub fn as_byte(&self) -> Option<i8> {
        match self {
            OwnedValue::Byte(value) => Some(*value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_byte(&self) -> bool {
        matches!(self, OwnedValue::Byte(_))
    }

    #[inline]
    pub fn as_short(&self) -> Option<i16> {
        match self {
            OwnedValue::Short(value) => Some(value.get()),
            _ => None,
        }
    }

    #[inline]
    pub fn is_short(&self) -> bool {
        matches!(self, OwnedValue::Short(_))
    }

    #[inline]
    pub fn as_int(&self) -> Option<i32> {
        match self {
            OwnedValue::Int(value) => Some(value.get()),
            _ => None,
        }
    }

    #[inline]
    pub fn is_int(&self) -> bool {
        matches!(self, OwnedValue::Int(_))
    }

    #[inline]
    pub fn as_long(&self) -> Option<i64> {
        match self {
            OwnedValue::Long(value) => Some(value.get()),
            _ => None,
        }
    }

    #[inline]
    pub fn is_long(&self) -> bool {
        matches!(self, OwnedValue::Long(_))
    }

    #[inline]
    pub fn as_float(&self) -> Option<f32> {
        match self {
            OwnedValue::Float(value) => Some(value.get()),
            _ => None,
        }
    }

    #[inline]
    pub fn is_float(&self) -> bool {
        matches!(self, OwnedValue::Float(_))
    }

    #[inline]
    pub fn as_double(&self) -> Option<f64> {
        match self {
            OwnedValue::Double(value) => Some(value.get()),
            _ => None,
        }
    }

    #[inline]
    pub fn is_double(&self) -> bool {
        matches!(self, OwnedValue::Double(_))
    }

    #[inline]
    pub fn as_byte_array(&self) -> Option<&[i8]> {
        match self {
            OwnedValue::ByteArray(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_byte_array(&self) -> bool {
        matches!(self, OwnedValue::ByteArray(_))
    }

    #[inline]
    pub fn as_string<'a>(&'a self) -> Option<ImmutableString<'a>> {
        match self {
            OwnedValue::String(value) => Some(ImmutableString {
                data: value.as_mutf8_bytes(),
            }),
            _ => None,
        }
    }

    #[inline]
    pub fn is_string(&self) -> bool {
        matches!(self, OwnedValue::String(_))
    }

    #[inline]
    pub fn as_list<'a>(&'a self) -> Option<ImmutableList<'a, O>> {
        match self {
            OwnedValue::List(value) => Some(ImmutableList {
                data: value.data.as_ptr(),
                _marker: PhantomData,
            }),
            _ => None,
        }
    }

    #[inline]
    pub fn is_list(&self) -> bool {
        matches!(self, OwnedValue::List(_))
    }

    #[inline]
    pub fn as_compound<'a>(&'a self) -> Option<ImmutableCompound<'a, O>> {
        match self {
            OwnedValue::Compound(value) => Some(ImmutableCompound {
                data: value.data.as_ptr(),
                _marker: PhantomData,
            }),
            _ => None,
        }
    }

    #[inline]
    pub fn is_compound(&self) -> bool {
        matches!(self, OwnedValue::Compound(_))
    }

    #[inline]
    pub fn as_int_array(&self) -> Option<&[byteorder::I32<O>]> {
        match self {
            OwnedValue::IntArray(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_int_array(&self) -> bool {
        matches!(self, OwnedValue::IntArray(_))
    }

    #[inline]
    pub fn as_long_array(&self) -> Option<&[byteorder::I64<O>]> {
        match self {
            OwnedValue::LongArray(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_long_array(&self) -> bool {
        matches!(self, OwnedValue::LongArray(_))
    }

    #[inline]
    pub fn get<'a, I: Index>(&'a self, index: I) -> Option<ImmutableValue<'a, O>> {
        index.index_dispatch(
            self,
            |value, index| match value {
                OwnedValue::List(value) => value.get(index),
                _ => None,
            },
            |value, key| match value {
                OwnedValue::Compound(value) => value.get(key),
                _ => None,
            },
        )
    }

    #[inline]
    pub fn write_to_vec<TARGET: ByteOrder>(&self) -> Result<Vec<u8>> {
        self.visit_scoped(|value| write_owned_to_vec::<O, TARGET>(value))
    }

    #[inline]
    pub fn write_to_writer<TARGET: ByteOrder>(&self, writer: impl Write) -> Result<()> {
        self.visit_scoped(|value| write_owned_to_writer::<O, TARGET>(value, writer))
    }
}

impl<O: ByteOrder> OwnedValue<O> {
    #[inline]
    pub fn as_byte_mut(&mut self) -> Option<&mut i8> {
        match self {
            OwnedValue::Byte(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn set_byte(&mut self, data: i8) -> bool {
        match self {
            OwnedValue::Byte(value) => {
                *value = data;
                true
            }
            _ => false,
        }
    }

    #[inline]
    pub fn update_byte(&mut self, f: impl FnOnce(i8) -> i8) -> bool {
        match self {
            OwnedValue::Byte(value) => {
                *value = f(*value);
                true
            }
            _ => false,
        }
    }

    #[inline]
    pub fn as_short_mut(&mut self) -> Option<&mut byteorder::I16<O>> {
        match self {
            OwnedValue::Short(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn set_short(&mut self, data: i16) -> bool {
        match self {
            OwnedValue::Short(value) => {
                value.set(data);
                true
            }
            _ => false,
        }
    }

    #[inline]
    pub fn update_short(&mut self, f: impl FnOnce(i16) -> i16) -> bool {
        match self {
            OwnedValue::Short(value) => {
                value.set(f(value.get()));
                true
            }
            _ => false,
        }
    }

    #[inline]
    pub fn as_int_mut(&mut self) -> Option<&mut byteorder::I32<O>> {
        match self {
            OwnedValue::Int(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn set_int(&mut self, data: i32) -> bool {
        match self {
            OwnedValue::Int(value) => {
                value.set(data);
                true
            }
            _ => false,
        }
    }

    #[inline]
    pub fn update_int(&mut self, f: impl FnOnce(i32) -> i32) -> bool {
        match self {
            OwnedValue::Int(value) => {
                value.set(f(value.get()));
                true
            }
            _ => false,
        }
    }

    #[inline]
    pub fn as_long_mut(&mut self) -> Option<&mut byteorder::I64<O>> {
        match self {
            OwnedValue::Long(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn set_long(&mut self, data: i64) -> bool {
        match self {
            OwnedValue::Long(value) => {
                value.set(data);
                true
            }
            _ => false,
        }
    }

    #[inline]
    pub fn update_long(&mut self, f: impl FnOnce(i64) -> i64) -> bool {
        match self {
            OwnedValue::Long(value) => {
                value.set(f(value.get()));
                true
            }
            _ => false,
        }
    }

    #[inline]
    pub fn as_float_mut(&mut self) -> Option<&mut byteorder::F32<O>> {
        match self {
            OwnedValue::Float(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn set_float(&mut self, data: f32) -> bool {
        match self {
            OwnedValue::Float(value) => {
                value.set(data);
                true
            }
            _ => false,
        }
    }

    #[inline]
    pub fn update_float(&mut self, f: impl FnOnce(f32) -> f32) -> bool {
        match self {
            OwnedValue::Float(value) => {
                value.set(f(value.get()));
                true
            }
            _ => false,
        }
    }

    #[inline]
    pub fn as_double_mut(&mut self) -> Option<&mut byteorder::F64<O>> {
        match self {
            OwnedValue::Double(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn set_double(&mut self, data: f64) -> bool {
        match self {
            OwnedValue::Double(value) => {
                value.set(data);
                true
            }
            _ => false,
        }
    }

    #[inline]
    pub fn update_double(&mut self, f: impl FnOnce(f64) -> f64) -> bool {
        match self {
            OwnedValue::Double(value) => {
                value.set(f(value.get()));
                true
            }
            _ => false,
        }
    }

    #[inline]
    pub fn as_byte_array_mut<'a>(&'a mut self) -> Option<VecViewMut<'a, i8>> {
        match self {
            OwnedValue::ByteArray(value) => {
                Some(unsafe { VecViewMut::new(&mut value.ptr, &mut value.len, &mut value.cap) })
            }
            _ => None,
        }
    }

    #[inline]
    pub fn as_string_mut<'a>(&'a mut self) -> Option<StringViewMut<'a>> {
        match self {
            OwnedValue::String(value) => {
                Some(unsafe { StringViewMut::new(&mut value.ptr, &mut value.len, &mut value.cap) })
            }
            _ => None,
        }
    }

    #[inline]
    pub fn as_list_mut<'a>(&'a mut self) -> Option<MutableList<'a, O>> {
        match self {
            OwnedValue::List(value) => Some(MutableList {
                data: unsafe {
                    VecViewMut::new(
                        &mut value.data.ptr,
                        &mut value.data.len,
                        &mut value.data.cap,
                    )
                },
                _marker: PhantomData,
            }),
            _ => None,
        }
    }

    #[inline]
    pub fn as_compound_mut<'a>(&'a mut self) -> Option<MutableCompound<'a, O>> {
        match self {
            OwnedValue::Compound(value) => Some(MutableCompound {
                data: unsafe {
                    VecViewMut::new(
                        &mut value.data.ptr,
                        &mut value.data.len,
                        &mut value.data.cap,
                    )
                },
                _marker: PhantomData,
            }),
            _ => None,
        }
    }

    #[inline]
    pub fn as_int_array_mut<'a>(&'a mut self) -> Option<VecViewMut<'a, byteorder::I32<O>>> {
        match self {
            OwnedValue::IntArray(value) => {
                Some(unsafe { VecViewMut::new(&mut value.ptr, &mut value.len, &mut value.cap) })
            }
            _ => None,
        }
    }

    #[inline]
    pub fn as_long_array_mut<'a>(&'a mut self) -> Option<VecViewMut<'a, byteorder::I64<O>>> {
        match self {
            OwnedValue::LongArray(value) => {
                Some(unsafe { VecViewMut::new(&mut value.ptr, &mut value.len, &mut value.cap) })
            }
            _ => None,
        }
    }

    #[inline]
    pub fn get_mut<'a, I: Index>(&'a mut self, index: I) -> Option<MutableValue<'a, O>> {
        index.index_dispatch_mut(
            self,
            |value, index| match value {
                OwnedValue::List(value) => value.get_mut(index),
                _ => None,
            },
            |value, key| match value {
                OwnedValue::Compound(value) => value.get_mut(key),
                _ => None,
            },
        )
    }
}

/// An owned NBT list.
///
/// This type represents a mutable NBT list that owns its data. All elements
/// in an NBT list must have the same tag type.
///
/// # Creating a List
///
/// The easiest way to create a list is using `From` conversions:
///
/// ```
/// use na_nbt::{OwnedValue, OwnedList};
/// use zerocopy::byteorder::BigEndian;
///
/// // Create from a Vec of bytes
/// let byte_list: OwnedValue<BigEndian> = vec![1i8, 2, 3].into();
///
/// // Create an empty list
/// let empty_list: OwnedList<BigEndian> = OwnedList::default();
/// ```
///
/// # Accessing Elements
///
/// ```
/// use na_nbt::{OwnedValue, OwnedList};
/// use zerocopy::byteorder::BigEndian;
///
/// let value: OwnedValue<BigEndian> = vec![10i8, 20, 30].into();
/// if let OwnedValue::List(list) = &value {
///     if let Some(elem) = list.get(1) {
///         assert_eq!(elem.as_byte(), Some(20));
///     }
/// }
/// ```
///
/// # Modifying Elements
///
/// ```
/// use na_nbt::{OwnedValue, OwnedList};
/// use zerocopy::byteorder::BigEndian;
///
/// let mut value: OwnedValue<BigEndian> = vec![1i8, 2, 3].into();
/// if let OwnedValue::List(ref mut list) = value {
///     list.push(4i8);
///     // list.remove(0);
/// }
/// ```
#[repr(transparent)]
pub struct OwnedList<O: ByteOrder> {
    pub(crate) data: VecViewOwn<u8>,
    pub(crate) _marker: PhantomData<O>,
}

impl<O: ByteOrder> Default for OwnedList<O> {
    fn default() -> Self {
        Self {
            data: vec![0, 0, 0, 0, 0].into(),
            _marker: PhantomData,
        }
    }
}

impl<O: ByteOrder> OwnedList<O> {
    pub(crate) unsafe fn write(self, dst: *mut u8) {
        unsafe {
            ptr::write(dst.cast(), self);
        }
    }

    pub(crate) unsafe fn read(src: *mut u8) -> Self {
        unsafe { ptr::read(src.cast()) }
    }
}

impl<O: ByteOrder> Drop for OwnedList<O> {
    fn drop(&mut self) {
        unsafe {
            let mut ptr = self.data.as_mut_ptr();

            let tag_id = *ptr.cast::<TagID>();
            ptr = ptr.add(1);

            if tag_id.is_primitive() {
                return;
            }

            let len = byteorder::U32::<O>::from_bytes(*ptr.cast()).get();
            ptr = ptr.add(4);

            match tag_id {
                TagID::ByteArray => {
                    for _ in 0..len {
                        VecViewOwn::<i8>::read(ptr);
                        ptr = ptr.add(tag_size(tag_id));
                    }
                }
                TagID::String => {
                    for _ in 0..len {
                        StringViewOwn::read(ptr);
                        ptr = ptr.add(tag_size(tag_id));
                    }
                }
                TagID::List => {
                    for _ in 0..len {
                        OwnedList::<O>::read(ptr);
                        ptr = ptr.add(tag_size(tag_id));
                    }
                }
                TagID::Compound => {
                    for _ in 0..len {
                        OwnedCompound::<O>::read(ptr);
                        ptr = ptr.add(tag_size(tag_id));
                    }
                }
                TagID::IntArray => {
                    for _ in 0..len {
                        VecViewOwn::<byteorder::I32<O>>::read(ptr);
                        ptr = ptr.add(tag_size(tag_id));
                    }
                }
                TagID::LongArray => {
                    for _ in 0..len {
                        VecViewOwn::<byteorder::I64<O>>::read(ptr);
                        ptr = ptr.add(tag_size(tag_id));
                    }
                }
                _ => unreachable_unchecked(),
            }
            debug_assert!(ptr.byte_offset_from_unsigned(self.data.as_mut_ptr()) == self.data.len());
        }
    }
}

impl<O: ByteOrder> IntoIterator for OwnedList<O> {
    type Item = OwnedValue<O>;
    type IntoIter = OwnedListIter<O>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let tag_id = self.tag_id();
        let remaining = self.len() as u32;
        let me = ManuallyDrop::new(self);
        let mut data = unsafe { ptr::read(&me.data) };
        let ptr = unsafe { data.as_mut_ptr().add(1 + 4) };
        OwnedListIter {
            tag_id,
            remaining,
            ptr,
            _data: data,
            _marker: PhantomData,
        }
    }
}

impl<O: ByteOrder> OwnedList<O> {
    #[inline]
    pub fn tag_id(&self) -> TagID {
        list_tag_id(self.data.as_ptr())
    }

    #[inline]
    pub fn len(&self) -> usize {
        list_len::<O>(self.data.as_ptr())
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        list_is_empty::<O>(self.data.as_ptr())
    }

    pub fn get<'a>(&'a self, index: usize) -> Option<ImmutableValue<'a, O>> {
        list_get(self.data.as_ptr(), index)
    }

    #[inline]
    pub fn iter<'a>(&'a self) -> ImmutableListIter<'a, O> {
        list_iter(self.data.as_ptr())
    }
}

impl<O: ByteOrder> OwnedList<O> {
    pub fn get_mut<'a>(&'a mut self, index: usize) -> Option<MutableValue<'a, O>> {
        list_get_mut(self.data.as_mut_ptr(), index)
    }

    #[inline]
    pub fn iter_mut<'a>(&'a mut self) -> MutableListIter<'a, O> {
        list_iter_mut(self.data.as_mut_ptr())
    }
}

impl<O: ByteOrder> OwnedList<O> {
    pub fn push<V: IntoOwnedValue<O>>(&mut self, value: V) {
        let mut data =
            unsafe { VecViewMut::new(&mut self.data.ptr, &mut self.data.len, &mut self.data.cap) };
        value.list_push(&mut data);
    }

    /// .
    ///
    /// # Safety
    ///
    /// .
    pub unsafe fn push_unchecked<V: IntoOwnedValue<O>>(&mut self, value: V) {
        let mut data =
            unsafe { VecViewMut::new(&mut self.data.ptr, &mut self.data.len, &mut self.data.cap) };

        unsafe { value.list_push_unchecked(&mut data) };
    }

    pub fn insert<V: IntoOwnedValue<O>>(&mut self, index: usize, value: V) {
        let mut data =
            unsafe { VecViewMut::new(&mut self.data.ptr, &mut self.data.len, &mut self.data.cap) };
        value.list_insert(&mut data, index);
    }

    /// .
    ///
    /// # Safety
    ///
    /// .
    pub unsafe fn insert_unchecked<V: IntoOwnedValue<O>>(&mut self, index: usize, value: V) {
        let mut data =
            unsafe { VecViewMut::new(&mut self.data.ptr, &mut self.data.len, &mut self.data.cap) };
        unsafe { value.list_insert_unchecked(&mut data, index) };
    }

    pub fn pop(&mut self) -> Option<OwnedValue<O>> {
        let mut data =
            unsafe { VecViewMut::new(&mut self.data.ptr, &mut self.data.len, &mut self.data.cap) };
        list_pop(&mut data)
    }

    /// Removes and returns the element at the given index.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    pub fn remove(&mut self, index: usize) -> OwnedValue<O> {
        let mut data =
            unsafe { VecViewMut::new(&mut self.data.ptr, &mut self.data.len, &mut self.data.cap) };
        list_remove(&mut data, index)
    }
}

/// An owned NBT compound (key-value map).
///
/// This type represents a mutable NBT compound that owns its data. Use it to
/// build compound structures or modify parsed compounds.
///
/// # Creating a Compound
///
/// ```
/// use na_nbt::OwnedCompound;
/// use zerocopy::byteorder::BigEndian;
///
/// let mut compound: OwnedCompound<BigEndian> = OwnedCompound::default();
/// compound.insert("name", "Steve");
/// compound.insert("health", 20i32);
/// compound.insert("score", 1000i64);
/// ```
///
/// # Accessing Values
///
/// ```
/// use na_nbt::{OwnedCompound, OwnedValue};
/// use zerocopy::byteorder::BigEndian;
///
/// let mut compound: OwnedCompound<BigEndian> = OwnedCompound::default();
/// compound.insert("level", 42i32);
///
/// if let Some(value) = compound.get("level") {
///     println!("Level: {:?}", value.as_int());
/// }
/// ```
///
/// # Iterating
///
/// ```
/// use na_nbt::{OwnedCompound, ReadableString, ScopedReadableValue};
/// use zerocopy::byteorder::BigEndian;
///
/// let mut compound: OwnedCompound<BigEndian> = OwnedCompound::default();
/// compound.insert("a", 1i32);
/// compound.insert("b", 2i32);
///
/// for (key, value) in compound.iter() {
///     println!("{}: {:?}", key.decode(), value.as_int());
/// }
/// ```
#[repr(transparent)]
pub struct OwnedCompound<O: ByteOrder> {
    pub(crate) data: VecViewOwn<u8>,
    pub(crate) _marker: PhantomData<O>,
}

impl<O: ByteOrder> Default for OwnedCompound<O> {
    fn default() -> Self {
        Self {
            data: vec![0].into(),
            _marker: PhantomData,
        }
    }
}

impl<O: ByteOrder> OwnedCompound<O> {
    pub(crate) unsafe fn write(self, dst: *mut u8) {
        unsafe {
            ptr::write(dst.cast(), self);
        }
    }

    pub(crate) unsafe fn read(src: *mut u8) -> Self {
        unsafe { ptr::read(src.cast()) }
    }
}

impl<O: ByteOrder> Drop for OwnedCompound<O> {
    fn drop(&mut self) {
        unsafe {
            let mut ptr = self.data.as_mut_ptr();

            loop {
                let tag_id = *ptr.cast();
                ptr = ptr.add(1);

                if tag_id == TagID::End {
                    cold_path();
                    debug_assert!(
                        ptr.byte_offset_from_unsigned(self.data.as_mut_ptr()) == self.data.len()
                    );
                    return;
                }

                let name_len = byteorder::U16::<O>::from_bytes(*ptr.cast()).get();
                ptr = ptr.add(2);

                ptr = ptr.add(name_len as usize);

                match tag_id {
                    TagID::ByteArray => {
                        VecViewOwn::<i8>::read(ptr);
                    }
                    TagID::String => {
                        StringViewOwn::read(ptr);
                    }
                    TagID::List => {
                        OwnedList::<O>::read(ptr);
                    }
                    TagID::Compound => {
                        OwnedCompound::<O>::read(ptr);
                    }
                    TagID::IntArray => {
                        VecViewOwn::<byteorder::I32<O>>::read(ptr);
                    }
                    TagID::LongArray => {
                        VecViewOwn::<byteorder::I64<O>>::read(ptr);
                    }
                    _ => (),
                }

                ptr = ptr.add(tag_size(tag_id));
            }
        }
    }
}

impl<O: ByteOrder> IntoIterator for OwnedCompound<O> {
    type Item = (String, OwnedValue<O>);
    type IntoIter = OwnedCompoundIter<O>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let me = ManuallyDrop::new(self);
        let mut data = unsafe { ptr::read(&me.data) };
        let ptr = data.as_mut_ptr();
        OwnedCompoundIter {
            ptr,
            _data: data,
            _marker: PhantomData,
        }
    }
}

impl<O: ByteOrder> OwnedCompound<O> {
    pub fn get<'a>(&'a self, key: &str) -> Option<ImmutableValue<'a, O>> {
        compound_get(self.data.as_ptr(), key)
    }

    #[inline]
    pub fn iter<'a>(&'a self) -> ImmutableCompoundIter<'a, O> {
        compound_iter(self.data.as_ptr())
    }
}

impl<O: ByteOrder> OwnedCompound<O> {
    pub fn get_mut<'a>(&'a mut self, key: &str) -> Option<MutableValue<'a, O>> {
        compound_get_mut(self.data.as_mut_ptr(), key)
    }

    #[inline]
    pub fn iter_mut<'a>(&'a mut self) -> MutableCompoundIter<'a, O> {
        compound_iter_mut(self.data.as_mut_ptr())
    }
}

impl<O: ByteOrder> OwnedCompound<O> {
    pub fn insert<V: IntoOwnedValue<O>>(&mut self, key: &str, value: V) -> Option<OwnedValue<O>> {
        let mut data =
            unsafe { VecViewMut::new(&mut self.data.ptr, &mut self.data.len, &mut self.data.cap) };
        value.compound_insert(&mut data, key)
    }

    pub fn remove(&mut self, key: &str) -> Option<OwnedValue<O>> {
        let mut data =
            unsafe { VecViewMut::new(&mut self.data.ptr, &mut self.data.len, &mut self.data.cap) };
        compound_remove(&mut data, key)
    }
}
