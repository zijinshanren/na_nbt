//! Tests for reading and writing NBT data

use na_nbt::{ScopedReadableValue as _, read_borrowed};
use zerocopy::byteorder::{BigEndian, LittleEndian};

/// Helper to create a simple byte NBT document
fn create_byte_nbt(value: i8) -> Vec<u8> {
    vec![
        0x01, // Tag::Byte
        0x00,
        0x00, // empty name (length = 0)
        value as u8,
    ]
}

/// Helper to create a simple short NBT document (big endian)
fn create_short_nbt_be(value: i16) -> Vec<u8> {
    let bytes = value.to_be_bytes();
    vec![
        0x02, // Tag::Short
        0x00, 0x00, // empty name
        bytes[0], bytes[1],
    ]
}

/// Helper to create a simple int NBT document (big endian)
fn create_int_nbt_be(value: i32) -> Vec<u8> {
    let bytes = value.to_be_bytes();
    vec![
        0x03, // Tag::Int
        0x00, 0x00, // empty name
        bytes[0], bytes[1], bytes[2], bytes[3],
    ]
}

/// Helper to create a simple long NBT document (big endian)
fn create_long_nbt_be(value: i64) -> Vec<u8> {
    let bytes = value.to_be_bytes();
    vec![
        0x04, // Tag::Long
        0x00, 0x00, // empty name
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ]
}

/// Helper to create a simple float NBT document (big endian)
fn create_float_nbt_be(value: f32) -> Vec<u8> {
    let bytes = value.to_be_bytes();
    vec![
        0x05, // Tag::Float
        0x00, 0x00, // empty name
        bytes[0], bytes[1], bytes[2], bytes[3],
    ]
}

/// Helper to create a simple double NBT document (big endian)
fn create_double_nbt_be(value: f64) -> Vec<u8> {
    let bytes = value.to_be_bytes();
    vec![
        0x06, // Tag::Double
        0x00, 0x00, // empty name
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ]
}

/// Helper to create a byte array NBT document (big endian)
fn create_byte_array_nbt_be(data: &[i8]) -> Vec<u8> {
    let len = data.len() as u32;
    let len_bytes = len.to_be_bytes();
    let mut result = vec![
        0x07, // Tag::ByteArray
        0x00,
        0x00, // empty name
        len_bytes[0],
        len_bytes[1],
        len_bytes[2],
        len_bytes[3],
    ];
    for &b in data {
        result.push(b as u8);
    }
    result
}

/// Helper to create a string NBT document (big endian)
fn create_string_nbt_be(s: &str) -> Vec<u8> {
    let len = s.len() as u16;
    let len_bytes = len.to_be_bytes();
    let mut result = vec![
        0x08, // Tag::String
        0x00,
        0x00, // empty name
        len_bytes[0],
        len_bytes[1],
    ];
    result.extend_from_slice(s.as_bytes());
    result
}

/// Helper to create an int array NBT document (big endian)
fn create_int_array_nbt_be(data: &[i32]) -> Vec<u8> {
    let len = data.len() as u32;
    let len_bytes = len.to_be_bytes();
    let mut result = vec![
        0x0B, // Tag::IntArray
        0x00,
        0x00, // empty name
        len_bytes[0],
        len_bytes[1],
        len_bytes[2],
        len_bytes[3],
    ];
    for &val in data {
        result.extend_from_slice(&val.to_be_bytes());
    }
    result
}

/// Helper to create a long array NBT document (big endian)
fn create_long_array_nbt_be(data: &[i64]) -> Vec<u8> {
    let len = data.len() as u32;
    let len_bytes = len.to_be_bytes();
    let mut result = vec![
        0x0C, // Tag::LongArray
        0x00,
        0x00, // empty name
        len_bytes[0],
        len_bytes[1],
        len_bytes[2],
        len_bytes[3],
    ];
    for &val in data {
        result.extend_from_slice(&val.to_be_bytes());
    }
    result
}

#[test]
fn test_read_byte() {
    let data = create_byte_nbt(42);
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();
    assert_eq!(root.as_byte(), Some(42));
}

#[test]
fn test_read_byte_negative() {
    let data = create_byte_nbt(-1);
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();
    assert_eq!(root.as_byte(), Some(-1));
}

#[test]
fn test_read_short() {
    let data = create_short_nbt_be(1234);
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();
    assert_eq!(root.as_short(), Some(1234));
}

#[test]
fn test_read_int() {
    let data = create_int_nbt_be(123456);
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();
    assert_eq!(root.as_int(), Some(123456));
}

#[test]
fn test_read_long() {
    let data = create_long_nbt_be(9876543210);
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();
    assert_eq!(root.as_long(), Some(9876543210));
}

#[test]
fn test_read_float() {
    let data = create_float_nbt_be(std::f32::consts::PI);
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();
    let v = root.as_float().unwrap();
    assert!((v - std::f32::consts::PI).abs() < 0.001);
}

#[test]
fn test_read_double() {
    let data = create_double_nbt_be(std::f64::consts::E);
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();
    let v = root.as_double().unwrap();
    assert!((v - std::f64::consts::E).abs() < 0.0000001);
}

