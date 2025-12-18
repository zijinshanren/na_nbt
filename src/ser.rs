use std::{io::Write, marker::PhantomData, ptr};

use serde::{Serialize, ser};
use zerocopy::byteorder;

use crate::{ByteOrder, Error, Result, Tag, cold_path};

pub struct Serializer<O: ByteOrder> {
    vec: Vec<u8>,
    marker: PhantomData<O>,
}

impl<O: ByteOrder> Serializer<O> {
    unsafe fn write_compound_item<T>(&mut self, name: &str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let old_len = self.vec.len();
        let encoded = simd_cesu8::mutf8::encode(name);
        let name_len = encoded.len();
        self.vec.reserve(1 + 2 + name_len);
        unsafe {
            let write_ptr = self.vec.as_mut_ptr().add(old_len);
            // tag_id placeholder (will be filled later)
            // name_len
            ptr::write(
                write_ptr.add(1).cast(),
                byteorder::U16::<O>::new(name_len as u16).to_bytes(),
            );
            // name
            ptr::copy_nonoverlapping(encoded.as_ptr(), write_ptr.add(1 + 2), name_len);
            self.vec.set_len(old_len + 1 + 2 + name_len);
        }
        let tag_id = value.serialize(&mut *self)?;
        unsafe { *self.vec.get_unchecked_mut(old_len) = tag_id as u8 };
        Ok(())
    }

    // Tag::List[Tag::Compound]
    unsafe fn write_list_of_compound_begin(&mut self, len: usize) -> Result<&mut Self> {
        if len > u32::MAX as usize {
            cold_path();
            return Err(Error::ListTooLong(len));
        }
        unsafe {
            let old_len = self.vec.len();
            self.vec.reserve(5);
            let write_ptr = self.vec.as_mut_ptr().add(old_len);
            ptr::write(write_ptr, Tag::Compound as u8);
            ptr::write(
                write_ptr.add(1).cast(),
                byteorder::U32::<O>::new(len as u32).to_bytes(),
            );
            self.vec.set_len(old_len + 5);
        }
        Ok(&mut *self)
    }

    // Tag::Compound { "" : value }
    unsafe fn write_list_of_compound_item<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unsafe {
            let old_len = self.vec.len();
            self.vec.reserve(3);
            let write_ptr = self.vec.as_mut_ptr().add(old_len);
            ptr::write(write_ptr.add(1).cast(), [0u8; 2]);
            self.vec.set_len(old_len + 3);
            let tag_id = value.serialize(&mut *self)?;
            *self.vec.get_unchecked_mut(old_len) = tag_id as u8;
            self.vec.push(Tag::End as u8);
        }
        Ok(())
    }
}

#[inline]
pub fn to_vec<O: ByteOrder>(value: &(impl ?Sized + Serialize)) -> Result<Vec<u8>> {
    let mut serializer = Serializer::<O> {
        vec: vec![0u8; 3],
        marker: PhantomData,
    };
    let tag_id = value.serialize(&mut serializer)?;
    if tag_id == Tag::End {
        cold_path();
        return Ok(vec![0]);
    }
    unsafe { *serializer.vec.get_unchecked_mut(0) = tag_id as u8 };
    Ok(serializer.vec)
}

#[inline]
pub fn to_writer<O: ByteOrder>(
    writer: &mut impl Write,
    value: &(impl ?Sized + Serialize),
) -> Result<()> {
    let vec = to_vec::<O>(value)?;
    writer.write_all(&vec).map_err(Error::IO)
}

impl<O: ByteOrder> ser::Serializer for &mut Serializer<O> {
    type Ok = Tag;
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    #[inline]
    fn serialize_bool(self, v: bool) -> std::result::Result<Self::Ok, Self::Error> {
        self.vec.push(v as u8);
        Ok(Tag::Byte)
    }

    #[inline]
    fn serialize_i8(self, v: i8) -> std::result::Result<Self::Ok, Self::Error> {
        self.vec.push(v as u8);
        Ok(Tag::Byte)
    }

