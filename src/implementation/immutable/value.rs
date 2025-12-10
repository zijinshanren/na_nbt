use std::{borrow::Cow, hint::unreachable_unchecked, marker::PhantomData, slice};

use zerocopy::byteorder;

use crate::{
    implementation::immutable::{mark::Mark, util::tag_size},
    index::Index,
    util::{ByteOrder, cold_path},
};

pub trait Document: Send + Sync + Clone + 'static {}

impl<T: Send + Sync + Clone + 'static> Document for T {}

#[derive(Clone)]
pub enum ImmutableValue<'doc, O: ByteOrder, D: Document> {
    End,
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(ImmutableArray<'doc, i8, D>),
    String(ImmutableString<'doc, D>),
    List(ImmutableList<'doc, O, D>),
    Compound(ImmutableCompound<'doc, O, D>),
    IntArray(ImmutableArray<'doc, byteorder::I32<O>, D>),
    LongArray(ImmutableArray<'doc, byteorder::I64<O>, D>),
}

impl<'doc, O: ByteOrder, D: Document> ImmutableValue<'doc, O, D> {
    pub unsafe fn read(tag_id: u8, data: *const u8, mark: *const Mark, doc: D) -> Self {
        unsafe {
            macro_rules! get {
                ($t:tt, $l:tt) => {{
                    ImmutableValue::$t(ImmutableArray {
                        data: slice::from_raw_parts(
                            data.add(std::mem::size_of::<byteorder::$l<O>>()).cast(),
                            byteorder::$l::<O>::from_bytes(*data.cast()).get() as usize,
                        ),
                        _doc: doc,
                    })
                }};
            }

            macro_rules! get_composite {
                ($t:tt, $s:tt) => {
                    ImmutableValue::$t($s {
                        data,
                        mark: mark.add(1),
                        doc,
                        _marker: PhantomData,
                    })
                };
            }

            match tag_id {
                0 => ImmutableValue::End,
                1 => ImmutableValue::Byte(*data.cast()),
                2 => ImmutableValue::Short(byteorder::I16::<O>::from_bytes(*data.cast()).get()),
                3 => ImmutableValue::Int(byteorder::I32::<O>::from_bytes(*data.cast()).get()),
                4 => ImmutableValue::Long(byteorder::I64::<O>::from_bytes(*data.cast()).get()),
                5 => ImmutableValue::Float(byteorder::F32::<O>::from_bytes(*data.cast()).get()),
                6 => ImmutableValue::Double(byteorder::F64::<O>::from_bytes(*data.cast()).get()),
                7 => get!(ByteArray, U32),
                8 => get!(String, U16),
                9 => get_composite!(List, ImmutableList),
                10 => get_composite!(Compound, ImmutableCompound),
                11 => get!(IntArray, U32),
                12 => get!(LongArray, U32),
                _ => unreachable_unchecked(),
            }
        }
    }
}

impl<'doc, O: ByteOrder, D: Document> ImmutableValue<'doc, O, D> {
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
    pub fn as_byte_array<'a>(&'a self) -> Option<&'a [i8]>
    where
        'doc: 'a,
    {
        match self {
            ImmutableValue::ByteArray(value) => Some(value.data),
            _ => None,
        }
    }

    #[inline]
    pub fn is_byte_array(&self) -> bool {
        matches!(self, ImmutableValue::ByteArray(_))
    }

    #[inline]
    pub fn as_string<'a>(&'a self) -> Option<&'a ImmutableString<'doc, D>>
    where
        'doc: 'a,
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
    pub fn as_list<'a>(&'a self) -> Option<&'a ImmutableList<'doc, O, D>>
    where
        'doc: 'a,
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
    pub fn as_compound<'a>(&'a self) -> Option<&'a ImmutableCompound<'doc, O, D>>
    where
        'doc: 'a,
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
    pub fn as_int_array<'a>(&'a self) -> Option<&'a [byteorder::I32<O>]>
    where
        'doc: 'a,
    {
        match self {
            ImmutableValue::IntArray(value) => Some(value.data),
            _ => None,
        }
    }

    #[inline]
    pub fn is_int_array(&self) -> bool {
        matches!(self, ImmutableValue::IntArray(_))
    }

    #[inline]
    pub fn as_long_array<'a>(&'a self) -> Option<&'a [byteorder::I64<O>]>
    where
        'doc: 'a,
    {
        match self {
            ImmutableValue::LongArray(value) => Some(value.data),
            _ => None,
        }
    }

    #[inline]
    pub fn is_long_array(&self) -> bool {
        matches!(self, ImmutableValue::LongArray(_))
    }

    #[inline]
    pub fn get<I: Index>(&self, index: I) -> Option<ImmutableValue<'doc, O, D>> {
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
}

#[derive(Clone)]
pub struct ImmutableArray<'doc, T, D: Document> {
    data: &'doc [T],
    _doc: D,
}

impl<'doc, T, D: Document> ImmutableArray<'doc, T, D> {
    #[inline]
    pub fn as_slice<'a>(&'a self) -> &'a [T]
    where
        'doc: 'a,
    {
        self.data
    }
}

