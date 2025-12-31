use std::{
    borrow::Cow,
    hint::{assert_unchecked, unreachable_unchecked},
    io::Read,
    marker::PhantomData,
};

use serde::{
    Deserialize,
    de::{self, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess},
};
use zerocopy::byteorder;

use crate::{ByteOrder, Error, Result, TagID, cold_path};

#[inline]
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
        Err(Error::REMAIN(deserializer.input.len()))
    }
}

#[inline]
pub fn from_slice_be<'de, T>(input: &'de [u8]) -> Result<T>
where
    T: Deserialize<'de>,
{
    from_slice::<zerocopy::byteorder::BigEndian, T>(input)
}

#[inline]
pub fn from_slice_le<'de, T>(input: &'de [u8]) -> Result<T>
where
    T: Deserialize<'de>,
{
    from_slice::<zerocopy::byteorder::LittleEndian, T>(input)
}

#[inline]
pub fn from_reader<O: ByteOrder, T, R: Read>(mut reader: R) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).map_err(Error::IO)?;
    from_slice::<O, T>(&buf)
}

#[inline]
pub fn from_reader_be<T, R: Read>(reader: R) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    from_reader::<zerocopy::byteorder::BigEndian, T, R>(reader)
}

#[inline]
pub fn from_reader_le<T, R: Read>(reader: R) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    from_reader::<zerocopy::byteorder::LittleEndian, T, R>(reader)
}

pub struct Deserializer<'de, O: ByteOrder> {
    current_tag: TagID,
    input: &'de [u8],
    marker: PhantomData<O>,
}

macro_rules! check_bounds {
    ($len:expr, $input:expr) => {
        if $len > $input.len() {
            cold_path();
            return Err(Error::EOF);
        }
    };
}

