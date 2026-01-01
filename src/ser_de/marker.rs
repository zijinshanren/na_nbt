//! Serialization markers for native NBT types.
//!
//! By default, `serde` serializes values as heterogeneous sequences. When serializing
//! heterogeneous values with `na_nbt`, each value is wrapped as:
//! ```text
//! List [ Compound { "" : <value> }, ... ]
//! ```
//! This representation is inefficient. The marker modules in this file indicate to `na_nbt`
//! that values should be serialized as more efficient native NBT types instead.
//!
//! # Available Markers
//!
//! | Module | NBT Type | Usage |
//! |--------|----------|-------|
//! | [`byte_array`] | TAG_Byte_Array | `#[serde(with = "na_nbt::byte_array")]` |
//! | [`int_array`] | TAG_Int_Array | `#[serde(with = "na_nbt::int_array")]` |
//! | [`long_array`] | TAG_Long_Array | `#[serde(with = "na_nbt::long_array")]` |
//! | [`list`] | TAG_List | `#[serde(with = "na_nbt::list")]` |

pub mod byte_array {
    //! Serializer and deserializer for NBT `TAG_Byte_Array`.
    //!
    //! Use this module with `#[serde(with = "na_nbt::byte_array")]` on fields of type
    //! `Vec<i8>` or `&[i8]` to serialize them as NBT byte arrays instead of generic
    //! sequences.
    //!
    //! # Examples
    //!
    //! ```rust
    //! use serde::{Serialize, Deserialize};
    //! #[derive(Serialize, Deserialize)]
    //! struct MyStruct {
    //!     #[serde(with = "na_nbt::byte_array")]
    //!     data: Vec<i8>,
    //! }
    //! ```

    use std::slice;

    use serde::{Deserializer, Serializer, de::Visitor};

    /// Serializes a slice of signed bytes as a NBT `TAG_Byte_Array`.
    ///
    /// This function is intended for use with `#[serde(with = "na_nbt::byte_array")]`.
    pub fn serialize<S>(data: &[i8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // SAFETY: i8 and u8 have the same memory representation and alignment.
        // The transmute effectively reinterprets the signedness of the bytes without copying.
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

        /// Visits a byte sequence and converts it to a `Vec<i8>`.
        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            // SAFETY: i8 and u8 have the same memory representation and alignment.
            // The transmute effectively reinterprets the signedness of the bytes.
            Ok(unsafe { slice::from_raw_parts(v.as_ptr() as *const i8, v.len()).to_vec() })
        }
    }

    /// Deserializes a NBT `TAG_Byte_Array` as a `Vec<i8>`.
    ///
    /// This function is intended for use with `#[serde(with = "na_nbt::byte_array")]`.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<i8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(ByteArrayVisitor)
    }
}

pub mod int_array {
    //! Serializer and deserializer for NBT `TAG_Int_Array`.
    //!
    //! Use this module with `#[serde(with = "na_nbt::int_array")]` on fields of type
    //! `Vec<T>` where `T: Into<i32> + From<i32> + Copy` (e.g., `Vec<i32>`) to serialize them as
    //! NBT int arrays instead of generic sequences.
    //!
    //! # Examples
    //!
    //! ```rust
    //! use serde::{Serialize, Deserialize};
    //! #[derive(Serialize, Deserialize)]
    //! struct MyStruct {
    //!     #[serde(with = "na_nbt::int_array")]
    //!     data: Vec<i32>,
    //! }
    //! ```
    //!
    //! This can also be used with types that implement `Into<i32>` and `From<i32>`, such as
    //! `zerocopy::byteorder::I32<Endian>`:
    //!
    //! ```rust
    //! use serde::{Serialize, Deserialize};
    //! use na_nbt::BigEndian;
    //! #[derive(Serialize, Deserialize)]
    //! struct MyStruct {
    //!     #[serde(with = "na_nbt::int_array")]
    //!     data: Vec<zerocopy::byteorder::I32<BigEndian>>,
    //! }
    //! ```

    use std::marker::PhantomData;

    use serde::{
        Deserializer, Serialize, Serializer,
        de::{SeqAccess, Visitor},
        ser::SerializeTuple,
    };

    /// Wrapper type for serializing slices as NBT `TAG_Int_Array`.
    ///
    /// This wrapper uses an internal marker name (`"na_nbt:int_array"`) to signal
    /// to `na_nbt` that the sequence should be serialized as an IntArray.
    struct IntArray<'a, T>(&'a [T]);

    impl<'a, T> Serialize for IntArray<'a, T>
    where
        T: Into<i32> + Copy,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut seq = serializer.serialize_tuple(self.0.len())?;
            for item in self.0 {
                // Convert each item to i32. The Into trait is infallible by design,
                // so no error handling is needed here.
                seq.serialize_element(&(*item).into())?;
            }
            seq.end()
        }
    }

    /// Serializes a slice as a NBT `TAG_Int_Array`.
    ///
    /// This function is intended for use with `#[serde(with = "na_nbt::int_array")]`.
    pub fn serialize<T, S>(data: &[T], serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Into<i32> + Copy,
        S: Serializer,
    {
        // The marker name "na_nbt:int_array" is an internal implementation detail
        // and not part of the stable public API.
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
                // Convert each i32 to the target type. The From trait is infallible.
                vec.push(item.into());
            }
            Ok(vec)
        }
    }

    /// Deserializes a NBT `TAG_Int_Array` as a `Vec<T>`.
    ///
    /// This function is intended for use with `#[serde(with = "na_nbt::int_array")]`.
    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: From<i32>,
    {
        deserializer.deserialize_seq(IntArrayVisitor(PhantomData))
    }
}

