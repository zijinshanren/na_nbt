//! Serde deserializer for NBT (Named Binary Tag) format.
//!
//! This module provides a [`serde::Deserializer`] implementation for deserializing
//! Rust types from NBT binary data.
//!
//! # Quick Start
//!
//! ```ignore
//! use serde::Deserialize;
//! use na_nbt::{from_slice_be, from_reader_be};
//!
//! #[derive(Deserialize)]
//! struct Player {
//!     name: String,
//!     health: f32,
//!     inventory: Vec<Item>,
//! }
//!
//! #[derive(Deserialize)]
//! struct Item {
//!     id: i32,
//!     count: i32,
//! }
//!
//! // From a byte slice
//! let player: Player = from_slice_be(&data)?;
//!
//! // From a file or any std::io::Read
//! let player: Player = from_reader_be(file)?;
//! ```
//!
//! # NBT to Rust Type Mapping
//!
//! | NBT Tag | Rust Types |
//! |---------|------------|
//! | `Byte` | `i8`, `u8`, `bool` |
//! | `Short` | `i16`, `u16` |
//! | `Int` | `i32`, `u32`, `char` |
//! | `Long` | `i64`, `u64` |
//! | `Float` | `f32` |
//! | `Double` | `f64` |
//! | `ByteArray` | `Vec<i8>` (with `#[serde(with = "na_nbt::byte_array")]`), `&[u8]`/`Vec<u8>` (with `serde_bytes`) |
//! | `String` | `&str`, `String` |
//! | `List` | `Vec<T>`, `[T; N]` |
//! | `Compound` | structs, `HashMap<String, T>` |
//! | `IntArray` | `Vec<i32>` (via List or `#[serde(with = "na_nbt::int_array")]`) |
//! | `LongArray` | `Vec<i64>` (via List or `#[serde(with = "na_nbt::long_array")]`) |
//!
//! # Enum Support
//!
//! All Rust enum variants are supported:
//!
//! ```ignore
//! #[derive(Deserialize)]
//! enum Data {
//!     // Unit variant: deserialized from Int (variant index)
//!     Empty,
//!     
//!     // Newtype variant: Compound { "Value": <inner> }
//!     Value(i32),
//!     
//!     // Tuple variant: Compound { "Point": List[Compound] }
//!     Point(i32, i32),
//!     
//!     // Struct variant: Compound { "Player": Compound { fields... } }
//!     Player { name: String, health: i32 },
//! }
//! ```
//!
//! # Byte Order
//!
//! NBT supports both big-endian (Java Edition) and little-endian (Bedrock Edition):
//!
//! ```ignore
//! use zerocopy::byteorder::{BigEndian, LittleEndian};
//!
//! // Java Edition (big-endian)
//! let data: Player = from_slice::<BigEndian, _>(&bytes)?;
//! let data: Player = from_slice_be(&bytes)?;  // convenience
//!
//! // Bedrock Edition (little-endian)
//! let data: Player = from_slice::<LittleEndian, _>(&bytes)?;
//! let data: Player = from_slice_le(&bytes)?;  // convenience
//! ```
//!
//! # Error Handling
//!
//! Deserialization can fail with these errors:
//! - [`Error::EndOfFile`] - Data truncated unexpectedly
//! - [`Error::InvalidTagType`] - Unknown NBT tag encountered
//! - [`Error::TrailingData`] - Extra bytes after root tag
//! - [`Error::TagMismatch`] - Type mismatch (e.g., expected Int, got String)
//!
//! [`Error::EndOfFile`]: crate::Error::EndOfFile
//! [`Error::InvalidTagType`]: crate::Error::InvalidTagType
//! [`Error::TrailingData`]: crate::Error::TrailingData
//! [`Error::TagMismatch`]: crate::Error::TagMismatch

use std::{borrow::Cow, io::Read, marker::PhantomData};

use serde::{
    Deserialize,
    de::{self, EnumAccess, MapAccess, SeqAccess, VariantAccess},
};
use zerocopy::byteorder;

