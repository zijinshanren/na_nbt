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