pub mod long_array {
    //! Serializer and deserializer for NBT `TAG_Long_Array`.
    //!
    //! Use this module with `#[serde(with = "na_nbt::long_array")]` on fields of type
    //! `Vec<T>` where `T: Into<i64> + From<i64> + Copy` (e.g., `Vec<i64>`) to serialize them as
    //! NBT long arrays instead of generic sequences.
    //!
    //! # Examples
    //!
    //! ```rust
    //! use serde::{Serialize, Deserialize};
    //! #[derive(Serialize, Deserialize)]
    //! struct MyStruct {
    //!     #[serde(with = "na_nbt::long_array")]
    //!     data: Vec<i64>,
    //! }
    //! ```
    //!
    //! This can also be used with types that implement `Into<i64>` and `From<i64>`, such as
    //! `zerocopy::byteorder::I64<Endian>`:
    //!
    //! ```rust
    //! use serde::{Serialize, Deserialize};
    //! use na_nbt::BigEndian;
    //! #[derive(Serialize, Deserialize)]
    //! struct MyStruct {
    //!     #[serde(with = "na_nbt::long_array")]
    //!     data: Vec<zerocopy::byteorder::I64<BigEndian>>,
    //! }
    //! ```

    use std::marker::PhantomData;

    use serde::{
        Deserializer, Serialize, Serializer,
        de::{SeqAccess, Visitor},
        ser::SerializeTuple,
    };

    /// Wrapper type for serializing slices as NBT `TAG_Long_Array`.
    ///
    /// This wrapper uses an internal marker name (`"na_nbt:long_array"`) to signal
    /// to `na_nbt` that the sequence should be serialized as a LongArray.
    struct LongArray<'a, T>(&'a [T]);

    impl<'a, T> Serialize for LongArray<'a, T>
    where
        T: Into<i64> + Copy,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut seq = serializer.serialize_tuple(self.0.len())?;
            for item in self.0 {
                // Convert each item to i64. The Into trait is infallible by design,
                // so no error handling is needed here.
                seq.serialize_element(&(*item).into())?;
            }
            seq.end()
        }
    }

    /// Serializes a slice as a NBT `TAG_Long_Array`.
    ///
    /// This function is intended for use with `#[serde(with = "na_nbt::long_array")]`.
    pub fn serialize<T, S>(data: &[T], serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Into<i64> + Copy,
        S: Serializer,
    {
        // The marker name "na_nbt:long_array" is an internal implementation detail
        // and not part of the stable public API.
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
                // Convert each i64 to the target type. The From trait is infallible.
                vec.push(item.into());
            }
            Ok(vec)
        }
    }

    /// Deserializes a NBT `TAG_Long_Array` as a `Vec<T>`.
    ///
    /// This function is intended for use with `#[serde(with = "na_nbt::long_array")]`.
    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: From<i64>,
    {
        deserializer.deserialize_seq(LongArrayVisitor(PhantomData::<T>))
    }
}

pub mod list {
    //! Serializer and deserializer for NBT `TAG_List`.
    //!
    //! Use this module with `#[serde(with = "na_nbt::list")]` on fields to serialize them
    //! as homogeneous NBT lists instead of heterogeneous compound-wrapped sequences.
    //!
    //! # Requirements
    //!
    //! - All items must serialize to the **same NBT type**. If items serialize to
    //!   different NBT types, serialization will fail with `na_nbt::Error`.
    //!
    //! # Empty Lists
    //!
    //! If the collection is empty, it will be serialized as an empty list with the
    //! `TAG_End` type marker. This follows the Minecraft NBT implementation.
    //!
    //! # Examples
    //!
    //! ```rust
    //! use serde::{Serialize, Deserialize};
    //! #[derive(Serialize, Deserialize)]
    //! struct MyStruct {
    //!     #[serde(with = "na_nbt::list")]
    //!     items: Vec<f64>,
    //! }
    //! ```

    use std::marker::PhantomData;

    use serde::{
        Deserialize, Deserializer, Serialize, Serializer,
        de::{SeqAccess, Visitor},
        ser::SerializeSeq,
    };

    /// Wrapper type for serializing iterators as NBT `TAG_List`.
    ///
    /// This wrapper uses an internal marker name (`"na_nbt:list"`) to signal
    /// to `na_nbt` that the sequence should be serialized as a homogeneous list.
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
            // Only provide an exact size hint if the iterator specifies one
            // (i.e., lower bound equals upper bound).
            let mut seq = serializer.serialize_seq((Some(lo) == hi).then_some(lo))?;
            for item in iter {
                seq.serialize_element(&item)?;
            }
            seq.end()
        }
    }

    /// Serializes a value as a NBT `TAG_List`.
    ///
    /// This function is intended for use with `#[serde(with = "na_nbt::list")]`.
    pub fn serialize<T, S>(data: T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: IntoIterator,
        T::IntoIter: Clone,
        T::Item: Serialize,
        S: Serializer,
    {
        // The marker name "na_nbt:list" is an internal implementation detail
        // and not part of the stable public API.
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

    /// Deserializes a NBT `TAG_List` as a `Vec<T>`.
    ///
    /// This function is intended for use with `#[serde(with = "na_nbt::list")]`.
    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
    where
        T: Deserialize<'de>,
        D: Deserializer<'de>,
    {
        // The newtype marker ensures that data serialized with this marker
        // is correctly deserialized as a native NBT list.
        deserializer.deserialize_newtype_struct("na_nbt:list", ListVisitor(PhantomData))
    }
}
