use std::marker::PhantomData;

use zerocopy::byteorder;

use crate::{
    ByteOrder, ConfigMut, ConfigRef, MutString, MutVec, NBT, NBTBase, OwnCompound, OwnList,
    OwnString, OwnTypedList, OwnVec, PrimitiveNBTBase, TagID,
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
    type Element = End;
    type TypeRef<'a, Config: ConfigRef> = ();
    type TypeMut<'a, Config: ConfigMut> = &'a mut ();
    type Type<O: ByteOrder> = ();

    #[inline]
    fn dispatch<A, R>(
        a: A,
        end: impl FnOnce(A) -> R,
        _: impl FnOnce(A) -> R,
        _: impl FnOnce(A) -> R,
    ) -> R {
        end(a)
    }
}

impl NBTBase for Byte {
    const TAG_ID: TagID = TagID::Byte;
    type Element = Byte;
    type TypeRef<'a, Config: ConfigRef> = i8;
    type TypeMut<'a, Config: ConfigMut> = &'a mut i8;
    type Type<O: ByteOrder> = i8;

    #[inline]
    fn dispatch<A, R>(
        a: A,
        _: impl FnOnce(A) -> R,
        normal: impl FnOnce(A) -> R,
        _: impl FnOnce(A) -> R,
    ) -> R {
        normal(a)
    }
}

impl NBTBase for Short {
    const TAG_ID: TagID = TagID::Short;
    type Element = Short;
    type TypeRef<'a, Config: ConfigRef> = i16;
    type TypeMut<'a, Config: ConfigMut> = &'a mut byteorder::I16<Config::ByteOrder>;
    type Type<O: ByteOrder> = byteorder::I16<O>;

    #[inline]
    fn dispatch<A, R>(
        a: A,
        _: impl FnOnce(A) -> R,
        normal: impl FnOnce(A) -> R,
        _: impl FnOnce(A) -> R,
    ) -> R {
        normal(a)
    }
}

impl NBTBase for Int {
    const TAG_ID: TagID = TagID::Int;
    type Element = Int;
    type TypeRef<'a, Config: ConfigRef> = i32;
    type TypeMut<'a, Config: ConfigMut> = &'a mut byteorder::I32<Config::ByteOrder>;
    type Type<O: ByteOrder> = byteorder::I32<O>;

    #[inline]
    fn dispatch<A, R>(
        a: A,
        _: impl FnOnce(A) -> R,
        normal: impl FnOnce(A) -> R,
        _: impl FnOnce(A) -> R,
    ) -> R {
        normal(a)
    }
}

impl NBTBase for Long {
    const TAG_ID: TagID = TagID::Long;
    type Element = Long;
    type TypeRef<'a, Config: ConfigRef> = i64;
    type TypeMut<'a, Config: ConfigMut> = &'a mut byteorder::I64<Config::ByteOrder>;
    type Type<O: ByteOrder> = byteorder::I64<O>;

    #[inline]
    fn dispatch<A, R>(
        a: A,
        _: impl FnOnce(A) -> R,
        normal: impl FnOnce(A) -> R,
        _: impl FnOnce(A) -> R,
    ) -> R {
        normal(a)
    }
}

impl NBTBase for Float {
    const TAG_ID: TagID = TagID::Float;
    type Element = Float;
    type TypeRef<'a, Config: ConfigRef> = f32;
    type TypeMut<'a, Config: ConfigMut> = &'a mut byteorder::F32<Config::ByteOrder>;
    type Type<O: ByteOrder> = byteorder::F32<O>;

    #[inline]
    fn dispatch<A, R>(
        a: A,
        _: impl FnOnce(A) -> R,
        normal: impl FnOnce(A) -> R,
        _: impl FnOnce(A) -> R,
    ) -> R {
        normal(a)
    }
}

impl NBTBase for Double {
    const TAG_ID: TagID = TagID::Double;
    type Element = Double;
    type TypeRef<'a, Config: ConfigRef> = f64;
    type TypeMut<'a, Config: ConfigMut> = &'a mut byteorder::F64<Config::ByteOrder>;
    type Type<O: ByteOrder> = byteorder::F64<O>;

    #[inline]
    fn dispatch<A, R>(
        a: A,
        _: impl FnOnce(A) -> R,
        normal: impl FnOnce(A) -> R,
        _: impl FnOnce(A) -> R,
    ) -> R {
        normal(a)
    }
}

