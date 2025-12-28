use std::slice;

use serde::{Deserializer, Serialize, Serializer, de};

struct IntArrayRef<'a>(&'a [i32]);

impl<'a> Serialize for IntArrayRef<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeTuple;
        let mut tup = serializer.serialize_tuple(self.0.len())?;
        for &val in self.0 {
            tup.serialize_element(&val)?;
        }
        tup.end()
    }
}

struct LongArrayRef<'a>(&'a [i64]);

impl<'a> Serialize for LongArrayRef<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeTuple;
        let mut tup = serializer.serialize_tuple(self.0.len())?;
        for &val in self.0 {
            tup.serialize_element(&val)?;
        }
        tup.end()
    }
}

struct IntArrayVisitor;

impl<'de> de::Visitor<'de> for IntArrayVisitor {
    type Value = Vec<i32>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an NBT IntArray or List<Int>")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let mut vec = Vec::with_capacity(seq.size_hint().unwrap_or(0));
        while let Some(val) = seq.next_element()? {
            vec.push(val);
        }
        Ok(vec)
    }
}

struct LongArrayVisitor;

impl<'de> de::Visitor<'de> for LongArrayVisitor {
    type Value = Vec<i64>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an NBT LongArray or List<Long>")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let mut vec = Vec::with_capacity(seq.size_hint().unwrap_or(0));
        while let Some(val) = seq.next_element()? {
            vec.push(val);
        }
        Ok(vec)
    }
}

struct ByteArrayVisitor;

impl<'de> de::Visitor<'de> for ByteArrayVisitor {
    type Value = Vec<i8>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a byte array")
    }

    fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<Self::Value, E> {
        Ok(unsafe { slice::from_raw_parts(v.as_ptr() as *const i8, v.len()).to_vec() })
    }

    fn visit_byte_buf<E: de::Error>(self, v: Vec<u8>) -> Result<Self::Value, E> {
        Ok(unsafe { std::mem::transmute::<Vec<u8>, Vec<i8>>(v) })
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let mut vec = Vec::with_capacity(seq.size_hint().unwrap_or(0));
        while let Some(val) = seq.next_element()? {
            vec.push(val);
        }
        Ok(vec)
    }
}

// ============================================================================
// Serde modules for #[serde(with = "...")]
// ============================================================================

pub mod byte_array {
    use super::*;

    /// Serialize `&[i8]` as NBT `ByteArray`.
    ///
    /// The bytes are written directly without any intermediate allocation.
    pub fn serialize<S>(data: &[i8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(unsafe {
            slice::from_raw_parts(data.as_ptr() as *const u8, data.len())
        })
    }

    /// Deserialize `Vec<i8>` from NBT `ByteArray`.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<i8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(ByteArrayVisitor)
    }
}

pub mod int_array {
    use super::*;

    /// Serialize `&[i32]` as NBT `IntArray`.
    ///
    /// Serializes directly from the slice reference without copying data.
    /// The integers are written with proper byte order conversion.
    pub fn serialize<S>(data: &[i32], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_newtype_struct("na_nbt:int_array", &IntArrayRef(data))
    }

    /// Deserialize `Vec<i32>` from NBT `IntArray` or `List<Int>`.
    ///
    /// The deserializer automatically detects the tag type.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<i32>, D::Error>
    where
        D: Deserializer<'de>,
    {
        // The deserializer auto-detects IntArray tags in deserialize_seq
        deserializer.deserialize_seq(IntArrayVisitor)
    }
}

pub mod long_array {
    use super::*;

    /// Serialize `&[i64]` as NBT `LongArray`.
    ///
    /// Serializes directly from the slice reference without copying data.
    /// The longs are written with proper byte order conversion.
    pub fn serialize<S>(data: &[i64], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_newtype_struct("na_nbt:long_array", &LongArrayRef(data))
    }

    /// Deserialize `Vec<i64>` from NBT `LongArray` or `List<Long>`.
    ///
    /// The deserializer automatically detects the tag type.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<i64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        // The deserializer auto-detects LongArray tags in deserialize_seq
        deserializer.deserialize_seq(LongArrayVisitor)
    }
}

/// Serde module for serializing `Vec<T>` / `&[T]` as native NBT `List`.
///
/// By default, `Vec<T>` is serialized as `List[Compound{"": value}, ...]` because
/// serde sequences don't carry type information. This module serializes directly
/// as a native NBT `List<T>` where all elements have the same tag type.
///
/// # Example
///
/// ```ignore
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct Player {
///     #[serde(with = "na_nbt::list")]
///     scores: Vec<i32>,  // Serializes as List<Int> instead of List<Compound>
/// }
/// ```
///
/// # Panics
///
/// Serialization will return an error if elements have different NBT types.
pub mod list {
    use serde::ser::SerializeSeq;

    use super::*;

    struct ListRef<'a, T>(&'a [T]);

    impl<'a, T: Serialize> Serialize for ListRef<'a, T> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
            for val in self.0 {
                seq.serialize_element(val)?;
            }
            seq.end()
        }
    }

    /// Serialize `&[T]` as native NBT `List<T>`.
    ///
    /// Elements are serialized directly without wrapping in compounds.
    /// All elements must serialize to the same NBT tag type.
    pub fn serialize<T: Serialize, S>(data: &[T], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_newtype_struct("na_nbt:list", &ListRef(data))
    }

    struct ListVisitor<T>(std::marker::PhantomData<T>);

    impl<'de, T: de::Deserialize<'de>> de::Visitor<'de> for ListVisitor<T> {
        type Value = Vec<T>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("an NBT List")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let mut vec = Vec::with_capacity(seq.size_hint().unwrap_or(0));
            while let Some(val) = seq.next_element()? {
                vec.push(val);
            }
            Ok(vec)
        }

        fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            // After the newtype wrapper sets native_list flag, deserialize the seq
            deserializer.deserialize_seq(self)
        }
    }

    /// Deserialize `Vec<T>` from native NBT `List<T>`.
    ///
    /// Uses `deserialize_newtype_struct` to signal native list mode,
    /// which skips compound unwrapping for list elements.
    pub fn deserialize<'de, T: de::Deserialize<'de>, D>(deserializer: D) -> Result<Vec<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_newtype_struct("na_nbt:list", ListVisitor(std::marker::PhantomData))
    }
}