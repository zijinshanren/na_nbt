use std::{hint::unreachable_unchecked, marker::PhantomData, mem::ManuallyDrop, ptr};

use zerocopy::byteorder;

use crate::{
    ByteOrder, ImmutableCompound, ImmutableList, ImmutableString, ImmutableValue, IntoOwnedValue,
    MutableCompound, MutableList, MutableValue, Tag, cold_path,
    implementation::mutable::{
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
    index::Index,
    view::{StringViewMut, StringViewOwn, VecViewMut, VecViewOwn},
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

#[repr(u8)]
pub enum OwnedValue<O: ByteOrder> {
    End = 0,
    Byte(i8) = 1,
    Short(byteorder::I16<O>) = 2,
    Int(byteorder::I32<O>) = 3,
    Long(byteorder::I64<O>) = 4,
    Float(byteorder::F32<O>) = 5,
    Double(byteorder::F64<O>) = 6,
    ByteArray(VecViewOwn<i8>) = 7,
    String(StringViewOwn) = 8,
    List(OwnedList<O>) = 9,
    Compound(OwnedCompound<O>) = 10,
    IntArray(VecViewOwn<byteorder::I32<O>>) = 11,
    LongArray(VecViewOwn<byteorder::I64<O>>) = 12,
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

    pub(crate) unsafe fn read(tag_id: Tag, src: *mut u8) -> Self {
        unsafe {
            match tag_id {
                Tag::End => OwnedValue::End,
                Tag::Byte => OwnedValue::Byte(ptr::read(src.cast())),
                Tag::Short => OwnedValue::Short(ptr::read(src.cast())),
                Tag::Int => OwnedValue::Int(ptr::read(src.cast())),
                Tag::Long => OwnedValue::Long(ptr::read(src.cast())),
                Tag::Float => OwnedValue::Float(ptr::read(src.cast())),
                Tag::Double => OwnedValue::Double(ptr::read(src.cast())),
                Tag::ByteArray => OwnedValue::ByteArray(VecViewOwn::read(src.cast())),
                Tag::String => OwnedValue::String(StringViewOwn::read(src.cast())),
                Tag::List => OwnedValue::List(OwnedList::read(src.cast())),
                Tag::Compound => OwnedValue::Compound(OwnedCompound::read(src.cast())),
                Tag::IntArray => OwnedValue::IntArray(VecViewOwn::read(src.cast())),
                Tag::LongArray => OwnedValue::LongArray(VecViewOwn::read(src.cast())),
            }
        }
    }
}

impl<O: ByteOrder> OwnedValue<O> {
    #[inline]
    pub fn tag_id(&self) -> Tag {
        unsafe { *(self as *const Self as *const Tag) }
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

            let tag_id = *ptr.cast::<Tag>();
            ptr = ptr.add(1);

            if tag_id.is_primitive() {
                return;
            }

            let len = byteorder::U32::<O>::from_bytes(*ptr.cast()).get();
            ptr = ptr.add(4);

            match tag_id {
                Tag::ByteArray => {
                    for _ in 0..len {
                        VecViewOwn::<i8>::read(ptr);
                        ptr = ptr.add(tag_size(tag_id));
                    }
                }
                Tag::String => {
                    for _ in 0..len {
                        StringViewOwn::read(ptr);
                        ptr = ptr.add(tag_size(tag_id));
                    }
                }
                Tag::List => {
                    for _ in 0..len {
                        OwnedList::<O>::read(ptr);
                        ptr = ptr.add(tag_size(tag_id));
                    }
                }
                Tag::Compound => {
                    for _ in 0..len {
                        OwnedCompound::<O>::read(ptr);
                        ptr = ptr.add(tag_size(tag_id));
                    }
                }
                Tag::IntArray => {
                    for _ in 0..len {
                        VecViewOwn::<byteorder::I32<O>>::read(ptr);
                        ptr = ptr.add(tag_size(tag_id));
                    }
                }
                Tag::LongArray => {
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
    pub fn tag_id(&self) -> Tag {
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

    pub fn remove(&mut self, index: usize) -> OwnedValue<O> {
        let mut data =
            unsafe { VecViewMut::new(&mut self.data.ptr, &mut self.data.len, &mut self.data.cap) };
        list_remove(&mut data, index)
    }
}

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

                if tag_id == Tag::End {
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
                    Tag::ByteArray => {
                        VecViewOwn::<i8>::read(ptr);
                    }
                    Tag::String => {
                        StringViewOwn::read(ptr);
                    }
                    Tag::List => {
                        OwnedList::<O>::read(ptr);
                    }
                    Tag::Compound => {
                        OwnedCompound::<O>::read(ptr);
                    }
                    Tag::IntArray => {
                        VecViewOwn::<byteorder::I32<O>>::read(ptr);
                    }
                    Tag::LongArray => {
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