pub type ImmutableString<'doc, D> = ImmutableArray<'doc, u8, D>;

impl<'doc, D: Document> ImmutableString<'doc, D> {
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
pub struct ImmutableList<'doc, O: ByteOrder, D: Document> {
    data: *const u8,
    mark: *const Mark,
    doc: D,
    _marker: PhantomData<(&'doc (), O)>,
}

unsafe impl<'doc, O: ByteOrder, D: Document> Send for ImmutableList<'doc, O, D> {}
unsafe impl<'doc, O: ByteOrder, D: Document> Sync for ImmutableList<'doc, O, D> {}

impl<'doc, O: ByteOrder, D: Document> IntoIterator for ImmutableList<'doc, O, D> {
    type Item = ImmutableValue<'doc, O, D>;
    type IntoIter = ImmutableListIter<'doc, O, D>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        ImmutableListIter {
            tag_id: self.tag_id(),
            remaining: self.len() as u32,
            data: unsafe { self.data.add(1 + 4) },
            mark: self.mark,
            doc: self.doc,
            _marker: PhantomData,
        }
    }
}

impl<'doc, O: ByteOrder, D: Document> ImmutableList<'doc, O, D> {
    #[inline]
    pub fn tag_id(&self) -> u8 {
        unsafe { *self.data.cast() }
    }

    #[inline]
    pub fn len(&self) -> usize {
        unsafe { byteorder::U32::<O>::from_bytes(*self.data.add(1).cast()).get() as usize }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, index: usize) -> Option<ImmutableValue<'doc, O, D>> {
        if index >= self.len() {
            cold_path();
            return None;
        }

        macro_rules! get {
            ($t: tt, $l: tt) => {
                unsafe {
                    let mut ptr = self.data.add(1 + 4);
                    for _ in 0..index {
                        let len = byteorder::$l::<O>::from_bytes(*ptr.cast()).get();
                        ptr = ptr.add(std::mem::size_of::<byteorder::$l<O>>() + len as usize);
                    }
                    let len = byteorder::$l::<O>::from_bytes(*ptr.cast()).get();
                    Some(ImmutableValue::$t(ImmutableArray {
                        data: slice::from_raw_parts(
                            ptr.add(std::mem::size_of::<byteorder::$l<O>>()).cast(),
                            len as usize,
                        ),
                        _doc: self.doc.clone(),
                    }))
                }
            };
        }

        macro_rules! get_composite {
            ($t:tt, $s:tt) => {
                unsafe {
                    let mut ptr = self.data.add(1 + 4);
                    let mut mark = self.mark;
                    for _ in 0..index {
                        ptr = (*mark).store.end_pointer;
                        mark = mark.add((*mark).store.flat_next_mark as usize);
                    }
                    Some(ImmutableValue::$t($s {
                        doc: self.doc.clone(),
                        data: ptr,
                        mark: mark.add(1),
                        _marker: PhantomData,
                    }))
                }
            };
        }

        match self.tag_id() {
            0 => Some(ImmutableValue::End),
            1 => Some(ImmutableValue::Byte(unsafe {
                *self.data.add(1 + 4 + index).cast()
            })),
            2 => Some(ImmutableValue::Short(unsafe {
                byteorder::I16::<O>::from_bytes(*self.data.add(1 + 4 + index * 2).cast()).get()
            })),
            3 => Some(ImmutableValue::Int(unsafe {
                byteorder::I32::<O>::from_bytes(*self.data.add(1 + 4 + index * 4).cast()).get()
            })),
            4 => Some(ImmutableValue::Long(unsafe {
                byteorder::I64::<O>::from_bytes(*self.data.add(1 + 4 + index * 8).cast()).get()
            })),
            5 => Some(ImmutableValue::Float(unsafe {
                byteorder::F32::<O>::from_bytes(*self.data.add(1 + 4 + index * 4).cast()).get()
            })),
            6 => Some(ImmutableValue::Double(unsafe {
                byteorder::F64::<O>::from_bytes(*self.data.add(1 + 4 + index * 8).cast()).get()
            })),
            7 => get!(ByteArray, U32),
            8 => get!(String, U16),
            9 => get_composite!(List, ImmutableList),
            10 => get_composite!(Compound, ImmutableCompound),
            11 => get!(IntArray, U32),
            12 => get!(LongArray, U32),
            _ => unsafe { unreachable_unchecked() },
        }
    }

    #[inline]
    pub fn iter(&self) -> ImmutableListIter<'doc, O, D> {
        ImmutableListIter {
            tag_id: self.tag_id(),
            remaining: self.len() as u32,
            data: unsafe { self.data.add(1 + 4) },
            mark: self.mark,
            doc: self.doc.clone(),
            _marker: PhantomData,
        }
    }
}

#[derive(Clone)]
pub struct ImmutableListIter<'doc, O: ByteOrder, D: Document> {
    tag_id: u8,
    remaining: u32,
    data: *const u8,
    mark: *const Mark,
    doc: D,
    _marker: PhantomData<(&'doc (), O)>,
}

