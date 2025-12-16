use std::{io::Write, marker::PhantomData};

use zerocopy::byteorder;

use crate::{
    ByteOrder, ImmutableCompound, ImmutableList, ImmutableString, ImmutableValue, IntoOwnedValue,
    OwnedValue, Result, ScopedReadableValue as _, Tag,
    index::Index,
    mutable::{
        iter::{ImmutableCompoundIter, ImmutableListIter, MutableCompoundIter, MutableListIter},
        util::{
            SIZE_USIZE, compound_get, compound_get_mut, compound_iter, compound_iter_mut,
            compound_remove, list_get, list_get_mut, list_is_empty, list_iter, list_iter_mut,
            list_len, list_pop, list_remove, list_tag_id,
        },
    },
    view::{StringViewMut, VecViewMut},
    write_owned_to_vec, write_owned_to_writer,
};

pub enum MutableValue<'s, O: ByteOrder> {
    End,
    Byte(&'s mut i8),
    Short(&'s mut byteorder::I16<O>),
    Int(&'s mut byteorder::I32<O>),
    Long(&'s mut byteorder::I64<O>),
    Float(&'s mut byteorder::F32<O>),
    Double(&'s mut byteorder::F64<O>),
    ByteArray(VecViewMut<'s, i8>),
    String(StringViewMut<'s>),
    List(MutableList<'s, O>),
    Compound(MutableCompound<'s, O>),
    IntArray(VecViewMut<'s, byteorder::I32<O>>),
    LongArray(VecViewMut<'s, byteorder::I64<O>>),
}

impl<'s, O: ByteOrder> MutableValue<'s, O> {
    /// .
    ///
    /// # Safety
    ///
    /// .
    pub unsafe fn read(tag_id: Tag, data: *mut u8) -> Self {
        unsafe {
            macro_rules! get {
                ($t:tt, $l:tt) => {{
                    let ptr_ref = &mut *(data.cast());
                    let len_ref = &mut *(data.add(SIZE_USIZE).cast());
                    let cap_ref = &mut *(data.add(SIZE_USIZE * 2).cast());
                    MutableValue::$t($l::new(ptr_ref, len_ref, cap_ref))
                }};
            }

            macro_rules! get_composite {
                ($t:tt, $l:tt) => {{
                    let ptr_ref = &mut *(data.cast());
                    let len_ref = &mut *(data.add(SIZE_USIZE).cast());
                    let cap_ref = &mut *(data.add(SIZE_USIZE * 2).cast());
                    MutableValue::$t($l {
                        data: VecViewMut::new(ptr_ref, len_ref, cap_ref),
                        _marker: PhantomData,
                    })
                }};
            }

            match tag_id {
                Tag::End => MutableValue::End,
                Tag::Byte => MutableValue::Byte(&mut *data.cast()),
                Tag::Short => MutableValue::Short(&mut *data.cast()),
                Tag::Int => MutableValue::Int(&mut *data.cast()),
                Tag::Long => MutableValue::Long(&mut *data.cast()),
                Tag::Float => MutableValue::Float(&mut *data.cast()),
                Tag::Double => MutableValue::Double(&mut *data.cast()),
                Tag::ByteArray => get!(ByteArray, VecViewMut),
                Tag::String => get!(String, StringViewMut),
                Tag::List => get_composite!(List, MutableList),
                Tag::Compound => get_composite!(Compound, MutableCompound),
                Tag::IntArray => get!(IntArray, VecViewMut),
                Tag::LongArray => get!(LongArray, VecViewMut),
            }
        }
    }
}

impl<'s, O: ByteOrder> MutableValue<'s, O> {
    #[inline]
    pub fn tag_id(&self) -> Tag {
        match self {
            MutableValue::End => Tag::End,
            MutableValue::Byte(_) => Tag::Byte,
            MutableValue::Short(_) => Tag::Short,
            MutableValue::Int(_) => Tag::Int,
            MutableValue::Long(_) => Tag::Long,
            MutableValue::Float(_) => Tag::Float,
            MutableValue::Double(_) => Tag::Double,
            MutableValue::ByteArray(_) => Tag::ByteArray,
            MutableValue::String(_) => Tag::String,
            MutableValue::List(_) => Tag::List,
            MutableValue::Compound(_) => Tag::Compound,
            MutableValue::IntArray(_) => Tag::IntArray,
            MutableValue::LongArray(_) => Tag::LongArray,
        }
    }

    #[inline]
    pub fn as_end(&self) -> Option<()> {
        match self {
            MutableValue::End => Some(()),
            _ => None,
        }
    }

    #[inline]
    pub fn is_end(&self) -> bool {
        matches!(self, MutableValue::End)
    }

    #[inline]
    pub fn as_byte(&self) -> Option<i8> {
        match self {
            MutableValue::Byte(value) => Some(**value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_byte(&self) -> bool {
        matches!(self, MutableValue::Byte(_))
    }

    #[inline]
    pub fn as_short(&self) -> Option<i16> {
        match self {
            MutableValue::Short(value) => Some(value.get()),
            _ => None,
        }
    }

    #[inline]
    pub fn is_short(&self) -> bool {
        matches!(self, MutableValue::Short(_))
    }

    #[inline]
    pub fn as_int(&self) -> Option<i32> {
        match self {
            MutableValue::Int(value) => Some(value.get()),
            _ => None,
        }
    }

    #[inline]
    pub fn is_int(&self) -> bool {
        matches!(self, MutableValue::Int(_))
    }

    #[inline]
    pub fn as_long(&self) -> Option<i64> {
        match self {
            MutableValue::Long(value) => Some(value.get()),
            _ => None,
        }
    }

    #[inline]
    pub fn is_long(&self) -> bool {
        matches!(self, MutableValue::Long(_))
    }

    #[inline]
    pub fn as_float(&self) -> Option<f32> {
        match self {
            MutableValue::Float(value) => Some(value.get()),
            _ => None,
        }
    }

    #[inline]
    pub fn is_float(&self) -> bool {
        matches!(self, MutableValue::Float(_))
    }

    #[inline]
    pub fn as_double(&self) -> Option<f64> {
        match self {
            MutableValue::Double(value) => Some(value.get()),
            _ => None,
        }
    }

    #[inline]
    pub fn is_double(&self) -> bool {
        matches!(self, MutableValue::Double(_))
    }

    #[inline]
    pub fn as_byte_array<'a>(&'a self) -> Option<&'a [i8]>
    where
        's: 'a,
    {
        match self {
            MutableValue::ByteArray(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_byte_array(&self) -> bool {
        matches!(self, MutableValue::ByteArray(_))
    }

    #[inline]
    pub fn as_string<'a>(&'a self) -> Option<ImmutableString<'a>>
    where
        's: 'a,
    {
        match self {
            MutableValue::String(value) => Some(ImmutableString {
                data: value.as_mutf8_bytes(),
            }),
            _ => None,
        }
    }

    #[inline]
    pub fn is_string(&self) -> bool {
        matches!(self, MutableValue::String(_))
    }

    #[inline]
    pub fn as_list<'a>(&'a self) -> Option<ImmutableList<'a, O>>
    where
        's: 'a,
    {
        match self {
            MutableValue::List(value) => Some(ImmutableList {
                data: value.data.as_ptr(),
                _marker: PhantomData,
            }),
            _ => None,
        }
    }

    #[inline]
    pub fn is_list(&self) -> bool {
        matches!(self, MutableValue::List(_))
    }

    #[inline]
    pub fn as_compound<'a>(&'a self) -> Option<ImmutableCompound<'a, O>>
    where
        's: 'a,
    {
        match self {
            MutableValue::Compound(value) => Some(ImmutableCompound {
                data: value.data.as_ptr(),
                _marker: PhantomData,
            }),
            _ => None,
        }
    }

    #[inline]
    pub fn is_compound(&self) -> bool {
        matches!(self, MutableValue::Compound(_))
    }

    #[inline]
    pub fn as_int_array<'a>(&'a self) -> Option<&'a [byteorder::I32<O>]>
    where
        's: 'a,
    {
        match self {
            MutableValue::IntArray(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_int_array(&self) -> bool {
        matches!(self, MutableValue::IntArray(_))
    }

    #[inline]
    pub fn as_long_array<'a>(&'a self) -> Option<&'a [byteorder::I64<O>]>
    where
        's: 'a,
    {
        match self {
            MutableValue::LongArray(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_long_array(&self) -> bool {
        matches!(self, MutableValue::LongArray(_))
    }

    #[inline]
    pub fn get<'a, I: Index>(&'a self, index: I) -> Option<ImmutableValue<'a, O>>
    where
        's: 'a,
    {
        index.index_dispatch(
            self,
            |value, index| match value {
                MutableValue::List(value) => value.get(index),
                _ => None,
            },
            |value, key| match value {
                MutableValue::Compound(value) => value.get(key),
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

impl<'s, O: ByteOrder> MutableValue<'s, O> {
    #[inline]
    pub fn as_byte_mut<'a>(&'a mut self) -> Option<&'a mut i8>
    where
        's: 'a,
    {
        match self {
            MutableValue::Byte(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn set_byte(&mut self, data: i8) -> bool {
        match self {
            MutableValue::Byte(value) => {
                **value = data;
                true
            }
            _ => false,
        }
    }

    #[inline]
    pub fn update_byte(&mut self, f: impl FnOnce(i8) -> i8) -> bool {
        match self {
            MutableValue::Byte(value) => {
                **value = f(**value);
                true
            }
            _ => false,
        }
    }

    #[inline]
    pub fn as_short_mut<'a>(&'a mut self) -> Option<&'a mut byteorder::I16<O>>
    where
        's: 'a,
    {
        match self {
            MutableValue::Short(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn set_short(&mut self, data: i16) -> bool {
        match self {
            MutableValue::Short(value) => {
                value.set(data);
                true
            }
            _ => false,
        }
    }

    #[inline]
    pub fn update_short(&mut self, f: impl FnOnce(i16) -> i16) -> bool {
        match self {
            MutableValue::Short(value) => {
                value.set(f(value.get()));
                true
            }
            _ => false,
        }
    }

    #[inline]
    pub fn as_int_mut<'a>(&'a mut self) -> Option<&'a mut byteorder::I32<O>>
    where
        's: 'a,
    {
        match self {
            MutableValue::Int(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn set_int(&mut self, data: i32) -> bool {
        match self {
            MutableValue::Int(value) => {
                value.set(data);
                true
            }
            _ => false,
        }
    }

    #[inline]
    pub fn update_int(&mut self, f: impl FnOnce(i32) -> i32) -> bool {
        match self {
            MutableValue::Int(value) => {
                value.set(f(value.get()));
                true
            }
            _ => false,
        }
    }

    #[inline]
    pub fn as_long_mut<'a>(&'a mut self) -> Option<&'a mut byteorder::I64<O>>
    where
        's: 'a,
    {
        match self {
            MutableValue::Long(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn set_long(&mut self, data: i64) -> bool {
        match self {
            MutableValue::Long(value) => {
                value.set(data);
                true
            }
            _ => false,
        }
    }

    #[inline]
    pub fn update_long(&mut self, f: impl FnOnce(i64) -> i64) -> bool {
        match self {
            MutableValue::Long(value) => {
                value.set(f(value.get()));
                true
            }
            _ => false,
        }
    }

    #[inline]
    pub fn as_float_mut<'a>(&'a mut self) -> Option<&'a mut byteorder::F32<O>>
    where
        's: 'a,
    {
        match self {
            MutableValue::Float(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn set_float(&mut self, data: f32) -> bool {
        match self {
            MutableValue::Float(value) => {
                value.set(data);
                true
            }
            _ => false,
        }
    }

    #[inline]
    pub fn update_float(&mut self, f: impl FnOnce(f32) -> f32) -> bool {
        match self {
            MutableValue::Float(value) => {
                value.set(f(value.get()));
                true
            }
            _ => false,
        }
    }

    #[inline]
    pub fn as_double_mut<'a>(&'a mut self) -> Option<&'a mut byteorder::F64<O>>
    where
        's: 'a,
    {
        match self {
            MutableValue::Double(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn set_double(&mut self, data: f64) -> bool {
        match self {
            MutableValue::Double(value) => {
                value.set(data);
                true
            }
            _ => false,
        }
    }

    #[inline]
    pub fn update_double(&mut self, f: impl FnOnce(f64) -> f64) -> bool {
        match self {
            MutableValue::Double(value) => {
                value.set(f(value.get()));
                true
            }
            _ => false,
        }
    }

    #[inline]
    pub fn as_byte_array_mut<'a>(&'a mut self) -> Option<&'a mut VecViewMut<'s, i8>>
    where
        's: 'a,
    {
        match self {
            MutableValue::ByteArray(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn as_string_mut<'a>(&'a mut self) -> Option<&'a mut StringViewMut<'s>>
    where
        's: 'a,
    {
        match self {
            MutableValue::String(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn as_list_mut<'a>(&'a mut self) -> Option<&'a mut MutableList<'s, O>>
    where
        's: 'a,
    {
        match self {
            MutableValue::List(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn as_compound_mut<'a>(&'a mut self) -> Option<&'a mut MutableCompound<'s, O>>
    where
        's: 'a,
    {
        match self {
            MutableValue::Compound(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn as_int_array_mut<'a>(&'a mut self) -> Option<&'a mut VecViewMut<'s, byteorder::I32<O>>>
    where
        's: 'a,
    {
        match self {
            MutableValue::IntArray(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn as_long_array_mut<'a>(&'a mut self) -> Option<&'a mut VecViewMut<'s, byteorder::I64<O>>>
    where
        's: 'a,
    {
        match self {
            MutableValue::LongArray(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn get_mut<'a, I: Index>(&'a mut self, index: I) -> Option<MutableValue<'a, O>>
    where
        's: 'a,
    {
        index.index_dispatch_mut(
            self,
            |value, index| match value {
                MutableValue::List(value) => value.get_mut(index),
                _ => None,
            },
            |value, key| match value {
                MutableValue::Compound(value) => value.get_mut(key),
                _ => None,
            },
        )
    }
}

pub struct MutableList<'s, O: ByteOrder> {
    pub(crate) data: VecViewMut<'s, u8>,
    pub(crate) _marker: PhantomData<O>,
}

impl<'s, O: ByteOrder> IntoIterator for MutableList<'s, O> {
    type Item = MutableValue<'s, O>;
    type IntoIter = MutableListIter<'s, O>;

    #[inline]
    fn into_iter(mut self) -> Self::IntoIter {
        MutableListIter {
            tag_id: self.tag_id(),
            remaining: self.len() as u32,
            data: unsafe { self.data.as_mut_ptr().add(1 + 4) },
            _marker: PhantomData,
        }
    }
}

impl<'s, O: ByteOrder> MutableList<'s, O> {
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

    pub fn get<'a>(&'a self, index: usize) -> Option<ImmutableValue<'a, O>>
    where
        's: 'a,
    {
        list_get(self.data.as_ptr(), index)
    }

    #[inline]
    pub fn iter<'a>(&'a self) -> ImmutableListIter<'a, O>
    where
        's: 'a,
    {
        list_iter(self.data.as_ptr())
    }
}

impl<'s, O: ByteOrder> MutableList<'s, O> {
    pub fn get_mut<'a>(&'a mut self, index: usize) -> Option<MutableValue<'a, O>>
    where
        's: 'a,
    {
        list_get_mut(self.data.as_mut_ptr(), index)
    }

    #[inline]
    pub fn iter_mut<'a>(&'a mut self) -> MutableListIter<'a, O>
    where
        's: 'a,
    {
        list_iter_mut(self.data.as_mut_ptr())
    }
}

impl<'s, O: ByteOrder> MutableList<'s, O> {
    pub fn push<V: IntoOwnedValue<O>>(&mut self, value: V) {
        value.list_push(&mut self.data);
    }

    /// .
    ///
    /// # Safety
    ///
    /// .
    pub unsafe fn push_unchecked<V: IntoOwnedValue<O>>(&mut self, value: V) {
        unsafe { value.list_push_unchecked(&mut self.data) };
    }

    pub fn insert<V: IntoOwnedValue<O>>(&mut self, index: usize, value: V) {
        value.list_insert(&mut self.data, index);
    }

    /// .
    ///
    /// # Safety
    ///
    /// .
    pub unsafe fn insert_unchecked<V: IntoOwnedValue<O>>(&mut self, index: usize, value: V) {
        unsafe { value.list_insert_unchecked(&mut self.data, index) };
    }

    pub fn pop(&mut self) -> Option<OwnedValue<O>> {
        list_pop(&mut self.data)
    }

    pub fn remove(&mut self, index: usize) -> OwnedValue<O> {
        list_remove(&mut self.data, index)
    }
}

pub struct MutableCompound<'s, O: ByteOrder> {
    pub(crate) data: VecViewMut<'s, u8>,
    pub(crate) _marker: PhantomData<O>,
}

impl<'s, O: ByteOrder> IntoIterator for MutableCompound<'s, O> {
    type Item = (ImmutableString<'s>, MutableValue<'s, O>);
    type IntoIter = MutableCompoundIter<'s, O>;

    #[inline]
    fn into_iter(mut self) -> Self::IntoIter {
        MutableCompoundIter {
            data: self.data.as_mut_ptr(),
            _marker: PhantomData,
        }
    }
}

impl<'s, O: ByteOrder> MutableCompound<'s, O> {
    pub fn get<'a>(&'a self, key: &str) -> Option<ImmutableValue<'a, O>>
    where
        's: 'a,
    {
        compound_get(self.data.as_ptr(), key)
    }

    #[inline]
    pub fn iter<'a>(&'a self) -> ImmutableCompoundIter<'a, O>
    where
        's: 'a,
    {
        compound_iter(self.data.as_ptr())
    }
}

impl<'s, O: ByteOrder> MutableCompound<'s, O> {
    pub fn get_mut<'a>(&'a mut self, key: &str) -> Option<MutableValue<'a, O>>
    where
        's: 'a,
    {
        compound_get_mut(self.data.as_mut_ptr(), key)
    }

    #[inline]
    pub fn iter_mut<'a>(&'a mut self) -> MutableCompoundIter<'a, O>
    where
        's: 'a,
    {
        compound_iter_mut(self.data.as_mut_ptr())
    }
}

impl<'s, O: ByteOrder> MutableCompound<'s, O> {
    pub fn insert<V: IntoOwnedValue<O>>(&mut self, key: &str, value: V) -> Option<OwnedValue<O>> {
        value.compound_insert(&mut self.data, key)
    }

    pub fn remove(&mut self, key: &str) -> Option<OwnedValue<O>> {
        compound_remove(&mut self.data, key)
    }
}