    #[inline]
    fn serialize_i16(self, v: i16) -> std::result::Result<Self::Ok, Self::Error> {
        self.vec
            .extend_from_slice(&byteorder::I16::<O>::new(v).to_bytes());
        Ok(Tag::Short)
    }

    #[inline]
    fn serialize_i32(self, v: i32) -> std::result::Result<Self::Ok, Self::Error> {
        self.vec
            .extend_from_slice(&byteorder::I32::<O>::new(v).to_bytes());
        Ok(Tag::Int)
    }

    #[inline]
    fn serialize_i64(self, v: i64) -> std::result::Result<Self::Ok, Self::Error> {
        self.vec
            .extend_from_slice(&byteorder::I64::<O>::new(v).to_bytes());
        Ok(Tag::Long)
    }

    #[inline]
    fn serialize_u8(self, v: u8) -> std::result::Result<Self::Ok, Self::Error> {
        self.vec.push(v);
        Ok(Tag::Byte)
    }

    #[inline]
    fn serialize_u16(self, v: u16) -> std::result::Result<Self::Ok, Self::Error> {
        self.serialize_i16(v as i16)
    }

    #[inline]
    fn serialize_u32(self, v: u32) -> std::result::Result<Self::Ok, Self::Error> {
        self.serialize_i32(v as i32)
    }

    #[inline]
    fn serialize_u64(self, v: u64) -> std::result::Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    #[inline]
    fn serialize_f32(self, v: f32) -> std::result::Result<Self::Ok, Self::Error> {
        self.vec
            .extend_from_slice(&byteorder::F32::<O>::new(v).to_bytes());
        Ok(Tag::Float)
    }

    #[inline]
    fn serialize_f64(self, v: f64) -> std::result::Result<Self::Ok, Self::Error> {
        self.vec
            .extend_from_slice(&byteorder::F64::<O>::new(v).to_bytes());
        Ok(Tag::Double)
    }

    #[inline]
    fn serialize_char(self, v: char) -> std::result::Result<Self::Ok, Self::Error> {
        self.serialize_u32(v as u32)
    }

    #[inline]
    fn serialize_str(self, v: &str) -> std::result::Result<Self::Ok, Self::Error> {
        let encoded = simd_cesu8::mutf8::encode(v);
        let len = encoded.len();
        unsafe {
            let old_len = self.vec.len();
            self.vec.reserve(2 + len);
            let write_ptr = self.vec.as_mut_ptr().add(old_len);
            ptr::write(
                write_ptr.cast(),
                byteorder::U16::<O>::new(len as u16).to_bytes(),
            );
            ptr::copy_nonoverlapping(encoded.as_ptr(), write_ptr.add(2), len);
            self.vec.set_len(old_len + 2 + len);
        }
        Ok(Tag::String)
    }

    #[inline]
    fn serialize_bytes(self, v: &[u8]) -> std::result::Result<Self::Ok, Self::Error> {
        let len = v.len();
        unsafe {
            let old_len = self.vec.len();
            self.vec.reserve(4 + len);
            let write_ptr = self.vec.as_mut_ptr().add(old_len);
            ptr::write(
                write_ptr.cast(),
                byteorder::U32::<O>::new(len as u32).to_bytes(),
            );
            ptr::copy_nonoverlapping(v.as_ptr(), write_ptr.add(4), len);
            self.vec.set_len(old_len + 4 + len);
        }
        Ok(Tag::ByteArray)
    }

    #[inline]
    fn serialize_none(self) -> std::result::Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_unit(self) -> std::result::Result<Self::Ok, Self::Error> {
        self.vec.push(Tag::End as u8);
        Ok(Tag::Compound)
    }

    #[inline]
    fn serialize_unit_struct(
        self,
        _name: &'static str,
    ) -> std::result::Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> std::result::Result<Self::Ok, Self::Error> {
        self.serialize_u32(variant_index)
    }

