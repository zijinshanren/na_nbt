use std::{io::Write, marker::PhantomData, ptr};

use base64ct::Encoding;
use serde::{Serialize, ser};
use zerocopy::byteorder;

use crate::{ByteOrder, Error, Result, TagID, cold_path, tag_of};

/// Serializes a value to a byte vector in NBT format.
///
/// The output includes the root tag ID byte followed by the serialized data.
/// Use `to_vec_be` or `to_vec_le` for convenience when the byte order is known.
#[inline]
pub fn to_vec<O: ByteOrder>(value: &(impl ?Sized + Serialize)) -> Result<Vec<u8>> {
    let tag_id = tag_of(value);
    let mut serializer = Serializer::<O> {
        vec: vec![tag_id as u8, 0, 0],
        _marker: PhantomData,
    };
    value.serialize(&mut serializer)?;
    Ok(serializer.vec)
}

/// Convenience function for serializing with big-endian byte order.
#[inline]
pub fn to_vec_be(value: &(impl ?Sized + Serialize)) -> Result<Vec<u8>> {
    to_vec::<zerocopy::byteorder::BigEndian>(value)
}

/// Convenience function for serializing with little-endian byte order.
#[inline]
pub fn to_vec_le(value: &(impl ?Sized + Serialize)) -> Result<Vec<u8>> {
    to_vec::<zerocopy::byteorder::LittleEndian>(value)
}

/// Serializes a value to a writer in NBT format.
///
/// The output includes the root tag ID byte followed by the serialized data.
/// Use `to_writer_be` or `to_writer_le` for convenience when the byte order is known.
#[inline]
pub fn to_writer<O: ByteOrder>(
    writer: &mut impl Write,
    value: &(impl ?Sized + Serialize),
) -> Result<()> {
    let vec = to_vec::<O>(value)?;
    writer.write_all(&vec).map_err(Error::IO)
}

/// Convenience function for writing with big-endian byte order.
#[inline]
pub fn to_writer_be(writer: &mut impl Write, value: &(impl ?Sized + Serialize)) -> Result<()> {
    to_writer::<zerocopy::byteorder::BigEndian>(writer, value)
}

/// Convenience function for writing with little-endian byte order.
#[inline]
pub fn to_writer_le(writer: &mut impl Write, value: &(impl ?Sized + Serialize)) -> Result<()> {
    to_writer::<zerocopy::byteorder::LittleEndian>(writer, value)
}

/// NBT serializer that writes to an internal byte vector.
///
/// This type implements `serde::Serializer` for the NBT format.
/// The `O` generic parameter specifies the byte order (BigEndian or LittleEndian).
pub struct Serializer<O: ByteOrder> {
    vec: Vec<u8>,
    _marker: PhantomData<O>,
}

// Writes a length-prefixed MUTF-8 string to the destination pointer.
// The length is written as a u16 in the specified byte order, followed by the encoded bytes.
// Safety: dst must have valid write access for 2 + encoded.len() bytes.
unsafe fn write_string<O: ByteOrder>(dst: *mut u8, encoded: &[u8]) -> Result<()> {
    unsafe {
        let len = encoded.len();
        if len >= u16::MAX as usize {
            cold_path();
            return Err(Error::LEN(len));
        }
        ptr::write(dst.cast(), byteorder::U16::<O>::new(len as u16).to_bytes());
        ptr::copy_nonoverlapping(encoded.as_ptr(), dst.add(2), len);
        Ok(())
    }
}

// Writes a compound tag header: tag ID byte followed by a length-prefixed name string.
// Used at the start of each compound entry.
// Safety: dst must have valid write access for 1 + 2 + encoded.len() bytes.
unsafe fn write_compound_header<O: ByteOrder>(
    dst: *mut u8,
    tag_id: TagID,
    encoded: &[u8],
) -> Result<()> {
    unsafe {
        ptr::write(dst, tag_id as u8);
        write_string::<O>(dst.add(1).cast(), encoded)
    }
}

// Writes a wrapped list header: TAG_Compound byte followed by length as u32.
// Wrapped lists store heterogeneous elements by wrapping each in a Compound with empty name.
// Safety: dst must have valid write access for 1 + 4 bytes.
unsafe fn write_wrapped_list_header<O: ByteOrder>(dst: *mut u8, len: usize) -> Result<()> {
    unsafe {
        if len > u32::MAX as usize {
            cold_path();
            return Err(Error::LEN(len));
        }
        ptr::write(dst, TagID::Compound as u8);
        ptr::write(
            dst.add(1).cast(),
            byteorder::U32::<O>::new(len as u32).to_bytes(),
        );
        Ok(())
    }
}

