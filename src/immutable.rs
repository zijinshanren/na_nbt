//! Zero-copy NBT value types with immutable access.
//!
//! This module contains the zero-copy value types that reference NBT data directly
//! from the source bytes without copying. These types provide the fastest read-only
//! access to NBT data.
//!
//! # Types
//!
//! - [`BorrowedValue`] - Value that borrows from a byte slice (via [`read_borrowed`])
//! - [`SharedValue`] - Value with `Arc` ownership for thread-safe sharing (via [`read_shared`])
//! - [`ReadonlyValue`] - The underlying generic type for both
//!
//! # Container Types
//!
//! - [`ReadonlyArray`] - Zero-copy view of byte/int/long arrays
//! - [`ReadonlyString`] - Zero-copy view of NBT strings (Modified UTF-8)
//! - [`ReadonlyList`] - Zero-copy view of NBT lists
//! - [`ReadonlyCompound`] - Zero-copy view of NBT compounds
//!
//! # When to Use
//!
//! Use zero-copy types when:
//! - You only need to read NBT data, not modify it
//! - Performance is critical
//! - You want to minimize memory allocations
//!
//! # Example
//!
//! ```
//! use na_nbt::read_borrowed;
//! use zerocopy::byteorder::BigEndian;
//!
//! let data = [
//!     0x0a, 0x00, 0x00,  // Compound with empty name
//!     0x03, 0x00, 0x01, b'x', 0x00, 0x00, 0x00, 0x2a,  // Int "x" = 42
//!     0x00,  // End
//! ];
//!
//! let doc = read_borrowed::<BigEndian>(&data).unwrap();
//! let root = doc.root();
//!
//! assert_eq!(root.get("x").and_then(|v| v.as_int()), Some(42));
//! ```

use std::{any::TypeId, io::Write, ptr};
#[cfg(feature = "shared")]
use std::sync::Arc;

#[cfg(feature = "shared")]
use bytes::Bytes;
use zerocopy::{IntoBytes, byteorder};

use crate::{ByteOrder, Error, Result, Tag, cold_path};

mod mark;
mod read;
mod trait_impl;
mod util;
mod value;
mod write;

pub use value::{
    ReadonlyArray, ReadonlyCompound, ReadonlyCompoundIter, ReadonlyList, ReadonlyListIter,
    ReadonlyString, ReadonlyValue,
};

/// A zero-copy NBT value that borrows from a byte slice.
///
/// This is the return type of [`read_borrowed`]. It provides fast, read-only access
/// to NBT data without copying. The value is only valid while the source slice lives.
///
/// # Example
///
/// ```
/// use na_nbt::read_borrowed;
/// use zerocopy::byteorder::BigEndian;
///
/// let data = [0x0a, 0x00, 0x00, 0x00]; // Empty compound
/// let doc = read_borrowed::<BigEndian>(&data).unwrap();
/// let root = doc.root(); // BorrowedValue<BigEndian>
///
/// assert!(root.is_compound());
/// ```
///
/// # Generic Parameter
///
/// - `O` - The byte order of the NBT data ([`BigEndian`](zerocopy::byteorder::BigEndian)
///   for Java Edition, [`LittleEndian`](zerocopy::byteorder::LittleEndian) for Bedrock)
pub type BorrowedValue<'s, O> = value::ReadonlyValue<'s, O, ()>;

/// Parses NBT from a byte slice with zero-copy borrowing.
///
/// This function performs a zero-copy parse of the NBT data. The returned
/// [`BorrowedDocument`] borrows from the input slice, so the slice must
/// outlive the document.
///
/// # Arguments
///
/// * `source` - The byte slice containing NBT data
///
/// # Type Parameters
///
/// * `O` - The byte order of the input data
///
/// # Returns
///
/// A `Result` containing the parsed document or an [`Error`].
///
/// # Example
///
/// ```
/// use na_nbt::read_borrowed;
/// use zerocopy::byteorder::BigEndian;
///
/// let data = [0x0a, 0x00, 0x00, 0x00]; // Empty compound
/// let doc = read_borrowed::<BigEndian>(&data)?;
/// let root = doc.root();
/// # Ok::<(), na_nbt::Error>(())
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - The data is truncated ([`Error::EndOfFile`])
/// - An invalid tag type is encountered ([`Error::InvalidTagType`])
/// - Extra data remains after parsing ([`Error::TrailingData`])
pub fn read_borrowed<'s, O: ByteOrder>(source: &'s [u8]) -> Result<BorrowedDocument<'s, O>> {
    unsafe {
        read::read_unsafe::<O, _>(source.as_ptr(), source.len(), |mark| BorrowedDocument {
            mark,
            source: source.as_ptr(),
            _marker: core::marker::PhantomData::<(&'s (), O)>,
        })
    }
}

