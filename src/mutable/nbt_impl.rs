use crate::{
    NBT, NBTBase,
    tag::{
        Byte, ByteArray, Compound, Double, End, Float, Int, IntArray, List, Long, LongArray, Short,
        String, TypedList,
    },
};

pub trait MutableGenericNBTImpl: NBTBase {}

pub trait MutableNBTImpl: MutableGenericNBTImpl {}

impl MutableGenericNBTImpl for End {}

impl MutableNBTImpl for End {}

impl MutableGenericNBTImpl for Byte {}

impl MutableNBTImpl for Byte {}

impl MutableGenericNBTImpl for Short {}

impl MutableNBTImpl for Short {}

impl MutableGenericNBTImpl for Int {}

impl MutableNBTImpl for Int {}

impl MutableGenericNBTImpl for Long {}

impl MutableNBTImpl for Long {}

impl MutableGenericNBTImpl for Float {}

impl MutableNBTImpl for Float {}

impl MutableGenericNBTImpl for Double {}

impl MutableNBTImpl for Double {}

impl MutableGenericNBTImpl for ByteArray {}

impl MutableNBTImpl for ByteArray {}

impl MutableGenericNBTImpl for String {}

impl MutableNBTImpl for String {}

impl MutableGenericNBTImpl for List {}

impl MutableNBTImpl for List {}

impl MutableGenericNBTImpl for Compound {}

impl MutableNBTImpl for Compound {}

impl MutableGenericNBTImpl for IntArray {}

impl MutableNBTImpl for IntArray {}

impl MutableGenericNBTImpl for LongArray {}

impl MutableNBTImpl for LongArray {}

impl<T: NBT> MutableGenericNBTImpl for TypedList<T> {}