impl<O: ByteOrder> Serializer<O> {
    // Writes a compound entry: tag ID, name length (u16), name (MUTF-8 encoded), then the value.
    fn write_compound_item<T>(&mut self, name: &str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let tag_id = tag_of(value);
        unsafe {
            let old_len = self.vec.len();
            let encoded = simd_cesu8::mutf8::encode(name);
            let name_len = encoded.len();
            self.vec.reserve(1 + 2 + name_len);
            let write_ptr = self.vec.as_mut_ptr().add(old_len);
            ptr::write(write_ptr, tag_id as u8);
            ptr::write(
                write_ptr.add(1).cast(),
                byteorder::U16::<O>::new(name_len as u16).to_bytes(),
            );
            ptr::copy_nonoverlapping(encoded.as_ptr(), write_ptr.add(3), name_len);
            self.vec.set_len(old_len + 1 + 2 + name_len);
        }
        value.serialize(&mut *self)
    }

    // Writes a wrapped item: Compound { "" : <value> }.
    // This representation allows any NBT type to be stored in a homogeneous list structure.
    // Used by Option, heterogeneous sequences, tuples, and more.
    fn write_wrapped_item<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let tag_id = tag_of(value);
        unsafe {
            let old_len = self.vec.len();
            self.vec.reserve(1 + 2);
            let write_ptr = self.vec.as_mut_ptr().add(old_len);
            ptr::write(write_ptr, tag_id as u8);
            ptr::write(write_ptr.add(1).cast(), [0u8; 2]);
            self.vec.set_len(old_len + 1 + 2);
        }
        value.serialize(&mut *self)?;
        self.vec.push(TagID::End as u8);
        Ok(())
    }

    // Writes a length-prefixed MUTF-8 string to the output buffer.
    // Encodes the string using MUTF-8 (modified UTF-8) as required by NBT.
    fn write_string(&mut self, encoded: &[u8]) -> Result<()> {
        unsafe {
            self.vec.reserve(2 + encoded.len());
            let old_len = self.vec.len();
            let write_ptr = self.vec.as_mut_ptr().add(old_len);
            write_string::<O>(write_ptr, encoded)?;
            self.vec.set_len(old_len + 2 + encoded.len());
            Ok(())
        }
    }

    // Writes a compound tag header to the output buffer.
    // Format: tag ID byte + name length (u16) + name bytes.
    fn write_compound_header(&mut self, tag_id: TagID, encoded: &[u8]) -> Result<()> {
        unsafe {
            self.vec.reserve(1 + 2 + encoded.len());
            let old_len = self.vec.len();
            let write_ptr = self.vec.as_mut_ptr().add(old_len);
            write_compound_header::<O>(write_ptr, tag_id, encoded)?;
            self.vec.set_len(old_len + 1 + 2 + encoded.len());
            Ok(())
        }
    }

    // Writes a wrapped list header to the output buffer.
    // Format: TAG_Compound byte + length (u32).
    // The list elements are stored as Compound { "" : <value> }.
    fn write_wrapped_list_header(&mut self, len: usize) -> Result<()> {
        unsafe {
            let old_len = self.vec.len();
            self.vec.reserve(1 + 4);
            let write_ptr = self.vec.as_mut_ptr().add(old_len);
            write_wrapped_list_header::<O>(write_ptr, len)?;
            self.vec.set_len(old_len + 1 + 4);
            Ok(())
        }
    }
}

