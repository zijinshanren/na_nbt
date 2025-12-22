use std::{borrow::Cow, io::Write, marker::PhantomData, ops::Deref, ptr, slice};

use zerocopy::byteorder;

use crate::{
    ByteOrder, EMPTY_COMPOUND, EMPTY_LIST, NBT, NBTImpl, Result, TagByte, TagByteArray,
    TagCompound, TagDouble, TagEnd, TagFloat, TagID, TagInt, TagIntArray, TagList, TagLong,
    TagLongArray, TagShort, TagString, cold_path,
    immutable::{
        mark::Mark,
        trait_impl::Config,
        util::{ImmutableNBTImpl as _, tag_size},
    },
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
pub trait Document: Send + Sync + Clone + Never + 'static {}

pub trait Never {
    unsafe fn never() -> Self;
}

impl<T: Send + Sync + Clone + Never + 'static> Document for T {}

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
#[derive(Clone, Default)]
pub enum ReadonlyValue<'doc, O: ByteOrder, D: Document> {
    /// End tag (0) - marks the end of a compound.
    #[default]
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
    pub unsafe fn read(tag_id: TagID, data: *const u8, mark: *const Mark, doc: D) -> Self {
        unsafe {
            match tag_id {
                TagID::End => ReadonlyValue::End,
                TagID::Byte => ReadonlyValue::Byte(TagByte::read::<O, D>(data, mark, doc)),
                TagID::Short => ReadonlyValue::Short(TagShort::read::<O, D>(data, mark, doc)),
                TagID::Int => ReadonlyValue::Int(TagInt::read::<O, D>(data, mark, doc)),
                TagID::Long => ReadonlyValue::Long(TagLong::read::<O, D>(data, mark, doc)),
                TagID::Float => ReadonlyValue::Float(TagFloat::read::<O, D>(data, mark, doc)),
                TagID::Double => ReadonlyValue::Double(TagDouble::read::<O, D>(data, mark, doc)),
                TagID::ByteArray => {
                    ReadonlyValue::ByteArray(TagByteArray::read::<O, D>(data, mark, doc))
                }
                TagID::String => ReadonlyValue::String(TagString::read::<O, D>(data, mark, doc)),
                TagID::List => ReadonlyValue::List(TagList::read::<O, D>(data, mark, doc)),
                TagID::Compound => {
                    ReadonlyValue::Compound(TagCompound::read::<O, D>(data, mark, doc))
                }
                TagID::IntArray => {
                    ReadonlyValue::IntArray(TagIntArray::read::<O, D>(data, mark, doc))
                }
                TagID::LongArray => {
                    ReadonlyValue::LongArray(TagLongArray::read::<O, D>(data, mark, doc))
                }
            }
        }
    }
}

