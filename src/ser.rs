use std::{io::Write, marker::PhantomData, ptr};

use serde::{Serialize, ser};
use zerocopy::byteorder;

use crate::{ByteOrder, Error, Result, TagID, cold_path};

/// Internal mode for tracking array serialization.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum ArrayMode {
    #[default]
    None,
    IntArray,
    LongArray,
}

/// NBT serializer implementing [`serde::Serializer`].
///
/// This serializer converts Rust types to NBT binary data using serde's
/// serialization framework.
///
/// For most use cases, prefer the convenience functions [`to_vec`], [`to_vec_be`],
/// [`to_writer`], etc. rather than using this type directly.
pub struct Serializer<O: ByteOrder> {
    vec: Vec<u8>,
    marker: PhantomData<O>,
    array_mode: ArrayMode,
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
            return Err(Error::LEN(len));
        }
        unsafe {
            let old_len = self.vec.len();
            self.vec.reserve(5);
            let write_ptr = self.vec.as_mut_ptr().add(old_len);
            ptr::write(write_ptr, TagID::Compound as u8);
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
            if tag_id == TagID::Compound {
                // unwrap the compound
                let current_len = self.vec.len();
                let compound_begin = self.vec.as_mut_ptr().add(old_len + 3);
                ptr::copy(
                    compound_begin,
                    compound_begin.sub(3),
                    current_len - old_len - 3,
                );
                self.vec.set_len(current_len - 3);
            } else {
                *self.vec.get_unchecked_mut(old_len) = tag_id as u8;
                self.vec.push(TagID::End as u8);
            }
        }
        Ok(())
    }
}