impl<'a, O: ByteOrder> ser::Serializer for &'a mut Serializer<O> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = SeqSerializer<'a, O>;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = MapSerializer<'a, O>;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    // Byte
    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        self.serialize_u8(v as u8)
    }

    // Byte
    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        self.vec.push(v as u8);
        Ok(())
    }

    // Short
    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        self.vec
            .extend_from_slice(&byteorder::I16::<O>::new(v).to_bytes());
        Ok(())
    }

    // Int
    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        self.vec
            .extend_from_slice(&byteorder::I32::<O>::new(v).to_bytes());
        Ok(())
    }

    // Long
    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        self.vec
            .extend_from_slice(&byteorder::I64::<O>::new(v).to_bytes());
        Ok(())
    }

    // IntArray[4]
    #[cfg(feature = "i128")]
    #[inline]
    fn serialize_i128(self, v: i128) -> Result<Self::Ok> {
        let v = v as u128;
        let x1: u32 = (v >> 96) as u32;
        let x2: u32 = ((v >> 64) & 0xFFFFFFFF) as u32;
        let x3: u32 = ((v >> 32) & 0xFFFFFFFF) as u32;
        let x4: u32 = (v & 0xFFFFFFFF) as u32;
        unsafe {
            let old_len = self.vec.len();
            self.vec.reserve(4 + 4 * 4);
            let write_ptr = self.vec.as_mut_ptr().add(old_len);
            ptr::write(write_ptr.cast(), byteorder::U32::<O>::new(4).to_bytes());
            ptr::write(
                write_ptr.add(4).cast(),
                byteorder::U32::<O>::new(x1).to_bytes(),
            );
            ptr::write(
                write_ptr.add(8).cast(),
                byteorder::U32::<O>::new(x2).to_bytes(),
            );
            ptr::write(
                write_ptr.add(12).cast(),
                byteorder::U32::<O>::new(x3).to_bytes(),
            );
            ptr::write(
                write_ptr.add(16).cast(),
                byteorder::U32::<O>::new(x4).to_bytes(),
            );
            self.vec.set_len(old_len + 4 + 4 * 4);
        }
        Ok(())
    }

    // Byte
    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        self.serialize_i8(v as i8)
    }

    // Short
    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        self.serialize_i16(v as i16)
    }

    // Int
    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        self.serialize_i32(v as i32)
    }

    // Long
    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        self.serialize_i64(v as i64)
    }

    // IntArray[4]
    #[cfg(feature = "i128")]
    #[inline]
    fn serialize_u128(self, v: u128) -> Result<Self::Ok> {
        self.serialize_i128(v as i128)
    }

    // Float
    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        self.vec
            .extend_from_slice(&byteorder::F32::<O>::new(v).to_bytes());
        Ok(())
    }

    // Double
    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        self.vec
            .extend_from_slice(&byteorder::F64::<O>::new(v).to_bytes());
        Ok(())
    }

    // Int
    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        self.serialize_i32(v as i32)
    }

    // String
    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        let encoded = simd_cesu8::mutf8::encode(v);
        self.write_string(&encoded)
    }

    // ByteArray
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        let len = v.len();
        if len >= u32::MAX as usize {
            cold_path();
            return Err(Error::LEN(len));
        }
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
        Ok(())
    }

    // Compound { }
    fn serialize_none(self) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    // Compound { "" : <value> }
    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        self.write_wrapped_item(value)
    }

    // Compound { }
    fn serialize_unit(self) -> Result<Self::Ok> {
        self.vec.push(TagID::End as u8);
        Ok(())
    }

    // Compound { }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    // Int
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok> {
        self.serialize_u32(variant_index)
    }

    // <value>
    fn serialize_newtype_struct<T>(self, name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        // todo: direct x_array_serializer
        match name {
            "na_nbt:int_array" => {
                value.serialize(ArraySerializer {
                    start_pos: 0,            // unused
                    element_tag: TagID::End, // unused
                    len: None,               // unused
                    size: 4,
                    serializer: &mut *self,
                })
            }
            "na_nbt:long_array" => {
                value.serialize(ArraySerializer {
                    start_pos: 0,            // unused
                    element_tag: TagID::End, // unused
                    len: None,               // unused
                    size: 8,
                    serializer: &mut *self,
                })
            }
            "na_nbt:list" => {
                value.serialize(ArraySerializer {
                    start_pos: self.vec.len(),
                    element_tag: TagID::End,
                    len: Some(0),
                    size: 0, // unused
                    serializer: &mut *self,
                })
            }
            _ => value.serialize(&mut *self),
        }
    }

    // Compound { "<variant>" : <value> }
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        self.write_compound_item(variant, value)?;
        self.vec.push(TagID::End as u8);
        Ok(())
    }

    // List [ Compound { "" : <value> }, ... ]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        if let Some(len) = len
            && len > u32::MAX as usize
        {
            cold_path();
            return Err(Error::LEN(len));
        }
        let list_len = len.map_or(Some(0), |_| None);
        unsafe {
            let old_len = self.vec.len();
            self.vec.reserve(1 + 4);
            let write_ptr = self.vec.as_mut_ptr().add(old_len);
            ptr::write(write_ptr, TagID::Compound as u8);
            ptr::write(
                write_ptr.add(1).cast(),
                byteorder::U32::<O>::new(len.unwrap_or(0) as u32).to_bytes(),
            );
            self.vec.set_len(old_len + 1 + 4);
            Ok(SeqSerializer {
                len_pos: old_len + 1,
                len: list_len,
                serializer: &mut *self,
            })
        }
    }

    // List [ Compound { "" : <value> }, ... ]
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.write_wrapped_list_header(len)?;
        Ok(&mut *self)
    }

    // List [ Compound { "" : <value> }, ... ]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_tuple(len)
    }

    // Compound { "<variant>" : List [ Compound { "" : <value> }, ... ] }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        let encoded = simd_cesu8::mutf8::encode(variant);
        let name_len = encoded.len();
        unsafe {
            let old_len = self.vec.len();
            self.vec.reserve(1 + 2 + name_len + 1 + 4);
            let write_ptr = self.vec.as_mut_ptr().add(old_len);
            write_compound_header::<O>(write_ptr, TagID::List, &encoded)?;
            write_wrapped_list_header::<O>(write_ptr.add(3 + name_len).cast(), len)?;
            self.vec.set_len(old_len + 1 + 2 + name_len + 1 + 4);
        }
        Ok(&mut *self)
    }

    // Compound
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        #[cfg(not(debug_assertions))]
        {
            Ok(MapSerializer {
                tag_pos: usize::MAX,
                serializer: &mut *self,
            })
        }
        #[cfg(debug_assertions)]
        {
            Ok(MapSerializer {
                tag_pos: None,
                serializer: &mut *self,
            })
        }
    }

    // Compound
    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(&mut *self)
    }

    // Compound { "<variant>" : Compound }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        let encoded = simd_cesu8::mutf8::encode(variant);
        self.write_compound_header(TagID::Compound, &encoded)?;
        Ok(&mut *self)
    }
}

