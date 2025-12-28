pub mod byte_array {
    use std::slice;

    use serde::{
        Deserializer, Serializer,
        de::{SeqAccess, Visitor},
    };

    pub fn serialize<S>(data: &[i8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(unsafe {
            slice::from_raw_parts(data.as_ptr() as *const u8, data.len())
        })
    }

    struct ByteArrayVisitor;

    impl<'de> Visitor<'de> for ByteArrayVisitor {
        type Value = Vec<i8>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a byte array")
        }

        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(unsafe { slice::from_raw_parts(v.as_ptr() as *const i8, v.len()).to_vec() })
        }

        fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(unsafe { std::mem::transmute::<Vec<u8>, Vec<i8>>(v) })
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut vec = Vec::with_capacity(seq.size_hint().unwrap_or(0));
            while let Some(item) = seq.next_element()? {
                vec.push(item);
            }
            Ok(vec)
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<i8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(ByteArrayVisitor)
    }
}

pub mod int_array {
    use std::marker::PhantomData;

    use serde::{
        Deserializer, Serialize, Serializer,
        de::{SeqAccess, Visitor},
        ser::SerializeSeq,
    };

    struct IntArray<'a, T>(&'a [T]);

    impl<'a, T> Serialize for IntArray<'a, T>
    where
        T: Into<i32> + Copy,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
            for item in self.0 {
                seq.serialize_element(&(*item).into())?;
            }
            seq.end()
        }
    }

    pub fn serialize<T, S>(data: &[T], serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Into<i32> + Copy,
        S: Serializer,
    {
        serializer.serialize_newtype_struct("na_nbt:int_array", &IntArray(data))
    }

    struct IntArrayVisitor<T>(PhantomData<T>);

    impl<'de, T> Visitor<'de> for IntArrayVisitor<T>
    where
        T: From<i32>,
    {
        type Value = Vec<T>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("an int array")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut vec = Vec::with_capacity(seq.size_hint().unwrap_or(0));
            while let Some(item) = seq.next_element::<i32>()? {
                vec.push(item.into());
            }
            Ok(vec)
        }
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: From<i32>,
    {
        deserializer.deserialize_seq(IntArrayVisitor(PhantomData))
    }
}

pub mod long_array {
    use std::marker::PhantomData;

    use serde::{
        Deserializer, Serialize, Serializer,
        de::{SeqAccess, Visitor},
        ser::SerializeSeq,
    };

    struct LongArray<'a, T>(&'a [T]);

    impl<'a, T> Serialize for LongArray<'a, T>
    where
        T: Into<i64> + Copy,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
            for item in self.0 {
                seq.serialize_element(&(*item).into())?;
            }
            seq.end()
        }
    }

    pub fn serialize<T, S>(data: &[T], serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Into<i64> + Copy,
        S: Serializer,
    {
        serializer.serialize_newtype_struct("na_nbt:long_array", &LongArray(data))
    }

    struct LongArrayVisitor<T>(PhantomData<T>);

    impl<'de, T> Visitor<'de> for LongArrayVisitor<T>
    where
        T: From<i64>,
    {
        type Value = Vec<T>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a long array")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut vec = Vec::with_capacity(seq.size_hint().unwrap_or(0));
            while let Some(item) = seq.next_element::<i64>()? {
                vec.push(item.into());
            }
            Ok(vec)
        }
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: From<i64>,
    {
        deserializer.deserialize_seq(LongArrayVisitor(PhantomData::<T>))
    }
}

pub mod list {
    use std::marker::PhantomData;

    use serde::{
        Deserialize, Deserializer, Serialize, Serializer,
        de::{SeqAccess, Visitor},
        ser::SerializeSeq,
    };

    struct List<T>(T);

    impl<T> Serialize for List<T>
    where
        T: Iterator + Clone,
        <T as Iterator>::Item: Serialize,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let iter = self.0.clone();
            let (lo, hi) = iter.size_hint();
            let mut seq = serializer.serialize_seq((Some(lo) == hi).then_some(lo))?;
            for item in iter {
                seq.serialize_element(&item)?;
            }
            seq.end()
        }
    }

    pub fn serialize<T, S>(data: T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: IntoIterator,
        T::IntoIter: Clone,
        T::Item: Serialize,
        S: Serializer,
    {
        serializer.serialize_newtype_struct("na_nbt:list", &List(data.into_iter()))
    }

    struct ListVisitor<T>(PhantomData<T>);

    impl<'de, T> Visitor<'de> for ListVisitor<T>
    where
        T: Deserialize<'de>,
    {
        type Value = Vec<T>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a list")
        }

        fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_seq(self)
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut vec = Vec::with_capacity(seq.size_hint().unwrap_or(0));
            while let Some(item) = seq.next_element()? {
                vec.push(item);
            }
            Ok(vec)
        }
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
    where
        T: Deserialize<'de>,
        D: Deserializer<'de>,
    {
        deserializer.deserialize_newtype_struct("na_nbt:list", ListVisitor(PhantomData))
    }
}
