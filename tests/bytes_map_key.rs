//! Tests for bytes as map keys using base64 encoding.

use na_nbt::{from_slice_be, from_slice_le, to_vec_be, to_vec_le};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

/// Wrapper for Vec<u8> that serializes as bytes (via base64 for map keys).
///
/// This is needed because Vec<u8> serializes as a seq by default, which
/// KeySerializer doesn't support. By using the serde_bytes pattern,
/// we can serialize Vec<u8> as bytes directly.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ByteVec(pub Vec<u8>);

impl Deref for ByteVec {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Hash for ByteVec {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl Serialize for ByteVec {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Use serialize_bytes which will be base64-encoded for map keys
        serializer.serialize_bytes(&self.0)
    }
}

impl<'de> Deserialize<'de> for ByteVec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // For map keys, this will deserialize from base64
        struct ByteVecVisitor;

        impl<'de2> serde::de::Visitor<'de2> for ByteVecVisitor {
            type Value = ByteVec;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a byte array")
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(ByteVec(v.to_vec()))
            }

            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(ByteVec(v))
            }
        }

        deserializer.deserialize_byte_buf(ByteVecVisitor)
    }
}

fn round_trip_be<T>(value: &T) -> T
where
    T: for<'de> Deserialize<'de> + Serialize + PartialEq + std::fmt::Debug,
{
    let serialized = to_vec_be(value).unwrap();
    from_slice_be(&serialized).unwrap()
}

fn round_trip_le<T>(value: &T) -> T
where
    T: for<'de> Deserialize<'de> + Serialize + PartialEq + std::fmt::Debug,
{
    let serialized = to_vec_le(value).unwrap();
    from_slice_le(&serialized).unwrap()
}

fn round_trip_both<T>(value: &T)
where
    T: for<'de> Deserialize<'de> + Serialize + PartialEq + std::fmt::Debug,
{
    assert_eq!(value, &round_trip_be(value), "BE round-trip failed");
    assert_eq!(value, &round_trip_le(value), "LE round-trip failed");
}

#[test]
fn test_hashmap_with_bytevec_keys() {
    let mut map: HashMap<ByteVec, i32> = HashMap::new();
    map.insert(ByteVec(b"hello".to_vec()), 42);
    map.insert(ByteVec(b"world".to_vec()), 99);

    round_trip_both(&map);
}

#[test]
fn test_hashmap_with_non_utf8_bytevec_keys() {
    let mut map: HashMap<ByteVec, String> = HashMap::new();
    map.insert(ByteVec(vec![0xFF, 0xFE, 0xFD]), "invalid utf8".to_string());
    map.insert(ByteVec(vec![0x00, 0x01, 0x02]), "null bytes".to_string());
    map.insert(ByteVec(b"\xff\xff".to_vec()), "double ff".to_string());

    round_trip_both(&map);
}

#[test]
fn test_hashmap_with_empty_bytevec_key() {
    let mut map: HashMap<ByteVec, i32> = HashMap::new();
    map.insert(ByteVec(vec![]), 42);
    map.insert(ByteVec(b"data".to_vec()), 99);

    round_trip_both(&map);
}

#[test]
fn test_serialization_only() {
    let mut map: HashMap<ByteVec, i32> = HashMap::new();
    map.insert(ByteVec(b"test".to_vec()), 1);
    map.insert(ByteVec(vec![0xFF, 0xFE]), 2);

    let serialized = to_vec_be(&map).unwrap();
    println!("Serialized {} bytes", serialized.len());

    let deserialized: HashMap<ByteVec, i32> = from_slice_be(&serialized).unwrap();
    assert_eq!(deserialized.get(&ByteVec(b"test".to_vec())), Some(&1));
    assert_eq!(deserialized.get(&ByteVec(vec![0xFF, 0xFE])), Some(&2));
}