impl<'de, O: ByteOrder> Deserializer<'de, O> {
    fn from_slice(input: &'de [u8]) -> Result<Self> {
        check_bounds!(1, input);
        let tag_id = TagID::from_u8(input[0])?;
        if tag_id.is_end() {
            cold_path();
            return Err(Error::INVALID(tag_id as u8));
        }
        check_bounds!(1 + 2, input);
        let name_len =
            byteorder::U16::<O>::from_bytes(unsafe { *input.as_ptr().add(1).cast() }).get();
        check_bounds!(1 + 2 + name_len as usize, input);
        Ok(Self {
            current_tag: tag_id,
            input: &input[1 + 2 + name_len as usize..],
            marker: PhantomData,
        })
    }

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
    fn parse_i128(&mut self) -> Result<i128> {
        check_bounds!(4 + 4 * 4, self.input);
        unsafe {
            let read_ptr = self.input.as_ptr().add(4);
            let x1 = byteorder::U32::<O>::from_bytes(*read_ptr.cast()).get();
            let x2 = byteorder::U32::<O>::from_bytes(*read_ptr.add(4).cast()).get();
            let x3 = byteorder::U32::<O>::from_bytes(*read_ptr.add(8).cast()).get();
            let x4 = byteorder::U32::<O>::from_bytes(*read_ptr.add(12).cast()).get();
            self.input = &self.input[4 + 4 * 4..];
            Ok(
                ((x1 as u128) << 96 | (x2 as u128) << 64 | (x3 as u128) << 32 | (x4 as u128))
                    as i128,
            )
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
        self.consume_end_tag()
    }

    fn skip_bytes(&mut self, n: usize) -> Result<()> {
        check_bounds!(n, self.input);
        self.input = &self.input[n..];
        Ok(())
    }

    fn deserialize_wrapped_list<V>(&mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        check_bounds!(1 + 4, self.input);
        let tag_id = self.input[0];
        if tag_id != TagID::Compound as u8 {
            cold_path();
            return Err(Error::INVALID(tag_id));
        }
        let length =
            byteorder::U32::<O>::from_bytes(unsafe { *self.input.as_ptr().add(1).cast() }).get();
        self.input = &self.input[1 + 4..];
        visitor.visit_seq(WrappedListAccess {
            remaining: length,
            deserializer: self,
        })
    }

    fn deserialize_wrapped_item<T>(&mut self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        const FORMAT_ERROR: &str =
            "wrapped item should contain exactly one element with empty name";

        // Compound { "" : <value> }
        check_bounds!(1 + 2, self.input);

        self.current_tag = TagID::from_u8(self.input[0])?;
        if self.current_tag.is_end() {
            cold_path();
            return Err(Error::MSG(FORMAT_ERROR.to_string()));
        }

        let name_len =
            byteorder::U16::<O>::from_bytes(unsafe { *self.input.as_ptr().add(1).cast() }).get();
        if name_len != 0 {
            cold_path();
            return Err(Error::MSG(FORMAT_ERROR.to_string()));
        }

        self.input = &self.input[3..];
        let value = seed.deserialize(&mut *self)?;

        self.consume_end_tag()?;

        Ok(value)
    }

    fn consume_end_tag(&mut self) -> Result<()> {
        check_bounds!(1, self.input);
        if self.input[0] != TagID::End as u8 {
            cold_path();
            return Err(Error::INVALID(self.input[0]));
        }
        self.input = &self.input[1..];
        Ok(())
    }

    fn parse_list_header(&mut self) -> Result<(TagID, u32)> {
        check_bounds!(1 + 4, self.input);
        let tag_id = TagID::from_u8(self.input[0])?;
        let length =
            byteorder::U32::<O>::from_bytes(unsafe { *self.input.as_ptr().add(1).cast() }).get();
        self.input = &self.input[1 + 4..];
        Ok((tag_id, length))
    }

    fn deserialize_array<V>(&mut self, element_tag_id: TagID, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        check_bounds!(4, self.input);
        let length = byteorder::U32::<O>::from_bytes(unsafe { *self.input.as_ptr().cast() }).get();
        self.input = &self.input[4..];
        visitor.visit_seq(ArrayAccess {
            element_tag_id,
            remaining: length,
            deserializer: self,
        })
    }
}

macro_rules! check_tag {
    ($expected:expr, $actual:expr, $ok:block) => {
        if $expected == $actual {
            $ok
        } else {
            cold_path();
            Err(Error::MISMATCH {
                expected: $expected,
                actual: $actual,
            })
        }
    };
}

impl<'de, O: ByteOrder> de::Deserializer<'de> for &mut Deserializer<'de, O> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.current_tag {
            TagID::End => Err(Error::INVALID(TagID::End as u8)),
            TagID::Byte => visitor.visit_i8(self.parse_i8()?),
            TagID::Short => visitor.visit_i16(self.parse_i16()?),
            TagID::Int => visitor.visit_i32(self.parse_i32()?),
            TagID::Long => visitor.visit_i64(self.parse_i64()?),
            TagID::Float => visitor.visit_f32(self.parse_f32()?),
            TagID::Double => visitor.visit_f64(self.parse_f64()?),
            TagID::ByteArray => visitor.visit_borrowed_bytes(self.parse_bytes()?),
            TagID::String => visitor.visit_str(self.parse_str()?.as_ref()),
            TagID::List => {
                let (tag_id, length) = self.parse_list_header()?;
                if tag_id == TagID::Compound {
                    visitor.visit_seq(WrappedListAccess {
                        remaining: length,
                        deserializer: self,
                    })
                } else {
                    visitor.visit_seq(ArrayAccess {
                        element_tag_id: tag_id,
                        remaining: length,
                        deserializer: self,
                    })
                }
            }
            TagID::Compound => visitor.visit_map(CompoundAccess { deserializer: self }),
            TagID::IntArray => self.deserialize_array(TagID::Int, visitor),
            TagID::LongArray => self.deserialize_array(TagID::Long, visitor),
        }
    }

    // Byte
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::Byte, self.current_tag, {
            visitor.visit_bool(self.parse_i8()? != 0)
        })
    }

    // Byte
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::Byte, self.current_tag, {
            visitor.visit_i8(self.parse_i8()?)
        })
    }

    // Short
    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::Short, self.current_tag, {
            visitor.visit_i16(self.parse_i16()?)
        })
    }

    // Int
    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::Int, self.current_tag, {
            visitor.visit_i32(self.parse_i32()?)
        })
    }

    // Long
    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::Long, self.current_tag, {
            visitor.visit_i64(self.parse_i64()?)
        })
    }

    // IntArray[4]
    #[cfg(feature = "i128")]
    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::IntArray, self.current_tag, {
            visitor.visit_i128(self.parse_i128()?)
        })
    }

    // Byte
    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::Byte, self.current_tag, {
            visitor.visit_u8(self.parse_i8()? as u8)
        })
    }

    // Short
    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::Short, self.current_tag, {
            visitor.visit_u16(self.parse_i16()? as u16)
        })
    }

    // Int
    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::Int, self.current_tag, {
            visitor.visit_u32(self.parse_i32()? as u32)
        })
    }

    // Long
    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::Long, self.current_tag, {
            visitor.visit_u64(self.parse_i64()? as u64)
        })
    }

    // IntArray[4]
    #[cfg(feature = "i128")]
    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::IntArray, self.current_tag, {
            visitor.visit_u128(self.parse_i128()? as u128)
        })
    }

    // Float
    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::Float, self.current_tag, {
            visitor.visit_f32(self.parse_f32()?)
        })
    }

    // Double
    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::Double, self.current_tag, {
            visitor.visit_f64(self.parse_f64()?)
        })
    }

    // Int
    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::Int, self.current_tag, {
            let value = self.parse_i32()? as u32;
            visitor.visit_char(char::from_u32(value).ok_or(Error::CHAR(value))?)
        })
    }

    // String
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::String, self.current_tag, {
            visitor.visit_str(self.parse_str()?.as_ref())
        })
    }

    // String
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::String, self.current_tag, {
            visitor.visit_string(self.parse_str()?.into_owned())
        })
    }

    // ByteArray
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::ByteArray, self.current_tag, {
            visitor.visit_bytes(self.parse_bytes()?)
        })
    }

    // ByteArray
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::List, self.current_tag, {
            visitor.visit_byte_buf(self.parse_bytes()?.to_vec())
        })
    }

    // Compound { "" : <value> } => Some(<value>)
    // Compound { } => None
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::Compound, self.current_tag, {
            check_bounds!(1, self.input);
            self.current_tag = TagID::from_u8(self.input[0])?;
            let value = if self.current_tag.is_end() {
                visitor.visit_none()
            } else {
                check_bounds!(1 + 2, self.input);
                self.input = &self.input[1 + 2..];
                visitor.visit_some(&mut *self)
            };
            check_bounds!(1, self.input);
            self.input = &self.input[1..];
            value
        })
    }

    // Compound { }
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::Compound, self.current_tag, {
            self.parse_unit()?;
            visitor.visit_unit()
        })
    }

    // Compound { }
    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match name {
            "na_nbt:list" => {
                check_tag!(TagID::List, self.current_tag, {
                    let (tag_id, length) = self.parse_list_header()?;
                    visitor.visit_seq(ArrayAccess {
                        element_tag_id: tag_id,
                        remaining: length,
                        deserializer: self,
                    })
                })
            }
            _ => visitor.visit_newtype_struct(self),
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.current_tag {
            TagID::IntArray => self.deserialize_array(TagID::Int, visitor),
            TagID::LongArray => self.deserialize_array(TagID::Long, visitor),
            TagID::List => self.deserialize_wrapped_list(visitor),
            _ => {
                cold_path();
                Err(Error::MISMATCH {
                    expected: TagID::List,
                    actual: self.current_tag,
                })
            }
        }
    }

    /// List [ Compound { "" : <value> }, ... ]
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::List, self.current_tag, {
            self.deserialize_wrapped_list(visitor)
        })
    }

    /// List [ Compound { "" : <value> }, ... ]
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::List, self.current_tag, {
            self.deserialize_wrapped_list(visitor)
        })
    }

    /// Compound
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        check_tag!(TagID::Compound, self.current_tag, {
            visitor.visit_map(CompoundAccess { deserializer: self })
        })
    }

    /// Compound
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    /// Int => unit_variant
    /// Compound { "<variant>" : <value> } => newtype_variant
    /// Compound { "<variant>" : Compound } => struct_variant
    /// Compound { "<variant>" : List [ Compound { "" : <value> }, ... ] } => tuple_variant
    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.current_tag {
            TagID::Int => visitor.visit_enum((self.parse_i32()? as u32).into_deserializer()),
            TagID::Compound => visitor.visit_enum(EnumVariantAccess { deserializer: self }),
            _ => {
                cold_path();
                Err(Error::MISMATCH {
                    expected: TagID::Compound,
                    actual: self.current_tag,
                })
            }
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_str(self.parse_str()?.as_ref())
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        const TAG_SIZE: [usize; 13] = [0, 1, 2, 4, 8, 4, 8, 1, 0, 0, 0, 4, 8];

        #[inline(always)]
        const fn tag_size(tag_id: TagID) -> usize {
            let tag_id = tag_id as u8;
            unsafe { assert_unchecked(tag_id < 13) };
            TAG_SIZE[tag_id as usize]
        }

        match self.current_tag {
            p if p.is_primitive() => self.skip_bytes(tag_size(p))?,
            a if a.is_array() => {
                check_bounds!(4, self.input);
                let length =
                    byteorder::U32::<O>::from_bytes(unsafe { *self.input.as_ptr().cast() }).get();
                self.skip_bytes(4 + length as usize * tag_size(a))?;
            }
            TagID::String => {
                check_bounds!(2, self.input);
                let length =
                    byteorder::U16::<O>::from_bytes(unsafe { *self.input.as_ptr().cast() }).get();
                self.skip_bytes(2 + length as usize)?;
            }
            TagID::List => {
                check_bounds!(5, self.input);
                let (element_tag, length) = self.parse_list_header()?;
                if element_tag.is_primitive() {
                    self.skip_bytes(length as usize * tag_size(element_tag))?;
                } else {
                    for _ in 0..length {
                        self.current_tag = element_tag;
                        self.deserialize_ignored_any(serde::de::IgnoredAny)?;
                    }
                }
            }
            TagID::Compound => loop {
                check_bounds!(1, self.input);
                self.current_tag = TagID::from_u8(self.input[0])?;
                self.input = &self.input[1..];
                if self.current_tag.is_end() {
                    break;
                }
                check_bounds!(2, self.input);
                let name_len =
                    byteorder::U16::<O>::from_bytes(unsafe { *self.input.as_ptr().cast() }).get();
                check_bounds!(2 + name_len as usize, self.input);
                self.input = &self.input[2 + name_len as usize..];
                if self.current_tag.is_primitive() {
                    self.skip_bytes(tag_size(self.current_tag))?;
                } else {
                    self.deserialize_ignored_any(serde::de::IgnoredAny)?;
                }
            },
            _ => unsafe { unreachable_unchecked() },
        }
        visitor.visit_unit()
    }
}

