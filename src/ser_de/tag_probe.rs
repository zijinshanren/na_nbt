use serde::{Serialize, ser};

use crate::{Error, Result, TagID};

pub struct TagProbe;

impl ser::Serializer for TagProbe {
    type Ok = TagID;
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        self.serialize_i8(v as i8)
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok> {
        Ok(TagID::Byte)
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok> {
        Ok(TagID::Short)
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok> {
        Ok(TagID::Int)
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok> {
        Ok(TagID::Long)
    }

    #[cfg(feature = "i128")]
    fn serialize_i128(self, _v: i128) -> Result<Self::Ok> {
        Ok(TagID::IntArray)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        self.serialize_i8(v as i8)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        self.serialize_i16(v as i16)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        self.serialize_i32(v as i32)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        self.serialize_i64(v as i64)
    }

    #[cfg(feature = "i128")]
    fn serialize_u128(self, v: u128) -> Result<Self::Ok> {
        self.serialize_i128(v as i128)
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok> {
        Ok(TagID::Float)
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok> {
        Ok(TagID::Double)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        self.serialize_i32(v as i32)
    }

    fn serialize_str(self, _v: &str) -> Result<Self::Ok> {
        Ok(TagID::String)
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok> {
        Ok(TagID::ByteArray)
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_some<T: ?Sized + Serialize>(self, _v: &T) -> Result<Self::Ok> {
        Ok(TagID::Compound)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Ok(TagID::Compound)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        idx: u32,
        _var: &'static str,
    ) -> Result<Self::Ok> {
        self.serialize_u32(idx)
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok> {
        match name {
            "na_nbt:byte_array" => Ok(TagID::ByteArray),
            "na_nbt:int_array" => Ok(TagID::IntArray),
            "na_nbt:long_array" => Ok(TagID::LongArray),
            "na_nbt:list" => Ok(TagID::List),
            _ => value.serialize(self),
        }
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _idx: u32,
        _var: &'static str,
        _value: &T,
    ) -> Result<Self::Ok> {
        Ok(TagID::Compound)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _idx: u32,
        _var: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(self)
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _idx: u32,
        _var: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Ok(self)
    }
}

impl ser::SerializeSeq for TagProbe {
    type Ok = TagID;
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<()> {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(TagID::List)
    }
}

impl ser::SerializeTuple for TagProbe {
    type Ok = TagID;
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<()> {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(TagID::List)
    }
}

impl ser::SerializeTupleStruct for TagProbe {
    type Ok = TagID;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<()> {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(TagID::List)
    }
}

impl ser::SerializeTupleVariant for TagProbe {
    type Ok = TagID;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<()> {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(TagID::Compound)
    }
}

impl ser::SerializeMap for TagProbe {
    type Ok = TagID;
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, _key: &T) -> Result<()> {
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<()> {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(TagID::Compound)
    }
}

impl ser::SerializeStruct for TagProbe {
    type Ok = TagID;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        _key: &'static str,
        _value: &T,
    ) -> Result<()> {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(TagID::Compound)
    }
}

impl ser::SerializeStructVariant for TagProbe {
    type Ok = TagID;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        _key: &'static str,
        _value: &T,
    ) -> Result<()> {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(TagID::Compound)
    }
}

#[inline]
pub fn tag_of<T: ?Sized + Serialize>(value: &T) -> TagID {
    unsafe { value.serialize(TagProbe).unwrap_unchecked() }
}
