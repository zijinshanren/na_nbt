//! Owned and mutable NBT value types.
//!
//! This module contains the owned value types that can be modified after parsing.
//! Unlike zero-copy types, these allocate memory and can outlive the source data.
//!
//! # Types
//!
//! ## Value Types
//!
//! - [`OwnedValue`] - Fully owned NBT value that can be modified
//! - [`MutableValue`] - Mutable borrowed view into an `OwnedValue`
//! - [`ImmutableValue`] - Immutable borrowed view into an `OwnedValue`
//!
//! ## Container Types
//!
//! - [`OwnedCompound`] / [`OwnedList`] - Owned compound and list types
//! - [`MutableCompound`] / [`MutableList`] - Mutable views into compounds and lists
//! - [`ImmutableCompound`] / [`ImmutableList`] / [`ImmutableString`] - Immutable views
//!
//! # When to Use
//!
//! Use owned types when you need to:
//! - Modify NBT data after parsing
//! - Keep values after the source bytes are dropped
//! - Build NBT structures from scratch
//! - Convert between endianness formats
//!
//! # Example
//!
//! ```
//! use na_nbt::{read_owned, OwnedValue, OwnedCompound};
//! use zerocopy::byteorder::BigEndian;
//!
//! // Parse existing NBT
//! let data = [0x0a, 0x00, 0x00, 0x00]; // Empty compound
//! let mut root: OwnedValue<BigEndian> = read_owned::<BigEndian, BigEndian>(&data).unwrap();
//!
//! // Modify the compound
//! if let OwnedValue::Compound(ref mut compound) = root {
//!     compound.insert("name", "Alex");
//!     compound.insert("health", 20i32);
//! }
//!
//! // Or build from scratch
//! let mut compound: OwnedCompound<BigEndian> = OwnedCompound::default();
//! compound.insert("x", 100i32);
//! compound.insert("y", 64i32);
//! compound.insert("z", -200i32);
//! ```

mod into_owned_value;
mod iter;
mod read;
mod trait_impl;
mod trait_impl_mut;
mod trait_impl_own;
mod util;
mod value;
mod value_mut;
mod value_own;
mod write;

use std::{any::TypeId, io::Write, ptr};

pub(crate) use into_owned_value::IntoOwnedValue;
pub use value::{ImmutableCompound, ImmutableList, ImmutableString, ImmutableValue};
pub use value_mut::{MutableCompound, MutableList, MutableValue};
pub use value_own::{OwnedCompound, OwnedList, OwnedValue};

use zerocopy::{IntoBytes, byteorder};

use crate::{
    ByteOrder, Error, Result, Tag, ValueScoped, cold_path,
    mutable::{
        read::{read_unsafe, read_unsafe_fallback},
        trait_impl::Config,
        write::{
            write_compound, write_compound_fallback, write_compound_to_writer,
            write_compound_to_writer_fallback, write_list, write_list_fallback,
            write_list_to_writer, write_list_to_writer_fallback,
        },
    },
};

