mod into_owned_value;
mod iter;
mod trait_impl;
mod trait_impl_mut;
mod trait_impl_own;
mod util;
mod value;
mod value_mut;
mod value_own;
mod write;

pub use into_owned_value::IntoOwnedValue;
pub use value::{ImmutableCompound, ImmutableList, ImmutableString, ImmutableValue, Name};
pub use value_mut::{MutableCompound, MutableList, MutableValue};
pub use value_own::{OwnedCompound, OwnedList, OwnedValue};

use zerocopy::{IntoBytes as _, byteorder};

use crate::{
    NbtError,
    implementation::mutable::write::{
        write_byte_array, write_compound, write_head, write_list, write_string,
    },
    util::ByteOrder,
};

pub fn write<'s, O: ByteOrder>(value: ImmutableValue<'s, O>) -> Result<Vec<u8>, NbtError> {
    let mut result = Vec::new();
    unsafe {
        match value {
            ImmutableValue::End => result.push(0),
            ImmutableValue::Byte(value) => {
                write_head::<O>(1, "", &mut result)?;
                result.push(value as u8);
            }
            ImmutableValue::Short(value) => {
                write_head::<O>(2, "", &mut result)?;
                result.extend_from_slice(&byteorder::I16::<O>::from(value).to_bytes());
            }
            ImmutableValue::Int(value) => {
                write_head::<O>(3, "", &mut result)?;
                result.extend_from_slice(&byteorder::I32::<O>::from(value).to_bytes());
            }
            ImmutableValue::Long(value) => {
                write_head::<O>(4, "", &mut result)?;
                result.extend_from_slice(&byteorder::I64::<O>::from(value).to_bytes());
            }
            ImmutableValue::Float(value) => {
                write_head::<O>(5, "", &mut result)?;
                result.extend_from_slice(&byteorder::F32::<O>::from(value).to_bytes());
            }
            ImmutableValue::Double(value) => {
                write_head::<O>(6, "", &mut result)?;
                result.extend_from_slice(&byteorder::F64::<O>::from(value).to_bytes());
            }
            ImmutableValue::ByteArray(value) => {
                write_head::<O>(7, "", &mut result)?;
                write_byte_array::<O>(value, &mut result)?;
            }
            ImmutableValue::String(value) => {
                write_head::<O>(8, "", &mut result)?;
                write_string::<O>(value.raw_bytes(), &mut result)?;
            }
            ImmutableValue::List(value) => {
                write_head::<O>(9, "", &mut result)?;
                write_list::<O>(value.data, &mut result)?;
            }
            ImmutableValue::Compound(value) => {
                write_head::<O>(10, "", &mut result)?;
                write_compound::<O>(value.data, &mut result)?;
            }
            ImmutableValue::IntArray(value) => {
                write_head::<O>(11, "", &mut result)?;
                result.extend_from_slice(&byteorder::U32::<O>::from(value.len() as u32).to_bytes());
                result.extend_from_slice(value.as_bytes());
            }
            ImmutableValue::LongArray(value) => {
                write_head::<O>(12, "", &mut result)?;
                result.extend_from_slice(&byteorder::U32::<O>::from(value.len() as u32).to_bytes());
                result.extend_from_slice(value.as_bytes());
            }
        }
    }
    Ok(result)
}