macro_rules! impl_key_parse {
    ($($name:ident: $visit:ident, $ty:ty),* $(,)?) => {
        $(fn $name<V>(self, visitor: V) -> Result<V::Value>
        where
            V: de::Visitor<'de>,
        {
            visitor.$visit(self.name.parse::<$ty>().map_err(|e| Error::MSG(e.to_string()))?)
        })*
    }
}

struct KeyDeserializer<'a> {
    name: &'a str,
}

impl<'a, 'de: 'a> de::Deserializer<'de> for KeyDeserializer<'a> {
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
        visitor.visit_str(self.name)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_string(self.name.to_string())
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_bytes(self.name.as_bytes())
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_byte_buf(self.name.as_bytes().to_vec())
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if self.name == "None" {
            visitor.visit_none()
        } else {
            visitor.visit_some(KeyDeserializer {
                name: self
                    .name
                    .strip_prefix("Some(")
                    .and_then(|s| s.strip_suffix(")"))
                    .ok_or(Error::MSG(format!(
                        "expected 'Some(<value>)' or 'None', got '{}'",
                        self.name
                    )))?,
            })
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if self.name.is_empty() {
            visitor.visit_unit()
        } else {
            Err(Error::MSG(format!(
                "expected empty string, got '{}'",
                self.name
            )))
        }
    }

    fn deserialize_unit_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if self.name == name {
            visitor.visit_unit()
        } else {
            Err(Error::MSG(format!(
                "expected '{}', got '{}'",
                name, self.name
            )))
        }
    }

    fn deserialize_newtype_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(KeyDeserializer {
            name: self
                .name
                .strip_prefix(format!("{}(", name).as_str())
                .and_then(|s| s.strip_suffix(")"))
                .ok_or(Error::MSG(format!(
                    "expected '{}(<value>)', got '{}'",
                    name, self.name
                )))?,
        })
    }

    fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::KEY)
    }

    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::KEY)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::KEY)
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::KEY)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::KEY)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::KEY)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_str(self.name)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    impl_key_parse!(
        deserialize_bool: visit_bool, bool,
        deserialize_i8: visit_i8, i8,
        deserialize_i16: visit_i16, i16,
        deserialize_i32: visit_i32, i32,
        deserialize_i64: visit_i64, i64,
        deserialize_u8: visit_u8, u8,
        deserialize_u16: visit_u16, u16,
        deserialize_u32: visit_u32, u32,
        deserialize_u64: visit_u64, u64,
        deserialize_f32: visit_f32, f32,
        deserialize_f64: visit_f64, f64,
        deserialize_char: visit_char, char,
    );

    #[cfg(feature = "i128")]
    impl_key_parse!(
        deserialize_i128: visit_i128, i128,
        deserialize_u128: visit_u128, u128,
    );
}