use crate::{ByteOrder, Error, Result, TagID, cold_path};

/// NBT deserializer implementing [`serde::Deserializer`].
///
/// This deserializer reads NBT binary data from a byte slice and converts it
/// to Rust types using serde's deserialization framework.
///
/// # Example
///
/// ```ignore
/// use na_nbt::de::Deserializer;
/// use serde::Deserialize;
/// use zerocopy::byteorder::BigEndian;
///
/// let mut de = Deserializer::<BigEndian>::from_slice(&data)?;
/// let player = Player::deserialize(&mut de)?;
/// ```
///
/// For most use cases, prefer the convenience functions [`from_slice`], [`from_slice_be`],
/// [`from_reader`], etc.
pub struct Deserializer<'de, O: ByteOrder> {
    current_tag: TagID,
    input: &'de [u8],
    marker: PhantomData<O>,
}

macro_rules! check_bounds {
    ($len:expr, $input:expr) => {
        if $len > $input.len() {
            cold_path();
            return Err(Error::EndOfFile);
        }
    };
}

impl<'de, O: ByteOrder> Deserializer<'de, O> {
    pub fn from_slice(input: &'de [u8]) -> Result<Self> {
        check_bounds!(1, input);
        let tag_id = input[0];
        if tag_id == TagID::End as u8 || tag_id > TagID::LongArray as u8 {
            cold_path();
            return Err(Error::InvalidTagType(tag_id));
        }
        check_bounds!(1 + 2, input);
        let name_len =
            byteorder::U16::<O>::from_bytes(unsafe { *input.as_ptr().add(1).cast() }).get();
        check_bounds!(1 + 2 + name_len as usize, input);
        Ok(Self {
            current_tag: unsafe { TagID::from_u8_unchecked(tag_id) },
            input: &input[1 + 2 + name_len as usize..],
            marker: PhantomData,
        })
    }
}

/// Deserialize an NBT value from a byte slice.
///
/// This is the main entry point for NBT deserialization. The byte order `O`
/// determines whether to read big-endian (Java Edition) or little-endian
/// (Bedrock Edition) data.
///
/// # Example
///
/// ```ignore
/// use na_nbt::de::from_slice;
/// use zerocopy::byteorder::BigEndian;
///
/// let player: Player = from_slice::<BigEndian, _>(&data)?;
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - The data is truncated ([`Error::EndOfFile`])
/// - An invalid tag type is encountered ([`Error::InvalidTagType`])
/// - There are extra bytes after the root tag ([`Error::TrailingData`])
/// - Type mismatch during deserialization ([`Error::TagMismatch`])
///
/// [`Error::EndOfFile`]: crate::Error::EndOfFile
/// [`Error::InvalidTagType`]: crate::Error::InvalidTagType
/// [`Error::TrailingData`]: crate::Error::TrailingData
/// [`Error::TagMismatch`]: crate::Error::TagMismatch
pub fn from_slice<'de, O: ByteOrder, T>(input: &'de [u8]) -> Result<T>
where
    T: Deserialize<'de>,
{
    let mut deserializer = Deserializer::<O>::from_slice(input)?;
    let value = T::deserialize(&mut deserializer)?;
    if deserializer.input.is_empty() {
        Ok(value)
    } else {
        cold_path();
        Err(Error::TrailingData(deserializer.input.len()))
    }
}

/// Convenience function for deserializing with big-endian byte order.
///
/// # Example
/// ```ignore
/// let player: Player = from_slice_be(&data)?;
/// ```
#[inline]
pub fn from_slice_be<'de, T>(input: &'de [u8]) -> Result<T>
where
    T: Deserialize<'de>,
{
    from_slice::<zerocopy::byteorder::BigEndian, T>(input)
}

/// Convenience function for deserializing with little-endian byte order.
///
/// # Example
/// ```ignore
/// let player: Player = from_slice_le(&data)?;
/// ```
#[inline]
pub fn from_slice_le<'de, T>(input: &'de [u8]) -> Result<T>
where
    T: Deserialize<'de>,
{
    from_slice::<zerocopy::byteorder::LittleEndian, T>(input)
}