/// Serialize a value to NBT binary data.
///
/// This is the main entry point for NBT serialization. The byte order `O`
/// determines whether to write big-endian (Java Edition) or little-endian
/// (Bedrock Edition) data.
///
/// # Example
///
/// ```ignore
/// use na_nbt::ser::to_vec;
/// use zerocopy::byteorder::BigEndian;
///
/// let bytes = to_vec::<BigEndian>(&player)?;
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - A list exceeds 2^32 elements ([`Error::ListTooLong`])
/// - A map has non-string keys ([`Error::KEY`])
/// - List elements have different types ([`Error::TagMismatch`])
///
/// [`Error::ListTooLong`]: crate::Error::ListTooLong
/// [`Error::KEY`]: crate::Error::KEY
/// [`Error::TagMismatch`]: crate::Error::TagMismatch
#[inline]
pub fn to_vec<O: ByteOrder>(value: &(impl ?Sized + Serialize)) -> Result<Vec<u8>> {
    let mut serializer = Serializer::<O> {
        vec: vec![0u8; 3],
        marker: PhantomData,
        array_mode: ArrayMode::None,
    };
    let tag_id = value.serialize(&mut serializer)?;
    if tag_id == TagID::End {
        cold_path();
        return Ok(vec![0]);
    }
    unsafe { *serializer.vec.get_unchecked_mut(0) = tag_id as u8 };
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

/// Serialize a value to an [`std::io::Write`] implementation.
///
/// This serializes the value to an internal buffer first, then writes
/// the entire buffer to the writer.
///
/// # Example
///
/// ```ignore
/// use na_nbt::ser::to_writer;
/// use std::fs::File;
/// use zerocopy::byteorder::BigEndian;
///
/// let mut file = File::create("player.nbt")?;
/// to_writer::<BigEndian>(&mut file, &player)?;
/// ```
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

impl<'a, O: ByteOrder> ser::Serializer for &'a mut Serializer<O> {
    type Ok = TagID;
    type Error = Error;

    type SerializeSeq = SeqSerializer<'a, O>;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = MapSerializer<'a, O>;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    #[inline]
    fn serialize_bool(self, v: bool) -> std::result::Result<Self::Ok, Self::Error> {
        self.vec.push(v as u8);
        Ok(TagID::Byte)
    }

    #[inline]
    fn serialize_i8(self, v: i8) -> std::result::Result<Self::Ok, Self::Error> {
        self.vec.push(v as u8);
        Ok(TagID::Byte)
    }

    #[inline]
    fn serialize_i16(self, v: i16) -> std::result::Result<Self::Ok, Self::Error> {
        self.vec
            .extend_from_slice(&byteorder::I16::<O>::new(v).to_bytes());
        Ok(TagID::Short)
    }

    #[inline]
    fn serialize_i32(self, v: i32) -> std::result::Result<Self::Ok, Self::Error> {
        self.vec
            .extend_from_slice(&byteorder::I32::<O>::new(v).to_bytes());
        Ok(TagID::Int)
    }

    #[inline]
    fn serialize_i64(self, v: i64) -> std::result::Result<Self::Ok, Self::Error> {
        self.vec
            .extend_from_slice(&byteorder::I64::<O>::new(v).to_bytes());
        Ok(TagID::Long)
    }

    #[cfg(feature = "i128")]
    #[inline]
    fn serialize_i128(self, v: i128) -> std::result::Result<Self::Ok, Self::Error> {
        self.serialize_u128(v as u128)
    }

    #[inline]
    fn serialize_u8(self, v: u8) -> std::result::Result<Self::Ok, Self::Error> {
        self.vec.push(v);
        Ok(TagID::Byte)
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

    /// Serialize u128 as IntArray with 4 elements (most significant to least significant)
    #[cfg(feature = "i128")]
    fn serialize_u128(self, v: u128) -> std::result::Result<Self::Ok, Self::Error> {
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
        Ok(TagID::IntArray)
    }

    #[inline]
    fn serialize_f32(self, v: f32) -> std::result::Result<Self::Ok, Self::Error> {
        self.vec
            .extend_from_slice(&byteorder::F32::<O>::new(v).to_bytes());
        Ok(TagID::Float)
    }

    #[inline]
    fn serialize_f64(self, v: f64) -> std::result::Result<Self::Ok, Self::Error> {
        self.vec
            .extend_from_slice(&byteorder::F64::<O>::new(v).to_bytes());
        Ok(TagID::Double)
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
        Ok(TagID::String)
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
        Ok(TagID::ByteArray)
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
        unsafe {
            let old_len = self.vec.len();
            self.vec.reserve(1 + 2);
            let write_ptr = self.vec.as_mut_ptr().add(old_len);
            ptr::write(write_ptr.add(1).cast(), [0u8; 2]);
            self.vec.set_len(old_len + 1 + 2);
            let tag_id = value.serialize(&mut *self)?;
            *self.vec.get_unchecked_mut(old_len) = tag_id as u8;
            self.vec.push(TagID::End as u8);
        }
        Ok(TagID::Compound)
    }

    #[inline]
    fn serialize_unit(self) -> std::result::Result<Self::Ok, Self::Error> {
        self.vec.push(TagID::End as u8);
        Ok(TagID::Compound)
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
        name: &'static str,
        value: &T,
    ) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        match name {
            "na_nbt:int_array" => {
                self.array_mode = ArrayMode::IntArray;
                let result = value.serialize(&mut *self);
                self.array_mode = ArrayMode::None;
                result
            }
            "na_nbt:long_array" => {
                self.array_mode = ArrayMode::LongArray;
                let result = value.serialize(&mut *self);
                self.array_mode = ArrayMode::None;
                result
            }
            _ => value.serialize(self),
        }
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
            self.vec.push(TagID::End as u8);
        }
        Ok(TagID::Compound)
    }

    #[inline]
    fn serialize_seq(
        self,
        len: Option<usize>,
    ) -> std::result::Result<Self::SerializeSeq, Self::Error> {
        let seq_len = match len {
            Some(_) => None,
            None => Some(0),
        };
        let len = len.unwrap_or(0);

        unsafe {
            let old_len = self.vec.len();
            self.write_list_of_compound_begin(len)?;
            Ok(SeqSerializer {
                start_pos: old_len,
                len: seq_len,
                serializer: &mut *self,
            })
        }
    }

    #[inline]
    fn serialize_tuple(self, len: usize) -> std::result::Result<Self::SerializeTuple, Self::Error> {
        match self.array_mode {
            ArrayMode::IntArray => {
                // Write IntArray: length (4 bytes) + data (4 bytes per element)
                unsafe {
                    let old_len = self.vec.len();
                    self.vec.reserve(4 + len * 4);
                    let write_ptr = self.vec.as_mut_ptr().add(old_len);
                    ptr::write(
                        write_ptr.cast(),
                        byteorder::U32::<O>::new(len as u32).to_bytes(),
                    );
                    self.vec.set_len(old_len + 4);
                }
                Ok(self)
            }
            ArrayMode::LongArray => {
                // Write LongArray: length (4 bytes) + data (8 bytes per element)
                unsafe {
                    let old_len = self.vec.len();
                    self.vec.reserve(4 + len * 8);
                    let write_ptr = self.vec.as_mut_ptr().add(old_len);
                    ptr::write(
                        write_ptr.cast(),
                        byteorder::U32::<O>::new(len as u32).to_bytes(),
                    );
                    self.vec.set_len(old_len + 4);
                }
                Ok(self)
            }
            ArrayMode::None => unsafe { self.write_list_of_compound_begin(len) },
        }
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> std::result::Result<Self::SerializeTupleStruct, Self::Error> {
        unsafe { self.write_list_of_compound_begin(len) }
    }

    // Tag::Compound { variant: Tag::List[Tag::Compound] }
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
            ptr::write(write_ptr, TagID::List as u8);
            // name_len
            ptr::write(
                write_ptr.add(1).cast(),
                byteorder::U16::<O>::new(name_len as u16).to_bytes(),
            );
            // name
            ptr::copy_nonoverlapping(encoded.as_ptr(), write_ptr.add(3), name_len);
            // list element tag
            ptr::write(write_ptr.add(3 + name_len), TagID::Compound as u8);
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
            ptr::write(write_ptr, TagID::Compound as u8);
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

pub struct SeqSerializer<'a, O: ByteOrder> {
    start_pos: usize,
    len: Option<u32>,
    serializer: &'a mut Serializer<O>,
}

impl<'a, O: ByteOrder> ser::SerializeSeq for SeqSerializer<'a, O> {
    type Ok = TagID;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        if let Some(ref mut len) = self.len {
            assert!(*len < u32::MAX, "list length too long");
            *len += 1;
        }
        unsafe { self.serializer.write_list_of_compound_item(value) }
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        unsafe {
            if let Some(len) = self.len {
                cold_path();
                ptr::write(
                    self.serializer
                        .vec
                        .as_mut_ptr()
                        .add(self.start_pos + 1)
                        .cast(),
                    byteorder::U32::<O>::new(len).to_bytes(),
                );
            }
            Ok(TagID::List)
        }
    }
}