// Serializer for native NBT arrays (ByteArray, IntArray, LongArray) or lists.
// Used by the newtype_struct markers to serialize homogeneous sequences.
// The size field indicates bytes per element (1, 4, or 8) or ignored for lists.
struct ArraySerializer<'a, O: ByteOrder> {
    start_pos: usize,
    element_tag: TagID,
    len: Option<u32>,
    size: usize,
    serializer: &'a mut Serializer<O>,
}

macro_rules! impl_unreachable_ser {
    ($($name:ident: $ty:ty),* $(,)?) => {
        $(fn $name(self, _v: $ty) -> Result<Self::Ok> {
            unreachable!()
        }
    )*
    };
}

impl<'a, O: ByteOrder> ser::Serializer for ArraySerializer<'a, O> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeMap = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = ser::Impossible<Self::Ok, Self::Error>;

    impl_unreachable_ser!(
        serialize_bool: bool,
        serialize_i8: i8,
        serialize_i16: i16,
        serialize_i32: i32,
        serialize_i64: i64,
        serialize_u8: u8,
        serialize_u16: u16,
        serialize_u32: u32,
        serialize_u64: u64,
        serialize_f32: f32,
        serialize_f64: f64,
        serialize_char: char,
        serialize_str: &str,
        serialize_bytes: &[u8],
    );

    fn serialize_none(self) -> Result<Self::Ok> {
        unreachable!()
    }

    fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        unreachable!()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        unreachable!()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok> {
        unreachable!()
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, _value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn serialize_seq(mut self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        if len.is_some() {
            self.len = None;
        }
        self.serializer
            .write_wrapped_list_header(len.unwrap_or(0))?;
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        if len > u32::MAX as usize {
            return Err(Error::LEN(len));
        }
        self.serializer.vec.reserve(self.size * len + 4);
        self.serializer
            .vec
            .extend_from_slice(&byteorder::U32::<O>::new(len as u32).to_bytes());
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        unreachable!()
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        unreachable!()
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        unreachable!()
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        unreachable!()
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        unreachable!()
    }
}

impl<'a, O: ByteOrder> ser::SerializeTuple for ArraySerializer<'a, O> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut *self.serializer)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a, O: ByteOrder> ser::SerializeSeq for ArraySerializer<'a, O> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        if let Some(len) = self.len {
            if len == u32::MAX {
                return Err(Error::LEN(len as usize));
            }
            self.len = Some(len + 1);
        }
        let tag_id = tag_of(value);
        if self.element_tag == TagID::End {
            cold_path();
            self.element_tag = tag_id;
        }
        if self.element_tag != tag_id {
            cold_path();
            return Err(Error::MISMATCH {
                expected: self.element_tag,
                actual: tag_id,
            });
        }
        value.serialize(&mut *self.serializer)
    }

    fn end(self) -> Result<Self::Ok> {
        unsafe {
            ptr::write(
                self.serializer.vec.as_mut_ptr().add(self.start_pos).cast(),
                self.element_tag as u8,
            );
            if let Some(len) = self.len {
                ptr::write(
                    self.serializer
                        .vec
                        .as_mut_ptr()
                        .add(self.start_pos + 1)
                        .cast(),
                    byteorder::U32::<O>::new(len).to_bytes(),
                );
            }
        }
        Ok(())
    }
}