/// Deserialize from any `std::io::Read` implementation.
///
/// This reads all data into a buffer first, then deserializes.
/// For better performance with large data, consider using `from_slice`
/// with memory-mapped files or pre-loaded buffers.
///
/// # Example
/// ```ignore
/// use std::fs::File;
/// let file = File::open("player.nbt")?;
/// let player: Player = from_reader::<BigEndian, _, _>(file)?;
/// ```
pub fn from_reader<O: ByteOrder, T, R: Read>(mut reader: R) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).map_err(Error::IO)?;
    from_slice::<O, T>(&buf)
}

/// Convenience function for deserializing from a reader with big-endian byte order.
///
/// # Example
/// ```ignore
/// use std::fs::File;
/// let file = File::open("player.nbt")?;
/// let player: Player = from_reader_be(file)?;
/// ```
#[inline]
pub fn from_reader_be<T, R: Read>(reader: R) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    from_reader::<zerocopy::byteorder::BigEndian, T, R>(reader)
}

/// Convenience function for deserializing from a reader with little-endian byte order.
///
/// # Example
/// ```ignore
/// use std::fs::File;
/// let file = File::open("player.nbt")?;
/// let player: Player = from_reader_le(file)?;
/// ```
#[inline]
pub fn from_reader_le<T, R: Read>(reader: R) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    from_reader::<zerocopy::byteorder::LittleEndian, T, R>(reader)
}

impl<'de, O: ByteOrder> Deserializer<'de, O> {
    fn parse_i8(&mut self) -> Result<i8> {
        check_bounds!(1, self.input);
        let value = self.input[0];
        self.input = &self.input[1..];
        Ok(value as i8)
    }

    fn parse_i16(&mut self) -> Result<i16> {
        check_bounds!(2, self.input);
        let value = byteorder::I16::<O>::from_bytes(unsafe { *self.input.as_ptr().cast() }).get();
        self.input = &self.input[2..];
        Ok(value)
    }

    fn parse_i32(&mut self) -> Result<i32> {
        check_bounds!(4, self.input);
        let value = byteorder::I32::<O>::from_bytes(unsafe { *self.input.as_ptr().cast() }).get();
        self.input = &self.input[4..];
        Ok(value)
    }

    fn parse_i64(&mut self) -> Result<i64> {
        check_bounds!(8, self.input);
        let value = byteorder::I64::<O>::from_bytes(unsafe { *self.input.as_ptr().cast() }).get();
        self.input = &self.input[8..];
        Ok(value)
    }

    #[cfg(feature = "i128")]
    fn parse_u128(&mut self) -> Result<u128> {
        check_bounds!(4 + 4 * 4, self.input);
        unsafe {
            let read_ptr = self.input.as_ptr().add(4);
            let x1 = byteorder::U32::<O>::from_bytes(*read_ptr.cast()).get();
            let x2 = byteorder::U32::<O>::from_bytes(*read_ptr.add(4).cast()).get();
            let x3 = byteorder::U32::<O>::from_bytes(*read_ptr.add(8).cast()).get();
            let x4 = byteorder::U32::<O>::from_bytes(*read_ptr.add(12).cast()).get();
            self.input = &self.input[4 + 4 * 4..];
            Ok((x1 as u128) << 96 | (x2 as u128) << 64 | (x3 as u128) << 32 | (x4 as u128))
        }
    }

    fn parse_f32(&mut self) -> Result<f32> {
        check_bounds!(4, self.input);
        let value = byteorder::F32::<O>::from_bytes(unsafe { *self.input.as_ptr().cast() }).get();
        self.input = &self.input[4..];
        Ok(value)
    }

    fn parse_f64(&mut self) -> Result<f64> {
        check_bounds!(8, self.input);
        let value = byteorder::F64::<O>::from_bytes(unsafe { *self.input.as_ptr().cast() }).get();
        self.input = &self.input[8..];
        Ok(value)
    }

