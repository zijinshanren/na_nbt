use na_nbt::de::from_slice;
use na_nbt::ser::to_vec;
use na_nbt::{from_slice_be, to_vec_be};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::f32;
use std::f64;
use zerocopy::byteorder::BigEndian;

// Test primitive types roundtrip
#[test]
fn test_roundtrip_primitives() {
    // i8 (Byte)
    let original: i8 = 42;
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: i8 = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);

    // bool
    let original = true;
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: bool = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);

    // i16 (Short)
    let original: i16 = -1234;
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: i16 = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);

    // i32 (Int)
    let original: i32 = -123456;
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: i32 = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);

    // i64 (Long)
    let original: i64 = -1234567890123;
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: i64 = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);

    // f32 (Float)
    let original: f32 = f32::consts::PI;
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: f32 = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert!((original - deserialized).abs() < 0.00001);

    // f64 (Double)
    let original: f64 = f64::consts::PI;
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: f64 = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert!((original - deserialized).abs() < 0.0000000001);
}

#[test]
fn test_roundtrip_string() {
    let original = "Hello, World!";
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: String = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);

    // Empty string
    let original = "";
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: String = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);

    // Unicode
    let original = "你好世界";
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: String = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

// Test bytes roundtrip
mod serde_bytes_impl {
    use serde::{Deserializer, Serializer, de};

