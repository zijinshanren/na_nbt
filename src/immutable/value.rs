use std::{borrow::Cow, io::Write, marker::PhantomData, slice};

use zerocopy::byteorder;

use crate::{
    ByteOrder, Result, Tag, cold_path,
    immutable::{mark::Mark, util::tag_size},
    index::Index,
    write_value_to_vec, write_value_to_writer,
};

pub trait Document: Send + Sync + Clone + 'static {}

impl<T: Send + Sync + Clone + 'static> Document for T {}

/// A zero-copy, immutable NBT value (Mark-based).
///
/// This type is optimized for fast reading and low memory usage using a "mark" system
/// to navigate the data without parsing everything upfront. It references the underlying
/// data source directly.
///
/// This is distinct from [`crate::mutable::ImmutableValue`], which is a pointer-based immutable view.
///
/// The generic parameters are:
/// * `'doc`: The lifetime of the underlying data.
/// * `O`: The byte order (endianness) of the data.
/// * `D`: The document type, which manages the lifetime of the data source (e.g., `()` for borrowed slices, `Arc<SharedDocument>` for shared ownership).
#[derive(Clone)]
pub enum ReadonlyValue<'doc, O: ByteOrder, D: Document> {
    /// End tag (0).
    End,
    /// Byte tag (1).
    Byte(i8),
    /// Short tag (2).
    Short(i16),
    /// Int tag (3).
    Int(i32),
    /// Long tag (4).
    Long(i64),
    /// Float tag (5).
    Float(f32),
    /// Double tag (6).
    Double(f64),
    /// Byte array tag (7).
    ByteArray(ReadonlyArray<'doc, i8, D>),
    /// String tag (8).
    String(ReadonlyString<'doc, D>),
    /// List tag (9).
    List(ReadonlyList<'doc, O, D>),
    /// Compound tag (10).
    Compound(ReadonlyCompound<'doc, O, D>),
    /// Int array tag (11).
    IntArray(ReadonlyArray<'doc, byteorder::I32<O>, D>),
    /// Long array tag (12).
    LongArray(ReadonlyArray<'doc, byteorder::I64<O>, D>),
}

impl<'doc, O: ByteOrder, D: Document> ReadonlyValue<'doc, O, D> {
    pub unsafe fn read(tag_id: Tag, data: *const u8, mark: *const Mark, doc: D) -> Self {
        unsafe {
            macro_rules! get {
                ($t:tt, $l:tt) => {{
                    ReadonlyValue::$t(ReadonlyArray {
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
                    ReadonlyValue::$t($s {
                        data: slice::from_raw_parts(
                            data,
                            (*mark).store.end_pointer.byte_offset_from_unsigned(data),
                        ),
                        mark: mark.add(1),
                        doc,
                        _marker: PhantomData,
                    })
                };
            }

            match tag_id {
                Tag::End => ReadonlyValue::End,
                Tag::Byte => ReadonlyValue::Byte(*data.cast()),
                Tag::Short => {
                    ReadonlyValue::Short(byteorder::I16::<O>::from_bytes(*data.cast()).get())
                }
                Tag::Int => ReadonlyValue::Int(byteorder::I32::<O>::from_bytes(*data.cast()).get()),
                Tag::Long => {
                    ReadonlyValue::Long(byteorder::I64::<O>::from_bytes(*data.cast()).get())
                }
                Tag::Float => {
                    ReadonlyValue::Float(byteorder::F32::<O>::from_bytes(*data.cast()).get())
                }
                Tag::Double => {
                    ReadonlyValue::Double(byteorder::F64::<O>::from_bytes(*data.cast()).get())
                }
                Tag::ByteArray => get!(ByteArray, U32),
                Tag::String => get!(String, U16),
                Tag::List => get_composite!(List, ReadonlyList),
                Tag::Compound => get_composite!(Compound, ReadonlyCompound),
                Tag::IntArray => get!(IntArray, U32),
                Tag::LongArray => get!(LongArray, U32),
            }
        }
    }
}

impl<'doc, O: ByteOrder, D: Document> ReadonlyValue<'doc, O, D> {
    #[inline]
    pub fn tag_id(&self) -> Tag {
        match self {
            ReadonlyValue::End => Tag::End,
            ReadonlyValue::Byte(_) => Tag::Byte,
            ReadonlyValue::Short(_) => Tag::Short,
            ReadonlyValue::Int(_) => Tag::Int,
            ReadonlyValue::Long(_) => Tag::Long,
            ReadonlyValue::Float(_) => Tag::Float,
            ReadonlyValue::Double(_) => Tag::Double,
            ReadonlyValue::ByteArray(_) => Tag::ByteArray,
            ReadonlyValue::String(_) => Tag::String,
            ReadonlyValue::List(_) => Tag::List,
            ReadonlyValue::Compound(_) => Tag::Compound,
            ReadonlyValue::IntArray(_) => Tag::IntArray,
            ReadonlyValue::LongArray(_) => Tag::LongArray,
        }
    }

    #[inline]
    pub fn as_end(&self) -> Option<()> {
        match self {
            ReadonlyValue::End => Some(()),
            _ => None,
        }
    }

    #[inline]
    pub fn is_end(&self) -> bool {
        matches!(self, ReadonlyValue::End)
    }

    #[inline]
    pub fn as_byte(&self) -> Option<i8> {
        match self {
            ReadonlyValue::Byte(value) => Some(*value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_byte(&self) -> bool {
        matches!(self, ReadonlyValue::Byte(_))
    }

    #[inline]
    pub fn as_short(&self) -> Option<i16> {
        match self {
            ReadonlyValue::Short(value) => Some(*value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_short(&self) -> bool {
        matches!(self, ReadonlyValue::Short(_))
    }

    #[inline]
    pub fn as_int(&self) -> Option<i32> {
        match self {
            ReadonlyValue::Int(value) => Some(*value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_int(&self) -> bool {
        matches!(self, ReadonlyValue::Int(_))
    }

    #[inline]
    pub fn as_long(&self) -> Option<i64> {
        match self {
            ReadonlyValue::Long(value) => Some(*value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_long(&self) -> bool {
        matches!(self, ReadonlyValue::Long(_))
    }

    #[inline]
    pub fn as_float(&self) -> Option<f32> {
        match self {
            ReadonlyValue::Float(value) => Some(*value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_float(&self) -> bool {
        matches!(self, ReadonlyValue::Float(_))
    }

    #[inline]
    pub fn as_double(&self) -> Option<f64> {
        match self {
            ReadonlyValue::Double(value) => Some(*value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_double(&self) -> bool {
        matches!(self, ReadonlyValue::Double(_))
    }

    #[inline]
    pub fn as_byte_array<'a>(&'a self) -> Option<&'a [i8]>
    where
        'doc: 'a,
    {
        match self {
            ReadonlyValue::ByteArray(value) => Some(value.data),
            _ => None,
        }
    }

    #[inline]
    pub fn is_byte_array(&self) -> bool {
        matches!(self, ReadonlyValue::ByteArray(_))
    }

    #[inline]
    pub fn as_string<'a>(&'a self) -> Option<&'a ReadonlyString<'doc, D>>
    where
        'doc: 'a,
    {
        match self {
            ReadonlyValue::String(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_string(&self) -> bool {
        matches!(self, ReadonlyValue::String(_))
    }

    #[inline]
    pub fn as_list<'a>(&'a self) -> Option<&'a ReadonlyList<'doc, O, D>>
    where
        'doc: 'a,
    {
        match self {
            ReadonlyValue::List(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_list(&self) -> bool {
        matches!(self, ReadonlyValue::List(_))
    }

    #[inline]
    pub fn as_compound<'a>(&'a self) -> Option<&'a ReadonlyCompound<'doc, O, D>>
    where
        'doc: 'a,
    {
        match self {
            ReadonlyValue::Compound(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_compound(&self) -> bool {
        matches!(self, ReadonlyValue::Compound(_))
    }

    #[inline]
    pub fn as_int_array<'a>(&'a self) -> Option<&'a [byteorder::I32<O>]>
    where
        'doc: 'a,
    {
        match self {
            ReadonlyValue::IntArray(value) => Some(value.data),
            _ => None,
        }
    }

    #[inline]
    pub fn is_int_array(&self) -> bool {
        matches!(self, ReadonlyValue::IntArray(_))
    }

    #[inline]
    pub fn as_long_array<'a>(&'a self) -> Option<&'a [byteorder::I64<O>]>
    where
        'doc: 'a,
    {
        match self {
            ReadonlyValue::LongArray(value) => Some(value.data),
            _ => None,
        }
    }

    #[inline]
    pub fn is_long_array(&self) -> bool {
        matches!(self, ReadonlyValue::LongArray(_))
    }

    #[inline]
    pub fn get<I: Index>(&self, index: I) -> Option<ReadonlyValue<'doc, O, D>> {
        index.index_dispatch(
            self,
            |value, index| match value {
                ReadonlyValue::List(value) => value.get(index),
                _ => None,
            },
            |value, key| match value {
                ReadonlyValue::Compound(value) => value.get(key),
                _ => None,
            },
        )
    }

    #[inline]
    pub fn write_to_vec<TARGET: ByteOrder>(&self) -> Result<Vec<u8>> {
        write_value_to_vec::<D, O, TARGET>(self)
    }

    #[inline]
    pub fn write_to_writer<TARGET: ByteOrder>(&self, writer: impl Write) -> Result<()> {
        write_value_to_writer::<D, O, TARGET>(self, writer)
    }
}

#[derive(Clone)]
pub struct ReadonlyArray<'doc, T, D: Document> {
    pub(crate) data: &'doc [T],
    _doc: D,
}

impl<'doc, T, D: Document> ReadonlyArray<'doc, T, D> {
    #[inline]
    pub fn as_slice<'a>(&'a self) -> &'a [T]
    where
        'doc: 'a,
    {
        self.data
    }
}

pub type ReadonlyString<'doc, D> = ReadonlyArray<'doc, u8, D>;

impl<'doc, D: Document> ReadonlyString<'doc, D> {
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
pub struct ReadonlyList<'doc, O: ByteOrder, D: Document> {
    pub(crate) data: &'doc [u8],
    pub(crate) mark: *const Mark,
    doc: D,
    _marker: PhantomData<O>,
}

unsafe impl<'doc, O: ByteOrder, D: Document> Send for ReadonlyList<'doc, O, D> {}
unsafe impl<'doc, O: ByteOrder, D: Document> Sync for ReadonlyList<'doc, O, D> {}

impl<'doc, O: ByteOrder, D: Document> IntoIterator for ReadonlyList<'doc, O, D> {
    type Item = ReadonlyValue<'doc, O, D>;
    type IntoIter = ReadonlyListIter<'doc, O, D>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        ReadonlyListIter {
            tag_id: self.tag_id(),
            remaining: self.len() as u32,
            data: unsafe { self.data.as_ptr().add(1 + 4) },
            mark: self.mark,
            doc: self.doc,
            _marker: PhantomData,
        }
    }
}

impl<'doc, O: ByteOrder, D: Document> ReadonlyList<'doc, O, D> {
    #[inline]
    pub fn tag_id(&self) -> Tag {
        unsafe { *self.data.as_ptr().cast() }
    }

    #[inline]
    pub fn len(&self) -> usize {
        unsafe { byteorder::U32::<O>::from_bytes(*self.data.as_ptr().add(1).cast()).get() as usize }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, index: usize) -> Option<ReadonlyValue<'doc, O, D>> {
        if index >= self.len() {
            cold_path();
            return None;
        }

        macro_rules! get {
            ($t: tt, $l: tt) => {
                unsafe {
                    let mut ptr = self.data.as_ptr().add(1 + 4);
                    for _ in 0..index {
                        let len = byteorder::$l::<O>::from_bytes(*ptr.cast()).get();
                        ptr = ptr.add(std::mem::size_of::<byteorder::$l<O>>() + len as usize);
                    }
                    let len = byteorder::$l::<O>::from_bytes(*ptr.cast()).get();
                    Some(ReadonlyValue::$t(ReadonlyArray {
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
                    let mut ptr = self.data.as_ptr().add(1 + 4);
                    let mut mark = self.mark;
                    for _ in 0..index {
                        ptr = (*mark).store.end_pointer;
                        mark = mark.add((*mark).store.flat_next_mark as usize);
                    }
                    Some(ReadonlyValue::$t($s {
                        doc: self.doc.clone(),
                        data: slice::from_raw_parts(
                            ptr,
                            (*mark).store.end_pointer.byte_offset_from_unsigned(ptr),
                        ),
                        mark: mark.add(1),
                        _marker: PhantomData,
                    }))
                }
            };
        }

        match self.tag_id() {
            Tag::End => Some(ReadonlyValue::End),
            Tag::Byte => Some(ReadonlyValue::Byte(unsafe {
                *self.data.as_ptr().add(1 + 4 + index).cast()
            })),
            Tag::Short => Some(ReadonlyValue::Short(unsafe {
                byteorder::I16::<O>::from_bytes(*self.data.as_ptr().add(1 + 4 + index * 2).cast())
                    .get()
            })),
            Tag::Int => Some(ReadonlyValue::Int(unsafe {
                byteorder::I32::<O>::from_bytes(*self.data.as_ptr().add(1 + 4 + index * 4).cast())
                    .get()
            })),
            Tag::Long => Some(ReadonlyValue::Long(unsafe {
                byteorder::I64::<O>::from_bytes(*self.data.as_ptr().add(1 + 4 + index * 8).cast())
                    .get()
            })),
            Tag::Float => Some(ReadonlyValue::Float(unsafe {
                byteorder::F32::<O>::from_bytes(*self.data.as_ptr().add(1 + 4 + index * 4).cast())
                    .get()
            })),
            Tag::Double => Some(ReadonlyValue::Double(unsafe {
                byteorder::F64::<O>::from_bytes(*self.data.as_ptr().add(1 + 4 + index * 8).cast())
                    .get()
            })),
            Tag::ByteArray => get!(ByteArray, U32),
            Tag::String => get!(String, U16),
            Tag::List => get_composite!(List, ReadonlyList),
            Tag::Compound => get_composite!(Compound, ReadonlyCompound),
            Tag::IntArray => get!(IntArray, U32),
            Tag::LongArray => get!(LongArray, U32),
        }
    }

    #[inline]
    pub fn iter(&self) -> ReadonlyListIter<'doc, O, D> {
        ReadonlyListIter {
            tag_id: self.tag_id(),
            remaining: self.len() as u32,
            data: unsafe { self.data.as_ptr().add(1 + 4) },
            mark: self.mark,
            doc: self.doc.clone(),
            _marker: PhantomData,
        }
    }
}

#[derive(Clone)]
pub struct ReadonlyListIter<'doc, O: ByteOrder, D: Document> {
    tag_id: Tag,
    remaining: u32,
    data: *const u8,
    mark: *const Mark,
    doc: D,
    _marker: PhantomData<(&'doc (), O)>,
}

unsafe impl<'doc, O: ByteOrder, D: Document> Send for ReadonlyListIter<'doc, O, D> {}
unsafe impl<'doc, O: ByteOrder, D: Document> Sync for ReadonlyListIter<'doc, O, D> {}

impl<'doc, O: ByteOrder, D: Document> Iterator for ReadonlyListIter<'doc, O, D> {
    type Item = ReadonlyValue<'doc, O, D>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            cold_path();
            return None;
        }

        self.remaining -= 1;

        let value =
            unsafe { ReadonlyValue::read(self.tag_id, self.data, self.mark, self.doc.clone()) };

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

impl<'doc, O: ByteOrder, D: Document> ExactSizeIterator for ReadonlyListIter<'doc, O, D> {}

#[derive(Clone)]
pub struct ReadonlyCompound<'doc, O: ByteOrder, D: Document> {
    pub(crate) data: &'doc [u8],
    pub(crate) mark: *const Mark,
    doc: D,
    _marker: PhantomData<O>,
}

unsafe impl<'doc, O: ByteOrder, D: Document> Send for ReadonlyCompound<'doc, O, D> {}
unsafe impl<'doc, O: ByteOrder, D: Document> Sync for ReadonlyCompound<'doc, O, D> {}

impl<'doc, O: ByteOrder, D: Document> IntoIterator for ReadonlyCompound<'doc, O, D> {
    type Item = (ReadonlyString<'doc, D>, ReadonlyValue<'doc, O, D>);
    type IntoIter = ReadonlyCompoundIter<'doc, O, D>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        ReadonlyCompoundIter {
            data: self.data.as_ptr(),
            mark: self.mark,
            doc: self.doc,
            _marker: PhantomData,
        }
    }
}

impl<'doc, O: ByteOrder, D: Document> ReadonlyCompound<'doc, O, D> {
    pub fn get(&self, key: &str) -> Option<ReadonlyValue<'doc, O, D>> {
        let name = simd_cesu8::mutf8::encode(key);
        unsafe {
            let mut ptr = self.data.as_ptr();
            let mut mark = self.mark;
            loop {
                let tag_id = *ptr.cast();
                ptr = ptr.add(1);

                if tag_id == Tag::End {
                    cold_path();
                    return None;
                }

                let name_len = byteorder::U16::<O>::from_bytes(*ptr.cast()).get();
                ptr = ptr.add(2);

                let name_bytes = core::slice::from_raw_parts(ptr, name_len as usize);
                ptr = ptr.add(name_len as usize);

                if name == name_bytes {
                    return Some(ReadonlyValue::read(tag_id, ptr, mark, self.doc.clone()));
                }

                let (data_advance, mark_advance) = tag_size::<O>(tag_id, ptr, mark);
                ptr = ptr.add(data_advance);
                mark = mark.add(mark_advance);
            }
        }
    }

    #[inline]
    pub fn iter(&self) -> ReadonlyCompoundIter<'doc, O, D> {
        ReadonlyCompoundIter {
            data: self.data.as_ptr(),
            mark: self.mark,
            doc: self.doc.clone(),
            _marker: PhantomData,
        }
    }
}

#[derive(Clone)]
pub struct ReadonlyCompoundIter<'doc, O: ByteOrder, D: Document> {
    data: *const u8,
    mark: *const Mark,
    doc: D,
    _marker: PhantomData<(&'doc (), O)>,
}

unsafe impl<'doc, O: ByteOrder, D: Document> Send for ReadonlyCompoundIter<'doc, O, D> {}
unsafe impl<'doc, O: ByteOrder, D: Document> Sync for ReadonlyCompoundIter<'doc, O, D> {}

impl<'doc, O: ByteOrder, D: Document> Iterator for ReadonlyCompoundIter<'doc, O, D> {
    type Item = (ReadonlyString<'doc, D>, ReadonlyValue<'doc, O, D>);

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let tag_id = *self.data.cast();

            if tag_id == Tag::End {
                cold_path();
                return None;
            }

            let name_len = byteorder::U16::<O>::from_bytes(*self.data.add(1).cast()).get();
            let name = ReadonlyString {
                data: slice::from_raw_parts(self.data.add(3), name_len as usize),
                _doc: self.doc.clone(),
            };

            let value = ReadonlyValue::read(
                tag_id,
                self.data.add(3 + name_len as usize),
                self.mark,
                self.doc.clone(),
            );

            self.data = self.data.add(1 + 2 + name_len as usize);

            let (data_advance, mark_advance) = tag_size::<O>(tag_id, self.data, self.mark);
            self.data = self.data.add(data_advance);
            self.mark = self.mark.add(mark_advance);

            Some((name, value))
        }
    }
}