/// A parsed NBT document that borrows from a byte slice.
///
/// This type is returned by [`read_borrowed`] and provides access to the
/// root NBT value through the [`root`](BorrowedDocument::root) method.
///
/// # Lifetime
///
/// The `'s` lifetime parameter ties this document to the source byte slice.
/// The document (and any values derived from it) cannot outlive the source data.
pub struct BorrowedDocument<'s, O: ByteOrder> {
    mark: Vec<mark::Mark>,
    source: *const u8,
    _marker: core::marker::PhantomData<(&'s (), O)>,
}

impl<'s, O: ByteOrder> BorrowedDocument<'s, O> {
    /// Returns the root value of the NBT document.
    ///
    /// The returned value has a lifetime tied to this document, not the
    /// original source slice. This allows the document to manage its
    /// internal parsing state.
    ///
    /// # Example
    ///
    /// ```
    /// use na_nbt::read_borrowed;
    /// use zerocopy::byteorder::BigEndian;
    ///
    /// let data = [0x0a, 0x00, 0x00, 0x00];
    /// let doc = read_borrowed::<BigEndian>(&data).unwrap();
    /// let root = doc.root();
    ///
    /// assert!(root.is_compound());
    /// ```
    #[inline]
    pub fn root<'doc>(&'doc self) -> BorrowedValue<'doc, O> {
        let root_tag = unsafe { *self.source.cast() };

        if root_tag == Tag::End {
            cold_path();
            return BorrowedValue::End;
        }

        let name_len = byteorder::U16::<O>::from_bytes(unsafe { *self.source.add(1).cast() }).get();

        unsafe {
            BorrowedValue::read(
                root_tag,
                self.source.add(3 + name_len as usize),
                self.mark.as_ptr(),
                (),
            )
        }
    }
}

unsafe impl<'s, O: ByteOrder> Send for BorrowedDocument<'s, O> {}
unsafe impl<'s, O: ByteOrder> Sync for BorrowedDocument<'s, O> {}

// SharedValue and read_shared require the "shared" feature (bytes crate)
#[cfg(feature = "shared")]
mod shared {
    use super::*;

    /// A zero-copy NBT value with shared ownership via `Arc`.
    ///
    /// This type is returned by [`read_shared`]. Unlike [`BorrowedValue`], it owns
    /// its data through `Arc`, making it `Clone`, `Send`, `Sync`, and `'static`.
    ///
    /// Use this when you need to:
    /// - Pass NBT values across thread boundaries
    /// - Store values without lifetime concerns
    /// - Clone values efficiently (only clones the `Arc`, not the data)
    ///
    /// # Example
    ///
    /// ```
    /// use na_nbt::read_shared;
    /// use bytes::Bytes;
    /// use zerocopy::byteorder::BigEndian;
    ///
    /// let data = Bytes::from_static(&[0x0a, 0x00, 0x00, 0x00]);
    /// let root = read_shared::<BigEndian>(data).unwrap();
    ///
    /// // Clone is cheap - just increments Arc refcount
    /// let cloned = root.clone();
    ///
    /// // Can send to another thread
    /// std::thread::spawn(move || {
    ///     assert!(cloned.is_compound());
    /// }).join().unwrap();
    /// ```
    pub type SharedValue<O> = value::ReadonlyValue<'static, O, Arc<SharedDocument>>;

