//! Serde modules for NBT's native array types.
//!
//! NBT has three specialized array types that are more efficient than `List`:
//!
//! | Module | NBT Tag | Element Type | Tag ID |
//! |--------|---------|--------------|--------|
//! | [`byte_array`] | `ByteArray` | `i8` | 7 |
//! | [`int_array`] | `IntArray` | `i32` | 11 |
//! | [`long_array`] | `LongArray` | `i64` | 12 |
//!
//! # Why Use Native Arrays?
//!
//! Native arrays are more compact than lists because they don't store per-element
//! type tags. For large arrays of integers, this can significantly reduce file size:
//!
//! | Format | Overhead per element |
//! |--------|---------------------|
//! | `List<Int>` | 5 bytes (1 tag + 4 data) |
//! | `IntArray` | 4 bytes (data only) |
//! | `List<Long>` | 9 bytes (1 tag + 8 data) |
//! | `LongArray` | 8 bytes (data only) |
//!
//! For Minecraft chunk data with thousands of block states, this matters!
//!
//! # Deserialization (Automatic)
//!
//! **Native arrays are automatically detected during deserialization!**
//! You can use plain `Vec<T>` types - the deserializer will correctly read
//! both native arrays and list formats:
//!
//! ```ignore
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct ChunkData {
//!     block_states: Vec<i64>,  // Auto-detects LongArray or List<Long>
//!     biomes: Vec<i32>,        // Auto-detects IntArray or List<Int>
//!     heightmap: Vec<i8>,      // Note: ByteArray requires byte_array module
//! }
//! ```
//!
//! # Serialization (Use These Modules)
//!
//! By default, `Vec<i8>`, `Vec<i32>`, and `Vec<i64>` serialize as `List` types.
//! Use `#[serde(with = "...")]` to serialize as native arrays:
//!
//! ```ignore
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct ChunkSection {
//!     #[serde(with = "na_nbt::long_array")]
//!     block_states: Vec<i64>,  // Serializes as LongArray
//!     
//!     #[serde(with = "na_nbt::int_array")]
//!     biomes: Vec<i32>,        // Serializes as IntArray
//!     
//!     #[serde(with = "na_nbt::byte_array")]
//!     heightmap: Vec<i8>,      // Serializes as ByteArray
//! }
//! ```
//!
//! # Performance
//!
//! These modules serialize directly from slice references (`&[T]`) without
//! intermediate allocations. The data is written in a single pass with proper
//! byte order conversion.
//!
//! # Complete Example
//!
//! ```ignore
//! use serde::{Serialize, Deserialize};
//! use na_nbt::{to_vec_be, from_slice_be};
//!
//! #[derive(Serialize, Deserialize, Debug, PartialEq)]
//! struct HeightMap {
//!     #[serde(with = "na_nbt::long_array")]
//!     data: Vec<i64>,
//! }
//!
//! let heightmap = HeightMap {
//!     data: vec![0i64; 256], // 256 packed height values
//! };
//!
//! // Serialize to NBT with native LongArray
//! let bytes = to_vec_be(&heightmap).unwrap();
//!
//! // Deserialize back
//! let loaded: HeightMap = from_slice_be(&bytes).unwrap();
//! assert_eq!(heightmap, loaded);
//! ```

use std::slice;

use serde::{Deserializer, Serialize, Serializer, de};

// ============================================================================
// Internal serialization helpers (zero-copy from slices)
// ============================================================================

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

// ============================================================================
// Deserialization visitors
// ============================================================================

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

/// Serde module for `Vec<i8>` as NBT `ByteArray`.
///
/// This module serializes byte vectors as NBT's native `ByteArray` (tag 7)
/// instead of `List<Byte>` (tag 9), which is more compact.
///
/// # Usage
///
/// ```ignore
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct BlockData {
///     #[serde(with = "na_nbt::byte_array")]
///     light_levels: Vec<i8>,
/// }
/// ```
///
/// # Note
///
/// This module uses serde's `serialize_bytes` which produces NBT `ByteArray`.
/// For deserialization, it expects `ByteArray` specifically (not `List<Byte>`).
/// If you need to read both formats, use a custom visitor.
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

/// Serde module for `Vec<i32>` as NBT `IntArray`.
///
/// This module serializes integer vectors as NBT's native `IntArray` (tag 11)
/// instead of `List<Int>` (tag 9). `IntArray` is more compact, saving 1 byte
/// per element (no per-element type tag).
///
/// # Usage
///
/// ```ignore
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct BiomeData {
///     #[serde(with = "na_nbt::int_array")]
///     biome_ids: Vec<i32>,  // Serialized as IntArray
/// }
/// ```
///
/// # Deserialization
///
/// The deserializer automatically detects `IntArray` tags, so this module
/// works for both reading and writing. You can also deserialize `IntArray`
/// to a plain `Vec<i32>` without this module - the detection is automatic.
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

/// Serde module for `Vec<i64>` as NBT `LongArray`.
///
/// This module serializes long integer vectors as NBT's native `LongArray` (tag 12)
/// instead of `List<Long>` (tag 9). `LongArray` is more compact, saving 1 byte
/// per element (no per-element type tag).
///
/// This is commonly used for Minecraft chunk data:
/// - Block states (packed into longs)
/// - Heightmaps
/// - Biome data (in newer versions)
///
/// # Usage
///
/// ```ignore
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct ChunkSection {
///     #[serde(with = "na_nbt::long_array")]
///     block_states: Vec<i64>,  // Packed block state data
///     
///     #[serde(with = "na_nbt::long_array")]
///     heightmap: Vec<i64>,     // World surface heightmap
/// }
/// ```
///
/// # Deserialization
///
/// The deserializer automatically detects `LongArray` tags, so this module
/// works for both reading and writing. You can also deserialize `LongArray`
/// to a plain `Vec<i64>` without this module - the detection is automatic.
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