    #[inline]
    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unsafe {
            self.write_compound_item(variant, value)?;
            self.vec.push(Tag::End as u8);
        }
        Ok(Tag::Compound)
    }

    #[inline]
    fn serialize_seq(
        self,
        len: Option<usize>,
    ) -> std::result::Result<Self::SerializeSeq, Self::Error> {
        if let Some(len) = len {
            unsafe { self.write_list_of_compound_begin(len) }
        } else {
            cold_path();
            Err(Error::ListLengthUnknown)
        }
    }

    #[inline]
    fn serialize_tuple(self, len: usize) -> std::result::Result<Self::SerializeTuple, Self::Error> {
        unsafe { self.write_list_of_compound_begin(len) }
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> std::result::Result<Self::SerializeTupleStruct, Self::Error> {
        unsafe { self.write_list_of_compound_begin(len) }
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> std::result::Result<Self::SerializeTupleVariant, Self::Error> {
        let encoded = simd_cesu8::mutf8::encode(variant);
        let name_len = encoded.len();
        unsafe {
            let old_len = self.vec.len();
            self.vec.reserve(1 + 2 + name_len + 1 + 4);
            let write_ptr = self.vec.as_mut_ptr().add(old_len);
            // tag_id list
            ptr::write(write_ptr, Tag::List as u8);
            // name_len
            ptr::write(
                write_ptr.add(1).cast(),
                byteorder::U16::<O>::new(name_len as u16).to_bytes(),
            );
            // name
            ptr::copy_nonoverlapping(encoded.as_ptr(), write_ptr.add(3), name_len);
            // list element tag
            ptr::write(write_ptr.add(3 + name_len), Tag::Compound as u8);
            // list length
            ptr::write(
                write_ptr.add(3 + name_len + 1).cast(),
                byteorder::U32::<O>::new(len as u32).to_bytes(),
            );
            self.vec.set_len(old_len + 1 + 2 + name_len + 1 + 4);
        }
        Ok(self)
    }

    #[inline]
    fn serialize_map(
        self,
        _len: Option<usize>,
    ) -> std::result::Result<Self::SerializeMap, Self::Error> {
        Ok(self)
    }

    #[inline]
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> std::result::Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> std::result::Result<Self::SerializeStructVariant, Self::Error> {
        let encoded = simd_cesu8::mutf8::encode(variant);
        let name_len = encoded.len();
        unsafe {
            let old_len = self.vec.len();
            self.vec.reserve(1 + 2 + name_len);
            let write_ptr = self.vec.as_mut_ptr().add(old_len);
            // tag_id compound
            ptr::write(write_ptr, Tag::Compound as u8);
            // name_len
            ptr::write(
                write_ptr.add(1).cast(),
                byteorder::U16::<O>::new(name_len as u16).to_bytes(),
            );
            // name
            ptr::copy_nonoverlapping(encoded.as_ptr(), write_ptr.add(3), name_len);
            self.vec.set_len(old_len + 1 + 2 + name_len);
        }
        Ok(self)
    }
}

impl<O: ByteOrder> ser::SerializeSeq for &mut Serializer<O> {
    type Ok = Tag;
    type Error = Error;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unsafe { self.write_list_of_compound_item(value) }
    }

    #[inline]
    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(Tag::List)
    }
}

impl<O: ByteOrder> ser::SerializeTuple for &mut Serializer<O> {
    type Ok = Tag;
    type Error = Error;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unsafe { self.write_list_of_compound_item(value) }
    }

    #[inline]
    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(Tag::List)
    }
}

impl<O: ByteOrder> ser::SerializeTupleStruct for &mut Serializer<O> {
    type Ok = Tag;
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unsafe { self.write_list_of_compound_item(value) }
    }

    #[inline]
    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(Tag::List)
    }
}

