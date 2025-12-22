//! Tests for compound and list NBT structures

use na_nbt::read_borrowed;
use zerocopy::byteorder::BigEndian;

/// Helper to create an empty compound NBT document (big endian)
fn create_empty_compound_nbt_be() -> Vec<u8> {
    vec![
        0x0A, // Tag::Compound
        0x00, 0x00, // empty name
        0x00, // Tag::End (compound terminator)
    ]
}

/// Helper to create a compound with a single byte entry
fn create_compound_with_byte_be(name: &str, value: i8) -> Vec<u8> {
    let name_len = name.len() as u16;
    let name_bytes = name_len.to_be_bytes();
    let mut result = vec![
        0x0A, // Tag::Compound
        0x00, 0x00, // empty root name
    ];
    // Add byte entry
    result.push(0x01); // Tag::Byte
    result.extend_from_slice(&name_bytes);
    result.extend_from_slice(name.as_bytes());
    result.push(value as u8);
    // End compound
    result.push(0x00);
    result
}

/// Helper to create a compound with multiple entries
fn create_compound_with_entries_be() -> Vec<u8> {
    let mut result = vec![
        0x0A, // Tag::Compound
        0x00, 0x00, // empty root name
    ];

    // Add byte entry "a" = 1
    result.push(0x01); // Tag::Byte
    result.extend_from_slice(&[0x00, 0x01]); // name length = 1
    result.push(b'a');
    result.push(1);

    // Add short entry "bb" = 256
    result.push(0x02); // Tag::Short
    result.extend_from_slice(&[0x00, 0x02]); // name length = 2
    result.extend_from_slice(b"bb");
    result.extend_from_slice(&256i16.to_be_bytes());

    // Add int entry "ccc" = 65536
    result.push(0x03); // Tag::Int
    result.extend_from_slice(&[0x00, 0x03]); // name length = 3
    result.extend_from_slice(b"ccc");
    result.extend_from_slice(&65536i32.to_be_bytes());

    // End compound
    result.push(0x00);
    result
}

/// Helper to create an empty list NBT document (big endian)
fn create_empty_list_nbt_be() -> Vec<u8> {
    vec![
        0x09, // Tag::List
        0x00, 0x00, // empty name
        0x00, // element type = End (empty list)
        0x00, 0x00, 0x00, 0x00, // length = 0
    ]
}

/// Helper to create a list of bytes
fn create_byte_list_nbt_be(values: &[i8]) -> Vec<u8> {
    let len = values.len() as u32;
    let len_bytes = len.to_be_bytes();
    let mut result = vec![
        0x09, // Tag::List
        0x00, 0x00, // empty name
        0x01, // element type = Byte
        len_bytes[0],
        len_bytes[1],
        len_bytes[2],
        len_bytes[3],
    ];
    for &v in values {
        result.push(v as u8);
    }
    result
}

