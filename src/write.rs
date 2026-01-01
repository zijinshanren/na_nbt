use std::{any::TypeId, io::Write, ptr, slice};

use zerocopy::byteorder;

use crate::{ByteOrder, Error, MUTF8Str, Result, TagID, Writable};

impl Writable for () {
    fn write_to_vec<TARGET: ByteOrder>(&self) -> Vec<u8> {
        vec![0]
    }

    fn write_to_writer<TARGET: ByteOrder>(&self, mut writer: impl Write) -> Result<()> {
        writer.write_all(&[0]).map_err(Error::IO)
    }
}

impl Writable for i8 {
    fn write_to_vec<TARGET: ByteOrder>(&self) -> Vec<u8> {
        unsafe {
            let mut buf = Vec::<u8>::with_capacity(4);
            let buf_ptr = buf.as_mut_ptr();
            ptr::write(buf_ptr.cast(), [TagID::Byte as u8, 0u8, 0u8, *self as u8]);
            buf.set_len(4);
            buf
        }
    }

    fn write_to_writer<TARGET: ByteOrder>(&self, mut writer: impl Write) -> Result<()> {
        writer
            .write_all(&[TagID::Byte as u8, 0u8, 0u8, *self as u8])
            .map_err(Error::IO)
    }
}

impl Writable for i16 {
    #[inline]
    fn write_to_vec<TARGET: ByteOrder>(&self) -> Vec<u8> {
        byteorder::I16::<TARGET>::new(*self).write_to_vec::<TARGET>()
    }

    #[inline]
    fn write_to_writer<TARGET: ByteOrder>(&self, writer: impl Write) -> Result<()> {
        byteorder::I16::<TARGET>::new(*self).write_to_writer::<TARGET>(writer)
    }
}

impl<O: ByteOrder> Writable for byteorder::I16<O> {
    fn write_to_vec<TARGET: ByteOrder>(&self) -> Vec<u8> {
        unsafe {
            let mut buf = Vec::<u8>::with_capacity(1 + 2 + 2);
            let buf_ptr = buf.as_mut_ptr();
            ptr::write(buf_ptr.cast(), [TagID::Short as u8, 0u8, 0u8]);
            ptr::write(
                buf_ptr.add(3).cast(),
                byteorder::I16::<TARGET>::new(self.get()).to_bytes(),
            );
            buf.set_len(1 + 2 + 2);
            buf
        }
    }

    fn write_to_writer<TARGET: ByteOrder>(&self, mut writer: impl Write) -> Result<()> {
        let mut buf = [0u8; 1 + 2 + 2];
        unsafe {
            ptr::write(buf.as_mut_ptr().cast(), [TagID::Short as u8, 0u8, 0u8]);
            ptr::write(
                buf.as_mut_ptr().add(3).cast(),
                byteorder::I16::<TARGET>::new(self.get()).to_bytes(),
            );
        }
        writer.write_all(&buf).map_err(Error::IO)
    }
}

impl Writable for i32 {
    #[inline]
    fn write_to_vec<TARGET: ByteOrder>(&self) -> Vec<u8> {
        byteorder::I32::<TARGET>::new(*self).write_to_vec::<TARGET>()
    }

    #[inline]
    fn write_to_writer<TARGET: ByteOrder>(&self, writer: impl Write) -> Result<()> {
        byteorder::I32::<TARGET>::new(*self).write_to_writer::<TARGET>(writer)
    }
}

impl<O: ByteOrder> Writable for byteorder::I32<O> {
    fn write_to_vec<TARGET: ByteOrder>(&self) -> Vec<u8> {
        unsafe {
            let mut buf = Vec::<u8>::with_capacity(1 + 2 + 4);
            let buf_ptr = buf.as_mut_ptr();
            ptr::write(buf_ptr.cast(), [TagID::Int as u8, 0u8, 0u8]);
            ptr::write(
                buf_ptr.add(3).cast(),
                byteorder::I32::<TARGET>::new(self.get()).to_bytes(),
            );
            buf.set_len(1 + 2 + 4);
            buf
        }
    }

    fn write_to_writer<TARGET: ByteOrder>(&self, mut writer: impl Write) -> Result<()> {
        let mut buf = [0u8; 1 + 2 + 4];
        unsafe {
            ptr::write(buf.as_mut_ptr().cast(), [TagID::Int as u8, 0u8, 0u8]);
            ptr::write(
                buf.as_mut_ptr().add(3).cast(),
                byteorder::I32::<TARGET>::new(self.get()).to_bytes(),
            );
        }
        writer.write_all(&buf).map_err(Error::IO)
    }
}