    fn parse_str(&mut self) -> Result<Cow<'de, str>> {
        check_bounds!(2, self.input);
        let length = byteorder::U16::<O>::from_bytes(unsafe { *self.input.as_ptr().cast() }).get();
        check_bounds!(2 + length as usize, self.input);
        let value = simd_cesu8::mutf8::decode_lossy(&self.input[2..2 + length as usize]);
        self.input = &self.input[2 + length as usize..];
        Ok(value)
    }

    fn parse_bytes(&mut self) -> Result<&'de [u8]> {
        check_bounds!(4, self.input);
        let length = byteorder::U32::<O>::from_bytes(unsafe { *self.input.as_ptr().cast() }).get();
        check_bounds!(4 + length as usize, self.input);
        let value = &self.input[4..4 + length as usize];
        self.input = &self.input[4 + length as usize..];
        Ok(value)
    }

    fn parse_unit(&mut self) -> Result<()> {
        check_bounds!(1, self.input);
        let value = self.input[0];
        if value != TagID::End as u8 {
            return Err(Error::InvalidTagType(value));
        }
        self.input = &self.input[1..];
        Ok(())
    }
}

macro_rules! check_tag {
    ($expected:expr, $actual:expr, $ok:block) => {
        if $expected == $actual {
            $ok
        } else {
            cold_path();
            Err(Error::TagMismatch($expected as u8, $actual as u8))
        }
    };
}