#[test]
fn test_read_byte_array() {
    let data = create_byte_array_nbt_be(&[1, 2, 3, 4, 5]);
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();
    let arr = root.as_byte_array().unwrap();
    assert_eq!(arr, &[1, 2, 3, 4, 5]);
}

#[test]
fn test_read_string() {
    let data = create_string_nbt_be("Hello, NBT!");
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();
    let s = root.as_string().unwrap();
    assert_eq!(s.decode().as_ref(), "Hello, NBT!");
}

#[test]
fn test_read_int_array() {
    let data = create_int_array_nbt_be(&[100, 200, 300]);
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();
    let arr = root.as_int_array().unwrap();
    let values: Vec<i32> = arr.iter().map(|x| x.get()).collect();
    assert_eq!(values, vec![100, 200, 300]);
}

#[test]
fn test_read_long_array() {
    let data = create_long_array_nbt_be(&[1000, 2000, 3000]);
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();
    let arr = root.as_long_array().unwrap();
    let values: Vec<i64> = arr.iter().map(|x| x.get()).collect();
    assert_eq!(values, vec![1000, 2000, 3000]);
}

#[test]
fn test_read_end_tag() {
    let data = vec![0x00]; // Tag::End
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();
    assert!(root.is_end());
}

#[test]
fn test_write_byte_to_vec() {
    let data = create_byte_nbt(42);
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();
    let written = root.write_to_vec::<BigEndian>().unwrap();

    // The written output should be: [Tag::Byte, 0, 0, 42]
    assert_eq!(written, vec![0x01, 0x00, 0x00, 42]);
}

#[test]
fn test_write_short_to_vec() {
    let data = create_short_nbt_be(1234);
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();
    let written = root.write_to_vec::<BigEndian>().unwrap();

    let expected_bytes = 1234i16.to_be_bytes();
    assert_eq!(
        written,
        vec![0x02, 0x00, 0x00, expected_bytes[0], expected_bytes[1]]
    );
}

#[test]
fn test_write_int_to_vec() {
    let data = create_int_nbt_be(123456);
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();
    let written = root.write_to_vec::<BigEndian>().unwrap();

    let expected_bytes = 123456i32.to_be_bytes();
    assert_eq!(
        written,
        vec![
            0x03,
            0x00,
            0x00,
            expected_bytes[0],
            expected_bytes[1],
            expected_bytes[2],
            expected_bytes[3]
        ]
    );
}

#[test]
fn test_write_endianness_conversion() {
    // Read as big endian, write as little endian
    let data = create_int_nbt_be(0x12345678);
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();
    let written = root.write_to_vec::<LittleEndian>().unwrap();

    let expected_bytes = 0x12345678i32.to_le_bytes();
    assert_eq!(
        written,
        vec![
            0x03,
            0x00,
            0x00,
            expected_bytes[0],
            expected_bytes[1],
            expected_bytes[2],
            expected_bytes[3]
        ]
    );
}

#[test]
fn test_roundtrip_byte() {
    let original = create_byte_nbt(127);
    let doc = read_borrowed::<BigEndian>(&original).unwrap();
    let root = doc.root();
    let written = root.write_to_vec::<BigEndian>().unwrap();

    let doc2 = read_borrowed::<BigEndian>(&written).unwrap();
    let root2 = doc2.root();

    assert_eq!(root.as_byte(), root2.as_byte());
}

#[test]
fn test_roundtrip_string() {
    let original = create_string_nbt_be("Test string");
    let doc = read_borrowed::<BigEndian>(&original).unwrap();
    let root = doc.root();
    let written = root.write_to_vec::<BigEndian>().unwrap();

    let doc2 = read_borrowed::<BigEndian>(&written).unwrap();
    let root2 = doc2.root();

    let s1 = root.as_string().unwrap();
    let s2 = root2.as_string().unwrap();
    assert_eq!(s1.decode().as_ref(), s2.decode().as_ref());
}

#[test]
fn test_error_eof() {
    let data = vec![0x01, 0x00]; // Incomplete byte tag
    let result = read_borrowed::<BigEndian>(&data);
    assert!(result.is_err());
}

#[test]
fn test_is_methods() {
    let data = create_byte_nbt(1);
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();
    assert!(root.is_byte());
    assert!(!root.is_short());
    assert!(!root.is_int());
    assert!(!root.is_long());
    assert!(!root.is_float());
    assert!(!root.is_double());
    assert!(!root.is_byte_array());
    assert!(!root.is_string());
    assert!(!root.is_list());
    assert!(!root.is_compound());
    assert!(!root.is_int_array());
    assert!(!root.is_long_array());
    assert!(!root.is_end());
}

#[test]
fn test_tag_id() {
    use na_nbt::Tag;

    let data = create_byte_nbt(1);
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();
    assert_eq!(root.tag_id(), Tag::Byte);

    let data = create_short_nbt_be(1);
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();
    assert_eq!(root.tag_id(), Tag::Short);

    let data = create_int_nbt_be(1);
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();
    assert_eq!(root.tag_id(), Tag::Int);
}