    /// Parses NBT from a `Bytes` buffer with shared ownership.
    ///
    /// The returned [`SharedValue`] owns the data via `Arc`, making it `Clone`,
    /// `Send`, `Sync`, and `'static`. This is ideal for multi-threaded scenarios
    /// or when you want to avoid lifetime management.
    ///
    /// # Arguments
    ///
    /// * `source` - A [`Bytes`] buffer containing NBT data
    ///
    /// # Type Parameters
    ///
    /// * `O` - The byte order of the input data
    ///
    /// # Returns
    ///
    /// The root NBT value, or an error if parsing fails.
    ///
    /// # Example
    ///
    /// ```
    /// use na_nbt::read_shared;
    /// use bytes::Bytes;
    /// use zerocopy::byteorder::BigEndian;
    ///
    /// let data = Bytes::from_static(&[0x0a, 0x00, 0x00, 0x00]);
    /// let root = read_shared::<BigEndian>(data)?;
    ///
    /// // Store in a struct with 'static lifetime
    /// struct MyStruct {
    ///     nbt: na_nbt::SharedValue<BigEndian>,
    /// }
    ///
    /// let s = MyStruct { nbt: root };
    /// # Ok::<(), na_nbt::Error>(())
    /// ```
    ///
    /// # Performance Note
    ///
    /// While parsing is still zero-copy, accessing the shared value has slightly
    /// more overhead than borrowed values due to `Arc` reference counting.
    /// Use [`read_borrowed`] when the borrowed lifetime is acceptable.
    pub fn read_shared<O: ByteOrder>(source: Bytes) -> Result<SharedValue<O>> {
        Ok(unsafe {
            read::read_unsafe::<O, _>(source.as_ptr(), source.len(), |mark| {
                Arc::new(SharedDocument { mark, source })
            })?
            .root()
        })
    }

    /// A parsed NBT document with shared ownership.
    ///
    /// This type holds the source data and parsing metadata for [`SharedValue`]s.
    /// It is managed through `Arc` and should not typically be used directly.
    pub struct SharedDocument {
        mark: Vec<mark::Mark>,
        source: Bytes,
    }

    impl SharedDocument {
        /// Returns the root value of the document.
        #[inline]
        pub fn root<O: ByteOrder>(self: Arc<Self>) -> SharedValue<O> {
            let root_tag = unsafe { Tag::from_u8_unchecked(*self.source.get_unchecked(0)) };

            if root_tag == Tag::End {
                cold_path();
                return SharedValue::End;
            }

            let name_len =
                byteorder::U16::<O>::from_bytes(unsafe { *self.source.as_ptr().add(1).cast() })
                    .get();

            unsafe {
                SharedValue::read(
                    root_tag,
                    self.source.as_ptr().add(3 + name_len as usize),
                    self.mark.as_ptr(),
                    self,
                )
            }
        }
    }
}

#[cfg(feature = "shared")]
pub use shared::{SharedDocument, SharedValue, read_shared};