impl<'doc, O: ByteOrder, D: Document> NBTImpl<'doc, Config<O, D>, ReadonlyValue<'doc, O, D>>
    for TagEnd
{
    fn extract(value: ReadonlyValue<'doc, O, D>) -> Option<Self::Type<'doc, Config<O, D>>> {
        match value {
            ReadonlyValue::End => Some(()),
            _ => None,
        }
    }

    fn peek<'a>(value: &'a ReadonlyValue<'doc, O, D>) -> Option<&'a Self::Type<'doc, Config<O, D>>>
    where
        'doc: 'a,
    {
        match value {
            ReadonlyValue::End => Some(&()),
            _ => None,
        }
    }
}

impl<'doc, O: ByteOrder, D: Document> NBTImpl<'doc, Config<O, D>, ReadonlyValue<'doc, O, D>>
    for TagByte
{
    fn extract(value: ReadonlyValue<'doc, O, D>) -> Option<Self::Type<'doc, Config<O, D>>> {
        match value {
            ReadonlyValue::Byte(value) => Some(value),
            _ => None,
        }
    }

    fn peek<'a>(value: &'a ReadonlyValue<'doc, O, D>) -> Option<&'a Self::Type<'doc, Config<O, D>>>
    where
        'doc: 'a,
    {
        match value {
            ReadonlyValue::Byte(value) => Some(value),
            _ => None,
        }
    }
}

impl<'doc, O: ByteOrder, D: Document> NBTImpl<'doc, Config<O, D>, ReadonlyValue<'doc, O, D>>
    for TagShort
{
    fn extract(value: ReadonlyValue<'doc, O, D>) -> Option<Self::Type<'doc, Config<O, D>>> {
        match value {
            ReadonlyValue::Short(value) => Some(value),
            _ => None,
        }
    }

    fn peek<'a>(value: &'a ReadonlyValue<'doc, O, D>) -> Option<&'a Self::Type<'doc, Config<O, D>>>
    where
        'doc: 'a,
    {
        match value {
            ReadonlyValue::Short(value) => Some(value),
            _ => None,
        }
    }
}

impl<'doc, O: ByteOrder, D: Document> NBTImpl<'doc, Config<O, D>, ReadonlyValue<'doc, O, D>>
    for TagInt
{
    fn extract(value: ReadonlyValue<'doc, O, D>) -> Option<Self::Type<'doc, Config<O, D>>> {
        match value {
            ReadonlyValue::Int(value) => Some(value),
            _ => None,
        }
    }

    fn peek<'a>(value: &'a ReadonlyValue<'doc, O, D>) -> Option<&'a Self::Type<'doc, Config<O, D>>>
    where
        'doc: 'a,
    {
        match value {
            ReadonlyValue::Int(value) => Some(value),
            _ => None,
        }
    }
}

impl<'doc, O: ByteOrder, D: Document> NBTImpl<'doc, Config<O, D>, ReadonlyValue<'doc, O, D>>
    for TagLong
{
    fn extract(value: ReadonlyValue<'doc, O, D>) -> Option<Self::Type<'doc, Config<O, D>>> {
        match value {
            ReadonlyValue::Long(value) => Some(value),
            _ => None,
        }
    }

    fn peek<'a>(value: &'a ReadonlyValue<'doc, O, D>) -> Option<&'a Self::Type<'doc, Config<O, D>>>
    where
        'doc: 'a,
    {
        match value {
            ReadonlyValue::Long(value) => Some(value),
            _ => None,
        }
    }
}

impl<'doc, O: ByteOrder, D: Document> NBTImpl<'doc, Config<O, D>, ReadonlyValue<'doc, O, D>>
    for TagFloat
{
    fn extract(value: ReadonlyValue<'doc, O, D>) -> Option<Self::Type<'doc, Config<O, D>>> {
        match value {
            ReadonlyValue::Float(value) => Some(value),
            _ => None,
        }
    }

    fn peek<'a>(value: &'a ReadonlyValue<'doc, O, D>) -> Option<&'a Self::Type<'doc, Config<O, D>>>
    where
        'doc: 'a,
    {
        match value {
            ReadonlyValue::Float(value) => Some(value),
            _ => None,
        }
    }
}

impl<'doc, O: ByteOrder, D: Document> NBTImpl<'doc, Config<O, D>, ReadonlyValue<'doc, O, D>>
    for TagDouble
{
    fn extract(value: ReadonlyValue<'doc, O, D>) -> Option<Self::Type<'doc, Config<O, D>>> {
        match value {
            ReadonlyValue::Double(value) => Some(value),
            _ => None,
        }
    }

    fn peek<'a>(value: &'a ReadonlyValue<'doc, O, D>) -> Option<&'a Self::Type<'doc, Config<O, D>>>
    where
        'doc: 'a,
    {
        match value {
            ReadonlyValue::Double(value) => Some(value),
            _ => None,
        }
    }
}

impl<'doc, O: ByteOrder, D: Document> NBTImpl<'doc, Config<O, D>, ReadonlyValue<'doc, O, D>>
    for TagByteArray
{
    fn extract(value: ReadonlyValue<'doc, O, D>) -> Option<Self::Type<'doc, Config<O, D>>> {
        match value {
            ReadonlyValue::ByteArray(value) => Some(value),
            _ => None,
        }
    }

    fn peek<'a>(value: &'a ReadonlyValue<'doc, O, D>) -> Option<&'a Self::Type<'doc, Config<O, D>>>
    where
        'doc: 'a,
    {
        match value {
            ReadonlyValue::ByteArray(value) => Some(value),
            _ => None,
        }
    }
}

impl<'doc, O: ByteOrder, D: Document> NBTImpl<'doc, Config<O, D>, ReadonlyValue<'doc, O, D>>
    for TagString
{
    fn extract(value: ReadonlyValue<'doc, O, D>) -> Option<Self::Type<'doc, Config<O, D>>> {
        match value {
            ReadonlyValue::String(value) => Some(value),
            _ => None,
        }
    }

    fn peek<'a>(value: &'a ReadonlyValue<'doc, O, D>) -> Option<&'a Self::Type<'doc, Config<O, D>>>
    where
        'doc: 'a,
    {
        match value {
            ReadonlyValue::String(value) => Some(value),
            _ => None,
        }
    }
}

impl<'doc, O: ByteOrder, D: Document> NBTImpl<'doc, Config<O, D>, ReadonlyValue<'doc, O, D>>
    for TagList
{
    fn extract(value: ReadonlyValue<'doc, O, D>) -> Option<Self::Type<'doc, Config<O, D>>> {
        match value {
            ReadonlyValue::List(value) => Some(value),
            _ => None,
        }
    }

    fn peek<'a>(value: &'a ReadonlyValue<'doc, O, D>) -> Option<&'a Self::Type<'doc, Config<O, D>>>
    where
        'doc: 'a,
    {
        match value {
            ReadonlyValue::List(value) => Some(value),
            _ => None,
        }
    }
}

impl<'doc, O: ByteOrder, D: Document> NBTImpl<'doc, Config<O, D>, ReadonlyValue<'doc, O, D>>
    for TagCompound
{
    fn extract(value: ReadonlyValue<'doc, O, D>) -> Option<Self::Type<'doc, Config<O, D>>> {
        match value {
            ReadonlyValue::Compound(value) => Some(value),
            _ => None,
        }
    }

    fn peek<'a>(value: &'a ReadonlyValue<'doc, O, D>) -> Option<&'a Self::Type<'doc, Config<O, D>>>
    where
        'doc: 'a,
    {
        match value {
            ReadonlyValue::Compound(value) => Some(value),
            _ => None,
        }
    }
}

impl<'doc, O: ByteOrder, D: Document> NBTImpl<'doc, Config<O, D>, ReadonlyValue<'doc, O, D>>
    for TagIntArray
{
    fn extract(value: ReadonlyValue<'doc, O, D>) -> Option<Self::Type<'doc, Config<O, D>>> {
        match value {
            ReadonlyValue::IntArray(value) => Some(value),
            _ => None,
        }
    }

    fn peek<'a>(value: &'a ReadonlyValue<'doc, O, D>) -> Option<&'a Self::Type<'doc, Config<O, D>>>
    where
        'doc: 'a,
    {
        match value {
            ReadonlyValue::IntArray(value) => Some(value),
            _ => None,
        }
    }
}

impl<'doc, O: ByteOrder, D: Document> NBTImpl<'doc, Config<O, D>, ReadonlyValue<'doc, O, D>>
    for TagLongArray
{
    fn extract(value: ReadonlyValue<'doc, O, D>) -> Option<Self::Type<'doc, Config<O, D>>> {
        match value {
            ReadonlyValue::LongArray(value) => Some(value),
            _ => None,
        }
    }

    fn peek<'a>(value: &'a ReadonlyValue<'doc, O, D>) -> Option<&'a Self::Type<'doc, Config<O, D>>>
    where
        'doc: 'a,
    {
        match value {
            ReadonlyValue::LongArray(value) => Some(value),
            _ => None,
        }
    }
}

impl<'doc, O: ByteOrder, D: Document> ReadonlyValue<'doc, O, D> {
    #[inline]
    pub fn tag_id(&self) -> TagID {
        match self {
            ReadonlyValue::End => TagID::End,
            ReadonlyValue::Byte(_) => TagID::Byte,
            ReadonlyValue::Short(_) => TagID::Short,
            ReadonlyValue::Int(_) => TagID::Int,
            ReadonlyValue::Long(_) => TagID::Long,
            ReadonlyValue::Float(_) => TagID::Float,
            ReadonlyValue::Double(_) => TagID::Double,
            ReadonlyValue::ByteArray(_) => TagID::ByteArray,
            ReadonlyValue::String(_) => TagID::String,
            ReadonlyValue::List(_) => TagID::List,
            ReadonlyValue::Compound(_) => TagID::Compound,
            ReadonlyValue::IntArray(_) => TagID::IntArray,
            ReadonlyValue::LongArray(_) => TagID::LongArray,
        }
    }

    #[inline]
    pub fn peek<'a, T: NBTImpl<'doc, Config<O, D>, Self>>(
        &'a self,
    ) -> Option<&'a T::Type<'doc, Config<O, D>>>
    where
        'doc: 'a,
    {
        T::peek(self)
    }

    #[inline]
    pub fn extract<T: NBTImpl<'doc, Config<O, D>, Self>>(
        self,
    ) -> Option<T::Type<'doc, Config<O, D>>> {
        T::extract(self)
    }

    #[inline]
    pub fn is<T: NBT>(&self) -> bool {
        self.tag_id() == T::TAG_ID
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
    pub(crate) _doc: D,
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
    pub(crate) doc: D,
    pub(crate) _marker: PhantomData<O>,
}

impl<'doc, O: ByteOrder, D: Document> Default for ReadonlyList<'doc, O, D> {
    fn default() -> Self {
        Self {
            data: &EMPTY_LIST,
            mark: ptr::null(),
            doc: unsafe { Never::never() },
            _marker: PhantomData,
        }
    }
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
    pub fn tag_id(&self) -> TagID {
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
            TagID::End => Some(ReadonlyValue::End),
            TagID::Byte => Some(ReadonlyValue::Byte(unsafe {
                *self.data.as_ptr().add(1 + 4 + index).cast()
            })),
            TagID::Short => Some(ReadonlyValue::Short(unsafe {
                byteorder::I16::<O>::from_bytes(*self.data.as_ptr().add(1 + 4 + index * 2).cast())
                    .get()
            })),
            TagID::Int => Some(ReadonlyValue::Int(unsafe {
                byteorder::I32::<O>::from_bytes(*self.data.as_ptr().add(1 + 4 + index * 4).cast())
                    .get()
            })),
            TagID::Long => Some(ReadonlyValue::Long(unsafe {
                byteorder::I64::<O>::from_bytes(*self.data.as_ptr().add(1 + 4 + index * 8).cast())
                    .get()
            })),
            TagID::Float => Some(ReadonlyValue::Float(unsafe {
                byteorder::F32::<O>::from_bytes(*self.data.as_ptr().add(1 + 4 + index * 4).cast())
                    .get()
            })),
            TagID::Double => Some(ReadonlyValue::Double(unsafe {
                byteorder::F64::<O>::from_bytes(*self.data.as_ptr().add(1 + 4 + index * 8).cast())
                    .get()
            })),
            TagID::ByteArray => get!(ByteArray, U32),
            TagID::String => get!(String, U16),
            TagID::List => get_composite!(List, ReadonlyList),
            TagID::Compound => get_composite!(Compound, ReadonlyCompound),
            TagID::IntArray => get!(IntArray, U32),
            TagID::LongArray => get!(LongArray, U32),
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

    pub fn as_end_list(&self) -> Option<ReadonlyPrimitiveList<'doc, O, D, (), ()>> {
        (self.tag_id() == TagID::End).then_some(ReadonlyPrimitiveList {
            data: unsafe {
                slice::from_raw_parts(self.data.as_ptr().add(1 + 4).cast(), self.len())
            },
            _doc: self.doc.clone(),
            _marker: PhantomData,
        })
    }

    pub fn as_byte_list(&self) -> Option<ReadonlyPrimitiveList<'doc, O, D, i8, i8>> {
        (self.tag_id() == TagID::Byte).then_some(ReadonlyPrimitiveList {
            data: unsafe {
                slice::from_raw_parts(self.data.as_ptr().add(1 + 4).cast(), self.len())
            },
            _doc: self.doc.clone(),
            _marker: PhantomData,
        })
    }

    pub fn as_short_list(
        &self,
    ) -> Option<ReadonlyPrimitiveList<'doc, O, D, byteorder::I16<O>, i16>> {
        (self.tag_id() == TagID::Short).then_some(ReadonlyPrimitiveList {
            data: unsafe {
                slice::from_raw_parts(self.data.as_ptr().add(1 + 4).cast(), self.len())
            },
            _doc: self.doc.clone(),
            _marker: PhantomData,
        })
    }

    pub fn as_int_list(&self) -> Option<ReadonlyPrimitiveList<'doc, O, D, byteorder::I32<O>, i32>> {
        (self.tag_id() == TagID::Int).then_some(ReadonlyPrimitiveList {
            data: unsafe {
                slice::from_raw_parts(self.data.as_ptr().add(1 + 4).cast(), self.len())
            },
            _doc: self.doc.clone(),
            _marker: PhantomData,
        })
    }

    pub fn as_long_list(
        &self,
    ) -> Option<ReadonlyPrimitiveList<'doc, O, D, byteorder::I64<O>, i64>> {
        (self.tag_id() == TagID::Long).then_some(ReadonlyPrimitiveList {
            data: unsafe {
                slice::from_raw_parts(self.data.as_ptr().add(1 + 4).cast(), self.len())
            },
            _doc: self.doc.clone(),
            _marker: PhantomData,
        })
    }

    pub fn as_float_list(
        &self,
    ) -> Option<ReadonlyPrimitiveList<'doc, O, D, byteorder::F32<O>, f32>> {
        (self.tag_id() == TagID::Float).then_some(ReadonlyPrimitiveList {
            data: unsafe {
                slice::from_raw_parts(self.data.as_ptr().add(1 + 4).cast(), self.len())
            },
            _doc: self.doc.clone(),
            _marker: PhantomData,
        })
    }

    pub fn as_double_list(
        &self,
    ) -> Option<ReadonlyPrimitiveList<'doc, O, D, byteorder::F64<O>, f64>> {
        (self.tag_id() == TagID::Double).then_some(ReadonlyPrimitiveList {
            data: unsafe {
                slice::from_raw_parts(self.data.as_ptr().add(1 + 4).cast(), self.len())
            },
            _doc: self.doc.clone(),
            _marker: PhantomData,
        })
    }
}

/// An iterator over the elements of a [`ReadonlyList`].
///
/// This iterator yields [`ReadonlyValue`]s and implements [`ExactSizeIterator`].
#[derive(Clone)]
pub struct ReadonlyListIter<'doc, O: ByteOrder, D: Document> {
    tag_id: TagID,
    remaining: u32,
    data: *const u8,
    mark: *const Mark,
    doc: D,
    _marker: PhantomData<(&'doc (), O)>,
}

impl<'doc, O: ByteOrder, D: Document> Default for ReadonlyListIter<'doc, O, D> {
    fn default() -> Self {
        Self {
            tag_id: TagID::End,
            remaining: 0,
            data: ptr::null(),
            mark: ptr::null(),
            doc: unsafe { Never::never() },
            _marker: PhantomData,
        }
    }
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
    pub(crate) doc: D,
    pub(crate) _marker: PhantomData<O>,
}

impl<'doc, O: ByteOrder, D: Document> Default for ReadonlyCompound<'doc, O, D> {
    fn default() -> Self {
        Self {
            data: &EMPTY_COMPOUND,
            mark: ptr::null(),
            doc: unsafe { Never::never() },
            _marker: PhantomData,
        }
    }
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

                if tag_id == TagID::End {
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

impl<'doc, O: ByteOrder, D: Document> Default for ReadonlyCompoundIter<'doc, O, D> {
    fn default() -> Self {
        Self {
            data: EMPTY_COMPOUND.as_ptr(),
            mark: ptr::null(),
            doc: unsafe { Never::never() },
            _marker: PhantomData,
        }
    }
}

unsafe impl<'doc, O: ByteOrder, D: Document> Send for ReadonlyCompoundIter<'doc, O, D> {}
unsafe impl<'doc, O: ByteOrder, D: Document> Sync for ReadonlyCompoundIter<'doc, O, D> {}

impl<'doc, O: ByteOrder, D: Document> Iterator for ReadonlyCompoundIter<'doc, O, D> {
    type Item = (ReadonlyString<'doc, D>, ReadonlyValue<'doc, O, D>);

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let tag_id = *self.data.cast();

            if tag_id == TagID::End {
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