impl<O: ByteOrder> ser::SerializeTupleVariant for &mut Serializer<O> {
    type Ok = Tag;
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unsafe { self.write_list_of_compound_item(value) }
    }

    #[inline]
    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        self.vec.push(Tag::End as u8);
        Ok(Tag::List)
    }
}

struct MapKeySerializer<'a, O: ByteOrder> {
    serializer: &'a mut Serializer<O>,
}

impl<'a, O: ByteOrder> ser::Serializer for MapKeySerializer<'a, O> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = ser::Impossible<(), Error>;
    type SerializeTuple = ser::Impossible<(), Error>;
    type SerializeTupleStruct = ser::Impossible<(), Error>;
    type SerializeTupleVariant = ser::Impossible<(), Error>;
    type SerializeMap = ser::Impossible<(), Error>;
    type SerializeStruct = ser::Impossible<(), Error>;
    type SerializeStructVariant = ser::Impossible<(), Error>;

    #[inline]
    fn serialize_str(self, v: &str) -> std::result::Result<Self::Ok, Self::Error> {
        unsafe {
            let old_len = self.serializer.vec.len();
            self.serializer.vec.reserve(2 + v.len());
            let write_ptr = self.serializer.vec.as_mut_ptr().add(old_len);
            ptr::write(
                write_ptr.cast(),
                byteorder::U16::<O>::new(v.len() as u16).to_bytes(),
            );
            ptr::copy_nonoverlapping(v.as_ptr(), write_ptr.add(2), v.len());
            self.serializer.vec.set_len(old_len + 2 + v.len());
        }
        Ok(())
    }

    #[inline]
    fn serialize_bool(self, _v: bool) -> std::result::Result<Self::Ok, Self::Error> {
        cold_path();
        Err(Error::KeyMustBeString)
    }

    #[inline]
    fn serialize_i8(self, _v: i8) -> std::result::Result<Self::Ok, Self::Error> {
        cold_path();
        Err(Error::KeyMustBeString)
    }

    #[inline]
    fn serialize_i16(self, _v: i16) -> std::result::Result<Self::Ok, Self::Error> {
        cold_path();
        Err(Error::KeyMustBeString)
    }

    #[inline]
    fn serialize_i32(self, _v: i32) -> std::result::Result<Self::Ok, Self::Error> {
        cold_path();
        Err(Error::KeyMustBeString)
    }

    #[inline]
    fn serialize_i64(self, _v: i64) -> std::result::Result<Self::Ok, Self::Error> {
        cold_path();
        Err(Error::KeyMustBeString)
    }

    #[inline]
    fn serialize_u8(self, _v: u8) -> std::result::Result<Self::Ok, Self::Error> {
        cold_path();
        Err(Error::KeyMustBeString)
    }

    #[inline]
    fn serialize_u16(self, _v: u16) -> std::result::Result<Self::Ok, Self::Error> {
        cold_path();
        Err(Error::KeyMustBeString)
    }

    #[inline]
    fn serialize_u32(self, _v: u32) -> std::result::Result<Self::Ok, Self::Error> {
        cold_path();
        Err(Error::KeyMustBeString)
    }

    #[inline]
    fn serialize_u64(self, _v: u64) -> std::result::Result<Self::Ok, Self::Error> {
        cold_path();
        Err(Error::KeyMustBeString)
    }

    #[inline]
    fn serialize_f32(self, _v: f32) -> std::result::Result<Self::Ok, Self::Error> {
        cold_path();
        Err(Error::KeyMustBeString)
    }

    #[inline]
    fn serialize_f64(self, _v: f64) -> std::result::Result<Self::Ok, Self::Error> {
        cold_path();
        Err(Error::KeyMustBeString)
    }

    #[inline]
    fn serialize_char(self, _v: char) -> std::result::Result<Self::Ok, Self::Error> {
        cold_path();
        Err(Error::KeyMustBeString)
    }

    #[inline]
    fn serialize_bytes(self, _v: &[u8]) -> std::result::Result<Self::Ok, Self::Error> {
        cold_path();
        Err(Error::KeyMustBeString)
    }

    #[inline]
    fn serialize_none(self) -> std::result::Result<Self::Ok, Self::Error> {
        cold_path();
        Err(Error::KeyMustBeString)
    }

    #[inline]
    fn serialize_some<T>(self, _value: &T) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        cold_path();
        Err(Error::KeyMustBeString)
    }

    #[inline]
    fn serialize_unit(self) -> std::result::Result<Self::Ok, Self::Error> {
        cold_path();
        Err(Error::KeyMustBeString)
    }

    #[inline]
    fn serialize_unit_struct(
        self,
        _name: &'static str,
    ) -> std::result::Result<Self::Ok, Self::Error> {
        cold_path();
        Err(Error::KeyMustBeString)
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> std::result::Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    #[inline]
    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        cold_path();
        Err(Error::KeyMustBeString)
    }

    #[inline]
    fn serialize_seq(
        self,
        _len: Option<usize>,
    ) -> std::result::Result<Self::SerializeSeq, Self::Error> {
        cold_path();
        Err(Error::KeyMustBeString)
    }

    #[inline]
    fn serialize_tuple(
        self,
        _len: usize,
    ) -> std::result::Result<Self::SerializeTuple, Self::Error> {
        cold_path();
        Err(Error::KeyMustBeString)
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> std::result::Result<Self::SerializeTupleStruct, Self::Error> {
        cold_path();
        Err(Error::KeyMustBeString)
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> std::result::Result<Self::SerializeTupleVariant, Self::Error> {
        cold_path();
        Err(Error::KeyMustBeString)
    }

    #[inline]
    fn serialize_map(
        self,
        _len: Option<usize>,
    ) -> std::result::Result<Self::SerializeMap, Self::Error> {
        cold_path();
        Err(Error::KeyMustBeString)
    }

    #[inline]
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> std::result::Result<Self::SerializeStruct, Self::Error> {
        cold_path();
        Err(Error::KeyMustBeString)
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> std::result::Result<Self::SerializeStructVariant, Self::Error> {
        cold_path();
        Err(Error::KeyMustBeString)
    }
}

impl<O: ByteOrder> ser::SerializeMap for &mut Serializer<O> {
    type Ok = Tag;
    type Error = Error;

    #[inline]
    fn serialize_key<T>(&mut self, _key: &T) -> std::result::Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!("use serialize_entry")
    }

    #[inline]
    fn serialize_value<T>(&mut self, _value: &T) -> std::result::Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!("use serialize_entry")
    }

    #[inline]
    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        self.vec.push(Tag::End as u8);
        Ok(Tag::Compound)
    }

    #[inline]
    fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> std::result::Result<(), Self::Error>
    where
        K: ?Sized + Serialize,
        V: ?Sized + Serialize,
    {
        let tag_pos = self.vec.len();
        key.serialize(MapKeySerializer { serializer: self })?;
        let tag_id = value.serialize(&mut **self)?;
        unsafe { *self.vec.get_unchecked_mut(tag_pos) = tag_id as u8 };
        Ok(())
    }
}

impl<O: ByteOrder> ser::SerializeStruct for &mut Serializer<O> {
    type Ok = Tag;
    type Error = Error;

    #[inline]
    fn serialize_field<T>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> std::result::Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unsafe { self.write_compound_item(key, value) }
    }

    #[inline]
    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        self.vec.push(Tag::End as u8);
        Ok(Tag::Compound)
    }
}

impl<O: ByteOrder> ser::SerializeStructVariant for &mut Serializer<O> {
    type Ok = Tag;
    type Error = Error;

    #[inline]
    fn serialize_field<T>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> std::result::Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unsafe { self.write_compound_item(key, value) }
    }

    #[inline]
    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        self.vec
            .extend_from_slice(&[Tag::End as u8, Tag::End as u8]);
        Ok(Tag::Compound)
    }
}