/// Parses NBT from bytes into an owned, mutable value.
///
/// This function parses NBT data and returns an [`OwnedValue`] that can be
/// modified. It supports converting between different byte orders during
/// parsing.
///
/// # Arguments
///
/// * `source` - The byte slice containing NBT data
///
/// # Type Parameters
///
/// * `SOURCE` - The byte order of the input data
/// * `STORE` - The byte order for the in-memory representation
///
/// # Returns
///
/// A `Result` containing the parsed `OwnedValue` or an [`Error`].
///
/// # Example
///
/// ```
/// use na_nbt::{read_owned, OwnedValue};
/// use zerocopy::byteorder::{BigEndian, LittleEndian};
///
/// // Read Java Edition NBT (BigEndian) into native representation
/// let data = [0x0a, 0x00, 0x00, 0x00];
/// let value: OwnedValue<BigEndian> = read_owned::<BigEndian, BigEndian>(&data)?;
///
/// // Convert from BigEndian source to LittleEndian storage
/// let value: OwnedValue<LittleEndian> = read_owned::<BigEndian, LittleEndian>(&data)?;
/// # Ok::<(), na_nbt::Error>(())
/// ```
///
/// # Performance
///
/// - **Fast path**: When `SOURCE == STORE`, parsing is optimized
/// - **Fallback**: When converting endianness, each value is converted individually
///
/// For best write performance, choose `STORE` to match your target format.
///
/// # Errors
///
/// Returns an error if:
/// - The data is truncated ([`Error::EndOfFile`])
/// - An invalid tag type is encountered ([`Error::InvalidTagType`])
/// - Extra data remains after parsing ([`Error::TrailingData`])
pub fn read_owned<SOURCE: ByteOrder, STORE: ByteOrder>(source: &[u8]) -> Result<OwnedValue<STORE>> {
    unsafe {
        macro_rules! check_bounds {
            ($required:expr) => {
                if source.len() < $required {
                    cold_path();
                    return Err(Error::EndOfFile);
                }
            };
        }

        let mut current_pos = source.as_ptr();
        let end_pos = source.as_ptr().add(source.len());

        check_bounds!(1);

        let tag_id = *current_pos;
        current_pos = current_pos.add(1);

        if tag_id == 0 {
            cold_path();
            return Ok(OwnedValue::End);
        }

        check_bounds!(1 + 2);
        let name_len = byteorder::U16::<SOURCE>::from_bytes(*current_pos.cast()).get();

        check_bounds!(1 + 2 + name_len as usize);
        current_pos = current_pos.add(2 + name_len as usize);

        let value = if TypeId::of::<SOURCE>() == TypeId::of::<STORE>() {
            let result = read_unsafe::<SOURCE>(tag_id, &mut current_pos, end_pos)?;
            Ok(std::mem::transmute::<OwnedValue<SOURCE>, OwnedValue<STORE>>(result))
        } else {
            read_unsafe_fallback::<SOURCE, STORE>(tag_id, &mut current_pos, end_pos)
        }?;

        if current_pos < end_pos {
            cold_path();
            return Err(Error::TrailingData(
                end_pos.byte_offset_from_unsigned(current_pos),
            ));
        }

        Ok(value)
    }
}