// Serializer for heterogeneous sequences.
// Wraps each element in a Compound { "" : <value> } to allow different types.
pub struct SeqSerializer<'a, O: ByteOrder> {
    len_pos: usize,
    len: Option<u32>,
    serializer: &'a mut Serializer<O>,
}

impl<'a, O: ByteOrder> ser::SerializeSeq for SeqSerializer<'a, O> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        if let Some(len) = self.len {
            cold_path();
            if len == u32::MAX {
                cold_path();
                return Err(Error::LEN(len as usize));
            }
            self.len = Some(len + 1);
        }
        self.serializer.write_wrapped_item(value)
    }

    fn end(self) -> Result<Self::Ok> {
        unsafe {
            if let Some(len) = self.len {
                cold_path();
                ptr::write(
                    self.serializer.vec.as_mut_ptr().add(self.len_pos).cast(),
                    byteorder::U32::<O>::new(len).to_bytes(),
                );
            }
            Ok(())
        }
    }
}

impl<O: ByteOrder> ser::SerializeTuple for &mut Serializer<O> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        self.write_wrapped_item(value)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<O: ByteOrder> ser::SerializeTupleStruct for &mut Serializer<O> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        self.write_wrapped_item(value)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<O: ByteOrder> ser::SerializeTupleVariant for &mut Serializer<O> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        self.write_wrapped_item(value)
    }

    fn end(self) -> Result<Self::Ok> {
        self.vec.push(TagID::End as u8);
        Ok(())
    }
}

// Serializer for map keys. NBT requires compound keys to be strings.
// Converts various Rust types to their string representation.
// See tag_probe.rs for the complete type-to-string mapping.
struct KeySerializer<'a, O: ByteOrder> {
    tag_id: TagID,
    serializer: &'a mut Serializer<O>,
}

impl<'a, O: ByteOrder> KeySerializer<'a, O> {
    fn write_wrapped<T>(self, name: &str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let encoded = simd_cesu8::mutf8::encode(name);
        let encoded_len = encoded.len();

        let pos = self.serializer.vec.len();
        value.serialize(KeySerializer {
            tag_id: self.tag_id,
            serializer: &mut *self.serializer,
        })?;

        unsafe {
            let old_len = self.serializer.vec.len();
            self.serializer.vec.reserve(encoded_len + 2); // name()
            let mut write_ptr = self.serializer.vec.as_mut_ptr().add(pos);
            write_ptr = write_ptr.add(1);
            let old_name_len = byteorder::U16::<O>::from_bytes(*write_ptr.cast()).get() as usize;
            let new_name_len = old_name_len + encoded_len + 2; // name()
            if new_name_len >= u16::MAX as usize {
                cold_path();
                return Err(Error::LEN(new_name_len));
            }
            ptr::write(
                write_ptr.cast(),
                byteorder::U16::<O>::new(new_name_len as u16).to_bytes(),
            );
            write_ptr = write_ptr.add(2);
            ptr::copy(write_ptr, write_ptr.add(encoded_len + 1), old_name_len); // move
            ptr::copy_nonoverlapping(encoded.as_ptr(), write_ptr, encoded_len); // name
            write_ptr = write_ptr.add(encoded_len);
            ptr::write(write_ptr.cast(), b'('); // (
            write_ptr = write_ptr.add(1 + old_name_len);
            ptr::write(write_ptr.cast(), b')'); // )
            self.serializer.vec.set_len(old_len + encoded_len + 2);
        }
        Ok(())
    }
}

macro_rules! impl_strlike_key {
    ($($name:ident: $ty:ty),* $(,)?) => {
        $(fn $name(self, v: $ty) -> Result<Self::Ok> {
            self.serialize_str(&v.to_string())
        })*
    }
}

