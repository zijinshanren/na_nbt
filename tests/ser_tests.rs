use na_nbt::Tag;
use na_nbt::ser::to_vec;
use serde::Serialize;
use std::{collections::HashMap, f32, f64};
use zerocopy::byteorder::{BigEndian, LittleEndian};

// Simple wrapper for serializing bytes
#[derive(Serialize)]
struct ByteVec(#[serde(with = "serde_bytes_impl")] Vec<u8>);

mod serde_bytes_impl {
    use serde::Serializer;

    pub fn serialize<S: Serializer>(data: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(data)
    }
}

// NBT root header size: tag_id (1 byte) + name_length (2 bytes, always 0 for empty name)
const ROOT_HEADER_SIZE: usize = 3;

// Helper to get payload (skip root header)
fn payload(data: &[u8]) -> &[u8] {
    &data[ROOT_HEADER_SIZE..]
}

// Helper to get root tag from serialized data
fn root_tag(data: &[u8]) -> u8 {
    data[0]
}

// Helper to read u16 in big-endian
fn read_be_u16(data: &[u8]) -> u16 {
    u16::from_be_bytes([data[0], data[1]])
}

// Helper to read u32 in big-endian
fn read_be_u32(data: &[u8]) -> u32 {
    u32::from_be_bytes([data[0], data[1], data[2], data[3]])
}

// Helper to read i32 in big-endian
fn read_be_i32(data: &[u8]) -> i32 {
    i32::from_be_bytes([data[0], data[1], data[2], data[3]])
}

// Helper to read i64 in big-endian
fn read_be_i64(data: &[u8]) -> i64 {
    i64::from_be_bytes([
        data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
    ])
}

// Helper to read i16 in big-endian
fn read_be_i16(data: &[u8]) -> i16 {
    i16::from_be_bytes([data[0], data[1]])
}

// Helper to read f32 in big-endian
fn read_be_f32(data: &[u8]) -> f32 {
    f32::from_be_bytes([data[0], data[1], data[2], data[3]])
}

// Helper to read f64 in big-endian
fn read_be_f64(data: &[u8]) -> f64 {
    f64::from_be_bytes([
        data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
    ])
}

#[test]
fn test_root_header_format() {
    // Verify the root header format: [tag_id, name_len_hi, name_len_lo]
    let result = to_vec::<BigEndian>(&42i8).unwrap();
    
    // First byte is tag_id (Byte = 1)
    assert_eq!(result[0], Tag::Byte as u8);
    // Next 2 bytes are name length (0 for empty name)
    assert_eq!(read_be_u16(&result[1..3]), 0);
    // Payload starts at byte 3
    assert_eq!(result[3], 42u8);
}

#[test]
fn test_serialize_primitives() {
    // Test i8 (Byte)
    let result = to_vec::<BigEndian>(&42i8).unwrap();
    assert_eq!(root_tag(&result), Tag::Byte as u8);
    assert_eq!(payload(&result), &[42u8]);

    // Test bool (serialized as i8)
    let result = to_vec::<BigEndian>(&true).unwrap();
    assert_eq!(root_tag(&result), Tag::Byte as u8);
    assert_eq!(payload(&result), &[1u8]);
    let result = to_vec::<BigEndian>(&false).unwrap();
    assert_eq!(payload(&result), &[0u8]);

    // Test i16 (Short)
    let result = to_vec::<BigEndian>(&0x1234i16).unwrap();
    assert_eq!(root_tag(&result), Tag::Short as u8);
    assert_eq!(payload(&result), &[0x12, 0x34]);

    // Test i32 (Int)
    let result = to_vec::<BigEndian>(&0x12345678i32).unwrap();
    assert_eq!(root_tag(&result), Tag::Int as u8);
    assert_eq!(payload(&result), &[0x12, 0x34, 0x56, 0x78]);

    // Test i64 (Long)
    let result = to_vec::<BigEndian>(&0x123456789ABCDEFi64).unwrap();
    assert_eq!(root_tag(&result), Tag::Long as u8);
    assert_eq!(read_be_i64(payload(&result)), 0x123456789ABCDEFi64);

    // Test f32 (Float)
    let result = to_vec::<BigEndian>(&f32::consts::PI).unwrap();
    assert_eq!(root_tag(&result), Tag::Float as u8);
    assert_eq!(payload(&result).len(), 4);
    assert!((read_be_f32(payload(&result)) - f32::consts::PI).abs() < 0.001);

    // Test f64 (Double)
    let result = to_vec::<BigEndian>(&f64::consts::PI).unwrap();
    assert_eq!(root_tag(&result), Tag::Double as u8);
    assert_eq!(payload(&result).len(), 8);
    assert!((read_be_f64(payload(&result)) - f64::consts::PI).abs() < 0.0000001);
}

#[test]
fn test_serialize_unsigned_as_signed() {
    // u8 serializes as i8
    let result = to_vec::<BigEndian>(&255u8).unwrap();
    assert_eq!(root_tag(&result), Tag::Byte as u8);
    assert_eq!(payload(&result), &[255u8]); // -1 as i8

    // u16 serializes as i16
    let result = to_vec::<BigEndian>(&0xFFFFu16).unwrap();
    assert_eq!(root_tag(&result), Tag::Short as u8);
    assert_eq!(read_be_i16(payload(&result)), -1i16);

    // u32 serializes as i32
    let result = to_vec::<BigEndian>(&0xFFFFFFFFu32).unwrap();
    assert_eq!(root_tag(&result), Tag::Int as u8);
    assert_eq!(read_be_i32(payload(&result)), -1i32);

    // u64 serializes as i64
    let result = to_vec::<BigEndian>(&0xFFFFFFFFFFFFFFFFu64).unwrap();
    assert_eq!(root_tag(&result), Tag::Long as u8);
    assert_eq!(read_be_i64(payload(&result)), -1i64);
}

#[test]
fn test_serialize_string_with_length_prefix() {
    // String serialization should include length prefix
    let result = to_vec::<BigEndian>(&"Hello").unwrap();
    assert_eq!(root_tag(&result), Tag::String as u8);

    let p = payload(&result);
    // First 2 bytes of payload should be string length (5)
    assert_eq!(read_be_u16(&p[0..2]), 5);
    // Following bytes should be the string content
    assert_eq!(&p[2..], b"Hello");

    // Empty string
    let result = to_vec::<BigEndian>(&"").unwrap();
    let p = payload(&result);
    assert_eq!(read_be_u16(&p[0..2]), 0);
    assert_eq!(p.len(), 2);
}

#[test]
fn test_serialize_string_with_unicode() {
    // Test with unicode characters (will be MUTF-8 encoded)
    let result = to_vec::<BigEndian>(&"你好").unwrap();
    assert_eq!(root_tag(&result), Tag::String as u8);

    let p = payload(&result);
    // Length should be encoded length, not char count
    let len = read_be_u16(&p[0..2]) as usize;
    assert!(len > 2); // MUTF-8 encoding of Chinese chars
    assert_eq!(p.len(), 2 + len);
}

#[test]
fn test_serialize_bytes_as_byte_array() {
    let bytes = ByteVec(vec![1, 2, 3, 4, 5]);
    let result = to_vec::<BigEndian>(&bytes).unwrap();
    assert_eq!(root_tag(&result), Tag::ByteArray as u8);

    let p = payload(&result);
    // First 4 bytes should be length (5)
    assert_eq!(read_be_u32(&p[0..4]), 5);
    // Following bytes should be the byte content
    assert_eq!(&p[4..], &[1, 2, 3, 4, 5]);
}

#[test]
fn test_serialize_unit() {
    // Unit serializes as empty compound with End tag
    let result = to_vec::<BigEndian>(&()).unwrap();
    assert_eq!(root_tag(&result), Tag::Compound as u8);
    assert_eq!(payload(&result), &[Tag::End as u8]);
}

#[test]
fn test_serialize_none_as_unit() {
    let opt: Option<i32> = None;
    let result = to_vec::<BigEndian>(&opt).unwrap();
    assert_eq!(root_tag(&result), Tag::Compound as u8);
    assert_eq!(payload(&result), &[Tag::End as u8]);
}

#[test]
fn test_serialize_some() {
    let opt: Option<i32> = Some(42);
    let result = to_vec::<BigEndian>(&opt).unwrap();
    // Some is serialized as a compound with single unnamed field
    assert_eq!(root_tag(&result), Tag::Compound as u8);
    let p = payload(&result);
    // Structure: [field_tag, name_len(2 bytes, 0), value, End]
    assert_eq!(p[0], Tag::Int as u8); // field tag
    assert_eq!(p[1], 0); // name len hi
    assert_eq!(p[2], 0); // name len lo
    assert_eq!(read_be_i32(&p[3..7]), 42); // value
    assert_eq!(p[7], Tag::End as u8); // end of compound
}

#[test]
fn test_serialize_struct() {
    #[derive(Serialize)]
    struct Player {
        health: i32,
        name: String,
    }

    let player = Player {
        health: 100,
        name: "Steve".to_string(),
    };

    let result = to_vec::<BigEndian>(&player).unwrap();
    assert_eq!(root_tag(&result), Tag::Compound as u8);

    // Should start with field entries and end with End tag
    assert!(!result.is_empty());
    assert_eq!(*result.last().unwrap(), Tag::End as u8);

    // Should contain field names
    assert!(result.windows(6).any(|w| w == b"health"));
    assert!(result.windows(4).any(|w| w == b"name"));
}

#[test]
fn test_serialize_nested_struct() {
    #[derive(Serialize)]
    struct Inner {
        value: i32,
    }

    #[derive(Serialize)]
    struct Outer {
        inner: Inner,
        count: i32,
    }

    let outer = Outer {
        inner: Inner { value: 42 },
        count: 10,
    };

    let result = to_vec::<BigEndian>(&outer).unwrap();
    assert_eq!(root_tag(&result), Tag::Compound as u8);

    // Should have End tags for both compounds
    assert!(!result.is_empty());
    // Count End tags
    let end_count = result.iter().filter(|&&b| b == Tag::End as u8).count();
    assert!(
        end_count >= 2,
        "Should have at least 2 End tags for nested compounds"
    );
}

#[test]
fn test_serialize_vec_as_list() {
    let vec = vec![1i32, 2i32, 3i32];
    let result = to_vec::<BigEndian>(&vec).unwrap();
    assert_eq!(root_tag(&result), Tag::List as u8);

    let p = payload(&result);
    // List structure: element_type (1 byte) + length (4 bytes) + elements
    assert_eq!(p[0], Tag::Int as u8); // Element type matches the vec content type
    assert_eq!(read_be_u32(&p[1..5]), 3); // 3 elements
}

#[test]
fn test_serialize_tuple() {
    let tuple = (1i32, 2i32, 3i32);
    let result = to_vec::<BigEndian>(&tuple).unwrap();
    assert_eq!(root_tag(&result), Tag::List as u8);

    let p = payload(&result);
    // Should have list header
    assert_eq!(p[0], Tag::Compound as u8);
    assert_eq!(read_be_u32(&p[1..5]), 3);
}

#[test]
fn test_serialize_tuple_struct() {
    #[derive(Serialize)]
    struct Point(i32, i32, i32);

    let point = Point(1, 2, 3);
    let result = to_vec::<BigEndian>(&point).unwrap();
    // Note: tuple_struct now serializes differently (user reverted the change)
    assert_eq!(root_tag(&result), Tag::List as u8);
}

#[test]
fn test_serialize_hashmap() {
    let mut map: HashMap<String, i32> = HashMap::new();
    map.insert("key1".to_string(), 10);
    map.insert("key2".to_string(), 20);

    let result = to_vec::<BigEndian>(&map).unwrap();
    assert_eq!(root_tag(&result), Tag::Compound as u8);

    // Should end with End tag
    assert_eq!(*result.last().unwrap(), Tag::End as u8);

    // Should contain field names
    assert!(result.windows(4).any(|w| w == b"key1"));
    assert!(result.windows(4).any(|w| w == b"key2"));
}

#[test]
fn test_serialize_map_with_str_key() {
    let mut map: HashMap<&str, i32> = HashMap::new();
    map.insert("hello", 42);

    let result = to_vec::<BigEndian>(&map).unwrap();
    assert_eq!(root_tag(&result), Tag::Compound as u8);
    assert!(!result.is_empty());
    assert_eq!(*result.last().unwrap(), Tag::End as u8);
}

#[test]
fn test_serialize_map_key_must_be_string() {
    let mut map: HashMap<i32, i32> = HashMap::new();
    map.insert(1, 10);

    let result = to_vec::<BigEndian>(&map);
    assert!(result.is_err());
}

#[test]
fn test_serialize_enum_unit_variant() {
    #[derive(Serialize)]
    enum Status {
        Active,
        Inactive,
        Pending,
    }

    // Unit variants serialize as their index (u32 -> Int tag)
    let result = to_vec::<BigEndian>(&Status::Active).unwrap();
    assert_eq!(root_tag(&result), Tag::Int as u8);
    assert_eq!(read_be_u32(payload(&result)), 0);

    let result = to_vec::<BigEndian>(&Status::Inactive).unwrap();
    assert_eq!(read_be_u32(payload(&result)), 1);

    let result = to_vec::<BigEndian>(&Status::Pending).unwrap();
    assert_eq!(read_be_u32(payload(&result)), 2);
}

#[test]
fn test_serialize_enum_newtype_variant() {
    #[derive(Serialize)]
    enum Data {
        Int(i32),
        Text(String),
    }

    let result = to_vec::<BigEndian>(&Data::Int(42)).unwrap();
    assert_eq!(root_tag(&result), Tag::Compound as u8);
    // Should be a compound with variant name as key, ending with End tag
    assert_eq!(*result.last().unwrap(), Tag::End as u8);
    assert!(result.windows(3).any(|w| w == b"Int"));

    let result = to_vec::<BigEndian>(&Data::Text("hello".to_string())).unwrap();
    assert_eq!(root_tag(&result), Tag::Compound as u8);
    assert_eq!(*result.last().unwrap(), Tag::End as u8);
    assert!(result.windows(4).any(|w| w == b"Text"));
}

#[test]
fn test_serialize_enum_tuple_variant() {
    #[derive(Serialize)]
    #[allow(dead_code)]
    enum Coord {
        Point2D(i32, i32),
        Point3D(i32, i32, i32),
    }

    let result = to_vec::<BigEndian>(&Coord::Point2D(1, 2)).unwrap();
    // Tuple variants serialize as Compound { variant: List[Compound] }
    assert_eq!(root_tag(&result), Tag::Compound as u8);
    // Should contain variant name and end with End tag
    assert_eq!(*result.last().unwrap(), Tag::End as u8);
    assert!(result.windows(7).any(|w| w == b"Point2D"));
}

#[test]
fn test_serialize_enum_struct_variant() {
    #[derive(Serialize)]
    #[allow(dead_code)]
    enum Entity {
        Player { name: String, health: i32 },
        Monster { kind: String, damage: i32 },
    }

    let result = to_vec::<BigEndian>(&Entity::Player {
        name: "Steve".to_string(),
        health: 100,
    })
    .unwrap();
    assert_eq!(root_tag(&result), Tag::Compound as u8);

    // Should have nested compounds and end with two End tags
    let end_count = result.iter().filter(|&&b| b == Tag::End as u8).count();
    assert!(end_count >= 2, "Should have End tags for nested compounds");
    assert!(result.windows(6).any(|w| w == b"Player"));
}

#[test]
fn test_serialize_newtype_struct() {
    #[derive(Serialize)]
    struct Wrapper(i32);

    let result = to_vec::<BigEndian>(&Wrapper(42)).unwrap();
    assert_eq!(root_tag(&result), Tag::Int as u8);
    // Newtype struct should serialize as the inner value
    assert_eq!(payload(&result), &[0x00, 0x00, 0x00, 42]);
}

#[test]
fn test_serialize_unit_struct() {
    #[derive(Serialize)]
    struct Empty;

    let result = to_vec::<BigEndian>(&Empty).unwrap();
    assert_eq!(root_tag(&result), Tag::Compound as u8);
    // Should be an empty compound with End tag
    assert_eq!(payload(&result), &[Tag::End as u8]);
}

#[test]
fn test_serialize_little_endian() {
    // Test that little endian serialization works
    let result = to_vec::<LittleEndian>(&0x12345678i32).unwrap();
    assert_eq!(result[0], Tag::Int as u8);
    // Name length in little endian (0)
    assert_eq!(result[1], 0);
    assert_eq!(result[2], 0);
    // Payload in little endian
    assert_eq!(&result[3..], &[0x78, 0x56, 0x34, 0x12]);

    let result = to_vec::<LittleEndian>(&0x1234i16).unwrap();
    assert_eq!(result[0], Tag::Short as u8);
    assert_eq!(&result[3..], &[0x34, 0x12]);
}

#[test]
fn test_serialize_char_as_int() {
    // Char is serialized as u32 (Int tag)
    let result = to_vec::<BigEndian>(&'A').unwrap();
    assert_eq!(root_tag(&result), Tag::Int as u8);
    assert_eq!(payload(&result).len(), 4);
    assert_eq!(read_be_u32(payload(&result)), 65);
}

#[test]
fn test_serialize_empty_vec() {
    let empty: Vec<i32> = vec![];
    let result = to_vec::<BigEndian>(&empty).unwrap();
    assert_eq!(root_tag(&result), Tag::List as u8);

    let p = payload(&result);
    // Empty list has element type End (no elements serialized to determine type)
    assert_eq!(p[0], Tag::End as u8);
    assert_eq!(read_be_u32(&p[1..5]), 0);
}

#[test]
fn test_serialize_empty_hashmap() {
    let empty: HashMap<String, i32> = HashMap::new();
    let result = to_vec::<BigEndian>(&empty).unwrap();
    assert_eq!(root_tag(&result), Tag::Compound as u8);

    // Empty compound should just be End tag
    assert_eq!(payload(&result), &[Tag::End as u8]);
}

#[test]
fn test_serialize_complex_nested_structure() {
    #[derive(Serialize)]
    struct Inventory {
        items: Vec<Item>,
    }

    #[derive(Serialize)]
    struct Item {
        id: i32,
        count: i32,
        name: String,
    }

    let inventory = Inventory {
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

    let result = to_vec::<BigEndian>(&inventory).unwrap();
    assert_eq!(root_tag(&result), Tag::Compound as u8);
    assert!(!result.is_empty());

    // Verify structure contains expected strings
    assert!(result.windows(5).any(|w| w == b"items"));
    assert!(result.windows(5).any(|w| w == b"Stone"));
    assert!(result.windows(4).any(|w| w == b"Wood"));
}

#[test]
fn test_serialize_array_types() {
    // Vec<u8> with our ByteVec wrapper should become ByteArray
    let bytes = ByteVec(vec![1u8, 2, 3, 4, 5]);
    let result = to_vec::<BigEndian>(&bytes).unwrap();
    assert_eq!(root_tag(&result), Tag::ByteArray as u8);

    let p = payload(&result);
    // ByteArray: length (4 bytes) + data
    assert_eq!(read_be_u32(&p[0..4]), 5);
    assert_eq!(&p[4..], &[1, 2, 3, 4, 5]);
}

#[test]
fn test_serialize_special_utf8() {
    // Test null character (should be encoded as modified UTF-8)
    let s = "hello\0world";
    let result = to_vec::<BigEndian>(&s).unwrap();
    assert_eq!(root_tag(&result), Tag::String as u8);

    let p = payload(&result);
    // Length prefix should be present
    let len = read_be_u16(&p[0..2]) as usize;
    assert!(len > 0);

    // Should use modified UTF-8 encoding for null (0xC0 0x80)
    // Total length should be 12: "hello" (5) + modified null (2) + "world" (5)
    assert_eq!(len, 12);
}