impl Writable for i64 {
    #[inline]
    fn write_to_vec<TARGET: ByteOrder>(&self) -> Vec<u8> {
        byteorder::I64::<TARGET>::new(*self).write_to_vec::<TARGET>()
    }

    #[inline]
    fn write_to_writer<TARGET: ByteOrder>(&self, writer: impl Write) -> Result<()> {
        byteorder::I64::<TARGET>::new(*self).write_to_writer::<TARGET>(writer)
    }
}

impl<O: ByteOrder> Writable for byteorder::I64<O> {
    fn write_to_vec<TARGET: ByteOrder>(&self) -> Vec<u8> {
        unsafe {
            let mut buf = Vec::<u8>::with_capacity(1 + 2 + 8);
            let buf_ptr = buf.as_mut_ptr();
            ptr::write(buf_ptr.cast(), [TagID::Long as u8, 0u8, 0u8]);
            ptr::write(
                buf_ptr.add(3).cast(),
                byteorder::I64::<TARGET>::new(self.get()).to_bytes(),
            );
            buf.set_len(1 + 2 + 8);
            buf
        }
    }

    fn write_to_writer<TARGET: ByteOrder>(&self, mut writer: impl Write) -> Result<()> {
        let mut buf = [0u8; 1 + 2 + 8];
        unsafe {
            ptr::write(buf.as_mut_ptr().cast(), [TagID::Long as u8, 0u8, 0u8]);
            ptr::write(
                buf.as_mut_ptr().add(3).cast(),
                byteorder::I64::<TARGET>::new(self.get()).to_bytes(),
            );
        }
        writer.write_all(&buf).map_err(Error::IO)
    }
}

impl Writable for f32 {
    #[inline]
    fn write_to_vec<TARGET: ByteOrder>(&self) -> Vec<u8> {
        byteorder::F32::<TARGET>::new(*self).write_to_vec::<TARGET>()
    }

    #[inline]
    fn write_to_writer<TARGET: ByteOrder>(&self, writer: impl Write) -> Result<()> {
        byteorder::F32::<TARGET>::new(*self).write_to_writer::<TARGET>(writer)
    }
}

impl<O: ByteOrder> Writable for byteorder::F32<O> {
    fn write_to_vec<TARGET: ByteOrder>(&self) -> Vec<u8> {
        unsafe {
            let mut buf = Vec::<u8>::with_capacity(1 + 2 + 4);
            let buf_ptr = buf.as_mut_ptr();
            ptr::write(buf_ptr.cast(), [TagID::Float as u8, 0u8, 0u8]);
            ptr::write(
                buf_ptr.add(3).cast(),
                byteorder::F32::<TARGET>::new(self.get()).to_bytes(),
            );
            buf.set_len(1 + 2 + 4);
            buf
        }
    }

    fn write_to_writer<TARGET: ByteOrder>(&self, mut writer: impl Write) -> Result<()> {
        let mut buf = [0u8; 1 + 2 + 4];
        unsafe {
            ptr::write(buf.as_mut_ptr().cast(), [TagID::Float as u8, 0u8, 0u8]);
            ptr::write(
                buf.as_mut_ptr().add(3).cast(),
                byteorder::F32::<TARGET>::new(self.get()).to_bytes(),
            );
        }
        writer.write_all(&buf).map_err(Error::IO)
    }
}

impl Writable for f64 {
    #[inline]
    fn write_to_vec<TARGET: ByteOrder>(&self) -> Vec<u8> {
        byteorder::F64::<TARGET>::new(*self).write_to_vec::<TARGET>()
    }

    #[inline]
    fn write_to_writer<TARGET: ByteOrder>(&self, writer: impl Write) -> Result<()> {
        byteorder::F64::<TARGET>::new(*self).write_to_writer::<TARGET>(writer)
    }
}

impl<O: ByteOrder> Writable for byteorder::F64<O> {
    fn write_to_vec<TARGET: ByteOrder>(&self) -> Vec<u8> {
        unsafe {
            let mut buf = Vec::<u8>::with_capacity(1 + 2 + 8);
            let buf_ptr = buf.as_mut_ptr();
            ptr::write(buf_ptr.cast(), [TagID::Double as u8, 0u8, 0u8]);
            ptr::write(
                buf_ptr.add(3).cast(),
                byteorder::F64::<TARGET>::new(self.get()).to_bytes(),
            );
            buf.set_len(1 + 2 + 8);
            buf
        }
    }

    fn write_to_writer<TARGET: ByteOrder>(&self, mut writer: impl Write) -> Result<()> {
        let mut buf = [0u8; 1 + 2 + 8];
        unsafe {
            ptr::write(buf.as_mut_ptr().cast(), [TagID::Double as u8, 0u8, 0u8]);
            ptr::write(
                buf.as_mut_ptr().add(3).cast(),
                byteorder::F64::<TARGET>::new(self.get()).to_bytes(),
            );
        }
        writer.write_all(&buf).map_err(Error::IO)
    }
}

