use std::{any::TypeId, io::Write, ptr, sync::Arc};

use bytes::Bytes;
use zerocopy::{IntoBytes, byteorder};

use crate::{ByteOrder, Error, Result, Tag, cold_path};

mod mark;
mod read;
mod trait_impl;
mod util;
mod value;
mod write;

pub type BorrowedValue<'s, O> = value::ImmutableValue<'s, O, ()>;

pub fn read_borrowed<'s, O: ByteOrder>(source: &'s [u8]) -> Result<BorrowedDocument<'s, O>> {
    unsafe {
        read::read_unsafe::<O, _>(source.as_ptr(), source.len(), |mark| BorrowedDocument {
            mark,
            source: source.as_ptr(),
            _marker: core::marker::PhantomData::<(&'s (), O)>,
        })
    }
}

pub struct BorrowedDocument<'s, O: ByteOrder> {
    mark: Vec<mark::Mark>,
    source: *const u8,
    _marker: core::marker::PhantomData<(&'s (), O)>,
}

impl<'s, O: ByteOrder> BorrowedDocument<'s, O> {
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

pub type SharedValue<O> = value::ImmutableValue<'static, O, Arc<SharedDocument>>;

pub fn read_shared<O: ByteOrder>(source: Bytes) -> Result<SharedValue<O>> {
    Ok(unsafe {
        read::read_unsafe::<O, _>(source.as_ptr(), source.len(), |mark| {
            Arc::new(SharedDocument { mark, source })
        })?
        .root()
    })
}

pub struct SharedDocument {
    mark: Vec<mark::Mark>,
    source: Bytes,
}

impl SharedDocument {
    #[inline]
    pub fn root<O: ByteOrder>(self: Arc<Self>) -> SharedValue<O> {
        let root_tag = unsafe { Tag::from_u8_unchecked(*self.source.get_unchecked(0)) };

        if root_tag == Tag::End {
            cold_path();
            return SharedValue::End;
        }

        let name_len =
            byteorder::U16::<O>::from_bytes(unsafe { *self.source.as_ptr().add(1).cast() }).get();

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

pub fn write_value_to_vec<'s, D: value::Document, SOURCE: ByteOrder, TARGET: ByteOrder>(
    value: &value::ImmutableValue<'s, SOURCE, D>,
) -> Result<Vec<u8>> {
    unsafe {
        match value {
            value::ImmutableValue::End => Ok(vec![0]),
            value::ImmutableValue::Byte(value) => {
                let mut buf = Vec::<u8>::with_capacity(4);
                let buf_ptr = buf.as_mut_ptr();
                ptr::write(buf_ptr.cast(), [Tag::Byte as u8, 0u8, 0u8, *value as u8]);
                buf.set_len(4);
                Ok(buf)
            }
            value::ImmutableValue::Short(value) => {
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
            value::ImmutableValue::Int(value) => {
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
            value::ImmutableValue::Long(value) => {
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
            value::ImmutableValue::Float(value) => {
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
            value::ImmutableValue::Double(value) => {
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
            value::ImmutableValue::ByteArray(value) => {
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
            value::ImmutableValue::String(value) => {
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
            value::ImmutableValue::List(value) => {
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
            value::ImmutableValue::Compound(value) => {
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
            value::ImmutableValue::IntArray(value) => {
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
            value::ImmutableValue::LongArray(value) => {
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

pub fn write_value_to_writer<
    's,
    D: value::Document,
    SOURCE: ByteOrder,
    TARGET: ByteOrder,
    W: Write,
>(
    mut writer: W,
    value: &value::ImmutableValue<'s, SOURCE, D>,
) -> Result<()> {
    unsafe {
        match value {
            value::ImmutableValue::End => writer.write_all(&[0]).map_err(Error::IO),
            value::ImmutableValue::Byte(value) => writer
                .write_all(&[Tag::Byte as u8, 0u8, 0u8, *value as u8])
                .map_err(Error::IO),
            value::ImmutableValue::Short(value) => {
                let mut buf = [0u8; 1 + 2 + 2];
                ptr::write(buf.as_mut_ptr().cast(), [Tag::Short as u8, 0u8, 0u8]);
                ptr::write(
                    buf.as_mut_ptr().add(3).cast(),
                    byteorder::I16::<TARGET>::new(*value).to_bytes(),
                );
                writer.write_all(&buf).map_err(Error::IO)
            }
            value::ImmutableValue::Int(value) => {
                let mut buf = [0u8; 1 + 2 + 4];
                ptr::write(buf.as_mut_ptr().cast(), [Tag::Int as u8, 0u8, 0u8]);
                ptr::write(
                    buf.as_mut_ptr().add(3).cast(),
                    byteorder::I32::<TARGET>::new(*value).to_bytes(),
                );
                writer.write_all(&buf).map_err(Error::IO)
            }
            value::ImmutableValue::Long(value) => {
                let mut buf = [0u8; 1 + 2 + 8];
                ptr::write(buf.as_mut_ptr().cast(), [Tag::Long as u8, 0u8, 0u8]);
                ptr::write(
                    buf.as_mut_ptr().add(3).cast(),
                    byteorder::I64::<TARGET>::new(*value).to_bytes(),
                );
                writer.write_all(&buf).map_err(Error::IO)
            }
            value::ImmutableValue::Float(value) => {
                let mut buf = [0u8; 1 + 2 + 4];
                ptr::write(buf.as_mut_ptr().cast(), [Tag::Float as u8, 0u8, 0u8]);
                ptr::write(
                    buf.as_mut_ptr().add(3).cast(),
                    byteorder::F32::<TARGET>::new(*value).to_bytes(),
                );
                writer.write_all(&buf).map_err(Error::IO)
            }
            value::ImmutableValue::Double(value) => {
                let mut buf = [0u8; 1 + 2 + 8];
                ptr::write(buf.as_mut_ptr().cast(), [Tag::Double as u8, 0u8, 0u8]);
                ptr::write(
                    buf.as_mut_ptr().add(3).cast(),
                    byteorder::F64::<TARGET>::new(*value).to_bytes(),
                );
                writer.write_all(&buf).map_err(Error::IO)
            }
            value::ImmutableValue::ByteArray(value) => {
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
            value::ImmutableValue::String(value) => {
                let mut buf_head = [0u8; 1 + 2 + 2];
                ptr::write(buf_head.as_mut_ptr().cast(), [Tag::String as u8, 0u8, 0u8]);
                ptr::write(
                    buf_head.as_mut_ptr().add(3).cast(),
                    byteorder::U16::<TARGET>::new(value.data.len() as u16).to_bytes(),
                );
                writer.write_all(&buf_head).map_err(Error::IO)?;
                writer.write_all(value.data.as_bytes()).map_err(Error::IO)
            }
            value::ImmutableValue::List(value) => {
                writer
                    .write_all(&[Tag::List as u8, 0u8, 0u8])
                    .map_err(Error::IO)?;
                if TypeId::of::<SOURCE>() == TypeId::of::<TARGET>() {
                    writer.write_all(value.data.as_bytes()).map_err(Error::IO)
                } else {
                    let size_written = write::write_list_to_writer_fallback::<SOURCE, TARGET, W>(
                        value.data.as_ptr(),
                        &mut writer,
                    )?;
                    debug_assert!(size_written == value.data.len());
                    Ok(())
                }
            }
            value::ImmutableValue::Compound(value) => {
                writer
                    .write_all(&[Tag::Compound as u8, 0u8, 0u8])
                    .map_err(Error::IO)?;
                if TypeId::of::<SOURCE>() == TypeId::of::<TARGET>() {
                    writer.write_all(value.data.as_bytes()).map_err(Error::IO)
                } else {
                    let size_written = write::write_compound_to_writer_fallback::<SOURCE, TARGET, W>(
                        value.data.as_ptr(),
                        &mut writer,
                    )?;
                    debug_assert!(size_written == value.data.len());
                    Ok(())
                }
            }
            value::ImmutableValue::IntArray(value) => {
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
            value::ImmutableValue::LongArray(value) => {
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
// todo: Read & Write trait

#[cfg(test)]
mod tests {
    use super::*;
    use zerocopy::{BE, LE};

    /// Helper to read back written NBT and verify roundtrip
    fn roundtrip_test<SOURCE: ByteOrder, TARGET: ByteOrder>(input: &[u8]) {
        let doc = read_borrowed::<SOURCE>(input).unwrap();
        let root = doc.root();

        // Test write_value_to_vec
        let vec_output = write_value_to_vec::<_, SOURCE, TARGET>(&root).unwrap();
        let doc2 = read_borrowed::<TARGET>(&vec_output).unwrap();
        
        // Verify we can read it back
        assert!(!matches!(doc2.root(), value::ImmutableValue::End) || matches!(root, value::ImmutableValue::End));

        // Test write_value_to_writer
        let mut writer_output = Vec::new();
        write_value_to_writer::<_, SOURCE, TARGET, _>(&mut writer_output, &root).unwrap();
        let doc3 = read_borrowed::<TARGET>(&writer_output).unwrap();

        // Both outputs should be identical
        assert_eq!(vec_output, writer_output);
        
        // Verify we can read it back
        assert!(!matches!(doc3.root(), value::ImmutableValue::End) || matches!(root, value::ImmutableValue::End));
    }

    #[test]
    fn test_end_tag() {
        // Minimal End tag NBT
        let input = [0u8]; // Tag::End
        let doc = read_borrowed::<BE>(&input).unwrap();
        
        let vec_out = write_value_to_vec::<_, BE, BE>(&doc.root()).unwrap();
        assert_eq!(vec_out, vec![0u8]);
        
        let mut writer_out = Vec::new();
        write_value_to_writer::<_, BE, BE, _>(&mut writer_out, &doc.root()).unwrap();
        assert_eq!(writer_out, vec![0u8]);
    }

    #[test]
    fn test_byte() {
        // Tag::Byte (1), name_len (0, 0), value (42)
        let input = [1u8, 0, 0, 42];
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_short() {
        // Tag::Short (2), name_len BE (0, 0), value BE (0x12, 0x34)
        let input = [2u8, 0, 0, 0x12, 0x34];
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_int() {
        // Tag::Int (3), name_len BE (0, 0), value BE
        let input = [3u8, 0, 0, 0x12, 0x34, 0x56, 0x78];
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_long() {
        // Tag::Long (4), name_len BE (0, 0), value BE
        let input = [4u8, 0, 0, 0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_float() {
        // Tag::Float (5), name_len BE (0, 0), value BE (1.0f32)
        let value_bytes = 1.0f32.to_be_bytes();
        let input = [5u8, 0, 0, value_bytes[0], value_bytes[1], value_bytes[2], value_bytes[3]];
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_double() {
        // Tag::Double (6), name_len BE (0, 0), value BE (1.0f64)
        let value_bytes = 1.0f64.to_be_bytes();
        let mut input = vec![6u8, 0, 0];
        input.extend_from_slice(&value_bytes);
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_byte_array() {
        // Tag::ByteArray (7), name_len BE (0, 0), array_len BE (0, 0, 0, 4), bytes
        let input = [7u8, 0, 0, 0, 0, 0, 4, 1, 2, 3, 4];
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_string() {
        // Tag::String (8), name_len BE (0, 0), string_len BE (0, 5), "hello"
        let input = [8u8, 0, 0, 0, 5, b'h', b'e', b'l', b'l', b'o'];
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_list_of_bytes() {
        // Tag::List (9), name_len BE (0, 0), element_tag (1=Byte), len BE (0, 0, 0, 3), bytes
        let input = [9u8, 0, 0, 1, 0, 0, 0, 3, 10, 20, 30];
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_list_of_shorts() {
        // Tag::List (9), name_len BE (0, 0), element_tag (2=Short), len BE (0, 0, 0, 2), shorts BE
        let input = [9u8, 0, 0, 2, 0, 0, 0, 2, 0x00, 0x10, 0x00, 0x20];
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_list_of_ints() {
        // Tag::List (9), name_len BE (0, 0), element_tag (3=Int), len BE (0, 0, 0, 2), ints BE
        let input = [9u8, 0, 0, 3, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 2];
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_list_of_longs() {
        // Tag::List (9), name_len BE (0, 0), element_tag (4=Long), len BE (0, 0, 0, 2), longs BE
        let input = [
            9u8, 0, 0, 4, 0, 0, 0, 2,
            0, 0, 0, 0, 0, 0, 0, 1,
            0, 0, 0, 0, 0, 0, 0, 2,
        ];
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_list_of_floats() {
        // Tag::List (9), name_len BE (0, 0), element_tag (5=Float), len BE (0, 0, 0, 2), floats BE
        let f1 = 1.5f32.to_be_bytes();
        let f2 = 2.5f32.to_be_bytes();
        let input = [
            9u8, 0, 0, 5, 0, 0, 0, 2,
            f1[0], f1[1], f1[2], f1[3],
            f2[0], f2[1], f2[2], f2[3],
        ];
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_list_of_doubles() {
        // Tag::List (9), name_len BE (0, 0), element_tag (6=Double), len BE (0, 0, 0, 2), doubles BE
        let d1 = 1.5f64.to_be_bytes();
        let d2 = 2.5f64.to_be_bytes();
        let mut input = vec![9u8, 0, 0, 6, 0, 0, 0, 2];
        input.extend_from_slice(&d1);
        input.extend_from_slice(&d2);
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_list_of_strings() {
        // Tag::List (9), name_len BE (0, 0), element_tag (8=String), len BE (0, 0, 0, 2), strings
        let input = [
            9u8, 0, 0, 8, 0, 0, 0, 2,
            0, 2, b'h', b'i',
            0, 3, b'b', b'y', b'e',
        ];
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_empty_compound() {
        // Tag::Compound (10), name_len BE (0, 0), Tag::End (0)
        let input = [10u8, 0, 0, 0];
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_compound_with_byte() {
        // Tag::Compound (10), name_len BE (0, 0), [Tag::Byte (1), name_len (0, 1), 'x', value], Tag::End
        let input = [10u8, 0, 0, 1, 0, 1, b'x', 42, 0];
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_compound_with_multiple_fields() {
        // Compound with Int and String fields
        let input = [
            10u8, 0, 0,           // Tag::Compound, name_len=0
            3, 0, 1, b'a',        // Tag::Int, name="a"
            0, 0, 0, 42,          // Int value = 42
            8, 0, 1, b'b',        // Tag::String, name="b"
            0, 5, b'h', b'e', b'l', b'l', b'o',  // String = "hello"
            0                     // Tag::End
        ];
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_nested_compound() {
        // Compound containing a compound
        let input = [
            10u8, 0, 0,           // Tag::Compound, name_len=0
            10, 0, 5, b'i', b'n', b'n', b'e', b'r',  // Tag::Compound, name="inner"
            1, 0, 1, b'x', 99,    // Tag::Byte, name="x", value=99
            0,                    // End inner
            0                     // End outer
        ];
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_int_array() {
        // Tag::IntArray (11), name_len BE (0, 0), array_len BE (0, 0, 0, 3), ints BE
        let input = [
            11u8, 0, 0, 0, 0, 0, 3,
            0, 0, 0, 1,
            0, 0, 0, 2,
            0, 0, 0, 3,
        ];
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_long_array() {
        // Tag::LongArray (12), name_len BE (0, 0), array_len BE (0, 0, 0, 2), longs BE
        let input = [
            12u8, 0, 0, 0, 0, 0, 2,
            0, 0, 0, 0, 0, 0, 0, 1,
            0, 0, 0, 0, 0, 0, 0, 2,
        ];
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_list_of_int_arrays() {
        // List of IntArrays
        let input = [
            9u8, 0, 0, 11, 0, 0, 0, 2,  // Tag::List, element=IntArray, len=2
            0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 2,  // IntArray[2]: [1, 2]
            0, 0, 0, 1, 0, 0, 0, 3,              // IntArray[1]: [3]
        ];
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_list_of_long_arrays() {
        // List of LongArrays
        let input = [
            9u8, 0, 0, 12, 0, 0, 0, 2,  // Tag::List, element=LongArray, len=2
            0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 10,  // LongArray[1]: [10]
            0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 20,  // LongArray[1]: [20]
        ];
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_list_of_compounds() {
        // List of Compounds
        let input = [
            9u8, 0, 0, 10, 0, 0, 0, 2,  // Tag::List, element=Compound, len=2
            1, 0, 1, b'x', 1, 0,       // Compound { x: Byte(1) }
            1, 0, 1, b'y', 2, 0,       // Compound { y: Byte(2) }
        ];
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_list_of_lists() {
        // List of Lists (List<List<Byte>>)
        let input = [
            9u8, 0, 0, 9, 0, 0, 0, 2,  // Tag::List, element=List, len=2
            1, 0, 0, 0, 2, 10, 20,     // List<Byte>[2]: [10, 20]
            1, 0, 0, 0, 1, 30,         // List<Byte>[1]: [30]
        ];
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_empty_list() {
        // Empty list of bytes
        let input = [9u8, 0, 0, 0, 0, 0, 0, 0]; // Tag::List, element=End, len=0
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_le_to_be() {
        // Test LE source to BE target
        // Tag::Int (3), name_len LE (0, 0), value LE (0x78, 0x56, 0x34, 0x12)
        let input = [3u8, 0, 0, 0x78, 0x56, 0x34, 0x12];
        roundtrip_test::<LE, BE>(&input);
        roundtrip_test::<LE, LE>(&input);
    }

    #[test]
    fn test_large_int_array() {
        // Test threshold behavior for IntArray (>128 elements)
        let mut input = vec![11u8, 0, 0]; // Tag::IntArray, name_len=0
        input.extend_from_slice(&[0, 0, 1, 0]); // len = 256 (BE)
        for i in 0..256u32 {
            input.extend_from_slice(&i.to_be_bytes());
        }
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_large_long_array() {
        // Test threshold behavior for LongArray (>128 elements)
        let mut input = vec![12u8, 0, 0]; // Tag::LongArray, name_len=0
        input.extend_from_slice(&[0, 0, 1, 0]); // len = 256 (BE)
        for i in 0..256u64 {
            input.extend_from_slice(&i.to_be_bytes());
        }
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_real_file_simple_player() {
        let input = include_bytes!("D:/cmp/simdnbt/simdnbt/tests/simple_player");
        roundtrip_test::<BE, BE>(input);
        roundtrip_test::<BE, LE>(input);
    }

    #[test]
    fn test_real_file_inttest() {
        let input = include_bytes!("D:/cmp/simdnbt/simdnbt/tests/inttest1023.nbt");
        roundtrip_test::<BE, BE>(input);
        roundtrip_test::<BE, LE>(input);
    }

    #[test]
    fn test_real_file_longtest() {
        let input = include_bytes!("D:/cmp/simdnbt/simdnbt/tests/longtest1024.nbt");
        roundtrip_test::<BE, BE>(input);
        roundtrip_test::<BE, LE>(input);
    }

    // Additional coverage tests

    #[test]
    fn test_list_of_byte_arrays() {
        // List of ByteArrays - covers Tag::ByteArray in write_list_fallback
        let input = [
            9u8, 0, 0, 7, 0, 0, 0, 2,  // Tag::List, element=ByteArray, len=2
            0, 0, 0, 3, 1, 2, 3,       // ByteArray[3]: [1, 2, 3]
            0, 0, 0, 2, 4, 5,          // ByteArray[2]: [4, 5]
        ];
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_compound_with_list() {
        // Compound containing a List - covers Tag::List in write_compound_fallback
        let input = [
            10u8, 0, 0,              // Tag::Compound, name_len=0
            9, 0, 4, b'l', b'i', b's', b't',  // Tag::List, name="list"
            1, 0, 0, 0, 3, 10, 20, 30,        // List<Byte>[3]
            0                        // Tag::End
        ];
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_compound_with_int_array() {
        // Compound containing IntArray - covers Tag::IntArray in write_compound_fallback
        let input = [
            10u8, 0, 0,              // Tag::Compound, name_len=0
            11, 0, 3, b'a', b'r', b'r',  // Tag::IntArray, name="arr"
            0, 0, 0, 2,              // len=2
            0, 0, 0, 1,              // 1
            0, 0, 0, 2,              // 2
            0                        // Tag::End
        ];
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_compound_with_long_array() {
        // Compound containing LongArray - covers Tag::LongArray in write_compound_fallback
        let input = [
            10u8, 0, 0,              // Tag::Compound, name_len=0
            12, 0, 3, b'a', b'r', b'r',  // Tag::LongArray, name="arr"
            0, 0, 0, 2,              // len=2
            0, 0, 0, 0, 0, 0, 0, 1,  // 1L
            0, 0, 0, 0, 0, 0, 0, 2,  // 2L
            0                        // Tag::End
        ];
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_compound_with_byte_array() {
        // Compound containing ByteArray - covers Tag::ByteArray in write_compound_fallback
        let input = [
            10u8, 0, 0,              // Tag::Compound, name_len=0
            7, 0, 4, b'd', b'a', b't', b'a',  // Tag::ByteArray, name="data"
            0, 0, 0, 4, 1, 2, 3, 4,  // ByteArray[4]
            0                        // Tag::End
        ];
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_compound_with_string() {
        // Compound containing String - covers Tag::String in write_compound_fallback
        let input = [
            10u8, 0, 0,              // Tag::Compound, name_len=0
            8, 0, 3, b'm', b's', b'g',  // Tag::String, name="msg"
            0, 5, b'h', b'e', b'l', b'l', b'o',  // String = "hello"
            0                        // Tag::End
        ];
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_empty_byte_array() {
        let input = [7u8, 0, 0, 0, 0, 0, 0]; // Tag::ByteArray, name_len=0, len=0
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_empty_int_array() {
        let input = [11u8, 0, 0, 0, 0, 0, 0]; // Tag::IntArray, name_len=0, len=0
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_empty_long_array() {
        let input = [12u8, 0, 0, 0, 0, 0, 0]; // Tag::LongArray, name_len=0, len=0
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_empty_string() {
        let input = [8u8, 0, 0, 0, 0]; // Tag::String, name_len=0, string_len=0
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_negative_values() {
        // Short with negative value
        let input = [2u8, 0, 0, 0xFF, 0xFE]; // -2 in BE
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);

        // Int with negative value
        let input = [3u8, 0, 0, 0xFF, 0xFF, 0xFF, 0xFE]; // -2 in BE
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);

        // Long with negative value
        let mut input = vec![4u8, 0, 0];
        input.extend_from_slice(&(-2i64).to_be_bytes());
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_min_max_values() {
        // i8 min/max
        let input_min = [1u8, 0, 0, 0x80]; // i8::MIN
        let input_max = [1u8, 0, 0, 0x7F]; // i8::MAX
        roundtrip_test::<BE, BE>(&input_min);
        roundtrip_test::<BE, LE>(&input_max);

        // i16 min/max
        let input_min = [2u8, 0, 0, 0x80, 0x00]; // i16::MIN
        let input_max = [2u8, 0, 0, 0x7F, 0xFF]; // i16::MAX
        roundtrip_test::<BE, BE>(&input_min);
        roundtrip_test::<BE, LE>(&input_max);

        // i32 min/max
        let input_min = [3u8, 0, 0, 0x80, 0x00, 0x00, 0x00]; // i32::MIN
        let input_max = [3u8, 0, 0, 0x7F, 0xFF, 0xFF, 0xFF]; // i32::MAX
        roundtrip_test::<BE, BE>(&input_min);
        roundtrip_test::<BE, LE>(&input_max);

        // i64 min/max
        let mut input_min = vec![4u8, 0, 0];
        input_min.extend_from_slice(&i64::MIN.to_be_bytes());
        let mut input_max = vec![4u8, 0, 0];
        input_max.extend_from_slice(&i64::MAX.to_be_bytes());
        roundtrip_test::<BE, BE>(&input_min);
        roundtrip_test::<BE, LE>(&input_max);
    }

    #[test]
    fn test_special_floats() {
        // Float: 0.0, -0.0, infinity, -infinity, NaN handling via roundtrip
        let zero = 0.0f32.to_be_bytes();
        let input = [5u8, 0, 0, zero[0], zero[1], zero[2], zero[3]];
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);

        let neg_zero = (-0.0f32).to_be_bytes();
        let input = [5u8, 0, 0, neg_zero[0], neg_zero[1], neg_zero[2], neg_zero[3]];
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);

        let inf = f32::INFINITY.to_be_bytes();
        let input = [5u8, 0, 0, inf[0], inf[1], inf[2], inf[3]];
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);

        let neg_inf = f32::NEG_INFINITY.to_be_bytes();
        let input = [5u8, 0, 0, neg_inf[0], neg_inf[1], neg_inf[2], neg_inf[3]];
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_special_doubles() {
        // Double: 0.0, infinity
        let zero = 0.0f64.to_be_bytes();
        let mut input = vec![6u8, 0, 0];
        input.extend_from_slice(&zero);
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);

        let inf = f64::INFINITY.to_be_bytes();
        let mut input = vec![6u8, 0, 0];
        input.extend_from_slice(&inf);
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_deeply_nested_compound() {
        // 3 levels deep: Compound { a: Compound { b: Compound { c: Byte } } }
        let input = [
            10u8, 0, 0,                     // outer Compound
            10, 0, 1, b'a',                 // Tag::Compound, name="a"
            10, 0, 1, b'b',                 // Tag::Compound, name="b"
            1, 0, 1, b'c', 42,              // Tag::Byte, name="c", value=42
            0,                              // End innermost
            0,                              // End middle
            0                               // End outer
        ];
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_deeply_nested_list() {
        // List<List<List<Byte>>>
        let input = [
            9u8, 0, 0, 9, 0, 0, 0, 1,  // List, element=List, len=1
            9, 0, 0, 0, 1,             // inner: List, element=List, len=1
            1, 0, 0, 0, 2, 10, 20,     // innermost: List<Byte>[2]
        ];
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_compound_all_types() {
        // Compound with all primitive types
        let mut input = vec![
            10u8, 0, 0,                     // Tag::Compound, name_len=0
            1, 0, 1, b'a', 42,              // Byte
            2, 0, 1, b'b', 0, 100,          // Short
            3, 0, 1, b'c', 0, 0, 0, 200,    // Int
        ];
        // Long
        input.extend_from_slice(&[4, 0, 1, b'd']);
        input.extend_from_slice(&300i64.to_be_bytes());
        // Float
        input.extend_from_slice(&[5, 0, 1, b'e']);
        input.extend_from_slice(&1.5f32.to_be_bytes());
        // Double
        input.extend_from_slice(&[6, 0, 1, b'f']);
        input.extend_from_slice(&2.5f64.to_be_bytes());
        // End
        input.push(0);
        
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_large_list_of_ints() {
        // List with >128 ints to test threshold in list fallback
        let mut input = vec![9u8, 0, 0, 3]; // Tag::List, element=Int
        input.extend_from_slice(&256u32.to_be_bytes()); // len=256
        for i in 0..256u32 {
            input.extend_from_slice(&i.to_be_bytes());
        }
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_large_list_of_longs() {
        // List with >128 longs to test threshold in list fallback
        let mut input = vec![9u8, 0, 0, 4]; // Tag::List, element=Long
        input.extend_from_slice(&256u32.to_be_bytes()); // len=256
        for i in 0..256u64 {
            input.extend_from_slice(&i.to_be_bytes());
        }
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_compound_with_large_int_array() {
        // Compound containing IntArray with >128 elements
        let mut input = vec![10u8, 0, 0];  // Tag::Compound
        input.extend_from_slice(&[11, 0, 3, b'a', b'r', b'r']); // Tag::IntArray, name="arr"
        input.extend_from_slice(&256u32.to_be_bytes()); // len=256
        for i in 0..256u32 {
            input.extend_from_slice(&i.to_be_bytes());
        }
        input.push(0); // End
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_compound_with_large_long_array() {
        // Compound containing LongArray with >128 elements
        let mut input = vec![10u8, 0, 0];  // Tag::Compound
        input.extend_from_slice(&[12, 0, 3, b'a', b'r', b'r']); // Tag::LongArray, name="arr"
        input.extend_from_slice(&256u32.to_be_bytes()); // len=256
        for i in 0..256u64 {
            input.extend_from_slice(&i.to_be_bytes());
        }
        input.push(0); // End
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    // ===== Tests for read_shared =====

    #[test]
    fn test_read_shared_byte() {
        use bytes::Bytes;
        let input = vec![1u8, 0, 0, 42u8]; // Tag::Byte, name="", value=42
        let bytes = Bytes::from(input);
        let value: super::SharedValue<BE> = super::read_shared(bytes).unwrap();
        assert!(value.is_byte());
        assert_eq!(value.as_byte(), Some(42));
    }

    #[test]
    fn test_read_shared_compound() {
        use bytes::Bytes;
        let mut input = vec![10u8, 0, 0]; // Tag::Compound, name=""
        input.extend_from_slice(&[1, 0, 1, b'a', 42u8]); // Byte named "a" = 42
        input.push(0); // End
        let bytes = Bytes::from(input);
        let value: super::SharedValue<BE> = super::read_shared(bytes).unwrap();
        assert!(value.is_compound());
    }

    #[test]
    fn test_read_shared_end_tag() {
        use bytes::Bytes;
        let input = vec![0u8]; // Tag::End (empty root)
        let bytes = Bytes::from(input);
        let value: super::SharedValue<BE> = super::read_shared(bytes).unwrap();
        assert!(value.is_end());
    }

    #[test]
    fn test_shared_document_root() {
        use bytes::Bytes;
        // Create a simple compound
        let mut input = vec![10u8, 0, 4, b't', b'e', b's', b't']; // Tag::Compound, name="test"
        input.extend_from_slice(&[1, 0, 1, b'x', 99u8]); // Byte named "x" = 99
        input.push(0); // End
        let bytes = Bytes::from(input);
        let value: super::SharedValue<BE> = super::read_shared(bytes).unwrap();
        assert!(value.is_compound());
    }

    // ===== Error path tests =====

    #[test]
    fn test_error_trailing_data_byte() {
        // Byte with extra trailing data
        let input = vec![1u8, 0, 0, 42u8, 0xFF]; // Tag::Byte, value=42, then extra byte
        let result = super::read_borrowed::<BE>(&input);
        let Err(e) = result else { panic!("Expected error") };
        assert!(matches!(e, super::Error::TrailingData(1)), "Expected TrailingData(1), got {:?}", e);
    }

    #[test]
    fn test_error_trailing_data_short() {
        // Short with extra trailing data
        let mut input = vec![2u8, 0, 0]; // Tag::Short, name=""
        input.extend_from_slice(&100i16.to_be_bytes());
        input.push(0xFF); // extra byte
        let result = super::read_borrowed::<BE>(&input);
        let Err(e) = result else { panic!("Expected error") };
        assert!(matches!(e, super::Error::TrailingData(1)), "Expected TrailingData(1), got {:?}", e);
    }

    #[test]
    fn test_error_trailing_data_int() {
        let mut input = vec![3u8, 0, 0]; // Tag::Int
        input.extend_from_slice(&123i32.to_be_bytes());
        input.extend_from_slice(&[0xFF, 0xFF]); // extra bytes
        let result = super::read_borrowed::<BE>(&input);
        let Err(e) = result else { panic!("Expected error") };
        assert!(matches!(e, super::Error::TrailingData(2)), "Expected TrailingData(2), got {:?}", e);
    }

    #[test]
    fn test_error_trailing_data_long() {
        let mut input = vec![4u8, 0, 0]; // Tag::Long
        input.extend_from_slice(&456i64.to_be_bytes());
        input.push(0xFF);
        let result = super::read_borrowed::<BE>(&input);
        let Err(e) = result else { panic!("Expected error") };
        assert!(matches!(e, super::Error::TrailingData(1)), "Expected TrailingData(1), got {:?}", e);
    }

    #[test]
    fn test_error_trailing_data_float() {
        let mut input = vec![5u8, 0, 0]; // Tag::Float
        input.extend_from_slice(&1.0f32.to_be_bytes());
        input.push(0xFF);
        let result = super::read_borrowed::<BE>(&input);
        let Err(e) = result else { panic!("Expected error") };
        assert!(matches!(e, super::Error::TrailingData(1)), "Expected TrailingData(1), got {:?}", e);
    }

    #[test]
    fn test_error_trailing_data_double() {
        let mut input = vec![6u8, 0, 0]; // Tag::Double
        input.extend_from_slice(&2.0f64.to_be_bytes());
        input.push(0xFF);
        let result = super::read_borrowed::<BE>(&input);
        let Err(e) = result else { panic!("Expected error") };
        assert!(matches!(e, super::Error::TrailingData(1)), "Expected TrailingData(1), got {:?}", e);
    }

    #[test]
    fn test_error_trailing_data_byte_array() {
        let mut input = vec![7u8, 0, 0]; // Tag::ByteArray
        input.extend_from_slice(&2u32.to_be_bytes()); // len=2
        input.extend_from_slice(&[1, 2]); // data
        input.push(0xFF); // extra
        let result = super::read_borrowed::<BE>(&input);
        let Err(e) = result else { panic!("Expected error") };
        assert!(matches!(e, super::Error::TrailingData(1)), "Expected TrailingData(1), got {:?}", e);
    }

    #[test]
    fn test_error_trailing_data_int_array() {
        let mut input = vec![11u8, 0, 0]; // Tag::IntArray
        input.extend_from_slice(&1u32.to_be_bytes()); // len=1
        input.extend_from_slice(&42i32.to_be_bytes());
        input.push(0xFF);
        let result = super::read_borrowed::<BE>(&input);
        let Err(e) = result else { panic!("Expected error") };
        assert!(matches!(e, super::Error::TrailingData(1)), "Expected TrailingData(1), got {:?}", e);
    }

    #[test]
    fn test_error_trailing_data_long_array() {
        let mut input = vec![12u8, 0, 0]; // Tag::LongArray
        input.extend_from_slice(&1u32.to_be_bytes()); // len=1
        input.extend_from_slice(&42i64.to_be_bytes());
        input.push(0xFF);
        let result = super::read_borrowed::<BE>(&input);
        let Err(e) = result else { panic!("Expected error") };
        assert!(matches!(e, super::Error::TrailingData(1)), "Expected TrailingData(1), got {:?}", e);
    }

    #[test]
    fn test_error_trailing_data_string() {
        let mut input = vec![8u8, 0, 0]; // Tag::String
        input.extend_from_slice(&3u16.to_be_bytes()); // len=3
        input.extend_from_slice(b"abc");
        input.push(0xFF);
        let result = super::read_borrowed::<BE>(&input);
        let Err(e) = result else { panic!("Expected error") };
        assert!(matches!(e, super::Error::TrailingData(1)), "Expected TrailingData(1), got {:?}", e);
    }

    #[test]
    fn test_error_invalid_tag_type() {
        // Tag type 13 is invalid
        let input = vec![13u8, 0, 0];
        let result = super::read_borrowed::<BE>(&input);
        let Err(e) = result else { panic!("Expected error") };
        assert!(matches!(e, super::Error::InvalidTagType(13)), "Expected InvalidTagType(13), got {:?}", e);
    }

    #[test]
    fn test_error_invalid_tag_type_high() {
        // Tag type 255 is invalid
        let input = vec![255u8, 0, 0];
        let result = super::read_borrowed::<BE>(&input);
        let Err(e) = result else { panic!("Expected error") };
        assert!(matches!(e, super::Error::InvalidTagType(255)), "Expected InvalidTagType(255), got {:?}", e);
    }

    #[test]
    fn test_error_unexpected_eof_empty() {
        let input: Vec<u8> = vec![];
        let result = super::read_borrowed::<BE>(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_unexpected_eof_name() {
        // Tag::Byte with incomplete name length
        let input = vec![1u8, 0]; // needs 2 more bytes for name len
        let result = super::read_borrowed::<BE>(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_unexpected_eof_value() {
        // Tag::Int with incomplete value
        let input = vec![3u8, 0, 0, 1, 2]; // needs 4 bytes for int, only 2
        let result = super::read_borrowed::<BE>(&input);
        assert!(result.is_err());
    }

    // ===== Edge case tests for List fallback threshold =====

    #[test]
    fn test_list_of_int_arrays_large() {
        // List of IntArrays where each array is >128 elements
        let mut input = vec![9u8, 0, 0, 11]; // Tag::List, element=IntArray
        input.extend_from_slice(&2u32.to_be_bytes()); // len=2 (two int arrays)
        for _ in 0..2 {
            input.extend_from_slice(&150u32.to_be_bytes()); // array len=150
            for i in 0..150u32 {
                input.extend_from_slice(&i.to_be_bytes());
            }
        }
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_list_of_long_arrays_large() {
        // List of LongArrays where each array is >128 elements
        let mut input = vec![9u8, 0, 0, 12]; // Tag::List, element=LongArray
        input.extend_from_slice(&2u32.to_be_bytes()); // len=2
        for _ in 0..2 {
            input.extend_from_slice(&150u32.to_be_bytes()); // array len=150
            for i in 0..150u64 {
                input.extend_from_slice(&i.to_be_bytes());
            }
        }
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_compound_with_large_list_of_ints() {
        // Compound containing a list with >128 ints
        let mut input = vec![10u8, 0, 0]; // Tag::Compound
        input.extend_from_slice(&[9, 0, 1, b'l', 3]); // Tag::List, name="l", element=Int
        input.extend_from_slice(&200u32.to_be_bytes()); // len=200
        for i in 0..200u32 {
            input.extend_from_slice(&i.to_be_bytes());
        }
        input.push(0); // End
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    #[test]
    fn test_compound_with_large_list_of_longs() {
        // Compound containing a list with >128 longs
        let mut input = vec![10u8, 0, 0]; // Tag::Compound
        input.extend_from_slice(&[9, 0, 1, b'l', 4]); // Tag::List, name="l", element=Long
        input.extend_from_slice(&200u32.to_be_bytes()); // len=200
        for i in 0..200u64 {
            input.extend_from_slice(&i.to_be_bytes());
        }
        input.push(0); // End
        roundtrip_test::<BE, BE>(&input);
        roundtrip_test::<BE, LE>(&input);
    }

    // ===== Tests for Tag methods =====

    #[test]
    fn test_tag_is_primitive() {
        use crate::Tag;
        assert!(Tag::End.is_primitive());
        assert!(Tag::Byte.is_primitive());
        assert!(Tag::Short.is_primitive());
        assert!(Tag::Int.is_primitive());
        assert!(Tag::Long.is_primitive());
        assert!(Tag::Float.is_primitive());
        assert!(Tag::Double.is_primitive());
        
        assert!(!Tag::ByteArray.is_primitive());
        assert!(!Tag::String.is_primitive());
        assert!(!Tag::List.is_primitive());
        assert!(!Tag::Compound.is_primitive());
        assert!(!Tag::IntArray.is_primitive());
        assert!(!Tag::LongArray.is_primitive());
    }

    #[test]
    fn test_tag_is_array() {
        use crate::Tag;
        assert!(Tag::ByteArray.is_array());
        assert!(Tag::IntArray.is_array());
        assert!(Tag::LongArray.is_array());
        
        assert!(!Tag::End.is_array());
        assert!(!Tag::Byte.is_array());
        assert!(!Tag::Short.is_array());
        assert!(!Tag::Int.is_array());
        assert!(!Tag::Long.is_array());
        assert!(!Tag::Float.is_array());
        assert!(!Tag::Double.is_array());
        assert!(!Tag::String.is_array());
        assert!(!Tag::List.is_array());
        assert!(!Tag::Compound.is_array());
    }

    #[test]
    fn test_tag_is_composite() {
        use crate::Tag;
        assert!(Tag::List.is_composite());
        assert!(Tag::Compound.is_composite());
        
        assert!(!Tag::End.is_composite());
        assert!(!Tag::Byte.is_composite());
        assert!(!Tag::Short.is_composite());
        assert!(!Tag::Int.is_composite());
        assert!(!Tag::Long.is_composite());
        assert!(!Tag::Float.is_composite());
        assert!(!Tag::Double.is_composite());
        assert!(!Tag::ByteArray.is_composite());
        assert!(!Tag::String.is_composite());
        assert!(!Tag::IntArray.is_composite());
        assert!(!Tag::LongArray.is_composite());
    }

    // ===== Tests for Value tag_id method =====

    #[test]
    fn test_value_tag_id_all_types() {
        // Test that each value type returns correct tag_id
        let mut input = vec![10u8, 0, 0]; // Tag::Compound
        // Add all primitive types
        input.push(1); input.push(0); input.push(1); input.push(b'a'); input.push(42); // Byte
        input.push(2); input.push(0); input.push(1); input.push(b'b'); input.extend_from_slice(&100i16.to_be_bytes()); // Short
        input.push(3); input.push(0); input.push(1); input.push(b'c'); input.extend_from_slice(&200i32.to_be_bytes()); // Int
        input.push(4); input.push(0); input.push(1); input.push(b'd'); input.extend_from_slice(&300i64.to_be_bytes()); // Long
        input.push(5); input.push(0); input.push(1); input.push(b'e'); input.extend_from_slice(&1.5f32.to_be_bytes()); // Float
        input.push(6); input.push(0); input.push(1); input.push(b'f'); input.extend_from_slice(&2.5f64.to_be_bytes()); // Double
        input.push(7); input.push(0); input.push(1); input.push(b'g'); input.extend_from_slice(&1u32.to_be_bytes()); input.push(99); // ByteArray
        input.push(8); input.push(0); input.push(1); input.push(b'h'); input.extend_from_slice(&2u16.to_be_bytes()); input.extend_from_slice(b"xy"); // String
        input.push(9); input.push(0); input.push(1); input.push(b'i'); input.push(1); input.extend_from_slice(&1u32.to_be_bytes()); input.push(7); // List of bytes
        input.push(10); input.push(0); input.push(1); input.push(b'j'); input.push(0); // Nested compound
        input.push(11); input.push(0); input.push(1); input.push(b'k'); input.extend_from_slice(&1u32.to_be_bytes()); input.extend_from_slice(&42i32.to_be_bytes()); // IntArray
        input.push(12); input.push(0); input.push(1); input.push(b'l'); input.extend_from_slice(&1u32.to_be_bytes()); input.extend_from_slice(&99i64.to_be_bytes()); // LongArray
        input.push(0); // End
        
        let doc = super::read_borrowed::<BE>(&input).unwrap();
        let doc_ref = doc.root();
        
        if let value::ImmutableValue::Compound(compound) = doc_ref {
            for (_, value) in compound.iter() {
                use crate::Tag;
                let tag_id = value.tag_id();
                match value {
                    value::ImmutableValue::End => assert_eq!(tag_id, Tag::End),
                    value::ImmutableValue::Byte(_) => assert_eq!(tag_id, Tag::Byte),
                    value::ImmutableValue::Short(_) => assert_eq!(tag_id, Tag::Short),
                    value::ImmutableValue::Int(_) => assert_eq!(tag_id, Tag::Int),
                    value::ImmutableValue::Long(_) => assert_eq!(tag_id, Tag::Long),
                    value::ImmutableValue::Float(_) => assert_eq!(tag_id, Tag::Float),
                    value::ImmutableValue::Double(_) => assert_eq!(tag_id, Tag::Double),
                    value::ImmutableValue::ByteArray(_) => assert_eq!(tag_id, Tag::ByteArray),
                    value::ImmutableValue::String(_) => assert_eq!(tag_id, Tag::String),
                    value::ImmutableValue::List(_) => assert_eq!(tag_id, Tag::List),
                    value::ImmutableValue::Compound(_) => assert_eq!(tag_id, Tag::Compound),
                    value::ImmutableValue::IntArray(_) => assert_eq!(tag_id, Tag::IntArray),
                    value::ImmutableValue::LongArray(_) => assert_eq!(tag_id, Tag::LongArray),
                }
            }
        } else {
            panic!("Expected compound");
        }
    }

    // ===== Tests for error path in reading =====

    #[test]
    fn test_error_bounds_check_byte_array() {
        // ByteArray with length that exceeds available data
        let mut input = vec![7u8, 0, 0]; // Tag::ByteArray
        input.extend_from_slice(&1000u32.to_be_bytes()); // claims 1000 bytes
        input.push(1); // but only 1 byte available
        let result = super::read_borrowed::<BE>(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_bounds_check_string() {
        // String with length that exceeds available data
        let mut input = vec![8u8, 0, 0]; // Tag::String
        input.extend_from_slice(&1000u16.to_be_bytes()); // claims 1000 bytes
        input.push(1); // but only 1 byte available
        let result = super::read_borrowed::<BE>(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_bounds_check_int_array() {
        // IntArray with length that exceeds available data
        let mut input = vec![11u8, 0, 0]; // Tag::IntArray
        input.extend_from_slice(&1000u32.to_be_bytes()); // claims 1000 ints
        input.extend_from_slice(&42i32.to_be_bytes()); // but only 1 int
        let result = super::read_borrowed::<BE>(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_bounds_check_long_array() {
        // LongArray with length that exceeds available data
        let mut input = vec![12u8, 0, 0]; // Tag::LongArray
        input.extend_from_slice(&1000u32.to_be_bytes()); // claims 1000 longs
        input.extend_from_slice(&99i64.to_be_bytes()); // but only 1 long
        let result = super::read_borrowed::<BE>(&input);
        assert!(result.is_err());
    }

    // ===== Tests for List iteration =====

    #[test]
    fn test_list_iteration_all_types() {
        // Create a compound with a list of each primitive type to test iteration
        let mut input = vec![10u8, 0, 0]; // Tag::Compound
        
        // List of Bytes
        input.push(9); input.push(0); input.push(4); input.extend_from_slice(b"list"); input.push(1); // List, element=Byte
        input.extend_from_slice(&3u32.to_be_bytes()); // len=3
        input.extend_from_slice(&[1u8, 2u8, 3u8]);
        
        input.push(0); // End
        
        let doc = super::read_borrowed::<BE>(&input).unwrap();
        let doc_ref = doc.root();
        if let value::ImmutableValue::Compound(compound) = doc_ref {
            if let Some(value::ImmutableValue::List(list)) = compound.get("list") {
                let mut count = 0;
                for _ in list.iter() {
                    count += 1;
                }
                assert_eq!(count, 3, "List should have 3 elements");
            } else {
                panic!("Expected to find 'list' as a List");
            }
        } else {
            panic!("Expected compound");
        }
    }

    #[test]
    fn test_compound_iteration_mixed() {
        // Compound with mixed types to test iteration
        let mut input = vec![10u8, 0, 0]; // Tag::Compound
        input.push(1); input.push(0); input.push(1); input.push(b'a'); input.push(42); // Byte
        input.push(2); input.push(0); input.push(1); input.push(b'b'); input.extend_from_slice(&100i16.to_be_bytes()); // Short
        input.push(3); input.push(0); input.push(1); input.push(b'c'); input.extend_from_slice(&200i32.to_be_bytes()); // Int
        input.push(0); // End
        
        let doc = super::read_borrowed::<BE>(&input).unwrap();
        let doc_ref = doc.root();
        if let value::ImmutableValue::Compound(compound) = doc_ref {
            let mut count = 0;
            for (name, _value) in compound.iter() {
                assert!(name.raw_bytes().len() > 0);
                count += 1;
            }
            assert_eq!(count, 3, "Compound should have 3 fields");
        } else {
            panic!("Expected compound");
        }
    }

    // ===== Tests for List iteration =====

    #[test]
    fn test_list_iteration_bytes() {
        // List of Bytes to test iteration
        let mut input = vec![9u8, 0, 0, 1]; // Tag::List, element=Byte
        input.extend_from_slice(&3u32.to_be_bytes()); // len=3
        input.extend_from_slice(&[10u8, 20u8, 30u8]);
        
        let doc = super::read_borrowed::<BE>(&input).unwrap();
        let doc_ref = doc.root();
        if let value::ImmutableValue::List(list) = doc_ref {
            let mut count = 0;
            for _ in list.iter() {
                count += 1;
            }
            assert_eq!(count, 3, "List should have 3 elements");
        } else {
            panic!("Expected list");
        }
    }

    // ===== Tests for index operations =====

    #[test]
    fn test_compound_get_by_name() {
        let mut input = vec![10u8, 0, 0]; // Tag::Compound
        input.push(1); input.push(0); input.push(3); input.extend_from_slice(b"key"); input.push(42); // Byte named "key"
        input.push(0); // End
        
        let doc = super::read_borrowed::<BE>(&input).unwrap();
        let doc_ref = doc.root();
        if let value::ImmutableValue::Compound(compound) = doc_ref {
            assert!(compound.get("key").is_some());
            assert!(compound.get("nonexistent").is_none());
        } else {
            panic!("Expected compound");
        }
    }

    #[test]
    fn test_list_get_by_index() {
        let mut input = vec![9u8, 0, 0, 1]; // Tag::List, element=Byte
        input.extend_from_slice(&3u32.to_be_bytes()); // len=3
        input.extend_from_slice(&[10u8, 20u8, 30u8]);
        
        let doc = super::read_borrowed::<BE>(&input).unwrap();
        let doc_ref = doc.root();
        if let value::ImmutableValue::List(list) = doc_ref {
            assert!(list.get(0).is_some());
            assert!(list.get(2).is_some());
            assert!(list.get(3).is_none());
        } else {
            panic!("Expected list");
        }
    }

    // ===== Tests for as_* and is_* methods =====

    #[test]
    fn test_value_as_end() {
        let end_val: value::ImmutableValue<BE, Vec<u8>> = value::ImmutableValue::End;
        assert!(end_val.as_end().is_some());
        assert!(end_val.is_end());
        
        let byte_val: value::ImmutableValue<BE, Vec<u8>> = value::ImmutableValue::Byte(42);
        assert!(byte_val.as_end().is_none());
        assert!(!byte_val.is_end());
    }

    #[test]
    fn test_value_as_byte() {
        let byte_val: value::ImmutableValue<BE, Vec<u8>> = value::ImmutableValue::Byte(42);
        assert_eq!(byte_val.as_byte(), Some(42));
        assert!(byte_val.is_byte());
        
        let short_val: value::ImmutableValue<BE, Vec<u8>> = value::ImmutableValue::Short(100);
        assert!(short_val.as_byte().is_none());
        assert!(!short_val.is_byte());
    }

    #[test]
    fn test_value_as_short() {
        let short_val: value::ImmutableValue<BE, Vec<u8>> = value::ImmutableValue::Short(1000);
        assert_eq!(short_val.as_short(), Some(1000));
        assert!(short_val.is_short());
        
        let int_val: value::ImmutableValue<BE, Vec<u8>> = value::ImmutableValue::Int(2000);
        assert!(int_val.as_short().is_none());
        assert!(!int_val.is_short());
    }

    #[test]
    fn test_value_as_int() {
        let int_val: value::ImmutableValue<BE, Vec<u8>> = value::ImmutableValue::Int(5000);
        assert_eq!(int_val.as_int(), Some(5000));
        assert!(int_val.is_int());
        
        let long_val: value::ImmutableValue<BE, Vec<u8>> = value::ImmutableValue::Long(10000);
        assert!(long_val.as_int().is_none());
        assert!(!long_val.is_int());
    }

    #[test]
    fn test_value_as_long() {
        let long_val: value::ImmutableValue<BE, Vec<u8>> = value::ImmutableValue::Long(999999);
        assert_eq!(long_val.as_long(), Some(999999));
        assert!(long_val.is_long());
        
        let float_val: value::ImmutableValue<BE, Vec<u8>> = value::ImmutableValue::Float(1.5);
        assert!(float_val.as_long().is_none());
        assert!(!float_val.is_long());
    }

    #[test]
    fn test_value_as_float() {
        let float_val: value::ImmutableValue<BE, Vec<u8>> = value::ImmutableValue::Float(3.14);
        assert!((float_val.as_float().unwrap() - 3.14).abs() < 0.01);
        assert!(float_val.is_float());
        
        let double_val: value::ImmutableValue<BE, Vec<u8>> = value::ImmutableValue::Double(2.71);
        assert!(double_val.as_float().is_none());
        assert!(!double_val.is_float());
    }

    #[test]
    fn test_value_as_double() {
        let double_val: value::ImmutableValue<BE, Vec<u8>> = value::ImmutableValue::Double(2.718281828);
        assert!((double_val.as_double().unwrap() - 2.718281828).abs() < 0.00001);
        assert!(double_val.is_double());
        
        let end_val: value::ImmutableValue<BE, Vec<u8>> = value::ImmutableValue::End;
        assert!(end_val.as_double().is_none());
        assert!(!end_val.is_double());
    }

    #[test]
    fn test_value_as_byte_array() {
        // Create a compound with a byte array to test as_byte_array
        let mut input = vec![10u8, 0, 0]; // Tag::Compound
        input.push(7); input.push(0); input.push(1); input.push(b'a');
        input.extend_from_slice(&2u32.to_be_bytes());
        input.extend_from_slice(&[99u8, 100u8]);
        input.push(0); // End
        
        let doc = super::read_borrowed::<BE>(&input).unwrap();
        let doc_ref = doc.root();
        if let value::ImmutableValue::Compound(compound) = doc_ref {
            if let Some(val) = compound.get("a") {
                assert!(val.as_byte_array().is_some());
                assert!(val.is_byte_array());
                assert_eq!(val.as_byte_array().unwrap().len(), 2);
            } else {
                panic!("Expected byte array field");
            }
        }
        
        let non_array: value::ImmutableValue<BE, Vec<u8>> = value::ImmutableValue::Byte(42);
        assert!(non_array.as_byte_array().is_none());
        assert!(!non_array.is_byte_array());
    }

    #[test]
    fn test_value_as_string() {
        // Create a compound with a string
        let mut input = vec![10u8, 0, 0]; // Tag::Compound
        input.push(8); input.push(0); input.push(1); input.push(b's');
        input.extend_from_slice(&5u16.to_be_bytes());
        input.extend_from_slice(b"hello");
        input.push(0); // End
        
        let doc = super::read_borrowed::<BE>(&input).unwrap();
        let doc_ref = doc.root();
        if let value::ImmutableValue::Compound(compound) = doc_ref {
            if let Some(val) = compound.get("s") {
                assert!(val.as_string().is_some());
                assert!(val.is_string());
            } else {
                panic!("Expected string field");
            }
        }
        
        let non_string: value::ImmutableValue<BE, Vec<u8>> = value::ImmutableValue::Short(42);
        assert!(non_string.as_string().is_none());
        assert!(!non_string.is_string());
    }

    #[test]
    fn test_value_as_list() {
        // Create a compound with a list
        let mut input = vec![10u8, 0, 0]; // Tag::Compound
        input.push(9); input.push(0); input.push(1); input.push(b'l');
        input.push(1); // element tag = Byte
        input.extend_from_slice(&2u32.to_be_bytes()); // list length = 2
        input.extend_from_slice(&[5u8, 10u8]);
        input.push(0); // End
        
        let doc = super::read_borrowed::<BE>(&input).unwrap();
        let doc_ref = doc.root();
        if let value::ImmutableValue::Compound(compound) = doc_ref {
            if let Some(val) = compound.get("l") {
                assert!(val.as_list().is_some());
                assert!(val.is_list());
            } else {
                panic!("Expected list field");
            }
        }
        
        let non_list: value::ImmutableValue<BE, Vec<u8>> = value::ImmutableValue::Int(999);
        assert!(non_list.as_list().is_none());
        assert!(!non_list.is_list());
    }

    #[test]
    fn test_value_as_compound() {
        // Create a compound with nested compound
        let mut input = vec![10u8, 0, 0]; // Tag::Compound
        input.push(10); input.push(0); input.push(1); input.push(b'c'); // nested compound
        input.push(0); // End of nested
        input.push(0); // End of outer
        
        let doc = super::read_borrowed::<BE>(&input).unwrap();
        let doc_ref = doc.root();
        if let value::ImmutableValue::Compound(compound) = doc_ref {
            if let Some(val) = compound.get("c") {
                assert!(val.as_compound().is_some());
                assert!(val.is_compound());
            } else {
                panic!("Expected compound field");
            }
        }
        
        let non_compound: value::ImmutableValue<BE, Vec<u8>> = value::ImmutableValue::Long(12345);
        assert!(non_compound.as_compound().is_none());
        assert!(!non_compound.is_compound());
    }

    #[test]
    fn test_value_as_int_array() {
        // Create a compound with an int array
        let mut input = vec![10u8, 0, 0]; // Tag::Compound
        input.push(11); input.push(0); input.push(1); input.push(b'i');
        input.extend_from_slice(&2u32.to_be_bytes()); // array length = 2
        input.extend_from_slice(&100i32.to_be_bytes());
        input.extend_from_slice(&200i32.to_be_bytes());
        input.push(0); // End
        
        let doc = super::read_borrowed::<BE>(&input).unwrap();
        let doc_ref = doc.root();
        if let value::ImmutableValue::Compound(compound) = doc_ref {
            if let Some(val) = compound.get("i") {
                assert!(val.as_int_array().is_some());
                assert!(val.is_int_array());
                assert_eq!(val.as_int_array().unwrap().len(), 2);
            } else {
                panic!("Expected int array field");
            }
        }
        
        let non_int_array: value::ImmutableValue<BE, Vec<u8>> = value::ImmutableValue::Float(1.5);
        assert!(non_int_array.as_int_array().is_none());
        assert!(!non_int_array.is_int_array());
    }

    #[test]
    fn test_value_as_long_array() {
        // Create a compound with a long array
        let mut input = vec![10u8, 0, 0]; // Tag::Compound
        input.push(12); input.push(0); input.push(1); input.push(b'l');
        input.extend_from_slice(&2u32.to_be_bytes()); // array length = 2
        input.extend_from_slice(&555i64.to_be_bytes());
        input.extend_from_slice(&777i64.to_be_bytes());
        input.push(0); // End
        
        let doc = super::read_borrowed::<BE>(&input).unwrap();
        let doc_ref = doc.root();
        if let value::ImmutableValue::Compound(compound) = doc_ref {
            if let Some(val) = compound.get("l") {
                assert!(val.as_long_array().is_some());
                assert!(val.is_long_array());
                assert_eq!(val.as_long_array().unwrap().len(), 2);
            } else {
                panic!("Expected long array field");
            }
        }
        
        let non_long_array: value::ImmutableValue<BE, Vec<u8>> = value::ImmutableValue::Double(3.14);
        assert!(non_long_array.as_long_array().is_none());
        assert!(!non_long_array.is_long_array());
    }

    // ===== Tests for ImmutableValue::get() method =====

    #[test]
    fn test_value_get_by_index() {
        // Test ImmutableValue::get() with usize index (for lists)
        let mut input = vec![9u8, 0, 0, 1]; // Tag::List, element=Byte
        input.extend_from_slice(&3u32.to_be_bytes()); // len=3
        input.extend_from_slice(&[10u8, 20u8, 30u8]);
        
        let doc = super::read_borrowed::<BE>(&input).unwrap();
        let doc_ref = doc.root();
        
        // Test valid index
        assert!(doc_ref.get(0usize).is_some());
        assert!(doc_ref.get(1usize).is_some());
        assert!(doc_ref.get(2usize).is_some());
        
        // Test out of bounds
        assert!(doc_ref.get(3usize).is_none());
        
        // Test on non-list (should return None)
        let byte_val: value::ImmutableValue<BE, Vec<u8>> = value::ImmutableValue::Byte(42);
        assert!(byte_val.get(0usize).is_none());
    }

    #[test]
    fn test_value_get_by_key() {
        // Test ImmutableValue::get() with &str key (for compounds)
        let mut input = vec![10u8, 0, 0]; // Tag::Compound
        input.push(1); input.push(0); input.push(4); input.extend_from_slice(b"name"); input.push(42); // Byte named "name"
        input.push(3); input.push(0); input.push(3); input.extend_from_slice(b"age"); input.extend_from_slice(&25i32.to_be_bytes()); // Int named "age"
        input.push(0); // End
        
        let doc = super::read_borrowed::<BE>(&input).unwrap();
        let doc_ref = doc.root();
        
        // Test valid keys
        assert!(doc_ref.get("name").is_some());
        assert!(doc_ref.get("age").is_some());
        
        // Test nonexistent key
        assert!(doc_ref.get("nonexistent").is_none());
        
        // Test on non-compound (should return None)
        let byte_val: value::ImmutableValue<BE, Vec<u8>> = value::ImmutableValue::Byte(42);
        assert!(byte_val.get("key").is_none());
    }

    // ===== Tests for ImmutableList methods =====

    #[test]
    fn test_list_tag_id() {
        let mut input = vec![9u8, 0, 0, 1]; // Tag::List, element=Byte
        input.extend_from_slice(&3u32.to_be_bytes()); // len=3
        input.extend_from_slice(&[10u8, 20u8, 30u8]);
        
        let doc = super::read_borrowed::<BE>(&input).unwrap();
        let doc_ref = doc.root();
        if let value::ImmutableValue::List(list) = doc_ref {
            assert_eq!(list.tag_id(), crate::Tag::Byte);
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_list_len_and_is_empty() {
        // Test empty list
        let mut input = vec![9u8, 0, 0, 1]; // Tag::List, element=Byte
        input.extend_from_slice(&0u32.to_be_bytes()); // len=0
        
        let doc = super::read_borrowed::<BE>(&input).unwrap();
        let doc_ref = doc.root();
        if let value::ImmutableValue::List(list) = doc_ref {
            assert_eq!(list.len(), 0);
            assert!(list.is_empty());
        } else {
            panic!("Expected list");
        }
        
        // Test non-empty list
        let mut input = vec![9u8, 0, 0, 1]; // Tag::List, element=Byte
        input.extend_from_slice(&2u32.to_be_bytes()); // len=2
        input.extend_from_slice(&[10u8, 20u8]);
        
        let doc = super::read_borrowed::<BE>(&input).unwrap();
        let doc_ref = doc.root();
        if let value::ImmutableValue::List(list) = doc_ref {
            assert_eq!(list.len(), 2);
            assert!(!list.is_empty());
        } else {
            panic!("Expected list");
        }
    }

    // ===== Tests for ImmutableCompound methods =====

    #[test]
    fn test_compound_get_various_types() {
        // Test compound.get() with various value types
        let mut input = vec![10u8, 0, 0]; // Tag::Compound
        input.push(1); input.push(0); input.push(1); input.push(b'a'); input.push(42); // Byte
        input.push(2); input.push(0); input.push(1); input.push(b'b'); input.extend_from_slice(&100i16.to_be_bytes()); // Short
        input.push(3); input.push(0); input.push(1); input.push(b'c'); input.extend_from_slice(&200i32.to_be_bytes()); // Int
        input.push(8); input.push(0); input.push(1); input.push(b's'); input.extend_from_slice(&5u16.to_be_bytes()); input.extend_from_slice(b"hello"); // String
        input.push(0); // End
        
        let doc = super::read_borrowed::<BE>(&input).unwrap();
        let doc_ref = doc.root();
        if let value::ImmutableValue::Compound(compound) = doc_ref {
            assert!(compound.get("a").is_some()); // Byte
            assert!(compound.get("b").is_some()); // Short
            assert!(compound.get("c").is_some()); // Int
            assert!(compound.get("s").is_some()); // String
            assert!(compound.get("nonexistent").is_none());
        } else {
            panic!("Expected compound");
        }
    }
}
