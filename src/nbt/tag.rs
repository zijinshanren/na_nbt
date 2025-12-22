use crate::{NBTBase, PrimitiveNBTBase, ReadableConfig, TagID};

macro_rules! define_primary_tag {
    ($($name:ident),* $(,)?) => {
        $(
            #[derive(Clone, Copy)]
            pub struct $name;
        )*
    };
}

define_primary_tag!(
    TagEnd,
    TagByte,
    TagShort,
    TagInt,
    TagLong,
    TagFloat,
    TagDouble,
    TagByteArray,
    TagString,
    TagList,
    TagCompound,
    TagIntArray,
    TagLongArray
);

impl NBTBase for TagEnd {
    const TAG_ID: TagID = TagID::End;
    type Type<'a, Config: ReadableConfig> = ();
}

impl NBTBase for TagByte {
    const TAG_ID: TagID = TagID::Byte;
    type Type<'a, Config: ReadableConfig> = i8;
}

impl NBTBase for TagShort {
    const TAG_ID: TagID = TagID::Short;
    type Type<'a, Config: ReadableConfig> = i16;
}

impl NBTBase for TagInt {
    const TAG_ID: TagID = TagID::Int;
    type Type<'a, Config: ReadableConfig> = i32;
}

impl NBTBase for TagLong {
    const TAG_ID: TagID = TagID::Long;
    type Type<'a, Config: ReadableConfig> = i64;
}

impl NBTBase for TagFloat {
    const TAG_ID: TagID = TagID::Float;
    type Type<'a, Config: ReadableConfig> = f32;
}

impl NBTBase for TagDouble {
    const TAG_ID: TagID = TagID::Double;
    type Type<'a, Config: ReadableConfig> = f64;
}

impl NBTBase for TagByteArray {
    const TAG_ID: TagID = TagID::ByteArray;
    type Type<'a, Config: ReadableConfig> = Config::ByteArray<'a>;
}

impl NBTBase for TagString {
    const TAG_ID: TagID = TagID::String;
    type Type<'a, Config: ReadableConfig> = Config::String<'a>;
}

impl NBTBase for TagList {
    const TAG_ID: TagID = TagID::List;
    type Type<'a, Config: ReadableConfig> = Config::List<'a>;
}

impl NBTBase for TagCompound {
    const TAG_ID: TagID = TagID::Compound;
    type Type<'a, Config: ReadableConfig> = Config::Compound<'a>;
}

impl NBTBase for TagIntArray {
    const TAG_ID: TagID = TagID::IntArray;
    type Type<'a, Config: ReadableConfig> = Config::IntArray<'a>;
}

impl NBTBase for TagLongArray {
    const TAG_ID: TagID = TagID::LongArray;
    type Type<'a, Config: ReadableConfig> = Config::LongArray<'a>;
}

macro_rules! primitive_tag {
    ($($name:ident),* $(,)?) => {
        $(
            impl PrimitiveNBTBase for $name {}
        )*
    };
}

primitive_tag!(
    TagEnd, TagByte, TagShort, TagInt, TagLong, TagFloat, TagDouble
);