impl Writable for [i8] {
    fn write_to_vec<TARGET: ByteOrder>(&self) -> Vec<u8> {
        unsafe {
            let payload = self.as_ptr().cast::<u8>();
            let len = self.len();
            let mut buf = Vec::<u8>::with_capacity(3 + 4 + len);
            let buf_ptr = buf.as_mut_ptr();
            ptr::write(buf_ptr.cast(), [TagID::ByteArray as u8, 0u8, 0u8]);
            ptr::write(
                buf_ptr.add(1 + 2).cast(),
                byteorder::U32::<TARGET>::new(len as u32).to_bytes(),
            );
            ptr::copy_nonoverlapping(payload, buf_ptr.add(1 + 2 + 4), len);
            buf.set_len(3 + 4 + len);
            buf
        }
    }

    fn write_to_writer<TARGET: ByteOrder>(&self, mut writer: impl Write) -> Result<()> {
        let mut buf_head = [0u8; 1 + 2 + 4];
        unsafe {
            ptr::write(
                buf_head.as_mut_ptr().cast(),
                [TagID::ByteArray as u8, 0u8, 0u8],
            );
            ptr::write(
                buf_head.as_mut_ptr().add(3).cast(),
                byteorder::U32::<TARGET>::new(self.len() as u32).to_bytes(),
            );
        }
        writer.write_all(&buf_head).map_err(Error::IO)?;
        let bytes = unsafe { slice::from_raw_parts(self.as_ptr().cast::<u8>(), self.len()) };
        writer.write_all(bytes).map_err(Error::IO)
    }
}

impl<const N: usize> Writable for [i8; N] {
    #[inline]
    fn write_to_vec<TARGET: ByteOrder>(&self) -> Vec<u8> {
        self.as_slice().write_to_vec::<TARGET>()
    }

    #[inline]
    fn write_to_writer<TARGET: ByteOrder>(&self, writer: impl Write) -> Result<()> {
        self.as_slice().write_to_writer::<TARGET>(writer)
    }
}

impl Writable for MUTF8Str {
    fn write_to_vec<TARGET: ByteOrder>(&self) -> Vec<u8> {
        unsafe {
            let bytes = self.as_bytes();
            let payload = bytes.as_ptr().cast::<u8>();
            let len = bytes.len();
            let mut buf = Vec::<u8>::with_capacity(3 + 2 + len);
            let buf_ptr = buf.as_mut_ptr();
            ptr::write(buf_ptr.cast(), [TagID::String as u8, 0u8, 0u8]);
            ptr::write(
                buf_ptr.add(1 + 2).cast(),
                byteorder::U16::<TARGET>::new(len as u16).to_bytes(),
            );
            ptr::copy_nonoverlapping(payload, buf_ptr.add(1 + 2 + 2), len);
            buf.set_len(3 + 2 + len);
            buf
        }
    }

    fn write_to_writer<TARGET: ByteOrder>(&self, mut writer: impl Write) -> Result<()> {
        let bytes = self.as_bytes();
        let mut buf_head = [0u8; 1 + 2 + 2];
        unsafe {
            ptr::write(
                buf_head.as_mut_ptr().cast(),
                [TagID::String as u8, 0u8, 0u8],
            );
            ptr::write(
                buf_head.as_mut_ptr().add(3).cast(),
                byteorder::U16::<TARGET>::new(bytes.len() as u16).to_bytes(),
            );
        }
        writer.write_all(&buf_head).map_err(Error::IO)?;
        writer.write_all(bytes).map_err(Error::IO)
    }
}

impl<O: ByteOrder> Writable for [byteorder::I32<O>] {
    fn write_to_vec<TARGET: ByteOrder>(&self) -> Vec<u8> {
        unsafe {
            let payload = self.as_ptr().cast::<u8>();
            let len = self.len();
            let len_bytes = std::mem::size_of_val(self);
            let mut buf = Vec::<u8>::with_capacity(3 + 4 + len_bytes);
            let mut buf_ptr = buf.as_mut_ptr();
            // head
            ptr::write(buf_ptr.cast(), [TagID::IntArray as u8, 0u8, 0u8]);
            // length
            ptr::write(
                buf_ptr.add(1 + 2).cast(),
                byteorder::U32::<TARGET>::new(len as u32).to_bytes(),
            );
            // data
            buf_ptr = buf_ptr.add(1 + 2 + 4);
            if TypeId::of::<O>() == TypeId::of::<TARGET>() {
                ptr::copy_nonoverlapping(payload, buf_ptr, len_bytes);
            } else {
                for element in self {
                    ptr::write(
                        buf_ptr.cast(),
                        byteorder::I32::<TARGET>::new(element.get()).to_bytes(),
                    );
                    buf_ptr = buf_ptr.add(4);
                }
            }
            buf.set_len(3 + 4 + len_bytes);
            buf
        }
    }

