//! Tests for immutable write fallbacks

use na_nbt::read_borrowed;
use std::io::Cursor;
use zerocopy::byteorder::{BigEndian, LittleEndian};

fn create_int_list_nbt_be(values: &[i32]) -> Vec<u8> {
    let len = values.len() as u32;
    let len_bytes = len.to_be_bytes();
    let mut result = vec![
        0x09, // Tag::List
        0x00,
        0x00, // empty name
        0x03, // element type = Int
        len_bytes[0],
        len_bytes[1],
        len_bytes[2],
        len_bytes[3],
    ];
    for &v in values {
        result.extend_from_slice(&v.to_be_bytes());
    }
    result
}

fn create_compound_int_entry_be(name: &str, value: i32) -> Vec<u8> {
    // Create a compound with a single int entry
    let name_len = name.len() as u16;
    let name_bytes = name_len.to_be_bytes();
    let mut result = vec![0x0A, 0x00, 0x00]; // Compound, empty root name
    result.push(0x03); // Tag::Int
    result.extend_from_slice(&name_bytes);
    result.extend_from_slice(name.as_bytes());
    result.extend_from_slice(&value.to_be_bytes());
    result.push(0x00); // End compound
    result
}

#[test]
fn test_write_list_endianness_conversion_to_vec() {
    let data = create_int_list_nbt_be(&[0x11223344, 0x55667788]);
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();

    // Write with target little-endian to force fallback conversion
    let written = root.write_to_vec::<LittleEndian>().unwrap();

    // The length and ints should be little-endian now
    let expected = vec![
        0x09, 0x00, 0x00, 0x03, 0x02, 0x00, 0x00,
        0x00, // header + element type + len (little-endian)
        0x44, 0x33, 0x22, 0x11, // 0x11223344 little-endian
        0x88, 0x77, 0x66, 0x55, // 0x55667788 little-endian
    ];
    assert_eq!(written, expected);
}

#[test]
fn test_write_compound_endianness_conversion_to_vec() {
    let data = create_compound_int_entry_be("a", 0x01020304);
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();

    let written = root.write_to_vec::<LittleEndian>().unwrap();

    // Expect int bytes to be little-endian
    // Name length (u16) will be converted to little-endian, so 0x0001 -> 0x01 0x00
    let expected_start = vec![0x0A, 0x00, 0x00, 0x03, 0x01, 0x00, b'a']; // compound + tag + name_len (LE) + name
    assert!(written.starts_with(&expected_start));
    // Find data bytes - last 5 bytes (4 bytes int + 0x00 end)
    let int_bytes = &written[written.len() - 5..written.len() - 1];
    assert_eq!(int_bytes, &[0x04, 0x03, 0x02, 0x01]);
}

#[test]
fn test_write_list_endianness_conversion_to_writer() {
    let data = create_int_list_nbt_be(&[0x10203040]);
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();

    let mut cursor = Cursor::new(Vec::new());
    root.write_to_writer::<LittleEndian>(&mut cursor).unwrap();
    let written = cursor.into_inner();

    // Int should be little endian
    let expected = vec![
        0x09, 0x00, 0x00, 0x03, 0x01, 0x00, 0x00, 0x00, 0x40, 0x30, 0x20, 0x10,
    ];
    assert_eq!(written, expected);
}