struct WrappedListAccess<'a, 'de: 'a, O: ByteOrder> {
    remaining: u32,
    deserializer: &'a mut Deserializer<'de, O>,
}

impl<'a, 'de, O: ByteOrder> SeqAccess<'de> for WrappedListAccess<'a, 'de, O> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;

        Ok(Some(self.deserializer.deserialize_wrapped_item(seed)?))
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.remaining as usize)
    }
}

struct ArrayAccess<'a, 'de: 'a, O: ByteOrder> {
    element_tag_id: TagID,
    remaining: u32,
    deserializer: &'a mut Deserializer<'de, O>,
}

impl<'a, 'de, O: ByteOrder> SeqAccess<'de> for ArrayAccess<'a, 'de, O> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;

        self.deserializer.current_tag = self.element_tag_id;
        Ok(Some(seed.deserialize(&mut *self.deserializer)?))
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.remaining as usize)
    }
}

struct CompoundAccess<'a, 'de: 'a, O: ByteOrder> {
    deserializer: &'a mut Deserializer<'de, O>,
}

impl<'a, 'de, O: ByteOrder> MapAccess<'de> for CompoundAccess<'a, 'de, O> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        check_bounds!(1, self.deserializer.input);
        self.deserializer.current_tag = TagID::from_u8(self.deserializer.input[0])?;
        if self.deserializer.current_tag.is_end() {
            self.deserializer.input = &self.deserializer.input[1..];
            return Ok(None);
        }
        check_bounds!(1 + 2, self.deserializer.input);
        let name_len = byteorder::U16::<O>::from_bytes(unsafe {
            *self.deserializer.input.as_ptr().add(1).cast()
        })
        .get();
        check_bounds!(1 + 2 + name_len as usize, self.deserializer.input);
        let name = &self.deserializer.input[1 + 2..1 + 2 + name_len as usize];
        self.deserializer.input = &self.deserializer.input[1 + 2 + name_len as usize..];
        let name = simd_cesu8::decode_lossy(name);
        Ok(Some(seed.deserialize(KeyDeserializer {
            name: name.as_ref(),
        })?))
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.deserializer)
    }
}

