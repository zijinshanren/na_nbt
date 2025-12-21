use std::ops::Deref;

use zerocopy::byteorder;

use crate::{
    ByteOrder, ReadableByteArrayList, ReadableByteList, ReadableCompound, ReadableCompoundList,
    ReadableDoubleList, ReadableEndList, ReadableFloatList, ReadableIntArrayList, ReadableIntList,
    ReadableList, ReadableListList, ReadableLongArrayList, ReadableLongList, ReadableShortList,
    ReadableString, ReadableStringList, ReadableValue, WritableCompound, WritableList,
    WritableValue,
};

pub trait ReadableConfig: Send + Sync + Sized + 'static {
    type ByteOrder: ByteOrder;
    type Value<'doc>: ReadableValue<'doc, Config = Self>;
    type ByteArray<'doc>: Deref<Target = [i8]> + Clone;
    type String<'doc>: ReadableString<'doc>;
    type List<'doc>: ReadableList<'doc, Config = Self, Item = Self::Value<'doc>>;
    type ListIter<'doc>: Iterator<Item = Self::Value<'doc>> + ExactSizeIterator + Clone + Default;
    type Compound<'doc>: ReadableCompound<'doc, Config = Self, Item = (Self::String<'doc>, Self::Value<'doc>)>;
    type CompoundIter<'doc>: Iterator<Item = (Self::String<'doc>, Self::Value<'doc>)>
        + Clone
        + Default;
    type IntArray<'doc>: Deref<Target = [byteorder::I32<Self::ByteOrder>]> + Clone;
    type LongArray<'doc>: Deref<Target = [byteorder::I64<Self::ByteOrder>]> + Clone;

    type EndList<'doc>: ReadableEndList<'doc, Config = Self, Item = ()>;
    type EndListIter<'doc>: Iterator<Item = ()> + ExactSizeIterator + Clone + Default;
    type ByteList<'doc>: ReadableByteList<'doc, Config = Self, Item = i8>;
    type ByteListIter<'doc>: Iterator<Item = i8> + ExactSizeIterator + Clone + Default;
    type ShortList<'doc>: ReadableShortList<'doc, Config = Self, Item = i16>;
    type ShortListIter<'doc>: Iterator<Item = i16> + ExactSizeIterator + Clone + Default;
    type IntList<'doc>: ReadableIntList<'doc, Config = Self, Item = i32>;
    type IntListIter<'doc>: Iterator<Item = i32> + ExactSizeIterator + Clone + Default;
    type LongList<'doc>: ReadableLongList<'doc, Config = Self, Item = i64>;
    type LongListIter<'doc>: Iterator<Item = i64> + ExactSizeIterator + Clone + Default;
    type FloatList<'doc>: ReadableFloatList<'doc, Config = Self, Item = f32>;
    type FloatListIter<'doc>: Iterator<Item = f32> + ExactSizeIterator + Clone + Default;
    type DoubleList<'doc>: ReadableDoubleList<'doc, Config = Self, Item = f64>;
    type DoubleListIter<'doc>: Iterator<Item = f64> + ExactSizeIterator + Clone + Default;
    type ByteArrayList<'doc>: ReadableByteArrayList<'doc, Config = Self, Item = Vec<i8>>;
    type ByteArrayListIter<'doc>: Iterator<Item = Vec<i8>> + ExactSizeIterator + Clone + Default;
    type StringList<'doc>: ReadableStringList<'doc, Config = Self, Item = String>;
    type StringListIter<'doc>: Iterator<Item = String> + ExactSizeIterator + Clone + Default;
    type ListList<'doc>: ReadableListList<'doc, Config = Self, Item = Self::List<'doc>>;
    type ListListIter<'doc>: Iterator<Item = Self::List<'doc>> + ExactSizeIterator + Clone + Default;
    type CompoundList<'doc>: ReadableCompoundList<'doc, Config = Self, Item = (String, Self::Value<'doc>)>;
    type CompoundListIter<'doc>: Iterator<Item = (String, Self::Value<'doc>)>
        + ExactSizeIterator
        + Clone
        + Default;
    type IntArrayList<'doc>: ReadableIntArrayList<'doc, Config = Self, Item = Vec<i32>>;
    type IntArrayListIter<'doc>: Iterator<Item = Vec<i32>> + ExactSizeIterator + Clone + Default;
    type LongArrayList<'doc>: ReadableLongArrayList<'doc, Config = Self, Item = Vec<i64>>;
    type LongArrayListIter<'doc>: Iterator<Item = Vec<i64>> + ExactSizeIterator + Clone + Default;
}

pub trait WritableConfig: ReadableConfig + Send + Sync + Sized + 'static {
    type ValueMut<'s>: WritableValue<'s, ConfigMut = Self>;
    type ListMut<'s>: WritableList<'s, ConfigMut = Self, Item = Self::ValueMut<'s>>;
    type ListIterMut<'s>: Iterator<Item = Self::ValueMut<'s>> + ExactSizeIterator;
    type CompoundMut<'s>: WritableCompound<'s, ConfigMut = Self, Item = (Self::String<'s>, Self::ValueMut<'s>)>;
    type CompoundIterMut<'s>: Iterator<Item = (Self::String<'s>, Self::ValueMut<'s>)>;
}
