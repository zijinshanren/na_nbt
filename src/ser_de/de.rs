use std::{borrow::Cow, marker::PhantomData};

use serde::{
    Deserialize,
    de::{self, EnumAccess, MapAccess, SeqAccess, VariantAccess},
};
use zerocopy::byteorder;

use crate::{ByteOrder, Error, Result, TagID, cold_path};

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
        check_bounds!(1, self.input);
        let value = self.input[0];
        if value != TagID::End as u8 {
            return Err(Error::INVALID(value));
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
        todo!()
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
                Err(Error::INVALID(tag_id))
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
        check_tag!(TagID::Compound, self.current_tag, {
            self.parse_unit()?;
            visitor.visit_unit()
        })
    }

    fn deserialize_newtype_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }
}
