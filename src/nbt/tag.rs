use std::marker::PhantomData;

use zerocopy::byteorder;

use crate::{
    ConfigMut, ConfigRef, NBT, NBTBase, PrimitiveNBTBase, StringViewMut, TagID, VecViewMut,
};

macro_rules! define_primary_tag {
    ($($name:ident),* $(,)?) => {
        $(
            #[derive(Clone, Copy)]
            pub struct $name;
        )*
    };
}

define_primary_tag!(
    End, Byte, Short, Int, Long, Float, Double, ByteArray, String, List, Compound, IntArray,
    LongArray
);

impl NBTBase for End {
    const TAG_ID: TagID = TagID::End;
    type Type<'a, Config: ConfigRef> = ();
    type TypeMut<'a, Config: ConfigMut> = &'a mut ();
}

impl NBTBase for Byte {
    const TAG_ID: TagID = TagID::Byte;
    type Type<'a, Config: ConfigRef> = i8;
    type TypeMut<'a, Config: ConfigMut> = &'a mut i8;
}

impl NBTBase for Short {
    const TAG_ID: TagID = TagID::Short;
    type Type<'a, Config: ConfigRef> = i16;
    type TypeMut<'a, Config: ConfigMut> = &'a mut byteorder::I16<Config::ByteOrder>;
}

impl NBTBase for Int {
    const TAG_ID: TagID = TagID::Int;
    type Type<'a, Config: ConfigRef> = i32;
    type TypeMut<'a, Config: ConfigMut> = &'a mut byteorder::I32<Config::ByteOrder>;
}

impl NBTBase for Long {
    const TAG_ID: TagID = TagID::Long;
    type Type<'a, Config: ConfigRef> = i64;
    type TypeMut<'a, Config: ConfigMut> = &'a mut byteorder::I64<Config::ByteOrder>;
}

impl NBTBase for Float {
    const TAG_ID: TagID = TagID::Float;
    type Type<'a, Config: ConfigRef> = f32;
    type TypeMut<'a, Config: ConfigMut> = &'a mut byteorder::F32<Config::ByteOrder>;
}

impl NBTBase for Double {
    const TAG_ID: TagID = TagID::Double;
    type Type<'a, Config: ConfigRef> = f64;
    type TypeMut<'a, Config: ConfigMut> = &'a mut byteorder::F64<Config::ByteOrder>;
}

impl NBTBase for ByteArray {
    const TAG_ID: TagID = TagID::ByteArray;
    type Type<'a, Config: ConfigRef> = Config::ByteArray<'a>;
    type TypeMut<'a, Config: ConfigMut> = VecViewMut<'a, i8>;
}

impl NBTBase for String {
    const TAG_ID: TagID = TagID::String;
    type Type<'a, Config: ConfigRef> = Config::String<'a>;
    type TypeMut<'a, Config: ConfigMut> = StringViewMut<'a>;
}

impl NBTBase for List {
    const TAG_ID: TagID = TagID::List;
    type Type<'a, Config: ConfigRef> = Config::List<'a>;
    type TypeMut<'a, Config: ConfigMut> = Config::ListMut<'a>;
}

impl NBTBase for Compound {
    const TAG_ID: TagID = TagID::Compound;
    type Type<'a, Config: ConfigRef> = Config::Compound<'a>;
    type TypeMut<'a, Config: ConfigMut> = Config::CompoundMut<'a>;
}

impl NBTBase for IntArray {
    const TAG_ID: TagID = TagID::IntArray;
    type Type<'a, Config: ConfigRef> = Config::IntArray<'a>;
    type TypeMut<'a, Config: ConfigMut> = VecViewMut<'a, byteorder::I32<Config::ByteOrder>>;
}

impl NBTBase for LongArray {
    const TAG_ID: TagID = TagID::LongArray;
    type Type<'a, Config: ConfigRef> = Config::LongArray<'a>;
    type TypeMut<'a, Config: ConfigMut> = VecViewMut<'a, byteorder::I64<Config::ByteOrder>>;
}

#[derive(Clone, Copy)]
pub struct TypedList<T: NBT>(PhantomData<T>);

impl<T: NBT> NBTBase for TypedList<T> {
    const TAG_ID: TagID = TagID::List;
    type Type<'a, Config: ConfigRef> = Config::TypedList<'a, T>;
}

macro_rules! primitive_tag {
    ($($name:ident),* $(,)?) => {
        $(
            impl PrimitiveNBTBase for $name {}
        )*
    };
}

primitive_tag!(End, Byte, Short, Int, Long, Float, Double);
