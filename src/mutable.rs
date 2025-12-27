mod compound_mut;
mod compound_own;
mod compound_ref;
mod config;
mod into_nbt;
mod list_mut;
mod list_own;
mod list_ref;
mod nbt_impl;
mod read;
mod size;
mod string_ref;
mod typed_list_mut;
mod typed_list_own;
mod typed_list_ref;
mod value_mut;
mod value_own;
mod value_ref;
// mod write;

use std::{
    any::TypeId,
    io::{BufRead, BufReader, Read},
};

pub use compound_mut::*;
pub use compound_own::*;
pub use compound_ref::*;
pub use config::*;
pub use into_nbt::*;
pub use list_mut::*;
pub use list_own::*;
pub use list_ref::*;
pub use read::*;
pub use size::*;
pub use string_ref::*;
pub use typed_list_mut::*;
pub use typed_list_own::*;
pub use typed_list_ref::*;
pub use value_mut::*;
pub use value_own::*;
pub use value_ref::*;
// pub use write::*;
use zerocopy::byteorder;

use crate::{ByteOrder, Error, Result, cold_path};

pub fn read_owned<SOURCE: ByteOrder, STORE: ByteOrder>(source: &[u8]) -> Result<OwnValue<STORE>> {
    unsafe {
        macro_rules! check_bounds {
            ($required:expr) => {
                if source.len() < $required {
                    cold_path();
                    return Err(Error::EOF);
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
            return Ok(OwnValue::End(()));
        }

        check_bounds!(1 + 2);
        let name_len = byteorder::U16::<SOURCE>::from_bytes(*current_pos.cast()).get();

        check_bounds!(1 + 2 + name_len as usize);
        current_pos = current_pos.add(2 + name_len as usize);

        let value = if TypeId::of::<SOURCE>() == TypeId::of::<STORE>() {
            let result = read_unsafe::<SOURCE>(tag_id, &mut current_pos, end_pos)?;
            Ok(std::mem::transmute::<OwnValue<SOURCE>, OwnValue<STORE>>(
                result,
            ))
        } else {
            read_unsafe_fallback::<SOURCE, STORE>(tag_id, &mut current_pos, end_pos)
        }?;

        if current_pos < end_pos {
            cold_path();
            return Err(Error::REMAIN(
                end_pos.byte_offset_from_unsigned(current_pos),
            ));
        }

        Ok(value)
    }
}

pub fn read_owned_from_reader<SOURCE: ByteOrder, STORE: ByteOrder>(
    reader: impl Read,
) -> Result<OwnValue<STORE>> {
    unsafe {
        let mut reader = BufReader::new(reader);

        let mut tag_id = [0u8];
        reader.read_exact(&mut tag_id).map_err(Error::IO)?;
        let tag_id = tag_id[0];

        if tag_id == 0 {
            cold_path();
            return Ok(OwnValue::End(()));
        }

        let mut name_len = [0u8; 2];
        reader.read_exact(&mut name_len).map_err(Error::IO)?;
        let name_len = byteorder::U16::<SOURCE>::from_bytes(name_len).get() as usize;
        {
            let mut skipped = 0;
            while skipped < name_len {
                let buf_len = reader.fill_buf().map_err(Error::IO)?.len();
                if buf_len == 0 {
                    cold_path();
                    return Err(Error::EOF);
                }
                let read = std::cmp::min(buf_len, name_len - skipped);
                reader.consume(read);
                skipped += read;
            }
        }

        let value = read_unsafe_from_reader::<SOURCE, STORE>(tag_id, &mut reader)?;

        {
            let remaining = reader.fill_buf().map_err(Error::IO)?.len();
            if remaining > 0 {
                cold_path();
                return Err(Error::REMAIN(remaining));
            }
        }

        Ok(value)
    }
}