pub(crate) fn write_value_to_vec<'s, D: value::Document, SOURCE: ByteOrder, TARGET: ByteOrder>(
    value: &value::ReadonlyValue<'s, SOURCE, D>,
) -> Result<Vec<u8>> {
    unsafe {
        match value {
            value::ReadonlyValue::End => Ok(vec![0]),
            value::ReadonlyValue::Byte(value) => {
                let mut buf = Vec::<u8>::with_capacity(4);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [Tag::Byte as u8, 0u8, 0u8, *value as u8]);
                buf.set_len(4);
                Ok(buf)
            }
            value::ReadonlyValue::Short(value) => {
                let mut buf = Vec::<u8>::with_capacity(1 + 2 + 2);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [Tag::Short as u8, 0u8, 0u8]);
                ptr::write(
                    buf_ptr.add(3).cast(),
                    byteorder::I16::<TARGET>::new(*value).to_bytes(),
                );
                buf.set_len(1 + 2 + 2);
                Ok(buf)
            }
            value::ReadonlyValue::Int(value) => {
                let mut buf = Vec::<u8>::with_capacity(1 + 2 + 4);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [Tag::Int as u8, 0u8, 0u8]);
                ptr::write(
                    buf_ptr.add(3).cast(),
                    byteorder::I32::<TARGET>::new(*value).to_bytes(),
                );
                buf.set_len(1 + 2 + 4);
                Ok(buf)
            }
            value::ReadonlyValue::Long(value) => {
                let mut buf = Vec::<u8>::with_capacity(1 + 2 + 8);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [Tag::Long as u8, 0u8, 0u8]);
                ptr::write(
                    buf_ptr.add(3).cast(),
                    byteorder::I64::<TARGET>::new(*value).to_bytes(),
                );
                buf.set_len(1 + 2 + 8);
                Ok(buf)
            }
            value::ReadonlyValue::Float(value) => {
                let mut buf = Vec::<u8>::with_capacity(1 + 2 + 4);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [Tag::Float as u8, 0u8, 0u8]);
                ptr::write(
                    buf_ptr.add(3).cast(),
                    byteorder::F32::<TARGET>::new(*value).to_bytes(),
                );
                buf.set_len(1 + 2 + 4);
                Ok(buf)
            }
            value::ReadonlyValue::Double(value) => {
                let mut buf = Vec::<u8>::with_capacity(1 + 2 + 8);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [Tag::Double as u8, 0u8, 0u8]);
                ptr::write(
                    buf_ptr.add(3).cast(),
                    byteorder::F64::<TARGET>::new(*value).to_bytes(),
                );
                buf.set_len(1 + 2 + 8);
                Ok(buf)
            }
            value::ReadonlyValue::ByteArray(value) => {
                let payload = value.data.as_ptr().cast::<u8>();
                let len = value.data.len();
                let mut buf = Vec::<u8>::with_capacity(3 + 4 + len);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [Tag::ByteArray as u8, 0u8, 0u8]);
                ptr::write(
                    buf_ptr.add(1 + 2).cast(),
                    byteorder::U32::<TARGET>::new(len as u32).to_bytes(),
                );
                ptr::copy_nonoverlapping(payload, buf_ptr.add(1 + 2 + 4), len);
                buf.set_len(3 + 4 + len);
                Ok(buf)
            }
            value::ReadonlyValue::String(value) => {
                let payload = value.data.as_ptr().cast::<u8>();
                let len = value.data.len();
                let mut buf = Vec::<u8>::with_capacity(3 + 2 + len);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [Tag::String as u8, 0u8, 0u8]);
                ptr::write(
                    buf_ptr.add(1 + 2).cast(),
                    byteorder::U16::<TARGET>::new(len as u16).to_bytes(),
                );
                ptr::copy_nonoverlapping(payload, buf_ptr.add(1 + 2 + 2), len);
                buf.set_len(3 + 2 + len);
                Ok(buf)
            }
            value::ReadonlyValue::List(value) => {
                let payload = value.data;
                let payload_len = payload.len();
                let mut buf = Vec::<u8>::with_capacity(3 + payload_len);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [Tag::List as u8, 0u8, 0u8]);
                ptr::copy_nonoverlapping(payload.as_ptr(), buf_ptr.add(3), payload_len);
                if TypeId::of::<SOURCE>() != TypeId::of::<TARGET>() {
                    let size_written =
                        write::write_list_fallback::<SOURCE, TARGET>(buf_ptr.add(1 + 2))?;
                    debug_assert!(size_written == payload_len);
                }
                buf.set_len(3 + payload_len);
                Ok(buf)
            }
            value::ReadonlyValue::Compound(value) => {
                let payload = value.data;
                let payload_len = payload.len();
                let mut buf = Vec::<u8>::with_capacity(3 + payload_len);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [Tag::Compound as u8, 0u8, 0u8]);
                ptr::copy_nonoverlapping(payload.as_ptr(), buf_ptr.add(3), payload_len);
                if TypeId::of::<SOURCE>() != TypeId::of::<TARGET>() {
                    let size_written =
                        write::write_compound_fallback::<SOURCE, TARGET>(buf_ptr.add(1 + 2))?;
                    debug_assert!(size_written == payload_len);
                }
                buf.set_len(3 + payload_len);
                Ok(buf)
            }
            value::ReadonlyValue::IntArray(value) => {
                let payload = value.data.as_ptr().cast::<u8>();
                let len = value.data.len();
                let len_bytes = std::mem::size_of_val(value.data);
                let mut buf = Vec::<u8>::with_capacity(3 + 4 + len_bytes);
                let mut buf_ptr = buf.as_mut_ptr();
                // head
                ptr::write(buf_ptr.cast(), [Tag::IntArray as u8, 0u8, 0u8]);
                // length
                ptr::write(
                    buf_ptr.add(1 + 2).cast(),
                    byteorder::U32::<TARGET>::new(len as u32).to_bytes(),
                );
                // data
                buf_ptr = buf_ptr.add(1 + 2 + 4);
                if TypeId::of::<SOURCE>() == TypeId::of::<TARGET>() {
                    ptr::copy_nonoverlapping(payload, buf_ptr, len_bytes);
                } else {
                    for element in value.data {
                        ptr::write(
                            buf_ptr.cast(),
                            byteorder::I32::<TARGET>::new(element.get()).to_bytes(),
                        );
                        buf_ptr = buf_ptr.add(4);
                    }
                }
                buf.set_len(3 + 4 + len_bytes);
                Ok(buf)
            }
            value::ReadonlyValue::LongArray(value) => {
                let payload = value.data.as_ptr().cast::<u8>();
                let len = value.data.len();
                let len_bytes = std::mem::size_of_val(value.data);
                let mut buf = Vec::<u8>::with_capacity(3 + 4 + len_bytes);
                let mut buf_ptr = buf.as_mut_ptr();
                // head
                ptr::write(buf_ptr.cast(), [Tag::LongArray as u8, 0u8, 0u8]);
                // length
                ptr::write(
                    buf_ptr.add(1 + 2).cast(),
                    byteorder::U32::<TARGET>::new(len as u32).to_bytes(),
                );
                // data
                buf_ptr = buf_ptr.add(1 + 2 + 4);
                if TypeId::of::<SOURCE>() == TypeId::of::<TARGET>() {
                    ptr::copy_nonoverlapping(payload, buf_ptr, len_bytes);
                } else {
                    for element in value.data {
                        ptr::write(
                            buf_ptr.cast(),
                            byteorder::I64::<TARGET>::new(element.get()).to_bytes(),
                        );
                        buf_ptr = buf_ptr.add(8);
                    }
                }
                buf.set_len(3 + 4 + len_bytes);
                Ok(buf)
            }
        }
    }
}

