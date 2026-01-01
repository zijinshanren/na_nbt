use std::{marker::PhantomData, mem::ManuallyDrop, ptr};

use zerocopy::byteorder;

use crate::{
    ByteOrder, CompoundOwn, ListOwn, NBT, NBTBase, StringOwn, TypedListOwn, VecOwn,
    tag::{
        Byte, ByteArray, Compound, Double, Float, Int, IntArray, List, Long, LongArray, Short,
        String,
    },
};

pub trait IntoNBT<O: ByteOrder> {
    type Tag: NBT;

    fn into_nbt(self) -> <Self::Tag as NBTBase>::Type<O>;
}

impl<O: ByteOrder> IntoNBT<O> for i8 {
    type Tag = Byte;

    #[inline]
    fn into_nbt(self) -> <Self::Tag as NBTBase>::Type<O> {
        self
    }
}

impl<O: ByteOrder> IntoNBT<O> for i16 {
    type Tag = Short;

    #[inline]
    fn into_nbt(self) -> <Self::Tag as NBTBase>::Type<O> {
        byteorder::I16::<O>::new(self)
    }
}

impl<O: ByteOrder, R: ByteOrder> IntoNBT<O> for byteorder::I16<R> {
    type Tag = Short;

    #[inline]
    fn into_nbt(self) -> <Self::Tag as NBTBase>::Type<O> {
        byteorder::I16::<O>::new(self.get())
    }
}

impl<O: ByteOrder> IntoNBT<O> for i32 {
    type Tag = Int;

    #[inline]
    fn into_nbt(self) -> <Self::Tag as NBTBase>::Type<O> {
        byteorder::I32::<O>::new(self)
    }
}

impl<O: ByteOrder, R: ByteOrder> IntoNBT<O> for byteorder::I32<R> {
    type Tag = Int;

    #[inline]
    fn into_nbt(self) -> <Self::Tag as NBTBase>::Type<O> {
        byteorder::I32::<O>::new(self.get())
    }
}

impl<O: ByteOrder> IntoNBT<O> for i64 {
    type Tag = Long;

    #[inline]
    fn into_nbt(self) -> <Self::Tag as NBTBase>::Type<O> {
        byteorder::I64::<O>::new(self)
    }
}

impl<O: ByteOrder, R: ByteOrder> IntoNBT<O> for byteorder::I64<R> {
    type Tag = Long;

    #[inline]
    fn into_nbt(self) -> <Self::Tag as NBTBase>::Type<O> {
        byteorder::I64::<O>::new(self.get())
    }
}

impl<O: ByteOrder> IntoNBT<O> for f32 {
    type Tag = Float;

    #[inline]
    fn into_nbt(self) -> <Self::Tag as NBTBase>::Type<O> {
        byteorder::F32::<O>::new(self)
    }
}

impl<O: ByteOrder, R: ByteOrder> IntoNBT<O> for byteorder::F32<R> {
    type Tag = Float;

    #[inline]
    fn into_nbt(self) -> <Self::Tag as NBTBase>::Type<O> {
        byteorder::F32::<O>::new(self.get())
    }
}

impl<O: ByteOrder> IntoNBT<O> for f64 {
    type Tag = Double;

    #[inline]
    fn into_nbt(self) -> <Self::Tag as NBTBase>::Type<O> {
        byteorder::F64::<O>::new(self)
    }
}

impl<O: ByteOrder, R: ByteOrder> IntoNBT<O> for byteorder::F64<R> {
    type Tag = Double;

    #[inline]
    fn into_nbt(self) -> <Self::Tag as NBTBase>::Type<O> {
        byteorder::F64::<O>::new(self.get())
    }
}

impl<O: ByteOrder> IntoNBT<O> for Vec<i8> {
    type Tag = ByteArray;

    #[inline]
    fn into_nbt(self) -> <Self::Tag as NBTBase>::Type<O> {
        self.into()
    }
}

impl<O: ByteOrder> IntoNBT<O> for &[i8] {
    type Tag = ByteArray;

    #[inline]
    fn into_nbt(self) -> <Self::Tag as NBTBase>::Type<O> {
        self.into()
    }
}

