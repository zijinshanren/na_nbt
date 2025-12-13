//! Tests for mutable read fallbacks

use na_nbt::read_owned;
use na_nbt::OwnedValue;
use na_nbt::ImmutableValue;
use zerocopy::byteorder::{BigEndian, LittleEndian};

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

#[test]
fn test_read_owned_list_fallback() {
    let data = create_int_list_nbt_be(&[0x01020304, 0x05060708]);
    let owned = read_owned::<BigEndian, LittleEndian>(&data).unwrap();

    if let OwnedValue::List(list) = owned {
        assert_eq!(list.len(), 2);
        let mut iter = list.into_iter();
        let v1 = iter.next().unwrap();
        if let OwnedValue::Int(i) = v1 {
            assert_eq!(i.get(), 0x01020304); // numeric value should match source content
        } else {
            panic!("expected int");
        }
    } else {
        panic!("expected list");
    }
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
fn test_read_owned_compound_fallback() {
    let data = create_compound_int_entry_be("a", 0x0A0B0C0D);
    let owned = read_owned::<BigEndian, LittleEndian>(&data).unwrap();

    if let OwnedValue::Compound(compound) = owned {
        let value = compound.get("a").unwrap();
        if let ImmutableValue::Int(i) = value {
            assert_eq!(i, 0x0A0B0C0D);
        } else {
            panic!("expected int");
        }
    } else {
        panic!("expected compound");
    }
}