    pub fn serialize<S: Serializer>(data: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(data)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
        struct ByteVisitor;
        impl<'de> de::Visitor<'de> for ByteVisitor {
            type Value = Vec<u8>;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("byte array")
            }
            fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<Self::Value, E> {
                Ok(v.to_vec())
            }
            fn visit_borrowed_bytes<E: de::Error>(self, v: &'de [u8]) -> Result<Self::Value, E> {
                Ok(v.to_vec())
            }
        }
        deserializer.deserialize_bytes(ByteVisitor)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct ByteVec(#[serde(with = "serde_bytes_impl")] Vec<u8>);

#[test]
fn test_roundtrip_bytes() {
    let original = ByteVec(vec![1, 2, 3, 4, 5]);
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: ByteVec = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

// Test struct roundtrip
#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct SimpleStruct {
    name: String,
    value: i32,
}

#[test]
fn test_roundtrip_struct() {
    let original = SimpleStruct {
        name: "test".to_string(),
        value: 42,
    };
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: SimpleStruct = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

// Test nested struct
#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct OuterStruct {
    inner: SimpleStruct,
    count: i32,
}

#[test]
fn test_roundtrip_nested_struct() {
    let original = OuterStruct {
        inner: SimpleStruct {
            name: "nested".to_string(),
            value: 100,
        },
        count: 5,
    };
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: OuterStruct = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

// Test HashMap roundtrip
#[test]
fn test_roundtrip_hashmap() {
    let mut original: HashMap<String, i32> = HashMap::new();
    original.insert("key1".to_string(), 10);
    original.insert("key2".to_string(), 20);
    original.insert("key3".to_string(), 30);

    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: HashMap<String, i32> = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

// Test empty HashMap
#[test]
fn test_roundtrip_empty_hashmap() {
    let original: HashMap<String, i32> = HashMap::new();
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: HashMap<String, i32> = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

// Test tuple roundtrip
#[test]
fn test_roundtrip_tuple() {
    let original = (1i32, 2i32, 3i32);
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: (i32, i32, i32) = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

// Test tuple struct
#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Point(i32, i32, i32);

#[test]
fn test_roundtrip_tuple_struct() {
    let original = Point(10, 20, 30);
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: Point = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

// Test newtype struct
#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Wrapper(i32);

#[test]
fn test_roundtrip_newtype_struct() {
    let original = Wrapper(42);
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: Wrapper = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

// Test unit struct
#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Empty;

#[test]
fn test_roundtrip_unit_struct() {
    let original = Empty;
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: Empty = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

// Test unit
#[test]
fn test_roundtrip_unit() {
    let original = ();
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let _deserialized: () = from_slice::<BigEndian, _>(&serialized).unwrap();
}

// Test enum unit variant
#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum Status {
    Active,
    Inactive,
    Pending,
}

#[test]
fn test_roundtrip_enum_unit_variant() {
    let original = Status::Active;
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: Status = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);

    let original = Status::Inactive;
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: Status = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);

    let original = Status::Pending;
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: Status = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

// Test enum newtype variant
#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum Data {
    Int(i32),
    Text(String),
}

#[test]
fn test_roundtrip_enum_newtype_variant() {
    let original = Data::Int(42);
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: Data = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);

    let original = Data::Text("hello".to_string());
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: Data = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

// Test enum tuple variant
#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum Coord {
    Point2D(i32, i32),
    Point3D(i32, i32, i32),
}

#[test]
fn test_roundtrip_enum_tuple_variant() {
    let original = Coord::Point2D(10, 20);
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: Coord = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);

    let original = Coord::Point3D(1, 2, 3);
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: Coord = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

// Test enum struct variant
#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum Entity {
    Player { name: String, health: i32 },
    Monster { kind: String, damage: i32 },
}

#[test]
fn test_roundtrip_enum_struct_variant() {
    let original = Entity::Player {
        name: "Steve".to_string(),
        health: 100,
    };
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: Entity = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);

    let original = Entity::Monster {
        kind: "Zombie".to_string(),
        damage: 10,
    };
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: Entity = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

// Test Vec (seq)
#[test]
fn test_roundtrip_vec() {
    let original = vec![1i32, 2, 3, 4, 5];
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: Vec<i32> = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

// Test empty Vec
#[test]
fn test_roundtrip_empty_vec() {
    let original: Vec<i32> = vec![];
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: Vec<i32> = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

// Test complex nested structure
#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Inventory {
    items: Vec<Item>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Item {
    id: i32,
    count: i32,
    name: String,
}

#[test]
fn test_roundtrip_complex_nested() {
    let original = Inventory {
        items: vec![
            Item {
                id: 1,
                count: 64,
                name: "Stone".to_string(),
            },
            Item {
                id: 2,
                count: 32,
                name: "Wood".to_string(),
            },
        ],
    };
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: Inventory = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

// Test char
#[test]
fn test_roundtrip_char() {
    let original = 'A';
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: char = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

// Test unsigned types (they serialize as signed but should roundtrip correctly)
#[test]
fn test_roundtrip_unsigned() {
    let original: u8 = 255;
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: u8 = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);

    let original: u16 = 65535;
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: u16 = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);

    let original: u32 = 4294967295;
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: u32 = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);

    let original: u64 = 18446744073709551615;
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: u64 = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

// Test Option Some
#[test]
fn test_roundtrip_option_some() {
    let original: Option<i32> = Some(42);
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: Option<i32> = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

// Test Option None
#[test]
fn test_roundtrip_option_none() {
    let original: Option<i32> = None;
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: Option<i32> = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

// Test struct with optional fields
#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct PlayerWithOptional {
    name: String,
    nickname: Option<String>,
}

#[test]
fn test_roundtrip_struct_with_option() {
    let original = PlayerWithOptional {
        name: "Player1".to_string(),
        nickname: Some("P1".to_string()),
    };
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: PlayerWithOptional = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);

    let original = PlayerWithOptional {
        name: "Player2".to_string(),
        nickname: None,
    };
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: PlayerWithOptional = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

// Test struct with all primitive types
#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct AllPrimitives {
    byte_val: i8,
    short_val: i16,
    int_val: i32,
    long_val: i64,
    float_val: f32,
    double_val: f64,
    string_val: String,
    bool_val: bool,
}

#[test]
fn test_roundtrip_all_primitives() {
    let original = AllPrimitives {
        byte_val: -128,
        short_val: -32768,
        int_val: -2147483648,
        long_val: -9223372036854775808,
        float_val: f32::consts::PI,
        double_val: f64::consts::E,
        string_val: "test".to_string(),
        bool_val: true,
    };
    let serialized = to_vec::<BigEndian>(&original).unwrap();
    let deserialized: AllPrimitives = from_slice::<BigEndian, _>(&serialized).unwrap();
    assert_eq!(original.byte_val, deserialized.byte_val);
    assert_eq!(original.short_val, deserialized.short_val);
    assert_eq!(original.int_val, deserialized.int_val);
    assert_eq!(original.long_val, deserialized.long_val);
    assert!((original.float_val - deserialized.float_val).abs() < 0.001);
    assert!((original.double_val - deserialized.double_val).abs() < 0.0000001);
    assert_eq!(original.string_val, deserialized.string_val);
    assert_eq!(original.bool_val, deserialized.bool_val);
}

// Test with LittleEndian
use zerocopy::byteorder::LittleEndian;

#[test]
fn test_roundtrip_little_endian() {
    let original = SimpleStruct {
        name: "little endian test".to_string(),
        value: 12345,
    };
    let serialized = to_vec::<LittleEndian>(&original).unwrap();
    let deserialized: SimpleStruct = from_slice::<LittleEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

// =============================================================================
// Native array serde module tests
// =============================================================================

#[test]
fn test_roundtrip_int_array() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct DataWithIntArray {
        #[serde(with = "na_nbt::int_array")]
        values: Vec<i32>,
    }

    let original = DataWithIntArray {
        values: vec![1, 2, 3, 4, 5, -100, 0, i32::MAX, i32::MIN],
    };

    let serialized = to_vec_be(&original).unwrap();
    let deserialized: DataWithIntArray = from_slice_be(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

#[test]
fn test_roundtrip_long_array() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct DataWithLongArray {
        #[serde(with = "na_nbt::long_array")]
        values: Vec<i64>,
    }

    let original = DataWithLongArray {
        values: vec![1, 2, 3, 4, 5, -100, 0, i64::MAX, i64::MIN],
    };

    let serialized = to_vec_be(&original).unwrap();
    let deserialized: DataWithLongArray = from_slice_be(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

#[test]
fn test_roundtrip_byte_array() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct DataWithByteArray {
        #[serde(with = "na_nbt::byte_array")]
        data: Vec<i8>,
    }

    let original = DataWithByteArray {
        data: vec![0i8, 1, 2, 3, -1, -128, -64],
    };

    let serialized = to_vec_be(&original).unwrap();
    let deserialized: DataWithByteArray = from_slice_be(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

#[test]
fn test_roundtrip_mixed_arrays() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct ChunkData {
        #[serde(with = "na_nbt::long_array")]
        block_states: Vec<i64>,
        #[serde(with = "na_nbt::int_array")]
        biomes: Vec<i32>,
        name: String,
    }

    let original = ChunkData {
        block_states: vec![0i64; 256],
        biomes: vec![1, 2, 3, 4],
        name: "test_chunk".to_string(),
    };

    let serialized = to_vec_be(&original).unwrap();
    let deserialized: ChunkData = from_slice_be(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

#[test]
fn test_int_array_vs_vec_i32_size() {
    // Compare serialized size of int_array module vs plain Vec<i32>
    #[derive(Serialize)]
    struct WithIntArray {
        #[serde(with = "na_nbt::int_array")]
        data: Vec<i32>,
    }

    #[derive(Serialize)]
    struct WithVec {
        data: Vec<i32>,
    }

    let int_array_data = WithIntArray {
        data: vec![1, 2, 3, 4, 5],
    };
    let vec_data = WithVec {
        data: vec![1, 2, 3, 4, 5],
    };

    let int_array_bytes = to_vec_be(&int_array_data).unwrap();
    let vec_bytes = to_vec_be(&vec_data).unwrap();

    // IntArray uses Tag::IntArray (more compact)
    // Vec<i32> uses Tag::List with Tag::Int elements
    // IntArray should be smaller or equal in size
    assert!(
        int_array_bytes.len() <= vec_bytes.len(),
        "int_array ({} bytes) should be <= Vec<i32> ({} bytes)",
        int_array_bytes.len(),
        vec_bytes.len()
    );
}

#[test]
fn test_native_int_array_to_vec_i32() {
    // Serialize using int_array module (produces native IntArray tag)
    #[derive(Serialize)]
    struct WriteData {
        #[serde(with = "na_nbt::int_array")]
        values: Vec<i32>,
    }

    // Deserialize to plain Vec<i32> (should auto-detect IntArray tag)
    #[derive(Deserialize, Debug, PartialEq)]
    struct ReadData {
        values: Vec<i32>,
    }

    let original = WriteData {
        values: vec![1, 2, 3, 4, 5],
    };

    let serialized = to_vec_be(&original).unwrap();
    let deserialized: ReadData = from_slice_be(&serialized).unwrap();

    // Vec<i32> should correctly read from IntArray tag
    assert_eq!(deserialized.values, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_native_long_array_to_vec_i64() {
    #[derive(Serialize)]
    struct WriteData {
        #[serde(with = "na_nbt::long_array")]
        values: Vec<i64>,
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct ReadData {
        values: Vec<i64>,
    }

    let original = WriteData {
        values: vec![100i64, 200, 300],
    };

    let serialized = to_vec_be(&original).unwrap();
    let deserialized: ReadData = from_slice_be(&serialized).unwrap();

    assert_eq!(deserialized.values, vec![100i64, 200, 300]);
}

// =============================================================================
// Additional array tests
// =============================================================================

#[test]
fn test_empty_int_array() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Data {
        #[serde(with = "na_nbt::int_array")]
        values: Vec<i32>,
    }

    let original = Data { values: vec![] };
    let serialized = to_vec_be(&original).unwrap();
    let deserialized: Data = from_slice_be(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

#[test]
fn test_empty_long_array() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Data {
        #[serde(with = "na_nbt::long_array")]
        values: Vec<i64>,
    }

    let original = Data { values: vec![] };
    let serialized = to_vec_be(&original).unwrap();
    let deserialized: Data = from_slice_be(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

#[test]
fn test_empty_byte_array() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Data {
        #[serde(with = "na_nbt::byte_array")]
        values: Vec<i8>,
    }

    let original = Data { values: vec![] };
    let serialized = to_vec_be(&original).unwrap();
    let deserialized: Data = from_slice_be(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

#[test]
fn test_int_array_edge_values() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Data {
        #[serde(with = "na_nbt::int_array")]
        values: Vec<i32>,
    }

    let original = Data {
        values: vec![i32::MIN, -1, 0, 1, i32::MAX],
    };
    let serialized = to_vec_be(&original).unwrap();
    let deserialized: Data = from_slice_be(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

#[test]
fn test_long_array_edge_values() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Data {
        #[serde(with = "na_nbt::long_array")]
        values: Vec<i64>,
    }

    let original = Data {
        values: vec![i64::MIN, -1, 0, 1, i64::MAX],
    };
    let serialized = to_vec_be(&original).unwrap();
    let deserialized: Data = from_slice_be(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

#[test]
fn test_byte_array_all_values() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Data {
        #[serde(with = "na_nbt::byte_array")]
        values: Vec<i8>,
    }

    // Test all possible byte values
    let original = Data {
        values: (0i8..=127).collect(),
    };
    let serialized = to_vec_be(&original).unwrap();
    let deserialized: Data = from_slice_be(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

#[test]
fn test_int_array_tag_is_correct() {
    use na_nbt::TagID;

    #[derive(Serialize)]
    struct Data {
        #[serde(with = "na_nbt::int_array")]
        values: Vec<i32>,
    }

    let data = Data {
        values: vec![1, 2, 3],
    };
    let bytes = to_vec_be(&data).unwrap();

    // First byte is the root compound tag
    assert_eq!(bytes[0], TagID::Compound as u8);
    // After compound tag (1) + name length (2) + name "values" (6) = offset 9
    // Then field tag should be IntArray (11)
    // Layout: [Tag::Compound, 0, 0, Tag::IntArray, 0, 6, 'v', 'a', 'l', 'u', 'e', 's', ...]
    assert_eq!(
        bytes[3],
        TagID::IntArray as u8,
        "Field tag should be IntArray (11)"
    );
}

#[test]
fn test_long_array_tag_is_correct() {
    use na_nbt::TagID;

    #[derive(Serialize)]
    struct Data {
        #[serde(with = "na_nbt::long_array")]
        values: Vec<i64>,
    }

    let data = Data {
        values: vec![1, 2, 3],
    };
    let bytes = to_vec_be(&data).unwrap();

    assert_eq!(bytes[0], TagID::Compound as u8);
    assert_eq!(
        bytes[3],
        TagID::LongArray as u8,
        "Field tag should be LongArray (12)"
    );
}

#[test]
fn test_byte_array_tag_is_correct() {
    use na_nbt::TagID;

    #[derive(Serialize)]
    struct Data {
        #[serde(with = "na_nbt::byte_array")]
        values: Vec<i8>,
    }

    let data = Data {
        values: vec![1, 2, 3],
    };
    let bytes = to_vec_be(&data).unwrap();

    assert_eq!(bytes[0], TagID::Compound as u8);
    assert_eq!(
        bytes[3],
        TagID::ByteArray as u8,
        "Field tag should be ByteArray (7)"
    );
}

#[test]
fn test_int_array_little_endian() {
    use na_nbt::de::from_slice;
    use na_nbt::ser::to_vec;
    use zerocopy::byteorder::LittleEndian;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Data {
        #[serde(with = "na_nbt::int_array")]
        values: Vec<i32>,
    }

    let original = Data {
        values: vec![0x01020304, 0x05060708],
    };
    let serialized = to_vec::<LittleEndian>(&original).unwrap();
    let deserialized: Data = from_slice::<LittleEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

#[test]
fn test_long_array_little_endian() {
    use na_nbt::de::from_slice;
    use na_nbt::ser::to_vec;
    use zerocopy::byteorder::LittleEndian;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Data {
        #[serde(with = "na_nbt::long_array")]
        values: Vec<i64>,
    }

    let original = Data {
        values: vec![0x0102030405060708i64, -1],
    };
    let serialized = to_vec::<LittleEndian>(&original).unwrap();
    let deserialized: Data = from_slice::<LittleEndian, _>(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

#[test]
fn test_large_int_array() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Data {
        #[serde(with = "na_nbt::int_array")]
        values: Vec<i32>,
    }

    let original = Data {
        values: (0..10000).collect(),
    };
    let serialized = to_vec_be(&original).unwrap();
    let deserialized: Data = from_slice_be(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

#[test]
fn test_large_long_array() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Data {
        #[serde(with = "na_nbt::long_array")]
        values: Vec<i64>,
    }

    let original = Data {
        values: (0i64..5000).collect(),
    };
    let serialized = to_vec_be(&original).unwrap();
    let deserialized: Data = from_slice_be(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

#[test]
fn test_multiple_arrays_in_struct() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct MultiArray {
        #[serde(with = "na_nbt::byte_array")]
        bytes: Vec<i8>,
        #[serde(with = "na_nbt::int_array")]
        ints: Vec<i32>,
        #[serde(with = "na_nbt::long_array")]
        longs: Vec<i64>,
        regular_list: Vec<i32>, // This stays as List<Int>
    }

    let original = MultiArray {
        bytes: vec![1, 2, 3],
        ints: vec![100, 200, 300],
        longs: vec![1000, 2000, 3000],
        regular_list: vec![10, 20, 30],
    };

    let serialized = to_vec_be(&original).unwrap();
    let deserialized: MultiArray = from_slice_be(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

#[test]
fn test_nested_struct_with_arrays() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Inner {
        #[serde(with = "na_nbt::int_array")]
        data: Vec<i32>,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Outer {
        name: String,
        inner: Inner,
    }

    let original = Outer {
        name: "test".to_string(),
        inner: Inner {
            data: vec![1, 2, 3, 4, 5],
        },
    };

    let serialized = to_vec_be(&original).unwrap();
    let deserialized: Outer = from_slice_be(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

#[test]
fn test_vec_of_structs_with_arrays() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Item {
        id: i32,
        #[serde(with = "na_nbt::int_array")]
        data: Vec<i32>,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Container {
        items: Vec<Item>,
    }

    let original = Container {
        items: vec![
            Item {
                id: 1,
                data: vec![10, 20],
            },
            Item {
                id: 2,
                data: vec![30, 40, 50],
            },
        ],
    };

    let serialized = to_vec_be(&original).unwrap();
    let deserialized: Container = from_slice_be(&serialized).unwrap();
    assert_eq!(original, deserialized);
}
