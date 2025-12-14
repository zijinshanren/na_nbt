//! Tests for trait implementations wrappers

use na_nbt::{ReadableValue, Value, read_borrowed};
use zerocopy::byteorder::BigEndian as BE;

fn create_byte_nbt(value: i8) -> Vec<u8> {
    vec![0x01, 0x00, 0x00, value as u8]
}

fn create_string_nbt_be(s: &str) -> Vec<u8> {
    let len = s.len() as u16;
    let len_bytes = len.to_be_bytes();
    let mut result = vec![0x08, 0x00, 0x00, len_bytes[0], len_bytes[1]];
    result.extend_from_slice(s.as_bytes());
    result
}

fn create_empty_list_nbt_be() -> Vec<u8> {
    vec![0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
}

fn create_empty_compound_nbt_be() -> Vec<u8> {
    vec![0x0A, 0x00, 0x00, 0x00]
}

#[test]
fn test_value_visit_byte() {
    let data = create_byte_nbt(123);
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let res = root.visit(|v| match v {
        Value::Byte(b) => b,
        _ => panic!("expected byte"),
    });
    assert_eq!(res, 123);
}

#[test]
fn test_value_visit_string() {
    let data = create_string_nbt_be("hello");
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let res = root.visit(|v| match v {
        Value::String(s) => s.decode().to_string(),
        _ => panic!("expected string"),
    });
    assert_eq!(res, "hello");
}

#[test]
fn test_value_visit_list_compound() {
    let data = create_empty_list_nbt_be();
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let list_tag = root.visit(|v| matches!(v, Value::List(_)));
    assert!(list_tag);

    let data = create_empty_compound_nbt_be();
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let comp_tag = root.visit(|v| matches!(v, Value::Compound(_)));
    assert!(comp_tag);
}
