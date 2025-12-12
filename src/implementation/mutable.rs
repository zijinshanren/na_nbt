mod into_owned_value;
mod iter;
mod read;
mod trait_impl;
mod trait_impl_mut;
mod trait_impl_own;
mod util;
mod value;
mod value_mut;
mod value_own;
// mod write;

use std::any::TypeId;

pub(crate) use into_owned_value::IntoOwnedValue;
pub use value::{ImmutableCompound, ImmutableList, ImmutableString, ImmutableValue};
pub use value_mut::{MutableCompound, MutableList, MutableValue};
pub use value_own::{OwnedCompound, OwnedList, OwnedValue};

use zerocopy::byteorder;

use crate::{
    ByteOrder, Error, Result, cold_path,
    implementation::mutable::read::{read_unsafe, read_unsafe_fallback},
};

// pub fn write<'s, O: ByteOrder>(value: ImmutableValue<'s, O>) -> Result<Vec<u8>> {
//     let mut result = Vec::new();
//     unsafe {
//         match value {
//             ImmutableValue::End => result.push(0),
//             ImmutableValue::Byte(value) => {
//                 write_head::<O>(1, "", &mut result)?;
//                 result.push(value as u8);
//             }
//             ImmutableValue::Short(value) => {
//                 write_head::<O>(2, "", &mut result)?;
//                 result.extend_from_slice(&byteorder::I16::<O>::from(value).to_bytes());
//             }
//             ImmutableValue::Int(value) => {
//                 write_head::<O>(3, "", &mut result)?;
//                 result.extend_from_slice(&byteorder::I32::<O>::from(value).to_bytes());
//             }
//             ImmutableValue::Long(value) => {
//                 write_head::<O>(4, "", &mut result)?;
//                 result.extend_from_slice(&byteorder::I64::<O>::from(value).to_bytes());
//             }
//             ImmutableValue::Float(value) => {
//                 write_head::<O>(5, "", &mut result)?;
//                 result.extend_from_slice(&byteorder::F32::<O>::from(value).to_bytes());
//             }
//             ImmutableValue::Double(value) => {
//                 write_head::<O>(6, "", &mut result)?;
//                 result.extend_from_slice(&byteorder::F64::<O>::from(value).to_bytes());
//             }
//             ImmutableValue::ByteArray(value) => {
//                 write_head::<O>(7, "", &mut result)?;
//                 write_byte_array::<O>(value, &mut result)?;
//             }
//             ImmutableValue::String(value) => {
//                 write_head::<O>(8, "", &mut result)?;
//                 write_string::<O>(value.raw_bytes(), &mut result)?;
//             }
//             ImmutableValue::List(value) => {
//                 write_head::<O>(9, "", &mut result)?;
//                 write_list::<O>(value.data, &mut result)?;
//             }
//             ImmutableValue::Compound(value) => {
//                 write_head::<O>(10, "", &mut result)?;
//                 write_compound::<O>(value.data, &mut result)?;
//             }
//             ImmutableValue::IntArray(value) => {
//                 write_head::<O>(11, "", &mut result)?;
//                 result.extend_from_slice(&byteorder::U32::<O>::from(value.len() as u32).to_bytes());
//                 result.extend_from_slice(value.as_bytes());
//             }
//             ImmutableValue::LongArray(value) => {
//                 write_head::<O>(12, "", &mut result)?;
//                 result.extend_from_slice(&byteorder::U32::<O>::from(value.len() as u32).to_bytes());
//                 result.extend_from_slice(value.as_bytes());
//             }
//         }
//     }
//     Ok(result)
// }

// pub fn write_owned<O: ByteOrder>(value: &OwnedValue<O>) -> Result<Vec<u8>> {
//     let mut result = Vec::new();
//     unsafe {
//         match value {
//             OwnedValue::End => result.push(0),
//             OwnedValue::Byte(value) => {
//                 write_head::<O>(1, "", &mut result)?;
//                 result.push(*value as u8);
//             }
//             OwnedValue::Short(value) => {
//                 write_head::<O>(2, "", &mut result)?;
//                 result.extend_from_slice(&value.to_bytes());
//             }
//             OwnedValue::Int(value) => {
//                 write_head::<O>(3, "", &mut result)?;
//                 result.extend_from_slice(&value.to_bytes());
//             }
//             OwnedValue::Long(value) => {
//                 write_head::<O>(4, "", &mut result)?;
//                 result.extend_from_slice(&value.to_bytes());
//             }
//             OwnedValue::Float(value) => {
//                 write_head::<O>(5, "", &mut result)?;
//                 result.extend_from_slice(&value.to_bytes());
//             }
//             OwnedValue::Double(value) => {
//                 write_head::<O>(6, "", &mut result)?;
//                 result.extend_from_slice(&value.to_bytes());
//             }
//             OwnedValue::ByteArray(value) => {
//                 write_head::<O>(7, "", &mut result)?;
//                 write_byte_array::<O>(value, &mut result)?;
//             }
//             OwnedValue::String(value) => {
//                 write_head::<O>(8, "", &mut result)?;
//                 write_string::<O>(value.as_mutf8_bytes(), &mut result)?;
//             }
//             OwnedValue::List(value) => {
//                 write_head::<O>(9, "", &mut result)?;
//                 write_list::<O>(value.data.as_ptr(), &mut result)?;
//             }
//             OwnedValue::Compound(value) => {
//                 write_head::<O>(10, "", &mut result)?;
//                 write_compound::<O>(value.data.as_ptr(), &mut result)?;
//             }
//             OwnedValue::IntArray(value) => {
//                 write_head::<O>(11, "", &mut result)?;
//                 result.extend_from_slice(&byteorder::U32::<O>::from(value.len() as u32).to_bytes());
//                 result.extend_from_slice(value.as_bytes());
//             }
//             OwnedValue::LongArray(value) => {
//                 write_head::<O>(12, "", &mut result)?;
//                 result.extend_from_slice(&byteorder::U32::<O>::from(value.len() as u32).to_bytes());
//                 result.extend_from_slice(value.as_bytes());
//             }
//         }
//     }
//     Ok(result)
// }

#[inline]
pub fn read_owned<SOURCE: ByteOrder, TARGET: ByteOrder>(
    source: &[u8],
) -> Result<OwnedValue<TARGET>> {
    unsafe {
        macro_rules! check_bounds {
            ($required:expr) => {
                if source.len() < $required {
                    cold_path();
                    return Err(Error::EndOfFile);
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
            return Ok(OwnedValue::End);
        }

        check_bounds!(1 + 2);
        let name_len = byteorder::U16::<SOURCE>::from_bytes(*current_pos.cast()).get();

        check_bounds!(1 + 2 + name_len as usize);
        current_pos = current_pos.add(2 + name_len as usize);

        let value = if TypeId::of::<SOURCE>() == TypeId::of::<TARGET>() {
            let result = read_unsafe::<SOURCE>(tag_id, &mut current_pos, end_pos)?;
            Ok(std::mem::transmute::<OwnedValue<SOURCE>, OwnedValue<TARGET>>(result))
        } else {
            read_unsafe_fallback::<SOURCE, TARGET>(tag_id, &mut current_pos, end_pos)
        }?;

        if current_pos < end_pos {
            cold_path();
            return Err(Error::TrailingData(
                end_pos.byte_offset_from_unsigned(current_pos),
            ));
        }

        Ok(value)
    }
}