impl<'de, O: ByteOrder> de::Deserializer<'de> for &mut Deserializer<'de, O> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.current_tag {
            TagID::End => Err(Error::InvalidTagType(TagID::End as u8)),
            TagID::Byte => visitor.visit_i8(self.parse_i8()?),
            TagID::Short => visitor.visit_i16(self.parse_i16()?),
            TagID::Int => visitor.visit_i32(self.parse_i32()?),
            TagID::Long => visitor.visit_i64(self.parse_i64()?),
            TagID::Float => visitor.visit_f32(self.parse_f32()?),
            TagID::Double => visitor.visit_f64(self.parse_f64()?),
            TagID::ByteArray => visitor.visit_borrowed_bytes(self.parse_bytes()?),
            TagID::String => visitor.visit_str(self.parse_str()?.as_ref()),
            TagID::List => {
                check_bounds!(1 + 4, self.input);
                let tag_id = self.input[0];
                if tag_id > TagID::LongArray as u8 {
                    cold_path();
                    return Err(Error::InvalidTagType(tag_id));
                }
                let length =
                    byteorder::U32::<O>::from_bytes(unsafe { *self.input.as_ptr().add(1).cast() })
                        .get();
                if tag_id == TagID::End as u8 && length > 0 {
                    cold_path();
                    return Err(Error::InvalidTagType(tag_id));
                }
                self.input = &self.input[1 + 4..];
                visitor.visit_seq(ListDeserializer {
                    tag_id: unsafe { TagID::from_u8_unchecked(tag_id) },
                    index: 0,
                    len: length,
                    deserializer: self,
                })
            }
            TagID::Compound => visitor.visit_map(CompoundAccess {
                deserializer: self,
                value_tag: TagID::End,
            }),
            TagID::IntArray => {
                check_bounds!(4, self.input);
                let length =
                    byteorder::U32::<O>::from_bytes(unsafe { *self.input.as_ptr().cast() }).get();
                self.input = &self.input[4..];
                visitor.visit_seq(ListDeserializer {
                    tag_id: TagID::Int,
                    index: 0,
                    len: length,
                    deserializer: self,
                })
            }
            TagID::LongArray => {
                check_bounds!(4, self.input);
                let length =
                    byteorder::U32::<O>::from_bytes(unsafe { *self.input.as_ptr().cast() }).get();
                self.input = &self.input[4..];
                visitor.visit_seq(ListDeserializer {
                    tag_id: TagID::Long,
                    index: 0,
                    len: length,
                    deserializer: self,
                })
            }
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::Byte, self.current_tag, {
            visitor.visit_bool(self.parse_i8()? != 0)
        })
    }

    fn deserialize_i8<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::Byte, self.current_tag, {
            visitor.visit_i8(self.parse_i8()?)
        })
    }

    fn deserialize_i16<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::Short, self.current_tag, {
            visitor.visit_i16(self.parse_i16()?)
        })
    }

    fn deserialize_i32<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::Int, self.current_tag, {
            visitor.visit_i32(self.parse_i32()?)
        })
    }

    fn deserialize_i64<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::Long, self.current_tag, {
            visitor.visit_i64(self.parse_i64()?)
        })
    }

    fn deserialize_u8<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::Byte, self.current_tag, {
            visitor.visit_u8(self.parse_i8()? as u8)
        })
    }

    fn deserialize_u16<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::Short, self.current_tag, {
            visitor.visit_u16(self.parse_i16()? as u16)
        })
    }

    fn deserialize_u32<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::Int, self.current_tag, {
            visitor.visit_u32(self.parse_i32()? as u32)
        })
    }

    fn deserialize_u64<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::Long, self.current_tag, {
            visitor.visit_u64(self.parse_i64()? as u64)
        })
    }

    #[cfg(feature = "i128")]
    fn deserialize_i128<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::IntArray, self.current_tag, {
            visitor.visit_i128(self.parse_u128()? as i128)
        })
    }

    /// Deserialize u128 from IntArray with 4 elements (most significant to least significant)
    #[cfg(feature = "i128")]
    fn deserialize_u128<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::IntArray, self.current_tag, {
            visitor.visit_u128(self.parse_u128()?)
        })
    }

    fn deserialize_f32<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::Float, self.current_tag, {
            visitor.visit_f32(self.parse_f32()?)
        })
    }

    fn deserialize_f64<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::Double, self.current_tag, {
            visitor.visit_f64(self.parse_f64()?)
        })
    }

    fn deserialize_char<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::Int, self.current_tag, {
            let value = self.parse_i32()? as u32;
            visitor.visit_char(char::from_u32(value).ok_or(Error::InvalidCharacter(value))?)
        })
    }

    fn deserialize_str<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::String, self.current_tag, {
            visitor.visit_str(self.parse_str()?.as_ref())
        })
    }

    fn deserialize_string<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::String, self.current_tag, {
            visitor.visit_string(self.parse_str()?.into_owned())
        })
    }

    fn deserialize_bytes<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::ByteArray, self.current_tag, {
            visitor.visit_borrowed_bytes(self.parse_bytes()?)
        })
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::ByteArray, self.current_tag, {
            visitor.visit_byte_buf(self.parse_bytes()?.to_vec())
        })
    }

    fn deserialize_option<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::Compound, self.current_tag, {
            check_bounds!(1, self.input);
            let tag_id = self.input[0];
            let value = if tag_id == TagID::End as u8 {
                visitor.visit_none()
            } else if tag_id <= TagID::LongArray as u8 {
                check_bounds!(1 + 2, self.input);
                self.input = &self.input[1 + 2..];
                self.current_tag = unsafe { TagID::from_u8_unchecked(tag_id) };
                visitor.visit_some(&mut *self)
            } else {
                cold_path();
                Err(Error::InvalidTagType(tag_id))
            };
            check_bounds!(1, self.input);
            self.input = &self.input[1..];
            value
        })
    }

    fn deserialize_unit<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::Compound, self.current_tag, {
            self.parse_unit()?;
            visitor.visit_unit()
        })
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        // Auto-detect format based on current tag
        match self.current_tag {
            TagID::IntArray => {
                // IntArray format: length (4 bytes) + i32 data
                check_bounds!(4, self.input);
                let length =
                    byteorder::U32::<O>::from_bytes(unsafe { *self.input.as_ptr().cast() }).get();
                self.input = &self.input[4..];
                visitor.visit_seq(ListDeserializer {
                    tag_id: TagID::Int,
                    index: 0,
                    len: length,
                    deserializer: self,
                })
            }
            TagID::LongArray => {
                // LongArray format: length (4 bytes) + i64 data
                check_bounds!(4, self.input);
                let length =
                    byteorder::U32::<O>::from_bytes(unsafe { *self.input.as_ptr().cast() }).get();
                self.input = &self.input[4..];
                visitor.visit_seq(ListDeserializer {
                    tag_id: TagID::Long,
                    index: 0,
                    len: length,
                    deserializer: self,
                })
            }
            TagID::List => {
                // Standard List format: element_tag (1 byte) + length (4 bytes)
                check_bounds!(1 + 4, self.input);
                let tag_id = self.input[0];
                if tag_id > TagID::LongArray as u8 {
                    cold_path();
                    return Err(Error::InvalidTagType(tag_id));
                }
                let length =
                    byteorder::U32::<O>::from_bytes(unsafe { *self.input.as_ptr().add(1).cast() })
                        .get();
                if tag_id == TagID::End as u8 && length > 0 {
                    cold_path();
                    return Err(Error::InvalidTagType(tag_id));
                }
                self.input = &self.input[1 + 4..];
                visitor.visit_seq(ListDeserializer {
                    tag_id: unsafe { TagID::from_u8_unchecked(tag_id) },
                    index: 0,
                    len: length,
                    deserializer: self,
                })
            }
            _ => {
                cold_path();
                Err(Error::TagMismatch(TagID::List as u8, self.current_tag as u8))
            }
        }
    }

    fn deserialize_tuple<V>(
        self,
        _len: usize,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        // Tuples are serialized as List<Compound> where each compound has a single unnamed field
        check_tag!(TagID::List, self.current_tag, {
            check_bounds!(1 + 4, self.input);
            let tag_id = self.input[0];
            if tag_id != TagID::Compound as u8 {
                cold_path();
                return Err(Error::TagMismatch(TagID::Compound as u8, tag_id));
            }
            let length =
                byteorder::U32::<O>::from_bytes(unsafe { *self.input.as_ptr().add(1).cast() })
                    .get();
            self.input = &self.input[1 + 4..];
            visitor.visit_seq(TupleDeserializer {
                index: 0,
                len: length,
                deserializer: self,
            })
        })
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::Compound, self.current_tag, {
            visitor.visit_map(CompoundAccess {
                deserializer: self,
                value_tag: TagID::End,
            })
        })
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::Compound, self.current_tag, {
            visitor.visit_map(CompoundAccess {
                deserializer: self,
                value_tag: TagID::End,
            })
        })
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.current_tag {
            // Unit variant - serialized as variant index (u32)
            TagID::Int => {
                let variant_index = self.parse_i32()? as u32;
                visitor.visit_enum(UnitVariantAccess { variant_index })
            }
            // Newtype, tuple, or struct variant - all serialized as compound
            // Format: Tag::Compound { variant_name: <value> }
            // - Newtype: inner tag is the value's tag
            // - Tuple: inner tag is Tag::List (List<Compound>)
            // - Struct: inner tag is Tag::Compound
            TagID::Compound => {
                check_bounds!(1, self.input);
                let tag_id = self.input[0];
                if tag_id == TagID::End as u8 || tag_id > TagID::LongArray as u8 {
                    cold_path();
                    return Err(Error::InvalidTagType(tag_id));
                }
                self.input = &self.input[1..];
                // Read variant name
                let variant_name = self.parse_str()?;
                visitor.visit_enum(VariantDeserializer {
                    deserializer: self,
                    variant_name,
                    variant_tag: unsafe { TagID::from_u8_unchecked(tag_id) },
                })
            }
            _ => {
                cold_path();
                Err(Error::TagMismatch(
                    TagID::Compound as u8,
                    self.current_tag as u8,
                ))
            }
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_str(self.parse_str()?.as_ref())
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

struct ListDeserializer<'a, 'de: 'a, O: ByteOrder> {
    tag_id: TagID,
    index: u32,
    len: u32,
    deserializer: &'a mut Deserializer<'de, O>,
}