    fn write_to_writer<TARGET: ByteOrder>(&self, mut writer: impl Write) -> Result<()> {
        let mut buf_head = [0u8; 1 + 2 + 4];
        unsafe {
            ptr::write(
                buf_head.as_mut_ptr().cast(),
                [TagID::IntArray as u8, 0u8, 0u8],
            );
            ptr::write(
                buf_head.as_mut_ptr().add(3).cast(),
                byteorder::U32::<TARGET>::new(self.len() as u32).to_bytes(),
            );
        }
        writer.write_all(&buf_head).map_err(Error::IO)?;
        if TypeId::of::<O>() == TypeId::of::<TARGET>() {
            let bytes =
                unsafe { slice::from_raw_parts(self.as_ptr().cast::<u8>(), self.len() * 4) };
            writer.write_all(bytes).map_err(Error::IO)
        } else {
            for element in self {
                writer
                    .write_all(&byteorder::I32::<TARGET>::new(element.get()).to_bytes())
                    .map_err(Error::IO)?;
            }
            Ok(())
        }
    }
}

impl<O: ByteOrder, const N: usize> Writable for [byteorder::I32<O>; N] {
    #[inline]
    fn write_to_vec<TARGET: ByteOrder>(&self) -> Vec<u8> {
        self.as_slice().write_to_vec::<TARGET>()
    }

    #[inline]
    fn write_to_writer<TARGET: ByteOrder>(&self, writer: impl Write) -> Result<()> {
        self.as_slice().write_to_writer::<TARGET>(writer)
    }
}

impl<O: ByteOrder> Writable for [byteorder::I64<O>] {
    fn write_to_vec<TARGET: ByteOrder>(&self) -> Vec<u8> {
        unsafe {
            let payload = self.as_ptr().cast::<u8>();
            let len = self.len();
            let len_bytes = std::mem::size_of_val(self);
            let mut buf = Vec::<u8>::with_capacity(3 + 4 + len_bytes);
            let mut buf_ptr = buf.as_mut_ptr();
            // head
            ptr::write(buf_ptr.cast(), [TagID::LongArray as u8, 0u8, 0u8]);
            // length
            ptr::write(
                buf_ptr.add(1 + 2).cast(),
                byteorder::U32::<TARGET>::new(len as u32).to_bytes(),
            );
            // data
            buf_ptr = buf_ptr.add(1 + 2 + 4);
            if TypeId::of::<O>() == TypeId::of::<TARGET>() {
                ptr::copy_nonoverlapping(payload, buf_ptr, len_bytes);
            } else {
                for element in self {
                    ptr::write(
                        buf_ptr.cast(),
                        byteorder::I64::<TARGET>::new(element.get()).to_bytes(),
                    );
                    buf_ptr = buf_ptr.add(8);
                }
            }
            buf.set_len(3 + 4 + len_bytes);
            buf
        }
    }

    fn write_to_writer<TARGET: ByteOrder>(&self, mut writer: impl Write) -> Result<()> {
        let mut buf_head = [0u8; 1 + 2 + 4];
        unsafe {
            ptr::write(
                buf_head.as_mut_ptr().cast(),
                [TagID::LongArray as u8, 0u8, 0u8],
            );
            ptr::write(
                buf_head.as_mut_ptr().add(3).cast(),
                byteorder::U32::<TARGET>::new(self.len() as u32).to_bytes(),
            );
        }
        writer.write_all(&buf_head).map_err(Error::IO)?;
        if TypeId::of::<O>() == TypeId::of::<TARGET>() {
            let bytes =
                unsafe { slice::from_raw_parts(self.as_ptr().cast::<u8>(), self.len() * 8) };
            writer.write_all(bytes).map_err(Error::IO)
        } else {
            for element in self {
                writer
                    .write_all(&byteorder::I64::<TARGET>::new(element.get()).to_bytes())
                    .map_err(Error::IO)?;
            }
            Ok(())
        }
    }
}

impl<O: ByteOrder, const N: usize> Writable for [byteorder::I64<O>; N] {
    #[inline]
    fn write_to_vec<TARGET: ByteOrder>(&self) -> Vec<u8> {
        self.as_slice().write_to_vec::<TARGET>()
    }

    #[inline]
    fn write_to_writer<TARGET: ByteOrder>(&self, writer: impl Write) -> Result<()> {
        self.as_slice().write_to_writer::<TARGET>(writer)
    }
}