pub(crate) fn write_owned_to_vec<'a, SOURCE: ByteOrder, TARGET: ByteOrder>(
    value: ValueScoped<'a, Config<SOURCE>>,
) -> Result<Vec<u8>> {
    unsafe {
        match value {
            ValueScoped::End => Ok(vec![0]),
            ValueScoped::Byte(value) => {
                let mut buf = Vec::<u8>::with_capacity(4);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [Tag::Byte as u8, 0u8, 0u8, value as u8]);
                buf.set_len(4);
                Ok(buf)
            }
            ValueScoped::Short(value) => {
                let mut buf = Vec::<u8>::with_capacity(1 + 2 + 2);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [Tag::Short as u8, 0u8, 0u8]);
                ptr::write(
                    buf_ptr.add(3).cast(),
                    byteorder::I16::<TARGET>::new(value).to_bytes(),
                );
                buf.set_len(1 + 2 + 2);
                Ok(buf)
            }
            ValueScoped::Int(value) => {
                let mut buf = Vec::<u8>::with_capacity(1 + 2 + 4);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [Tag::Int as u8, 0u8, 0u8]);
                ptr::write(
                    buf_ptr.add(3).cast(),
                    byteorder::I32::<TARGET>::new(value).to_bytes(),
                );
                buf.set_len(1 + 2 + 4);
                Ok(buf)
            }
            ValueScoped::Long(value) => {
                let mut buf = Vec::<u8>::with_capacity(1 + 2 + 8);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [Tag::Long as u8, 0u8, 0u8]);
                ptr::write(
                    buf_ptr.add(3).cast(),
                    byteorder::I64::<TARGET>::new(value).to_bytes(),
                );
                buf.set_len(1 + 2 + 8);
                Ok(buf)
            }
            ValueScoped::Float(value) => {
                let mut buf = Vec::<u8>::with_capacity(1 + 2 + 4);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [Tag::Float as u8, 0u8, 0u8]);
                ptr::write(
                    buf_ptr.add(3).cast(),
                    byteorder::F32::<TARGET>::new(value).to_bytes(),
                );
                buf.set_len(1 + 2 + 4);
                Ok(buf)
            }
            ValueScoped::Double(value) => {
                let mut buf = Vec::<u8>::with_capacity(1 + 2 + 8);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [Tag::Double as u8, 0u8, 0u8]);
                ptr::write(
                    buf_ptr.add(3).cast(),
                    byteorder::F64::<TARGET>::new(value).to_bytes(),
                );
                buf.set_len(1 + 2 + 8);
                Ok(buf)
            }
            ValueScoped::ByteArray(value) => {
                let payload = value.as_ptr().cast::<u8>();
                let len = value.len();
                let mut buf = Vec::<u8>::with_capacity(1 + 2 + 4 + len);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [Tag::ByteArray as u8, 0u8, 0u8]);
                ptr::write(
                    buf_ptr.add(1 + 2).cast(),
                    byteorder::U32::<TARGET>::new(len as u32).to_bytes(),
                );
                ptr::copy_nonoverlapping(payload, buf_ptr.add(1 + 2 + 4), len);
                buf.set_len(1 + 2 + 4 + len);
                Ok(buf)
            }
            ValueScoped::String(value) => {
                let payload = value.data.as_ptr().cast::<u8>();
                let len = value.data.len();
                let mut buf = Vec::<u8>::with_capacity(1 + 2 + len);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [Tag::String as u8, 0u8, 0u8]);
                ptr::write(
                    buf_ptr.add(1 + 2).cast(),
                    byteorder::U16::<TARGET>::new(len as u16).to_bytes(),
                );
                ptr::copy_nonoverlapping(payload, buf_ptr.add(1 + 2 + 2), len);
                buf.set_len(1 + 2 + len);
                Ok(buf)
            }
            ValueScoped::List(value) => {
                let payload = value.data;
                let mut buf = Vec::<u8>::with_capacity(1 + 2 + 4 + 128);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [Tag::List as u8, 0u8, 0u8]);
                buf.set_len(1 + 2);
                if TypeId::of::<SOURCE>() == TypeId::of::<TARGET>() {
                    write_list::<TARGET>(payload, &mut buf)?;
                } else {
                    write_list_fallback::<SOURCE, TARGET>(payload, &mut buf)?;
                }
                Ok(buf)
            }
            ValueScoped::Compound(value) => {
                let payload = value.data;
                let mut buf = Vec::<u8>::with_capacity(1 + 2 + 4 + 128);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [Tag::Compound as u8, 0u8, 0u8]);
                buf.set_len(1 + 2);
                if TypeId::of::<SOURCE>() == TypeId::of::<TARGET>() {
                    write_compound::<TARGET>(payload, &mut buf)?;
                } else {
                    write_compound_fallback::<SOURCE, TARGET>(payload, &mut buf)?;
                }
                Ok(buf)
            }
            ValueScoped::IntArray(value) => {
                let payload = value.as_ptr().cast::<u8>();
                let len = value.len();
                let mut buf = Vec::<u8>::with_capacity(1 + 2 + 4 + len);
                let mut buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [Tag::IntArray as u8, 0u8, 0u8]);
                ptr::write(
                    buf_ptr.add(1 + 2).cast(),
                    byteorder::U32::<TARGET>::new(len as u32).to_bytes(),
                );
                buf_ptr = buf_ptr.add(1 + 2 + 4);
                if TypeId::of::<SOURCE>() == TypeId::of::<TARGET>() {
                    ptr::copy_nonoverlapping(payload, buf_ptr, len * 4);
                } else {
                    for element in value {
                        ptr::write(
                            buf_ptr.cast(),
                            byteorder::I32::<TARGET>::new(element.get()).to_bytes(),
                        );
                        buf_ptr = buf_ptr.add(4);
                    }
                }
                buf.set_len(1 + 2 + 4 + len);
                Ok(buf)
            }
            ValueScoped::LongArray(value) => {
                let payload = value.as_ptr().cast::<u8>();
                let len = value.len();
                let mut buf = Vec::<u8>::with_capacity(1 + 2 + 8 + len);
                let mut buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [Tag::LongArray as u8, 0u8, 0u8]);
                ptr::write(
                    buf_ptr.add(1 + 2).cast(),
                    byteorder::U32::<TARGET>::new(len as u32).to_bytes(),
                );
                buf_ptr = buf_ptr.add(1 + 2 + 4);
                if TypeId::of::<SOURCE>() == TypeId::of::<TARGET>() {
                    ptr::copy_nonoverlapping(payload, buf_ptr, len * 8);
                } else {
                    for element in value {
                        ptr::write(
                            buf_ptr.cast(),
                            byteorder::I64::<TARGET>::new(element.get()).to_bytes(),
                        );
                        buf_ptr = buf_ptr.add(8);
                    }
                }
                buf.set_len(1 + 2 + 4 + len);
                Ok(buf)
            }
        }
    }
}