impl<'a, O: ByteOrder> ser::Serializer for KeySerializer<'a, O> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = ser::Impossible<Self::Ok, Error>;
    type SerializeTuple = ser::Impossible<Self::Ok, Error>;
    type SerializeTupleStruct = ser::Impossible<Self::Ok, Error>;
    type SerializeTupleVariant = ser::Impossible<Self::Ok, Error>;
    type SerializeMap = ser::Impossible<Self::Ok, Error>;
    type SerializeStruct = ser::Impossible<Self::Ok, Error>;
    type SerializeStructVariant = ser::Impossible<Self::Ok, Error>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        self.serialize_str(if v { "true" } else { "false" })
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        let encoded = simd_cesu8::mutf8::encode(v);
        self.serializer.write_compound_header(self.tag_id, &encoded)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        self.serialize_str(&base64ct::Base64::encode_string(v))
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        self.serialize_str("None")
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        self.write_wrapped("Some", value)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        self.serialize_str("")
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok> {
        self.serialize_str(name)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok> {
        Err(Error::KEY)
    }

    fn serialize_newtype_struct<T>(self, name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        self.write_wrapped(name, value)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::KEY)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(Error::KEY)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(Error::KEY)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(Error::KEY)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::KEY)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::KEY)
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(Error::KEY)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::KEY)
    }

    impl_strlike_key!(
        serialize_i8: i8,
        serialize_i16: i16,
        serialize_i32: i32,
        serialize_i64: i64,
        serialize_u8: u8,
        serialize_u16: u16,
        serialize_u32: u32,
        serialize_u64: u64,
        serialize_f32: f32,
        serialize_f64: f64,
        serialize_char: char,
    );

    #[cfg(feature = "i128")]
    impl_strlike_key!(
        serialize_i128: i128,
        serialize_u128: u128,
    );
}

// Serializer for maps and structs as NBT compounds.
// Tracks the position of the tag byte for key-value validation.
// Uses debug_assertions in debug mode to ensure correct usage.
pub struct MapSerializer<'a, O: ByteOrder> {
    #[cfg(not(debug_assertions))]
    tag_pos: usize,
    #[cfg(debug_assertions)]
    tag_pos: Option<usize>,
    serializer: &'a mut Serializer<O>,
}

impl<'a, O: ByteOrder> ser::SerializeMap for MapSerializer<'a, O> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        #[cfg(not(debug_assertions))]
        {
            self.tag_pos = self.serializer.vec.len();
        }
        #[cfg(debug_assertions)]
        {
            debug_assert!(
                self.tag_pos.is_none(),
                "serialize_key called without tag_pos consumed"
            );
            self.tag_pos = Some(self.serializer.vec.len());
        }
        key.serialize(KeySerializer {
            tag_id: TagID::End, // placeholder
            serializer: self.serializer,
        })?;
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        #[cfg(debug_assertions)]
        {
            debug_assert!(
                self.tag_pos.is_some(),
                "serialize_value called without serialize_key"
            );
        }
        let tag_id = tag_of(value);
        #[cfg(debug_assertions)]
        {
            unsafe { *self.serializer.vec.get_unchecked_mut(self.tag_pos.unwrap()) = tag_id as u8 };
        }
        #[cfg(not(debug_assertions))]
        {
            unsafe { *self.serializer.vec.get_unchecked_mut(self.tag_pos) = tag_id as u8 };
        }
        value.serialize(&mut *self.serializer)?;
        #[cfg(debug_assertions)]
        {
            self.tag_pos = None;
        }
        Ok(())
    }

    fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> Result<Self::Ok>
    where
        K: ?Sized + Serialize,
        V: ?Sized + Serialize,
    {
        #[cfg(debug_assertions)]
        {
            debug_assert!(self.tag_pos.is_none());
        }
        key.serialize(KeySerializer {
            tag_id: tag_of(value),
            serializer: self.serializer,
        })?;
        value.serialize(&mut *self.serializer)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        self.serializer.vec.push(TagID::End as u8);
        Ok(())
    }
}

impl<O: ByteOrder> ser::SerializeStruct for &mut Serializer<O> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        self.write_compound_item(key, value)
    }

    fn end(self) -> Result<Self::Ok> {
        self.vec.push(TagID::End as u8);
        Ok(())
    }
}

impl<O: ByteOrder> ser::SerializeStructVariant for &mut Serializer<O> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        self.write_compound_item(key, value)
    }

    fn end(self) -> Result<Self::Ok> {
        self.vec
            .extend_from_slice(&[TagID::End as u8, TagID::End as u8]);
        Ok(())
    }
}