unsafe impl<'doc, O: ByteOrder, D: Document> Send for ImmutableListIter<'doc, O, D> {}
unsafe impl<'doc, O: ByteOrder, D: Document> Sync for ImmutableListIter<'doc, O, D> {}

impl<'doc, O: ByteOrder, D: Document> Iterator for ImmutableListIter<'doc, O, D> {
    type Item = ImmutableValue<'doc, O, D>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            cold_path();
            return None;
        }

        self.remaining -= 1;

        let value =
            unsafe { ImmutableValue::read(self.tag_id, self.data, self.mark, self.doc.clone()) };

        let (data_advance, mark_advance) =
            unsafe { tag_size::<O>(self.tag_id, self.data, self.mark) };
        self.data = unsafe { self.data.add(data_advance) };
        self.mark = unsafe { self.mark.add(mark_advance) };

        Some(value)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.remaining as usize;
        (remaining, Some(remaining))
    }
}

impl<'doc, O: ByteOrder, D: Document> ExactSizeIterator for ImmutableListIter<'doc, O, D> {}

#[derive(Clone)]
pub struct ImmutableCompound<'doc, O: ByteOrder, D: Document> {
    data: *const u8,
    mark: *const Mark,
    doc: D,
    _marker: PhantomData<(&'doc (), O)>,
}

unsafe impl<'doc, O: ByteOrder, D: Document> Send for ImmutableCompound<'doc, O, D> {}
unsafe impl<'doc, O: ByteOrder, D: Document> Sync for ImmutableCompound<'doc, O, D> {}

impl<'doc, O: ByteOrder, D: Document> IntoIterator for ImmutableCompound<'doc, O, D> {
    type Item = (ImmutableString<'doc, D>, ImmutableValue<'doc, O, D>);
    type IntoIter = ImmutableCompoundIter<'doc, O, D>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        ImmutableCompoundIter {
            data: self.data,
            mark: self.mark,
            doc: self.doc,
            _marker: PhantomData,
        }
    }
}

impl<'doc, O: ByteOrder, D: Document> ImmutableCompound<'doc, O, D> {
    pub fn get(&self, key: &str) -> Option<ImmutableValue<'doc, O, D>> {
        let name = simd_cesu8::mutf8::encode(key);
        unsafe {
            let mut ptr = self.data;
            let mut mark = self.mark;
            loop {
                let tag_id = *ptr;
                ptr = ptr.add(1);

                if tag_id == 0 {
                    cold_path();
                    return None;
                }

                let name_len = byteorder::U16::<O>::from_bytes(*ptr.cast()).get();
                ptr = ptr.add(2);

                let name_bytes = core::slice::from_raw_parts(ptr, name_len as usize);
                ptr = ptr.add(name_len as usize);

                if name == name_bytes {
                    return Some(ImmutableValue::read(tag_id, ptr, mark, self.doc.clone()));
                }

                let (data_advance, mark_advance) = tag_size::<O>(tag_id, ptr, mark);
                ptr = ptr.add(data_advance);
                mark = mark.add(mark_advance);
            }
        }
    }

    #[inline]
    pub fn iter(&self) -> ImmutableCompoundIter<'doc, O, D> {
        ImmutableCompoundIter {
            data: self.data,
            mark: self.mark,
            doc: self.doc.clone(),
            _marker: PhantomData,
        }
    }
}

#[derive(Clone)]
pub struct ImmutableCompoundIter<'doc, O: ByteOrder, D: Document> {
    data: *const u8,
    mark: *const Mark,
    doc: D,
    _marker: PhantomData<(&'doc (), O)>,
}

unsafe impl<'doc, O: ByteOrder, D: Document> Send for ImmutableCompoundIter<'doc, O, D> {}
unsafe impl<'doc, O: ByteOrder, D: Document> Sync for ImmutableCompoundIter<'doc, O, D> {}

impl<'doc, O: ByteOrder, D: Document> Iterator for ImmutableCompoundIter<'doc, O, D> {
    type Item = (ImmutableString<'doc, D>, ImmutableValue<'doc, O, D>);

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let tag_id = *self.data.cast();

            if tag_id == 0 {
                cold_path();
                return None;
            }

            let name_len = byteorder::U16::<O>::from_bytes(*self.data.add(1).cast()).get();
            let name = ImmutableString {
                data: slice::from_raw_parts(self.data.add(3), name_len as usize),
                _doc: self.doc.clone(),
            };

            let value = ImmutableValue::read(
                tag_id,
                self.data.add(3 + name_len as usize),
                self.mark,
                self.doc.clone(),
            );

            let (data_advance, mark_advance) =
                tag_size::<O>(tag_id, self.data.add(3 + name_len as usize), self.mark);
            self.data = self.data.add(3 + name_len as usize + data_advance);
            self.mark = self.mark.add(mark_advance);

            Some((name, value))
        }
    }
}