impl<'a, 'de, O: ByteOrder> SeqAccess<'de> for ListDeserializer<'a, 'de, O> {
    type Error = Error;

    fn next_element_seed<T>(
        &mut self,
        seed: T,
    ) -> std::result::Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.index >= self.len {
            return Ok(None);
        }
        self.index += 1;
        self.deserializer.current_tag = self.tag_id;
        seed.deserialize(&mut *self.deserializer).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        Some((self.len - self.index) as usize)
    }
}

// For deserializing compounds (structs and maps)
struct CompoundAccess<'a, 'de: 'a, O: ByteOrder> {
    deserializer: &'a mut Deserializer<'de, O>,
    value_tag: TagID,
}

impl<'a, 'de, O: ByteOrder> MapAccess<'de> for CompoundAccess<'a, 'de, O> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        check_bounds!(1, self.deserializer.input);
        let tag_id = self.deserializer.input[0];

        if tag_id == TagID::End as u8 {
            self.deserializer.input = &self.deserializer.input[1..];
            return Ok(None);
        }

        if tag_id > TagID::LongArray as u8 {
            cold_path();
            return Err(Error::InvalidTagType(tag_id));
        }

        self.deserializer.input = &self.deserializer.input[1..];
        self.value_tag = unsafe { TagID::from_u8_unchecked(tag_id) };

        // Deserialize the key (field name as string)
        seed.deserialize(FieldNameDeserializer {
            deserializer: &mut *self.deserializer,
        })
        .map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        self.deserializer.current_tag = self.value_tag;
        seed.deserialize(&mut *self.deserializer)
    }
}

