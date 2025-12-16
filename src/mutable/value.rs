use std::{borrow::Cow, io::Write, marker::PhantomData, ptr, slice};

use zerocopy::byteorder;

use crate::{
    ByteOrder, Result, ScopedReadableValue as _, Tag,
    index::Index,
    mutable::{
        iter::{ImmutableCompoundIter, ImmutableListIter},
        util::{
            SIZE_USIZE, compound_get, compound_iter, list_get, list_is_empty, list_iter, list_len,
            list_tag_id,
        },
    },
    write_owned_to_vec, write_owned_to_writer,
};

/// A zero-copy, immutable NBT value (Pointer-based).
///
/// This type provides an immutable view into NBT data using direct pointers and slices.
///
/// This type is typically used when you need a lightweight, immutable view of data that might
/// be part of a mutable structure or when the mark-based overhead is not desired.
#[derive(Clone)]
pub enum ImmutableValue<'s, O: ByteOrder> {
    End,
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(&'s [i8]),
    String(ImmutableString<'s>),
    List(ImmutableList<'s, O>),
    Compound(ImmutableCompound<'s, O>),
    IntArray(&'s [byteorder::I32<O>]),
    LongArray(&'s [byteorder::I64<O>]),
}

impl<'s, O: ByteOrder> ImmutableValue<'s, O> {
    /// .
    ///
    /// # Safety
    ///
    /// .
    pub unsafe fn read(tag_id: Tag, data: *const u8) -> Self {
        unsafe {
            macro_rules! get {
                ($t:tt) => {{
                    let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                    let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
                    ImmutableValue::$t(slice::from_raw_parts(ptr, len))
                }};
                ($t:tt, $l:tt) => {{
                    let ptr = ptr::with_exposed_provenance(usize::from_ne_bytes(*data.cast()));
                    ImmutableValue::$t($l {
                        data: ptr,
                        _marker: PhantomData,
                    })
                }};
            }

            match tag_id {
                Tag::End => ImmutableValue::End,
                Tag::Byte => ImmutableValue::Byte(*data.cast()),
                Tag::Short => {
                    ImmutableValue::Short(byteorder::I16::<O>::from_bytes(*data.cast()).get())
                }
                Tag::Int => {
                    ImmutableValue::Int(byteorder::I32::<O>::from_bytes(*data.cast()).get())
                }
                Tag::Long => {
                    ImmutableValue::Long(byteorder::I64::<O>::from_bytes(*data.cast()).get())
                }
                Tag::Float => {
                    ImmutableValue::Float(byteorder::F32::<O>::from_bytes(*data.cast()).get())
                }
                Tag::Double => {
                    ImmutableValue::Double(byteorder::F64::<O>::from_bytes(*data.cast()).get())
                }
                Tag::ByteArray => get!(ByteArray),
                Tag::String => {
                    let addr = usize::from_ne_bytes(*data.cast());
                    let ptr = ptr::with_exposed_provenance::<u8>(addr);
                    let len = usize::from_ne_bytes(*data.add(SIZE_USIZE).cast());
                    ImmutableValue::String(ImmutableString {
                        data: slice::from_raw_parts(ptr, len),
                    })
                }
                Tag::List => get!(List, ImmutableList),
                Tag::Compound => get!(Compound, ImmutableCompound),
                Tag::IntArray => get!(IntArray),
                Tag::LongArray => get!(LongArray),
            }
        }
    }
}

impl<'s, O: ByteOrder> ImmutableValue<'s, O> {
    #[inline]
    pub fn tag_id(&self) -> Tag {
        match self {
            ImmutableValue::End => Tag::End,
            ImmutableValue::Byte(_) => Tag::Byte,
            ImmutableValue::Short(_) => Tag::Short,
            ImmutableValue::Int(_) => Tag::Int,
            ImmutableValue::Long(_) => Tag::Long,
            ImmutableValue::Float(_) => Tag::Float,
            ImmutableValue::Double(_) => Tag::Double,
            ImmutableValue::ByteArray(_) => Tag::ByteArray,
            ImmutableValue::String(_) => Tag::String,
            ImmutableValue::List(_) => Tag::List,
            ImmutableValue::Compound(_) => Tag::Compound,
            ImmutableValue::IntArray(_) => Tag::IntArray,
            ImmutableValue::LongArray(_) => Tag::LongArray,
        }
    }

    #[inline]
    pub fn as_end(&self) -> Option<()> {
        match self {
            ImmutableValue::End => Some(()),
            _ => None,
        }
    }

    #[inline]
    pub fn is_end(&self) -> bool {
        matches!(self, ImmutableValue::End)
    }

    #[inline]
    pub fn as_byte(&self) -> Option<i8> {
        match self {
            ImmutableValue::Byte(value) => Some(*value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_byte(&self) -> bool {
        matches!(self, ImmutableValue::Byte(_))
    }

    #[inline]
    pub fn as_short(&self) -> Option<i16> {
        match self {
            ImmutableValue::Short(value) => Some(*value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_short(&self) -> bool {
        matches!(self, ImmutableValue::Short(_))
    }

    #[inline]
    pub fn as_int(&self) -> Option<i32> {
        match self {
            ImmutableValue::Int(value) => Some(*value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_int(&self) -> bool {
        matches!(self, ImmutableValue::Int(_))
    }

    #[inline]
    pub fn as_long(&self) -> Option<i64> {
        match self {
            ImmutableValue::Long(value) => Some(*value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_long(&self) -> bool {
        matches!(self, ImmutableValue::Long(_))
    }

    #[inline]
    pub fn as_float(&self) -> Option<f32> {
        match self {
            ImmutableValue::Float(value) => Some(*value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_float(&self) -> bool {
        matches!(self, ImmutableValue::Float(_))
    }

    #[inline]
    pub fn as_double(&self) -> Option<f64> {
        match self {
            ImmutableValue::Double(value) => Some(*value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_double(&self) -> bool {
        matches!(self, ImmutableValue::Double(_))
    }

    #[inline]
    pub fn as_byte_array<'a>(&'a self) -> Option<&'a &'s [i8]>
    where
        's: 'a,
    {
        match self {
            ImmutableValue::ByteArray(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_byte_array(&self) -> bool {
        matches!(self, ImmutableValue::ByteArray(_))
    }

    #[inline]
    pub fn as_string<'a>(&'a self) -> Option<&'a ImmutableString<'s>>
    where
        's: 'a,
    {
        match self {
            ImmutableValue::String(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_string(&self) -> bool {
        matches!(self, ImmutableValue::String(_))
    }

    #[inline]
    pub fn as_list<'a>(&'a self) -> Option<&'a ImmutableList<'s, O>>
    where
        's: 'a,
    {
        match self {
            ImmutableValue::List(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_list(&self) -> bool {
        matches!(self, ImmutableValue::List(_))
    }

    #[inline]
    pub fn as_compound<'a>(&'a self) -> Option<&'a ImmutableCompound<'s, O>>
    where
        's: 'a,
    {
        match self {
            ImmutableValue::Compound(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_compound(&self) -> bool {
        matches!(self, ImmutableValue::Compound(_))
    }

    #[inline]
    pub fn as_int_array<'a>(&'a self) -> Option<&'a &'s [byteorder::I32<O>]>
    where
        's: 'a,
    {
        match self {
            ImmutableValue::IntArray(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_int_array(&self) -> bool {
        matches!(self, ImmutableValue::IntArray(_))
    }

    #[inline]
    pub fn as_long_array<'a>(&'a self) -> Option<&'a &'s [byteorder::I64<O>]>
    where
        's: 'a,
    {
        match self {
            ImmutableValue::LongArray(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_long_array(&self) -> bool {
        matches!(self, ImmutableValue::LongArray(_))
    }

    #[inline]
    pub fn get<I: Index>(&self, index: I) -> Option<ImmutableValue<'s, O>> {
        index.index_dispatch(
            self,
            |value, index| match value {
                ImmutableValue::List(value) => value.get(index),
                _ => None,
            },
            |value, key| match value {
                ImmutableValue::Compound(value) => value.get(key),
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

#[derive(Clone)]
pub struct ImmutableString<'s> {
    pub(crate) data: &'s [u8],
}

impl<'s> ImmutableString<'s> {
    #[inline]
    pub fn raw_bytes(&self) -> &[u8] {
        self.data
    }

    #[inline]
    pub fn decode<'a>(&'a self) -> Cow<'a, str> {
        simd_cesu8::mutf8::decode_lossy(self.data)
    }
}

#[derive(Clone)]
pub struct ImmutableList<'s, O: ByteOrder> {
    pub(crate) data: *const u8,
    pub(crate) _marker: PhantomData<(&'s (), O)>,
}

unsafe impl<'s, O: ByteOrder> Send for ImmutableList<'s, O> {}
unsafe impl<'s, O: ByteOrder> Sync for ImmutableList<'s, O> {}

impl<'s, O: ByteOrder> IntoIterator for ImmutableList<'s, O> {
    type Item = ImmutableValue<'s, O>;
    type IntoIter = ImmutableListIter<'s, O>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        ImmutableListIter {
            tag_id: self.tag_id(),
            remaining: self.len() as u32,
            data: unsafe { self.data.add(1 + 4) },
            _marker: PhantomData,
        }
    }
}

impl<'s, O: ByteOrder> ImmutableList<'s, O> {
    #[inline]
    pub fn tag_id(&self) -> Tag {
        list_tag_id(self.data)
    }

    #[inline]
    pub fn len(&self) -> usize {
        list_len::<O>(self.data)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        list_is_empty::<O>(self.data)
    }

    pub fn get(&self, index: usize) -> Option<ImmutableValue<'s, O>> {
        list_get(self.data, index)
    }

    #[inline]
    pub fn iter(&self) -> ImmutableListIter<'s, O> {
        list_iter(self.data)
    }
}

#[derive(Clone)]
pub struct ImmutableCompound<'s, O: ByteOrder> {
    pub(crate) data: *const u8,
    pub(crate) _marker: PhantomData<(&'s (), O)>,
}

unsafe impl<'s, O: ByteOrder> Send for ImmutableCompound<'s, O> {}
unsafe impl<'s, O: ByteOrder> Sync for ImmutableCompound<'s, O> {}

impl<'s, O: ByteOrder> IntoIterator for ImmutableCompound<'s, O> {
    type Item = (ImmutableString<'s>, ImmutableValue<'s, O>);
    type IntoIter = ImmutableCompoundIter<'s, O>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        ImmutableCompoundIter {
            data: self.data,
            _marker: PhantomData,
        }
    }
}

impl<'s, O: ByteOrder> ImmutableCompound<'s, O> {
    pub fn get(&self, key: &str) -> Option<ImmutableValue<'s, O>> {
        compound_get(self.data, key)
    }

    #[inline]
    pub fn iter(&self) -> ImmutableCompoundIter<'s, O> {
        compound_iter(self.data)
    }
}