struct EnumVariantAccess<'a, 'de: 'a, O: ByteOrder> {
    deserializer: &'a mut Deserializer<'de, O>,
}

impl<'a, 'de, O: ByteOrder> EnumAccess<'de> for EnumVariantAccess<'a, 'de, O> {
    type Error = Error;

    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: de::DeserializeSeed<'de>,
    {
        check_bounds!(1, self.deserializer.input);
        self.deserializer.current_tag = TagID::from_u8(self.deserializer.input[0])?;
        if self.deserializer.current_tag.is_end() {
            cold_path();
            return Err(Error::INVALID(self.deserializer.current_tag as u8));
        }
        self.deserializer.input = &self.deserializer.input[1..];
        let name = self.deserializer.parse_str()?;
        Ok((
            seed.deserialize(KeyDeserializer {
                name: name.as_ref(),
            })?,
            self,
        ))
    }
}

impl<'a, 'de, O: ByteOrder> VariantAccess<'de> for EnumVariantAccess<'a, 'de, O> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Err(Error::MSG(
            "expected newtype/tuple/struct variant, got unit variant".to_string(),
        ))
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        let value = seed.deserialize(&mut *self.deserializer)?;
        self.deserializer.consume_end_tag()?;
        Ok(value)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let value = de::Deserializer::deserialize_tuple(&mut *self.deserializer, len, visitor)?;
        self.deserializer.consume_end_tag()?;
        Ok(value)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let value = de::Deserializer::deserialize_map(&mut *self.deserializer, visitor)?;
        self.deserializer.consume_end_tag()?;
        Ok(value)
    }
}