// For deserializing field names in compounds
struct FieldNameDeserializer<'a, 'de: 'a, O: ByteOrder> {
    deserializer: &'a mut Deserializer<'de, O>,
}

impl<'de, 'a, O: ByteOrder> de::Deserializer<'de> for FieldNameDeserializer<'a, 'de, O> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_str(self.deserializer.parse_str()?.as_ref())
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_string(self.deserializer.parse_str()?.into_owned())
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char bytes byte_buf
        option unit unit_struct newtype_struct seq tuple tuple_struct
        map struct enum ignored_any
    }
}

// For deserializing tuples (each element is wrapped in a compound)
struct TupleDeserializer<'a, 'de: 'a, O: ByteOrder> {
    index: u32,
    len: u32,
    deserializer: &'a mut Deserializer<'de, O>,
}

impl<'a, 'de, O: ByteOrder> SeqAccess<'de> for TupleDeserializer<'a, 'de, O> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.index >= self.len {
            return Ok(None);
        }
        self.index += 1;

        // Each element is wrapped: tag_id (1), name_len (2, always 0), value, Tag::End (1)
        check_bounds!(1 + 2, self.deserializer.input);
        let tag_id = self.deserializer.input[0];
        if tag_id == TagID::End as u8 || tag_id > TagID::LongArray as u8 {
            cold_path();
            return Err(Error::InvalidTagType(tag_id));
        }
        // Verify empty name
        let name_len = byteorder::U16::<O>::from_bytes(unsafe {
            *self.deserializer.input.as_ptr().add(1).cast()
        })
        .get();
        if name_len != 0 {
            cold_path();
            return Err(Error::Message(
                "Expected empty name in tuple element".into(),
            ));
        }
        // Skip tag_id and empty name
        self.deserializer.input = &self.deserializer.input[1 + 2..];
        self.deserializer.current_tag = unsafe { TagID::from_u8_unchecked(tag_id) };

        let value = seed.deserialize(&mut *self.deserializer)?;

        // Consume the Tag::End
        check_bounds!(1, self.deserializer.input);
        if self.deserializer.input[0] != TagID::End as u8 {
            cold_path();
            return Err(Error::InvalidTagType(self.deserializer.input[0]));
        }
        self.deserializer.input = &self.deserializer.input[1..];

        Ok(Some(value))
    }
}

// For unit enum variants (index only)
struct UnitVariantAccess {
    variant_index: u32,
}