impl<O: ByteOrder> ser::SerializeSeq for &mut Serializer<O> {
    type Ok = TagID;
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
        Ok(TagID::List)
    }
}

impl<O: ByteOrder> ser::SerializeTuple for &mut Serializer<O> {
    type Ok = TagID;
    type Error = Error;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        match self.array_mode {
            ArrayMode::IntArray | ArrayMode::LongArray => {
                // In array mode, just serialize the raw value (i32 or i64)
                value.serialize(&mut **self)?;
                Ok(())
            }
            ArrayMode::None => unsafe { self.write_list_of_compound_item(value) },
        }
    }

    #[inline]
    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        match self.array_mode {
            ArrayMode::IntArray => Ok(TagID::IntArray),
            ArrayMode::LongArray => Ok(TagID::LongArray),
            ArrayMode::None => Ok(TagID::List),
        }
    }
}

impl<O: ByteOrder> ser::SerializeTupleStruct for &mut Serializer<O> {
    type Ok = TagID;
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
        Ok(TagID::List)
    }
}

impl<O: ByteOrder> ser::SerializeTupleVariant for &mut Serializer<O> {
    type Ok = TagID;
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
        self.vec.push(TagID::End as u8);
        Ok(TagID::Compound)
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
            self.serializer.vec.reserve(1 + 2 + v.len());
            let write_ptr = self.serializer.vec.as_mut_ptr().add(old_len);
            ptr::write(
                write_ptr.add(1).cast(),
                byteorder::U16::<O>::new(v.len() as u16).to_bytes(),
            );
            ptr::copy_nonoverlapping(v.as_ptr(), write_ptr.add(3), v.len());
            self.serializer.vec.set_len(old_len + 1 + 2 + v.len());
        }
        Ok(())
    }

    #[inline]
    fn serialize_bool(self, _v: bool) -> std::result::Result<Self::Ok, Self::Error> {
        cold_path();
        Err(Error::KEY)
    }

    #[inline]
    fn serialize_i8(self, _v: i8) -> std::result::Result<Self::Ok, Self::Error> {
        cold_path();
        Err(Error::KEY)
    }

    #[inline]
    fn serialize_i16(self, _v: i16) -> std::result::Result<Self::Ok, Self::Error> {
        cold_path();
        Err(Error::KEY)
    }

    #[inline]
    fn serialize_i32(self, _v: i32) -> std::result::Result<Self::Ok, Self::Error> {
        cold_path();
        Err(Error::KEY)
    }

    #[inline]
    fn serialize_i64(self, _v: i64) -> std::result::Result<Self::Ok, Self::Error> {
        cold_path();
        Err(Error::KEY)
    }

    #[inline]
    fn serialize_u8(self, _v: u8) -> std::result::Result<Self::Ok, Self::Error> {
        cold_path();
        Err(Error::KEY)
    }

    #[inline]
    fn serialize_u16(self, _v: u16) -> std::result::Result<Self::Ok, Self::Error> {
        cold_path();
        Err(Error::KEY)
    }

    #[inline]
    fn serialize_u32(self, _v: u32) -> std::result::Result<Self::Ok, Self::Error> {
        cold_path();
        Err(Error::KEY)
    }

    #[inline]
    fn serialize_u64(self, _v: u64) -> std::result::Result<Self::Ok, Self::Error> {
        cold_path();
        Err(Error::KEY)
    }

    #[inline]
    fn serialize_f32(self, _v: f32) -> std::result::Result<Self::Ok, Self::Error> {
        cold_path();
        Err(Error::KEY)
    }

    #[inline]
    fn serialize_f64(self, _v: f64) -> std::result::Result<Self::Ok, Self::Error> {
        cold_path();
        Err(Error::KEY)
    }

    #[inline]
    fn serialize_char(self, _v: char) -> std::result::Result<Self::Ok, Self::Error> {
        cold_path();
        Err(Error::KEY)
    }

    #[inline]
    fn serialize_bytes(self, _v: &[u8]) -> std::result::Result<Self::Ok, Self::Error> {
        cold_path();
        Err(Error::KEY)
    }

    #[inline]
    fn serialize_none(self) -> std::result::Result<Self::Ok, Self::Error> {
        cold_path();
        Err(Error::KEY)
    }

    #[inline]
    fn serialize_some<T>(self, _value: &T) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        cold_path();
        Err(Error::KEY)
    }

    #[inline]
    fn serialize_unit(self) -> std::result::Result<Self::Ok, Self::Error> {
        cold_path();
        Err(Error::KEY)
    }

    #[inline]
    fn serialize_unit_struct(
        self,
        _name: &'static str,
    ) -> std::result::Result<Self::Ok, Self::Error> {
        cold_path();
        Err(Error::KEY)
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
        Err(Error::KEY)
    }

    #[inline]
    fn serialize_seq(
        self,
        _len: Option<usize>,
    ) -> std::result::Result<Self::SerializeSeq, Self::Error> {
        cold_path();
        Err(Error::KEY)
    }

    #[inline]
    fn serialize_tuple(
        self,
        _len: usize,
    ) -> std::result::Result<Self::SerializeTuple, Self::Error> {
        cold_path();
        Err(Error::KEY)
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> std::result::Result<Self::SerializeTupleStruct, Self::Error> {
        cold_path();
        Err(Error::KEY)
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
        Err(Error::KEY)
    }

    #[inline]
    fn serialize_map(
        self,
        _len: Option<usize>,
    ) -> std::result::Result<Self::SerializeMap, Self::Error> {
        cold_path();
        Err(Error::KEY)
    }

    #[inline]
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> std::result::Result<Self::SerializeStruct, Self::Error> {
        cold_path();
        Err(Error::KEY)
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
        Err(Error::KEY)
    }
}