impl<O: ByteOrder, const N: usize> IntoNBT<O> for [i8; N] {
    type Tag = ByteArray;

    #[inline]
    fn into_nbt(self) -> <Self::Tag as NBTBase>::Type<O> {
        self.as_slice().into()
    }
}

impl<O: ByteOrder> IntoNBT<O> for VecOwn<i8> {
    type Tag = ByteArray;

    #[inline]
    fn into_nbt(self) -> <Self::Tag as NBTBase>::Type<O> {
        self
    }
}

impl<O: ByteOrder> IntoNBT<O> for std::string::String {
    type Tag = String;

    #[inline]
    fn into_nbt(self) -> <Self::Tag as NBTBase>::Type<O> {
        self.into()
    }
}

impl<O: ByteOrder> IntoNBT<O> for &str {
    type Tag = String;

    #[inline]
    fn into_nbt(self) -> <Self::Tag as NBTBase>::Type<O> {
        self.into()
    }
}

impl<O: ByteOrder> IntoNBT<O> for StringOwn {
    type Tag = String;

    #[inline]
    fn into_nbt(self) -> <Self::Tag as NBTBase>::Type<O> {
        self
    }
}

impl<O: ByteOrder> IntoNBT<O> for ListOwn<O> {
    type Tag = List;

    #[inline]
    fn into_nbt(self) -> <Self::Tag as NBTBase>::Type<O> {
        self
    }
}

impl<O: ByteOrder, T: NBT> IntoNBT<O> for TypedListOwn<O, T> {
    type Tag = List;

    #[inline]
    fn into_nbt(self) -> <Self::Tag as NBTBase>::Type<O> {
        let me = ManuallyDrop::new(self);
        ListOwn {
            data: unsafe { ptr::read(&me.data) },
            _marker: PhantomData,
        }
    }
}

impl<O: ByteOrder> IntoNBT<O> for CompoundOwn<O> {
    type Tag = Compound;

    #[inline]
    fn into_nbt(self) -> <Self::Tag as NBTBase>::Type<O> {
        self
    }
}

impl<O: ByteOrder> IntoNBT<O> for Vec<byteorder::I32<O>> {
    type Tag = IntArray;

    #[inline]
    fn into_nbt(self) -> <Self::Tag as NBTBase>::Type<O> {
        self.into()
    }
}

impl<O: ByteOrder> IntoNBT<O> for &[byteorder::I32<O>] {
    type Tag = IntArray;

    #[inline]
    fn into_nbt(self) -> <Self::Tag as NBTBase>::Type<O> {
        self.into()
    }
}

impl<O: ByteOrder, const N: usize> IntoNBT<O> for [byteorder::I32<O>; N] {
    type Tag = IntArray;

    #[inline]
    fn into_nbt(self) -> <Self::Tag as NBTBase>::Type<O> {
        self.as_slice().into()
    }
}

impl<O: ByteOrder> IntoNBT<O> for VecOwn<byteorder::I32<O>> {
    type Tag = IntArray;

    #[inline]
    fn into_nbt(self) -> <Self::Tag as NBTBase>::Type<O> {
        self
    }
}

impl<O: ByteOrder> IntoNBT<O> for Vec<byteorder::I64<O>> {
    type Tag = LongArray;

    #[inline]
    fn into_nbt(self) -> <Self::Tag as NBTBase>::Type<O> {
        self.into()
    }
}

impl<O: ByteOrder> IntoNBT<O> for &[byteorder::I64<O>] {
    type Tag = LongArray;

    #[inline]
    fn into_nbt(self) -> <Self::Tag as NBTBase>::Type<O> {
        self.into()
    }
}

impl<O: ByteOrder, const N: usize> IntoNBT<O> for [byteorder::I64<O>; N] {
    type Tag = LongArray;

    #[inline]
    fn into_nbt(self) -> <Self::Tag as NBTBase>::Type<O> {
        self.as_slice().into()
    }
}

impl<O: ByteOrder> IntoNBT<O> for VecOwn<byteorder::I64<O>> {
    type Tag = LongArray;

    #[inline]
    fn into_nbt(self) -> <Self::Tag as NBTBase>::Type<O> {
        self
    }
}
