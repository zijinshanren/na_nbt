use std::{borrow::Cow, io::Write, marker::PhantomData, ops::Deref, slice};

use zerocopy::byteorder;

use crate::{
    ByteOrder, Result, Tag, cold_path,
    immutable::{mark::Mark, util::tag_size},
    index::Index,
    write_value_to_vec, write_value_to_writer,
};

/// Marker trait for document ownership types.
///
/// This trait is automatically implemented for any type that is `Send + Sync + Clone + 'static`.
/// It is used to abstract over the ownership model of the underlying NBT data.
///
/// The two main implementations are:
/// - `()` - For borrowed values that reference external data
/// - `Arc<SharedDocument>` - For shared values with `Arc` ownership
pub trait Document: Send + Sync + Clone + 'static {}

impl<T: Send + Sync + Clone + 'static> Document for T {}

/// A zero-copy, immutable NBT value.
///
/// This is the core type for zero-copy NBT parsing. It references the original byte
/// slice directly without copying data, making parsing extremely fast.
///
/// # Type Aliases
///
/// You typically use this through type aliases:
/// - [`BorrowedValue`](crate::BorrowedValue) - When data lives in a borrowed slice
/// - [`SharedValue`](crate::SharedValue) - When data is wrapped in `Arc` for sharing
///
/// # Generic Parameters
///
/// - `'doc`: Lifetime of the underlying byte data
/// - `O`: Byte order ([`BigEndian`](zerocopy::byteorder::BigEndian) or [`LittleEndian`](zerocopy::byteorder::LittleEndian))
/// - `D`: Document type managing data ownership (`()` for borrowed, `Arc<...>` for shared)
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
/// | `String` | 8 | Modified UTF-8 string |
/// | `List` | 9 | Homogeneous list |
/// | `Compound` | 10 | Key-value map |
/// | `IntArray` | 11 | Array of ints |
/// | `LongArray` | 12 | Array of longs |
///
/// # Example
///
/// ```
/// use na_nbt::{read_borrowed, BorrowedValue};
/// use zerocopy::byteorder::BigEndian;
///
/// let data = [0x0a, 0x00, 0x00, 0x00]; // Empty compound
/// let doc = read_borrowed::<BigEndian>(&data).unwrap();
/// let root = doc.root();
///
/// match &root {
///     BorrowedValue::Compound(compound) => {
///         assert_eq!(compound.iter().count(), 0);
///     }
///     _ => panic!("Root is not a compound"),
/// }
/// ```
///
/// # See Also
///
/// - [`ImmutableValue`](crate::ImmutableValue) - Immutable view into owned data
/// - [`OwnedValue`](crate::OwnedValue) - Fully owned, mutable NBT value
#[derive(Clone)]
pub enum ReadonlyValue<'doc, O: ByteOrder, D: Document> {
    /// End tag (0) - marks the end of a compound.
    End,
    /// Byte tag (1) - a signed 8-bit integer.
    Byte(i8),
    /// Short tag (2) - a signed 16-bit integer.
    Short(i16),
    /// Int tag (3) - a signed 32-bit integer.
    Int(i32),
    /// Long tag (4) - a signed 64-bit integer.
    Long(i64),
    /// Float tag (5) - a 32-bit IEEE 754 floating point number.
    Float(f32),
    /// Double tag (6) - a 64-bit IEEE 754 floating point number.
    Double(f64),
    /// Byte array tag (7) - an array of signed bytes.
    ByteArray(ReadonlyArray<'doc, i8, D>),
    /// String tag (8) - a Modified UTF-8 encoded string.
    String(ReadonlyString<'doc, D>),
    /// List tag (9) - a list of values, all of the same type.
    List(ReadonlyList<'doc, O, D>),
    /// Compound tag (10) - a map of string keys to NBT values.
    Compound(ReadonlyCompound<'doc, O, D>),
    /// Int array tag (11) - an array of signed 32-bit integers.
    IntArray(ReadonlyArray<'doc, byteorder::I32<O>, D>),
    /// Long array tag (12) - an array of signed 64-bit integers.
    LongArray(ReadonlyArray<'doc, byteorder::I64<O>, D>),
}

impl<'doc, O: ByteOrder, D: Document> ReadonlyValue<'doc, O, D> {
    /// Reads a value from raw NBT data.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// - `tag_id` is a valid NBT tag type
    /// - `data` points to valid NBT data for the given tag type
    /// - `mark` points to valid parsing metadata
    /// - The data remains valid for the `'doc` lifetime
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
    pub fn as_byte_array<'a>(&'a self) -> Option<&'a ReadonlyArray<'doc, i8, D>>
    where
        'doc: 'a,
    {
        match self {
            ReadonlyValue::ByteArray(value) => Some(value),
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
    pub fn as_int_array<'a>(&'a self) -> Option<&'a ReadonlyArray<'doc, byteorder::I32<O>, D>>
    where
        'doc: 'a,
    {
        match self {
            ReadonlyValue::IntArray(value) => Some(value),
            _ => None,
        }
    }

    #[inline]
    pub fn is_int_array(&self) -> bool {
        matches!(self, ReadonlyValue::IntArray(_))
    }

    #[inline]
    pub fn as_long_array<'a>(&'a self) -> Option<&'a ReadonlyArray<'doc, byteorder::I64<O>, D>>
    where
        'doc: 'a,
    {
        match self {
            ReadonlyValue::LongArray(value) => Some(value),
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

    /// Serializes this value to a byte vector.
    ///
    /// The output includes the tag type and empty root name, making it a complete
    /// NBT document that can be written to a file or sent over the network.
    ///
    /// # Type Parameters
    ///
    /// * `TARGET` - The byte order for the output
    ///
    /// # Example
    ///
    /// ```
    /// use na_nbt::read_borrowed;
    /// use zerocopy::byteorder::{BigEndian, LittleEndian};
    ///
    /// let data = [0x0a, 0x00, 0x00, 0x00];
    /// let doc = read_borrowed::<BigEndian>(&data).unwrap();
    /// let root = doc.root();
    ///
    /// // Write as BigEndian (Java Edition)
    /// let bytes = root.write_to_vec::<BigEndian>().unwrap();
    ///
    /// // Or convert to LittleEndian (Bedrock Edition)
    /// let bytes = root.write_to_vec::<LittleEndian>().unwrap();
    /// ```
    #[inline]
    pub fn write_to_vec<TARGET: ByteOrder>(&self) -> Result<Vec<u8>> {
        write_value_to_vec::<D, O, TARGET>(self)
    }

    /// Serializes this value to a writer.
    ///
    /// Writes a complete NBT document (with tag type and empty root name) to
    /// any type implementing [`std::io::Write`].
    ///
    /// # Type Parameters
    ///
    /// * `TARGET` - The byte order for the output
    ///
    /// # Example
    ///
    /// ```
    /// use na_nbt::read_borrowed;
    /// use zerocopy::byteorder::BigEndian;
    /// use std::io::Cursor;
    ///
    /// let data = [0x0a, 0x00, 0x00, 0x00];
    /// let doc = read_borrowed::<BigEndian>(&data).unwrap();
    /// let root = doc.root();
    ///
    /// let mut buffer = Cursor::new(Vec::new());
    /// root.write_to_writer::<BigEndian>(&mut buffer).unwrap();
    /// ```
    #[inline]
    pub fn write_to_writer<TARGET: ByteOrder>(&self, writer: impl Write) -> Result<()> {
        write_value_to_writer::<D, O, TARGET>(self, writer)
    }
}

/// A zero-copy view of an NBT array (byte array, int array, or long array).
///
/// This type provides direct access to array data without copying. It implements
/// [`Deref`] to `[T]`, so you can use it like a slice.
///
/// # Type Parameters
///
/// - `'doc` - Lifetime of the underlying data
/// - `T` - Element type (`i8` for byte arrays, `I32<O>` for int arrays, `I64<O>` for long arrays)
/// - `D` - Document ownership type
///
/// # Example
///
/// ```
/// use na_nbt::{read_borrowed, BorrowedValue};
/// use zerocopy::byteorder::BigEndian;
///
/// # fn example() -> Option<()> {
/// let data = [
///     0x0a, 0x00, 0x00,  // Compound
///     0x07, 0x00, 0x01, b'a', 0x00, 0x00, 0x00, 0x03, 1, 2, 3,  // ByteArray "a" = [1,2,3]
///     0x00,  // End
/// ];
/// let doc = read_borrowed::<BigEndian>(&data).ok()?;
/// let root = doc.root();
/// let compound = root.as_compound()?;
/// let value = compound.get("a")?;
/// let array = value.as_byte_array()?;
///
/// // Use like a slice
/// assert_eq!(array.len(), 3);
/// assert_eq!(array[0], 1);
/// # Some(())
/// # }
/// ```
#[derive(Clone)]
pub struct ReadonlyArray<'doc, T, D: Document> {
    pub(crate) data: &'doc [T],
    _doc: D,
}

impl<'doc, T, D: Document> Deref for ReadonlyArray<'doc, T, D> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'doc, T, D: Document> ReadonlyArray<'doc, T, D> {
    /// Returns the array data as a slice.
    #[inline]
    pub fn as_slice<'a>(&'a self) -> &'a [T]
    where
        'doc: 'a,
    {
        self.data
    }
}

/// A zero-copy view of an NBT string.
///
/// NBT strings use Modified UTF-8 encoding (MUTF-8), which is similar to CESU-8.
/// Use [`decode`](ReadonlyString::decode) to convert to a Rust string.
///
/// # Example
///
/// ```
/// use na_nbt::{read_borrowed, BorrowedValue};
/// use zerocopy::byteorder::BigEndian;
///
/// # fn example() -> Option<()> {
/// let data = [
///     0x0a, 0x00, 0x00,  // Compound
///     0x08, 0x00, 0x01, b'n', 0x00, 0x05, b'H', b'e', b'l', b'l', b'o',  // String "n" = "Hello"
///     0x00,  // End
/// ];
/// let doc = read_borrowed::<BigEndian>(&data).ok()?;
/// let root = doc.root();
/// let compound = root.as_compound()?;
/// let name_value = compound.get("n")?;
/// let name_str = name_value.as_string()?;
///
/// assert_eq!(name_str.decode(), "Hello");
/// # Some(())
/// # }
/// ```
pub type ReadonlyString<'doc, D> = ReadonlyArray<'doc, u8, D>;

impl<'doc, D: Document> ReadonlyString<'doc, D> {
    /// Returns the raw MUTF-8 bytes of the string.
    ///
    /// For most ASCII strings, this is identical to UTF-8. Use [`decode`](Self::decode)
    /// for proper string conversion.
    #[inline]
    pub fn raw_bytes(&self) -> &[u8] {
        self.data
    }

    /// Decodes the MUTF-8 string to a Rust string.
    ///
    /// Returns a [`Cow<str>`](std::borrow::Cow) - borrowed if the string is valid UTF-8,
    /// owned if conversion was needed.
    ///
    /// Invalid sequences are replaced with the Unicode replacement character (U+FFFD).
    #[inline]
    pub fn decode<'a>(&'a self) -> Cow<'a, str> {
        simd_cesu8::mutf8::decode_lossy(self.data)
    }
}

/// A zero-copy view of an NBT list.
///
/// NBT lists are homogeneous - all elements have the same tag type. Use
/// [`tag_id`](ReadonlyList::tag_id) to get the element type.
///
/// This type implements [`IntoIterator`], so you can use it directly in a for loop.
///
/// # Example
///
/// ```
/// use na_nbt::{read_borrowed, BorrowedValue, Tag};
/// use zerocopy::byteorder::BigEndian;
///
/// # fn example() -> Option<()> {
/// let data = [
///     0x0a, 0x00, 0x00,  // Compound
///     0x09, 0x00, 0x01, b's', 0x03, 0x00, 0x00, 0x00, 0x02,  // List "s" of Int, length 2
///     0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02,  // [1, 2]
///     0x00,  // End
/// ];
/// let doc = read_borrowed::<BigEndian>(&data).ok()?;
/// let root = doc.root();
/// let compound = root.as_compound()?;
/// let value = compound.get("s")?;
/// let list = value.as_list()?.clone();
///
/// assert_eq!(list.tag_id(), Tag::Int);
/// assert_eq!(list.len(), 2);
///
/// for item in list {
///     // Each item is an Int
///     let _ = item.as_int();
/// }
/// # Some(())
/// # }
/// ```
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
    /// Returns the tag type of elements in this list.
    ///
    /// All elements in an NBT list have the same type.
    #[inline]
    pub fn tag_id(&self) -> Tag {
        unsafe { *self.data.as_ptr().cast() }
    }

    /// Returns the number of elements in this list.
    #[inline]
    pub fn len(&self) -> usize {
        unsafe { byteorder::U32::<O>::from_bytes(*self.data.as_ptr().add(1).cast()).get() as usize }
    }

    /// Returns `true` if this list contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the element at the given index, or `None` if out of bounds.
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

    /// Returns an iterator over the elements of this list.
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

/// An iterator over the elements of a [`ReadonlyList`].
///
/// This iterator yields [`ReadonlyValue`]s and implements [`ExactSizeIterator`].
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

/// A zero-copy view of an NBT compound.
///
/// NBT compounds are key-value maps where keys are strings and values can be
/// any NBT type. Use [`get`](ReadonlyCompound::get) to look up values by key,
/// or iterate to visit all entries.
///
/// This type implements [`IntoIterator`], yielding `(ReadonlyString, ReadonlyValue)` pairs.
///
/// # Example
///
/// ```
/// use na_nbt::{read_borrowed, BorrowedValue};
/// use zerocopy::byteorder::BigEndian;
///
/// # fn example() -> Option<()> {
/// let data = [
///     0x0a, 0x00, 0x00,  // Compound (root)
///     0x03, 0x00, 0x01, b'x', 0x00, 0x00, 0x00, 0x0a,  // Int "x" = 10
///     0x03, 0x00, 0x01, b'y', 0x00, 0x00, 0x00, 0x14,  // Int "y" = 20
///     0x00,  // End
/// ];
/// let doc = read_borrowed::<BigEndian>(&data).ok()?;
/// let root = doc.root();
/// let compound = root.as_compound()?.clone();
///
/// // Look up by key
/// let x = compound.get("x")?.as_int()?;
/// assert_eq!(x, 10);
///
/// // Iterate over all entries
/// for (key, value) in compound {
///     let _ = (key.decode(), value.tag_id());
/// }
/// # Some(())
/// # }
/// ```
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
    /// Returns the value associated with the given key, or `None` if not found.
    ///
    /// Key lookup uses MUTF-8 encoding internally to match NBT string format.
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

    /// Returns an iterator over the entries of this compound.
    ///
    /// Each entry is a `(ReadonlyString, ReadonlyValue)` pair.
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

/// An iterator over the entries of a [`ReadonlyCompound`].
///
/// Each iteration yields a `(ReadonlyString, ReadonlyValue)` pair representing
/// a key-value entry in the compound.
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
