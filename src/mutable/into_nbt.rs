use zerocopy::byteorder;

use crate::{
    ByteOrder, NBT, NBTBase, OwnCompound, OwnList, OwnString, OwnVec,
    tag::{
        Byte, ByteArray, Compound, Double, Float, Int, IntArray, List, Long, LongArray, Short,
        String,
    },
};

pub trait IntoNBT<O: ByteOrder>: Into<<Self::Tag as NBTBase>::Type<O>> {
    type Tag: NBT;
}

impl<O: ByteOrder> IntoNBT<O> for i8 {
    type Tag = Byte;
}

impl<O: ByteOrder> IntoNBT<O> for byteorder::I16<O> {
    type Tag = Short;
}

impl<O: ByteOrder> IntoNBT<O> for byteorder::I32<O> {
    type Tag = Int;
}

impl<O: ByteOrder> IntoNBT<O> for byteorder::I64<O> {
    type Tag = Long;
}

impl<O: ByteOrder> IntoNBT<O> for byteorder::F32<O> {
    type Tag = Float;
}

impl<O: ByteOrder> IntoNBT<O> for byteorder::F64<O> {
    type Tag = Double;
}

impl<O: ByteOrder> IntoNBT<O> for OwnVec<i8> {
    type Tag = ByteArray;
}

impl<O: ByteOrder> IntoNBT<O> for OwnString {
    type Tag = String;
}

impl<O: ByteOrder> IntoNBT<O> for OwnList<O> {
    type Tag = List;
}

impl<O: ByteOrder> IntoNBT<O> for OwnCompound<O> {
    type Tag = Compound;
}

impl<O: ByteOrder> IntoNBT<O> for OwnVec<byteorder::I32<O>> {
    type Tag = IntArray;
}

impl<O: ByteOrder> IntoNBT<O> for OwnVec<byteorder::I64<O>> {
    type Tag = LongArray;
}