/// Helper to create a list of ints
fn create_int_list_nbt_be(values: &[i32]) -> Vec<u8> {
    let len = values.len() as u32;
    let len_bytes = len.to_be_bytes();
    let mut result = vec![
        0x09, // Tag::List
        0x00, 0x00, // empty name
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

/// Helper to create a list of strings
fn create_string_list_nbt_be(values: &[&str]) -> Vec<u8> {
    let len = values.len() as u32;
    let len_bytes = len.to_be_bytes();
    let mut result = vec![
        0x09, // Tag::List
        0x00, 0x00, // empty name
        0x08, // element type = String
        len_bytes[0],
        len_bytes[1],
        len_bytes[2],
        len_bytes[3],
    ];
    for s in values {
        let str_len = s.len() as u16;
        result.extend_from_slice(&str_len.to_be_bytes());
        result.extend_from_slice(s.as_bytes());
    }
    result
}

/// Helper to create nested compounds
fn create_nested_compound_nbt_be() -> Vec<u8> {
    let mut result = vec![
        0x0A, // Tag::Compound
        0x00, 0x00, // empty root name
    ];

    // Add nested compound entry "inner"
    result.push(0x0A); // Tag::Compound
    result.extend_from_slice(&[0x00, 0x05]); // name length = 5
    result.extend_from_slice(b"inner");

    // Add byte inside nested compound
    result.push(0x01); // Tag::Byte
    result.extend_from_slice(&[0x00, 0x01]); // name length = 1
    result.push(b'x');
    result.push(42);

    // End inner compound
    result.push(0x00);

    // End outer compound
    result.push(0x00);
    result
}

#[test]
fn test_read_empty_compound() {
    let data = create_empty_compound_nbt_be();
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();

    assert!(root.is_compound());
    let compound = root.as_compound().unwrap();
    // Empty compound - get should return None for any key
    assert!(compound.get("anything").is_none());
}

#[test]
fn test_read_compound_with_byte() {
    let data = create_compound_with_byte_be("test", 42);
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();

    assert!(root.is_compound());
    let compound = root.as_compound().unwrap();
    let entry = compound.get("test").unwrap();
    assert_eq!(entry.as_byte(), Some(42));
}

#[test]
fn test_read_compound_with_multiple_entries() {
    let data = create_compound_with_entries_be();
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();

    assert!(root.is_compound());
    let compound = root.as_compound().unwrap();

    assert_eq!(compound.get("a").unwrap().as_byte(), Some(1));
    assert_eq!(compound.get("bb").unwrap().as_short(), Some(256));
    assert_eq!(compound.get("ccc").unwrap().as_int(), Some(65536));
}

#[test]
fn test_compound_get_nonexistent() {
    let data = create_compound_with_byte_be("test", 42);
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();

    let compound = root.as_compound().unwrap();
    assert!(compound.get("nonexistent").is_none());
}

#[test]
fn test_read_empty_list() {
    let data = create_empty_list_nbt_be();
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();

    assert!(root.is_list());
    let list = root.as_list().unwrap();
    assert_eq!(list.len(), 0);
    assert!(list.is_empty());
}

#[test]
fn test_read_byte_list() {
    let data = create_byte_list_nbt_be(&[1, 2, 3, 4, 5]);
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();

    assert!(root.is_list());
    let list = root.as_list().unwrap();
    assert_eq!(list.len(), 5);

    // Iterate and check values
    let bytes: Vec<i8> = list.iter().map(|v| v.as_byte().unwrap()).collect();
    assert_eq!(bytes, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_read_int_list() {
    let data = create_int_list_nbt_be(&[100, 200, 300]);
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();

    assert!(root.is_list());
    let list = root.as_list().unwrap();
    assert_eq!(list.len(), 3);

    let ints: Vec<i32> = list.iter().map(|v| v.as_int().unwrap()).collect();
    assert_eq!(ints, vec![100, 200, 300]);
}

#[test]
fn test_read_string_list() {
    let data = create_string_list_nbt_be(&["hello", "world"]);
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();

    assert!(root.is_list());
    let list = root.as_list().unwrap();
    assert_eq!(list.len(), 2);

    let strings: Vec<String> = list
        .iter()
        .map(|v| v.as_string().unwrap().decode().to_string())
        .collect();
    assert_eq!(strings, vec!["hello", "world"]);
}

#[test]
fn test_read_nested_compound() {
    let data = create_nested_compound_nbt_be();
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();

    assert!(root.is_compound());
    let outer = root.as_compound().unwrap();

    let inner = outer.get("inner").unwrap();
    assert!(inner.is_compound());
    let inner_compound = inner.as_compound().unwrap();

    assert_eq!(inner_compound.get("x").unwrap().as_byte(), Some(42));
}

#[test]
fn test_compound_iteration() {
    let data = create_compound_with_entries_be();
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();

    let compound = root.as_compound().unwrap();
    let mut count = 0;
    for (_name, _value) in compound.iter() {
        count += 1;
    }
    assert_eq!(count, 3);
}

#[test]
fn test_list_indexing() {
    let data = create_byte_list_nbt_be(&[10, 20, 30]);
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();

    let list = root.as_list().unwrap();
    assert_eq!(list.get(0).unwrap().as_byte(), Some(10));
    assert_eq!(list.get(1).unwrap().as_byte(), Some(20));
    assert_eq!(list.get(2).unwrap().as_byte(), Some(30));
    assert!(list.get(3).is_none());
}

#[test]
fn test_list_tag_id() {
    use na_nbt::TagID;

    let data = create_byte_list_nbt_be(&[1, 2, 3]);
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();

    let list = root.as_list().unwrap();
    assert_eq!(list.tag_id(), TagID::Byte);

    let data = create_int_list_nbt_be(&[1, 2, 3]);
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();

    let list = root.as_list().unwrap();
    assert_eq!(list.tag_id(), TagID::Int);
}

#[test]
fn test_list_into_iter() {
    let data = create_byte_list_nbt_be(&[1, 2, 3]);
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();

    if let na_nbt::BorrowedValue::List(list) = root {
        let values: Vec<i8> = list.into_iter().map(|v| v.as_byte().unwrap()).collect();
        assert_eq!(values, vec![1, 2, 3]);
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_compound_into_iter() {
    let data = create_compound_with_entries_be();
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();

    if let na_nbt::BorrowedValue::Compound(compound) = root {
        let entries: Vec<_> = compound
            .into_iter()
            .map(|(name, _value)| name.decode().to_string())
            .collect();
        assert_eq!(entries.len(), 3);
        assert!(entries.contains(&"a".to_string()));
        assert!(entries.contains(&"bb".to_string()));
        assert!(entries.contains(&"ccc".to_string()));
    } else {
        panic!("Expected compound");
    }
}