impl<'de> EnumAccess<'de> for UnitVariantAccess {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: de::DeserializeSeed<'de>,
    {
        let variant = seed.deserialize(de::value::U32Deserializer::new(self.variant_index))?;
        Ok((variant, self))
    }
}

impl<'de> VariantAccess<'de> for UnitVariantAccess {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        cold_path();
        Err(Error::Message("Expected unit variant".into()))
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        cold_path();
        Err(Error::Message("Expected unit variant".into()))
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        cold_path();
        Err(Error::Message("Expected unit variant".into()))
    }
}

// For newtype/tuple/struct enum variants
struct VariantDeserializer<'a, 'de: 'a, O: ByteOrder> {
    deserializer: &'a mut Deserializer<'de, O>,
    variant_name: Cow<'de, str>,
    variant_tag: TagID,
}

impl<'a, 'de, O: ByteOrder> EnumAccess<'de> for VariantDeserializer<'a, 'de, O> {
    type Error = Error;
    type Variant = VariantValueDeserializer<'a, 'de, O>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: de::DeserializeSeed<'de>,
    {
        let variant = match &self.variant_name {
            Cow::Borrowed(s) => seed.deserialize(de::value::BorrowedStrDeserializer::new(s))?,
            Cow::Owned(s) => seed.deserialize(de::value::StrDeserializer::new(s))?,
        };
        Ok((
            variant,
            VariantValueDeserializer {
                deserializer: self.deserializer,
                variant_tag: self.variant_tag,
            },
        ))
    }
}

// Separate struct for variant value deserialization
struct VariantValueDeserializer<'a, 'de: 'a, O: ByteOrder> {
    deserializer: &'a mut Deserializer<'de, O>,
    variant_tag: TagID,
}

impl<'a, 'de, O: ByteOrder> VariantAccess<'de> for VariantValueDeserializer<'a, 'de, O> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        cold_path();
        Err(Error::Message(
            "Expected newtype/tuple/struct variant".into(),
        ))
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        self.deserializer.current_tag = self.variant_tag;
        let value = seed.deserialize(&mut *self.deserializer)?;
        // Consume the outer compound's End tag
        check_bounds!(1, self.deserializer.input);
        if self.deserializer.input[0] != TagID::End as u8 {
            cold_path();
            return Err(Error::InvalidTagType(self.deserializer.input[0]));
        }
        self.deserializer.input = &self.deserializer.input[1..];
        Ok(value)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        // Tuple variant is serialized as a List inside the compound
        check_tag!(TagID::List, self.variant_tag, {
            check_bounds!(1 + 4, self.deserializer.input);
            let tag_id = self.deserializer.input[0];
            if tag_id != TagID::Compound as u8 {
                cold_path();
                return Err(Error::TagMismatch(TagID::Compound as u8, tag_id));
            }
            let length = byteorder::U32::<O>::from_bytes(unsafe {
                *self.deserializer.input.as_ptr().add(1).cast()
            })
            .get();
            self.deserializer.input = &self.deserializer.input[1 + 4..];
            let value = visitor.visit_seq(TupleDeserializer {
                index: 0,
                len: length,
                deserializer: self.deserializer,
            })?;
            // Consume the outer compound's End tag
            check_bounds!(1, self.deserializer.input);
            if self.deserializer.input[0] != TagID::End as u8 {
                cold_path();
                return Err(Error::InvalidTagType(self.deserializer.input[0]));
            }
            self.deserializer.input = &self.deserializer.input[1..];
            Ok(value)
        })
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        // Struct variant is serialized as a Compound inside the compound
        check_tag!(TagID::Compound, self.variant_tag, {
            let value = visitor.visit_map(CompoundAccess {
                deserializer: self.deserializer,
                value_tag: TagID::End,
            })?;
            // Consume the outer compound's End tag
            check_bounds!(1, self.deserializer.input);
            if self.deserializer.input[0] != TagID::End as u8 {
                cold_path();
                return Err(Error::InvalidTagType(self.deserializer.input[0]));
            }
            self.deserializer.input = &self.deserializer.input[1..];
            Ok(value)
        })
    }
}