pub struct MapSerializer<'a, O: ByteOrder> {
    #[cfg(not(debug_assertions))]
    tag_pos: usize,
    #[cfg(debug_assertions)]
    tag_pos: Option<usize>,
    serializer: &'a mut Serializer<O>,
}

impl<'a, O: ByteOrder> ser::SerializeMap for MapSerializer<'a, O> {
    type Ok = TagID;
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> std::result::Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        #[cfg(not(debug_assertions))]
        {
            self.tag_pos = self.serializer.vec.len();
            key.serialize(MapKeySerializer {
                serializer: self.serializer,
            })?;
        }
        #[cfg(debug_assertions)]
        {
            debug_assert!(
                self.tag_pos.is_none(),
                "serialize_key called without tag_pos consumed"
            );
            self.tag_pos = Some(self.serializer.vec.len());
            key.serialize(MapKeySerializer {
                serializer: self.serializer,
            })?;
        }
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        #[cfg(not(debug_assertions))]
        {
            let tag_id = value.serialize(&mut *self.serializer)?;
            unsafe { *self.serializer.vec.get_unchecked_mut(self.tag_pos) = tag_id as u8 };
            Ok(())
        }
        #[cfg(debug_assertions)]
        {
            debug_assert!(
                self.tag_pos.is_some(),
                "serialize_value called without serialize_key"
            );
            let tag_id = value.serialize(&mut *self.serializer)?;
            unsafe { *self.serializer.vec.get_unchecked_mut(self.tag_pos.unwrap()) = tag_id as u8 };
            self.tag_pos = None;
            Ok(())
        }
    }

    fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> std::result::Result<(), Self::Error>
    where
        K: ?Sized + Serialize,
        V: ?Sized + Serialize,
    {
        #[cfg(debug_assertions)]
        {
            debug_assert!(self.tag_pos.is_none());
        }
        let tag_pos = self.serializer.vec.len();
        key.serialize(MapKeySerializer {
            serializer: self.serializer,
        })?;
        let tag_id = value.serialize(&mut *self.serializer)?;
        unsafe { *self.serializer.vec.get_unchecked_mut(tag_pos) = tag_id as u8 };
        Ok(())
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        self.serializer.vec.push(TagID::End as u8);
        Ok(TagID::Compound)
    }
}

impl<O: ByteOrder> ser::SerializeStruct for &mut Serializer<O> {
    type Ok = TagID;
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
        self.vec.push(TagID::End as u8);
        Ok(TagID::Compound)
    }
}

impl<O: ByteOrder> ser::SerializeStructVariant for &mut Serializer<O> {
    type Ok = TagID;
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
            .extend_from_slice(&[TagID::End as u8, TagID::End as u8]);
        Ok(TagID::Compound)
    }
}