pub(crate) fn write_owned_to_writer<'a, SOURCE: ByteOrder, TARGET: ByteOrder>(
    value: ValueScoped<'a, Config<SOURCE>>,
    mut writer: impl Write,
) -> Result<()> {
    unsafe {
        match value {
            ValueScoped::End => writer.write_all(&[0]).map_err(Error::IO),
            ValueScoped::Byte(value) => writer
                .write_all(&[Tag::Byte as u8, 0u8, 0u8, value as u8])
                .map_err(Error::IO),
            ValueScoped::Short(value) => {
                let mut buf = [0u8; 1 + 2 + 2];
                ptr::write(buf.as_mut_ptr().cast(), [Tag::Short as u8, 0u8, 0u8]);
                ptr::write(
                    buf.as_mut_ptr().add(3).cast(),
                    byteorder::I16::<TARGET>::new(value).to_bytes(),
                );
                writer.write_all(&buf).map_err(Error::IO)
            }
            ValueScoped::Int(value) => {
                let mut buf = [0u8; 1 + 2 + 4];
                ptr::write(buf.as_mut_ptr().cast(), [Tag::Int as u8, 0u8, 0u8]);
                ptr::write(
                    buf.as_mut_ptr().add(3).cast(),
                    byteorder::I32::<TARGET>::new(value).to_bytes(),
                );
                writer.write_all(&buf).map_err(Error::IO)
            }
            ValueScoped::Long(value) => {
                let mut buf = [0u8; 1 + 2 + 8];
                ptr::write(buf.as_mut_ptr().cast(), [Tag::Long as u8, 0u8, 0u8]);
                ptr::write(
                    buf.as_mut_ptr().add(3).cast(),
                    byteorder::I64::<TARGET>::new(value).to_bytes(),
                );
                writer.write_all(&buf).map_err(Error::IO)
            }
            ValueScoped::Float(value) => {
                let mut buf = [0u8; 1 + 2 + 4];
                ptr::write(buf.as_mut_ptr().cast(), [Tag::Float as u8, 0u8, 0u8]);
                ptr::write(
                    buf.as_mut_ptr().add(3).cast(),
                    byteorder::F32::<TARGET>::new(value).to_bytes(),
                );
                writer.write_all(&buf).map_err(Error::IO)
            }
            ValueScoped::Double(value) => {
                let mut buf = [0u8; 1 + 2 + 8];
                ptr::write(buf.as_mut_ptr().cast(), [Tag::Double as u8, 0u8, 0u8]);
                ptr::write(
                    buf.as_mut_ptr().add(3).cast(),
                    byteorder::F64::<TARGET>::new(value).to_bytes(),
                );
                writer.write_all(&buf).map_err(Error::IO)
            }
            ValueScoped::ByteArray(value) => {
                let mut buf_head = [0u8; 1 + 2 + 4];
                ptr::write(
                    buf_head.as_mut_ptr().cast(),
                    [Tag::ByteArray as u8, 0u8, 0u8],
                );
                ptr::write(
                    buf_head.as_mut_ptr().add(3).cast(),
                    byteorder::U32::<TARGET>::new(value.len() as u32).to_bytes(),
                );
                writer.write_all(&buf_head).map_err(Error::IO)?;
                writer.write_all(value.as_bytes()).map_err(Error::IO)
            }
            ValueScoped::String(value) => {
                let mut buf_head = [0u8; 1 + 2 + 2];
                ptr::write(buf_head.as_mut_ptr().cast(), [Tag::String as u8, 0u8, 0u8]);
                ptr::write(
                    buf_head.as_mut_ptr().add(3).cast(),
                    byteorder::U16::<TARGET>::new(value.data.len() as u16).to_bytes(),
                );
                writer.write_all(&buf_head).map_err(Error::IO)?;
                writer.write_all(value.data.as_bytes()).map_err(Error::IO)
            }
            ValueScoped::List(value) => {
                writer
                    .write_all(&[Tag::List as u8, 0u8, 0u8])
                    .map_err(Error::IO)?;
                if TypeId::of::<SOURCE>() == TypeId::of::<TARGET>() {
                    write_list_to_writer::<TARGET>(value.data, &mut writer)?;
                } else {
                    write_list_to_writer_fallback::<SOURCE, TARGET>(value.data, &mut writer)?;
                }
                Ok(())
            }
            ValueScoped::Compound(value) => {
                writer
                    .write_all(&[Tag::Compound as u8, 0u8, 0u8])
                    .map_err(Error::IO)?;
                if TypeId::of::<SOURCE>() == TypeId::of::<TARGET>() {
                    write_compound_to_writer::<TARGET>(value.data, &mut writer)?;
                } else {
                    write_compound_to_writer_fallback::<SOURCE, TARGET>(value.data, &mut writer)?;
                }
                Ok(())
            }
            ValueScoped::IntArray(value) => {
                let mut buf_head = [0u8; 1 + 2 + 4];
                ptr::write(
                    buf_head.as_mut_ptr().cast(),
                    [Tag::IntArray as u8, 0u8, 0u8],
                );
                ptr::write(
                    buf_head.as_mut_ptr().add(3).cast(),
                    byteorder::U32::<TARGET>::new(value.len() as u32).to_bytes(),
                );
                writer.write_all(&buf_head).map_err(Error::IO)?;
                if TypeId::of::<SOURCE>() == TypeId::of::<TARGET>() {
                    writer.write_all(value.as_bytes()).map_err(Error::IO)
                } else {
                    for element in value {
                        writer
                            .write_all(&byteorder::I32::<TARGET>::new(element.get()).to_bytes())
                            .map_err(Error::IO)?;
                    }
                    Ok(())
                }
            }
            ValueScoped::LongArray(value) => {
                let mut buf_head = [0u8; 1 + 2 + 4];
                ptr::write(
                    buf_head.as_mut_ptr().cast(),
                    [Tag::LongArray as u8, 0u8, 0u8],
                );
                ptr::write(
                    buf_head.as_mut_ptr().add(3).cast(),
                    byteorder::U32::<TARGET>::new(value.len() as u32).to_bytes(),
                );
                writer.write_all(&buf_head).map_err(Error::IO)?;
                if TypeId::of::<SOURCE>() == TypeId::of::<TARGET>() {
                    writer.write_all(value.as_bytes()).map_err(Error::IO)
                } else {
                    for element in value {
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