pub(crate) fn write_value_to_writer<
    's,
    D: value::Document,
    SOURCE: ByteOrder,
    TARGET: ByteOrder,
>(
    value: &value::ReadonlyValue<'s, SOURCE, D>,
    mut writer: impl Write,
) -> Result<()> {
    unsafe {
        match value {
            value::ReadonlyValue::End => writer.write_all(&[0]).map_err(Error::IO),
            value::ReadonlyValue::Byte(value) => writer
                .write_all(&[Tag::Byte as u8, 0u8, 0u8, *value as u8])
                .map_err(Error::IO),
            value::ReadonlyValue::Short(value) => {
                let mut buf = [0u8; 1 + 2 + 2];
                ptr::write(buf.as_mut_ptr().cast(), [Tag::Short as u8, 0u8, 0u8]);
                ptr::write(
                    buf.as_mut_ptr().add(3).cast(),
                    byteorder::I16::<TARGET>::new(*value).to_bytes(),
                );
                writer.write_all(&buf).map_err(Error::IO)
            }
            value::ReadonlyValue::Int(value) => {
                let mut buf = [0u8; 1 + 2 + 4];
                ptr::write(buf.as_mut_ptr().cast(), [Tag::Int as u8, 0u8, 0u8]);
                ptr::write(
                    buf.as_mut_ptr().add(3).cast(),
                    byteorder::I32::<TARGET>::new(*value).to_bytes(),
                );
                writer.write_all(&buf).map_err(Error::IO)
            }
            value::ReadonlyValue::Long(value) => {
                let mut buf = [0u8; 1 + 2 + 8];
                ptr::write(buf.as_mut_ptr().cast(), [Tag::Long as u8, 0u8, 0u8]);
                ptr::write(
                    buf.as_mut_ptr().add(3).cast(),
                    byteorder::I64::<TARGET>::new(*value).to_bytes(),
                );
                writer.write_all(&buf).map_err(Error::IO)
            }
            value::ReadonlyValue::Float(value) => {
                let mut buf = [0u8; 1 + 2 + 4];
                ptr::write(buf.as_mut_ptr().cast(), [Tag::Float as u8, 0u8, 0u8]);
                ptr::write(
                    buf.as_mut_ptr().add(3).cast(),
                    byteorder::F32::<TARGET>::new(*value).to_bytes(),
                );
                writer.write_all(&buf).map_err(Error::IO)
            }
            value::ReadonlyValue::Double(value) => {
                let mut buf = [0u8; 1 + 2 + 8];
                ptr::write(buf.as_mut_ptr().cast(), [Tag::Double as u8, 0u8, 0u8]);
                ptr::write(
                    buf.as_mut_ptr().add(3).cast(),
                    byteorder::F64::<TARGET>::new(*value).to_bytes(),
                );
                writer.write_all(&buf).map_err(Error::IO)
            }
            value::ReadonlyValue::ByteArray(value) => {
                let mut buf_head = [0u8; 1 + 2 + 4];
                ptr::write(
                    buf_head.as_mut_ptr().cast(),
                    [Tag::ByteArray as u8, 0u8, 0u8],
                );
                ptr::write(
                    buf_head.as_mut_ptr().add(3).cast(),
                    byteorder::U32::<TARGET>::new(value.data.len() as u32).to_bytes(),
                );
                writer.write_all(&buf_head).map_err(Error::IO)?;
                writer.write_all(value.data.as_bytes()).map_err(Error::IO)
            }
            value::ReadonlyValue::String(value) => {
                let mut buf_head = [0u8; 1 + 2 + 2];
                ptr::write(buf_head.as_mut_ptr().cast(), [Tag::String as u8, 0u8, 0u8]);
                ptr::write(
                    buf_head.as_mut_ptr().add(3).cast(),
                    byteorder::U16::<TARGET>::new(value.data.len() as u16).to_bytes(),
                );
                writer.write_all(&buf_head).map_err(Error::IO)?;
                writer.write_all(value.data.as_bytes()).map_err(Error::IO)
            }
            value::ReadonlyValue::List(value) => {
                writer
                    .write_all(&[Tag::List as u8, 0u8, 0u8])
                    .map_err(Error::IO)?;
                if TypeId::of::<SOURCE>() == TypeId::of::<TARGET>() {
                    writer.write_all(value.data.as_bytes()).map_err(Error::IO)
                } else {
                    let size_written = write::write_list_to_writer_fallback::<SOURCE, TARGET>(
                        value.data.as_ptr(),
                        &mut writer,
                    )?;
                    debug_assert!(size_written == value.data.len());
                    Ok(())
                }
            }
            value::ReadonlyValue::Compound(value) => {
                writer
                    .write_all(&[Tag::Compound as u8, 0u8, 0u8])
                    .map_err(Error::IO)?;
                if TypeId::of::<SOURCE>() == TypeId::of::<TARGET>() {
                    writer.write_all(value.data.as_bytes()).map_err(Error::IO)
                } else {
                    let size_written = write::write_compound_to_writer_fallback::<SOURCE, TARGET>(
                        value.data.as_ptr(),
                        &mut writer,
                    )?;
                    debug_assert!(size_written == value.data.len());
                    Ok(())
                }
            }
            value::ReadonlyValue::IntArray(value) => {
                let mut buf_head = [0u8; 1 + 2 + 4];
                ptr::write(
                    buf_head.as_mut_ptr().cast(),
                    [Tag::IntArray as u8, 0u8, 0u8],
                );
                ptr::write(
                    buf_head.as_mut_ptr().add(3).cast(),
                    byteorder::U32::<TARGET>::new(value.data.len() as u32).to_bytes(),
                );
                writer.write_all(&buf_head).map_err(Error::IO)?;
                if TypeId::of::<SOURCE>() == TypeId::of::<TARGET>() {
                    writer.write_all(value.data.as_bytes()).map_err(Error::IO)
                } else {
                    for element in value.data {
                        writer
                            .write_all(&byteorder::I32::<TARGET>::new(element.get()).to_bytes())
                            .map_err(Error::IO)?;
                    }
                    Ok(())
                }
            }
            value::ReadonlyValue::LongArray(value) => {
                let mut buf_head = [0u8; 1 + 2 + 4];
                ptr::write(
                    buf_head.as_mut_ptr().cast(),
                    [Tag::LongArray as u8, 0u8, 0u8],
                );
                ptr::write(
                    buf_head.as_mut_ptr().add(3).cast(),
                    byteorder::U32::<TARGET>::new(value.data.len() as u32).to_bytes(),
                );
                writer.write_all(&buf_head).map_err(Error::IO)?;
                if TypeId::of::<SOURCE>() == TypeId::of::<TARGET>() {
                    writer.write_all(value.data.as_bytes()).map_err(Error::IO)
                } else {
                    for element in value.data {
                        writer
                            .write_all(&byteorder::I64::<TARGET>::new(element.get()).to_bytes())
                            .map_err(Error::IO)?;
                    }
                    Ok(())
                }
            }
        }
    }
}