impl NBTBase for ByteArray {
    const TAG_ID: TagID = TagID::ByteArray;
    type Element = ByteArray;
    type TypeRef<'a, Config: ConfigRef> = Config::ByteArray<'a>;
    type TypeMut<'a, Config: ConfigMut> = MutVec<'a, i8>;
    type Type<O: ByteOrder> = OwnVec<i8>;

    #[inline]
    fn dispatch<A, R>(
        a: A,
        _: impl FnOnce(A) -> R,
        normal: impl FnOnce(A) -> R,
        _: impl FnOnce(A) -> R,
    ) -> R {
        normal(a)
    }
}

impl NBTBase for String {
    const TAG_ID: TagID = TagID::String;
    type Element = String;
    type TypeRef<'a, Config: ConfigRef> = Config::String<'a>;
    type TypeMut<'a, Config: ConfigMut> = MutString<'a>;
    type Type<O: ByteOrder> = OwnString;

    #[inline]
    fn dispatch<A, R>(
        a: A,
        _: impl FnOnce(A) -> R,
        normal: impl FnOnce(A) -> R,
        _: impl FnOnce(A) -> R,
    ) -> R {
        normal(a)
    }
}

impl NBTBase for List {
    const TAG_ID: TagID = TagID::List;
    type Element = List;
    type TypeRef<'a, Config: ConfigRef> = Config::List<'a>;
    type TypeMut<'a, Config: ConfigMut> = Config::ListMut<'a>;
    type Type<O: ByteOrder> = OwnList<O>;

    #[inline]
    fn dispatch<A, R>(
        a: A,
        _: impl FnOnce(A) -> R,
        normal: impl FnOnce(A) -> R,
        _: impl FnOnce(A) -> R,
    ) -> R {
        normal(a)
    }
}

impl NBTBase for Compound {
    const TAG_ID: TagID = TagID::Compound;
    type Element = Compound;
    type TypeRef<'a, Config: ConfigRef> = Config::Compound<'a>;
    type TypeMut<'a, Config: ConfigMut> = Config::CompoundMut<'a>;
    type Type<O: ByteOrder> = OwnCompound<O>;

    #[inline]
    fn dispatch<A, R>(
        a: A,
        _: impl FnOnce(A) -> R,
        normal: impl FnOnce(A) -> R,
        _: impl FnOnce(A) -> R,
    ) -> R {
        normal(a)
    }
}

impl NBTBase for IntArray {
    const TAG_ID: TagID = TagID::IntArray;
    type Element = IntArray;
    type TypeRef<'a, Config: ConfigRef> = Config::IntArray<'a>;
    type TypeMut<'a, Config: ConfigMut> = MutVec<'a, byteorder::I32<Config::ByteOrder>>;
    type Type<O: ByteOrder> = OwnVec<byteorder::I32<O>>;

    #[inline]
    fn dispatch<A, R>(
        a: A,
        _: impl FnOnce(A) -> R,
        normal: impl FnOnce(A) -> R,
        _: impl FnOnce(A) -> R,
    ) -> R {
        normal(a)
    }
}

impl NBTBase for LongArray {
    const TAG_ID: TagID = TagID::LongArray;
    type Element = LongArray;
    type TypeRef<'a, Config: ConfigRef> = Config::LongArray<'a>;
    type TypeMut<'a, Config: ConfigMut> = MutVec<'a, byteorder::I64<Config::ByteOrder>>;
    type Type<O: ByteOrder> = OwnVec<byteorder::I64<O>>;

    #[inline]
    fn dispatch<A, R>(
        a: A,
        _: impl FnOnce(A) -> R,
        normal: impl FnOnce(A) -> R,
        _: impl FnOnce(A) -> R,
    ) -> R {
        normal(a)
    }
}

#[derive(Clone, Copy)]
pub struct TypedList<T: NBTBase>(PhantomData<T>);

impl<T: NBT> NBTBase for TypedList<T> {
    const TAG_ID: TagID = TagID::List;
    type Element = T;
    type TypeRef<'a, Config: ConfigRef> = Config::TypedList<'a, T>;
    type TypeMut<'a, Config: ConfigMut> = Config::TypedListMut<'a, T>;
    type Type<O: ByteOrder> = OwnTypedList<O, T>;

    #[inline]
    fn dispatch<A, R>(
        a: A,
        _: impl FnOnce(A) -> R,
        _: impl FnOnce(A) -> R,
        typed_list: impl FnOnce(A) -> R,
    ) -> R {
        typed_list(a)
    }
}

macro_rules! primitive_tag {
    ($($name:ident),* $(,)?) => {
        $(
            impl PrimitiveNBTBase for $name {}
        )*
    };
}

primitive_tag!(End, Byte, Short, Int, Long, Float, Double);
