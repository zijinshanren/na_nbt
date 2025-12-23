use std::marker::PhantomData;

use crate::{ConfigRef, NBT, NBTBase, PrimitiveNBTBase, TagID};

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
}

impl NBTBase for Byte {
    const TAG_ID: TagID = TagID::Byte;
    type Type<'a, Config: ConfigRef> = i8;
}

impl NBTBase for Short {
    const TAG_ID: TagID = TagID::Short;
    type Type<'a, Config: ConfigRef> = i16;
}

impl NBTBase for Int {
    const TAG_ID: TagID = TagID::Int;
    type Type<'a, Config: ConfigRef> = i32;
}

impl NBTBase for Long {
    const TAG_ID: TagID = TagID::Long;
    type Type<'a, Config: ConfigRef> = i64;
}

impl NBTBase for Float {
    const TAG_ID: TagID = TagID::Float;
    type Type<'a, Config: ConfigRef> = f32;
}

impl NBTBase for Double {
    const TAG_ID: TagID = TagID::Double;
    type Type<'a, Config: ConfigRef> = f64;
}

impl NBTBase for ByteArray {
    const TAG_ID: TagID = TagID::ByteArray;
    type Type<'a, Config: ConfigRef> = Config::ByteArray<'a>;
}

impl NBTBase for String {
    const TAG_ID: TagID = TagID::String;
    type Type<'a, Config: ConfigRef> = Config::String<'a>;
}

impl NBTBase for List {
    const TAG_ID: TagID = TagID::List;
    type Type<'a, Config: ConfigRef> = Config::List<'a>;
}

impl NBTBase for Compound {
    const TAG_ID: TagID = TagID::Compound;
    type Type<'a, Config: ConfigRef> = Config::Compound<'a>;
}

impl NBTBase for IntArray {
    const TAG_ID: TagID = TagID::IntArray;
    type Type<'a, Config: ConfigRef> = Config::IntArray<'a>;
}

impl NBTBase for LongArray {
    const TAG_ID: TagID = TagID::LongArray;
    type Type<'a, Config: ConfigRef> = Config::LongArray<'a>;
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
