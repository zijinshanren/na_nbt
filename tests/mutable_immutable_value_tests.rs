//! Tests for ScopedReadableValue impl on mutable module's ImmutableValue

use na_nbt::{
    read_owned, ByteOrder, ScopedReadableCompound, ScopedReadableList, ScopedReadableValue, Tag,
};
use zerocopy::byteorder::{BigEndian as BE, LittleEndian as LE};

// Helper functions to create test NBT data

fn create_byte_nbt(value: i8) -> Vec<u8> {
    // Compound with "root" containing Byte
    let mut data = vec![0x0A]; // Tag: Compound
    data.extend_from_slice(&0u16.to_be_bytes()); // Name length: 0 (root)
    data.push(0x01); // Tag: Byte
    data.extend_from_slice(&4u16.to_be_bytes()); // Name length: 4
    data.extend_from_slice(b"root"); // Name
    data.push(value as u8); // Value
    data.push(0x00); // End tag
    data
}

fn create_short_nbt(value: i16) -> Vec<u8> {
    let mut data = vec![0x0A];
    data.extend_from_slice(&0u16.to_be_bytes());
    data.push(0x02);
    data.extend_from_slice(&4u16.to_be_bytes());
    data.extend_from_slice(b"root");
    data.extend_from_slice(&value.to_be_bytes());
    data.push(0x00);
    data
}

fn create_int_nbt(value: i32) -> Vec<u8> {
    let mut data = vec![0x0A];
    data.extend_from_slice(&0u16.to_be_bytes());
    data.push(0x03);
    data.extend_from_slice(&4u16.to_be_bytes());
    data.extend_from_slice(b"root");
    data.extend_from_slice(&value.to_be_bytes());
    data.push(0x00);
    data
}

fn create_long_nbt(value: i64) -> Vec<u8> {
    let mut data = vec![0x0A];
    data.extend_from_slice(&0u16.to_be_bytes());
    data.push(0x04);
    data.extend_from_slice(&4u16.to_be_bytes());
    data.extend_from_slice(b"root");
    data.extend_from_slice(&value.to_be_bytes());
    data.push(0x00);
    data
}

fn create_float_nbt(value: f32) -> Vec<u8> {
    let mut data = vec![0x0A];
    data.extend_from_slice(&0u16.to_be_bytes());
    data.push(0x05);
    data.extend_from_slice(&4u16.to_be_bytes());
    data.extend_from_slice(b"root");
    data.extend_from_slice(&value.to_be_bytes());
    data.push(0x00);
    data
}

fn create_double_nbt(value: f64) -> Vec<u8> {
    let mut data = vec![0x0A];
    data.extend_from_slice(&0u16.to_be_bytes());
    data.push(0x06);
    data.extend_from_slice(&4u16.to_be_bytes());
    data.extend_from_slice(b"root");
    data.extend_from_slice(&value.to_be_bytes());
    data.push(0x00);
    data
}

fn create_byte_array_nbt(values: &[i8]) -> Vec<u8> {
    let mut data = vec![0x0A];
    data.extend_from_slice(&0u16.to_be_bytes());
    data.push(0x07);
    data.extend_from_slice(&4u16.to_be_bytes());
    data.extend_from_slice(b"root");
    data.extend_from_slice(&(values.len() as i32).to_be_bytes());
    for v in values {
        data.push(*v as u8);
    }
    data.push(0x00);
    data
}

fn create_string_nbt(value: &str) -> Vec<u8> {
    let mut data = vec![0x0A];
    data.extend_from_slice(&0u16.to_be_bytes());
    data.push(0x08);
    data.extend_from_slice(&4u16.to_be_bytes());
    data.extend_from_slice(b"root");
    data.extend_from_slice(&(value.len() as u16).to_be_bytes());
    data.extend_from_slice(value.as_bytes());
    data.push(0x00);
    data
}

fn create_int_array_nbt(values: &[i32]) -> Vec<u8> {
    let mut data = vec![0x0A];
    data.extend_from_slice(&0u16.to_be_bytes());
    data.push(0x0B);
    data.extend_from_slice(&4u16.to_be_bytes());
    data.extend_from_slice(b"root");
    data.extend_from_slice(&(values.len() as i32).to_be_bytes());
    for v in values {
        data.extend_from_slice(&v.to_be_bytes());
    }
    data.push(0x00);
    data
}

fn create_long_array_nbt(values: &[i64]) -> Vec<u8> {
    let mut data = vec![0x0A];
    data.extend_from_slice(&0u16.to_be_bytes());
    data.push(0x0C);
    data.extend_from_slice(&4u16.to_be_bytes());
    data.extend_from_slice(b"root");
    data.extend_from_slice(&(values.len() as i32).to_be_bytes());
    for v in values {
        data.extend_from_slice(&v.to_be_bytes());
    }
    data.push(0x00);
    data
}

fn create_int_list_nbt() -> Vec<u8> {
    let mut data = vec![0x0A];
    data.extend_from_slice(&0u16.to_be_bytes());
    data.push(0x09); // Tag: List
    data.extend_from_slice(&4u16.to_be_bytes());
    data.extend_from_slice(b"root");
    data.push(0x03); // Element type: Int
    data.extend_from_slice(&3i32.to_be_bytes()); // Length
    data.extend_from_slice(&1i32.to_be_bytes());
    data.extend_from_slice(&2i32.to_be_bytes());
    data.extend_from_slice(&3i32.to_be_bytes());
    data.push(0x00);
    data
}

fn create_compound_nbt() -> Vec<u8> {
    let mut data = vec![0x0A]; // Outer compound
    data.extend_from_slice(&0u16.to_be_bytes());
    data.push(0x0A); // Inner compound
    data.extend_from_slice(&4u16.to_be_bytes());
    data.extend_from_slice(b"root");
    data.push(0x03); // Int
    data.extend_from_slice(&1u16.to_be_bytes());
    data.extend_from_slice(b"x");
    data.extend_from_slice(&42i32.to_be_bytes());
    data.push(0x00); // End inner
    data.push(0x00); // End outer
    data
}

// ==================== ImmutableValue from OwnedValue.get() Tests ====================
// These test the mutable module's trait_impl.rs

#[test]
fn test_immutable_from_owned_tag_id() {
    let data = create_byte_nbt(42);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    let root = owned.get("root").unwrap();
    assert_eq!(ScopedReadableValue::tag_id(&root), Tag::Byte);
}

#[test]
fn test_immutable_from_owned_as_byte() {
    let data = create_byte_nbt(42);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    let root = owned.get("root").unwrap();
    assert_eq!(ScopedReadableValue::as_byte(&root), Some(42));
    assert!(ScopedReadableValue::is_byte(&root));
}

#[test]
fn test_immutable_from_owned_as_short() {
    let data = create_short_nbt(1234);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    let root = owned.get("root").unwrap();
    assert_eq!(ScopedReadableValue::as_short(&root), Some(1234));
    assert!(ScopedReadableValue::is_short(&root));
}

#[test]
fn test_immutable_from_owned_as_int() {
    let data = create_int_nbt(123456);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    let root = owned.get("root").unwrap();
    assert_eq!(ScopedReadableValue::as_int(&root), Some(123456));
    assert!(ScopedReadableValue::is_int(&root));
}

#[test]
fn test_immutable_from_owned_as_long() {
    let data = create_long_nbt(123456789012);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    let root = owned.get("root").unwrap();
    assert_eq!(ScopedReadableValue::as_long(&root), Some(123456789012));
    assert!(ScopedReadableValue::is_long(&root));
}

#[test]
fn test_immutable_from_owned_as_float() {
    let data = create_float_nbt(3.14);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    let root = owned.get("root").unwrap();
    let val = ScopedReadableValue::as_float(&root).unwrap();
    assert!((val - 3.14).abs() < 0.001);
    assert!(ScopedReadableValue::is_float(&root));
}

#[test]
fn test_immutable_from_owned_as_double() {
    let data = create_double_nbt(3.14159265359);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    let root = owned.get("root").unwrap();
    let val = ScopedReadableValue::as_double(&root).unwrap();
    assert!((val - 3.14159265359).abs() < 0.0000001);
    assert!(ScopedReadableValue::is_double(&root));
}

#[test]
fn test_immutable_from_owned_as_byte_array() {
    let data = create_byte_array_nbt(&[1, 2, 3, 4, 5]);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    let root = owned.get("root").unwrap();
    let arr = ScopedReadableValue::as_byte_array(&root).unwrap();
    assert_eq!(arr, &[1, 2, 3, 4, 5]);
    assert!(ScopedReadableValue::is_byte_array(&root));
}

#[test]
fn test_immutable_from_owned_as_string() {
    let data = create_string_nbt("hello");
    let owned = read_owned::<BE, BE>(&data).unwrap();
    let root = owned.get("root").unwrap();
    let s = ScopedReadableValue::as_string_scoped(&root).unwrap();
    assert_eq!(s.decode().as_ref(), "hello");
    assert!(ScopedReadableValue::is_string(&root));
}

#[test]
fn test_immutable_from_owned_as_int_array() {
    let data = create_int_array_nbt(&[1, 2, 3]);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    let root = owned.get("root").unwrap();
    let arr = ScopedReadableValue::as_int_array(&root).unwrap();
    assert_eq!(arr.len(), 3);
    assert!(ScopedReadableValue::is_int_array(&root));
}

#[test]
fn test_immutable_from_owned_as_long_array() {
    let data = create_long_array_nbt(&[1, 2, 3]);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    let root = owned.get("root").unwrap();
    let arr = ScopedReadableValue::as_long_array(&root).unwrap();
    assert_eq!(arr.len(), 3);
    assert!(ScopedReadableValue::is_long_array(&root));
}

#[test]
fn test_immutable_from_owned_as_list() {
    let data = create_int_list_nbt();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    let root = owned.get("root").unwrap();
    let list = ScopedReadableValue::as_list_scoped(&root).unwrap();
    assert_eq!(ScopedReadableList::len(&list), 3);
    assert!(ScopedReadableValue::is_list(&root));
}

#[test]
fn test_immutable_from_owned_as_compound() {
    let data = create_compound_nbt();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    let root = owned.get("root").unwrap();
    let comp = ScopedReadableValue::as_compound_scoped(&root).unwrap();
    let x = ScopedReadableCompound::get_scoped(&comp, "x").unwrap();
    assert_eq!(ScopedReadableValue::as_int(&x), Some(42));
    assert!(ScopedReadableValue::is_compound(&root));
}

#[test]
fn test_immutable_from_owned_type_mismatch() {
    let data = create_byte_nbt(42);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    let root = owned.get("root").unwrap();
    
    // Should return None for wrong types
    assert!(ScopedReadableValue::as_short(&root).is_none());
    assert!(ScopedReadableValue::as_int(&root).is_none());
    assert!(ScopedReadableValue::as_long(&root).is_none());
    assert!(ScopedReadableValue::as_float(&root).is_none());
    assert!(ScopedReadableValue::as_double(&root).is_none());
    assert!(ScopedReadableValue::as_string_scoped(&root).is_none());
    assert!(ScopedReadableValue::as_list_scoped(&root).is_none());
    assert!(ScopedReadableValue::as_compound_scoped(&root).is_none());
    assert!(ScopedReadableValue::as_byte_array(&root).is_none());
    assert!(ScopedReadableValue::as_int_array(&root).is_none());
    assert!(ScopedReadableValue::as_long_array(&root).is_none());
}

#[test]
fn test_immutable_from_owned_is_type_false() {
    let data = create_byte_nbt(42);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    let root = owned.get("root").unwrap();
    
    // Should return false for wrong types
    assert!(!ScopedReadableValue::is_short(&root));
    assert!(!ScopedReadableValue::is_int(&root));
    assert!(!ScopedReadableValue::is_long(&root));
    assert!(!ScopedReadableValue::is_float(&root));
    assert!(!ScopedReadableValue::is_double(&root));
    assert!(!ScopedReadableValue::is_string(&root));
    assert!(!ScopedReadableValue::is_list(&root));
    assert!(!ScopedReadableValue::is_compound(&root));
    assert!(!ScopedReadableValue::is_byte_array(&root));
    assert!(!ScopedReadableValue::is_int_array(&root));
    assert!(!ScopedReadableValue::is_long_array(&root));
}

// ==================== ScopedReadableList from owned ImmutableList ====================

#[test]
fn test_immutable_list_from_owned() {
    let data = create_int_list_nbt();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    let root = owned.get("root").unwrap();
    let list = ScopedReadableValue::as_list_scoped(&root).unwrap();
    
    assert_eq!(ScopedReadableList::tag_id(&list), Tag::Int);
    assert_eq!(ScopedReadableList::len(&list), 3);
    assert!(!ScopedReadableList::is_empty(&list));
    
    let v0 = ScopedReadableList::get_scoped(&list, 0).unwrap();
    assert_eq!(ScopedReadableValue::as_int(&v0), Some(1));
    
    let v1 = ScopedReadableList::get_scoped(&list, 1).unwrap();
    assert_eq!(ScopedReadableValue::as_int(&v1), Some(2));
    
    let v2 = ScopedReadableList::get_scoped(&list, 2).unwrap();
    assert_eq!(ScopedReadableValue::as_int(&v2), Some(3));
    
    assert!(ScopedReadableList::get_scoped(&list, 3).is_none());
}

#[test]
fn test_immutable_list_iter_scoped() {
    let data = create_int_list_nbt();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    let root = owned.get("root").unwrap();
    let list = ScopedReadableValue::as_list_scoped(&root).unwrap();
    
    let mut count = 0;
    for v in ScopedReadableList::iter_scoped(&list) {
        count += 1;
        assert!(ScopedReadableValue::is_int(&v));
    }
    assert_eq!(count, 3);
}

// ==================== ScopedReadableCompound from owned ImmutableCompound ====================

#[test]
fn test_immutable_compound_from_owned() {
    let data = create_compound_nbt();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    let root = owned.get("root").unwrap();
    let comp = ScopedReadableValue::as_compound_scoped(&root).unwrap();
    
    let x = ScopedReadableCompound::get_scoped(&comp, "x").unwrap();
    assert_eq!(ScopedReadableValue::as_int(&x), Some(42));
    
    assert!(ScopedReadableCompound::get_scoped(&comp, "nonexistent").is_none());
}

#[test]
fn test_immutable_compound_iter_scoped() {
    let data = create_compound_nbt();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    let root = owned.get("root").unwrap();
    let comp = ScopedReadableValue::as_compound_scoped(&root).unwrap();
    
    let mut count = 0;
    for (k, v) in ScopedReadableCompound::iter_scoped(&comp) {
        assert_eq!(k.decode(), "x");
        assert_eq!(ScopedReadableValue::as_int(&v), Some(42));
        count += 1;
    }
    assert_eq!(count, 1);
}

// ==================== get_scoped on ImmutableValue ====================

#[test]
fn test_immutable_value_get_scoped_str() {
    let data = create_compound_nbt();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    let root = owned.get("root").unwrap();
    
    let x = ScopedReadableValue::get_scoped(&root, "x").unwrap();
    assert_eq!(ScopedReadableValue::as_int(&x), Some(42));
    
    assert!(ScopedReadableValue::get_scoped(&root, "nonexistent").is_none());
}

#[test]
fn test_immutable_value_get_scoped_usize() {
    let data = create_int_list_nbt();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    let root = owned.get("root").unwrap();
    
    let v0 = ScopedReadableValue::get_scoped(&root, 0usize).unwrap();
    assert_eq!(ScopedReadableValue::as_int(&v0), Some(1));
    
    assert!(ScopedReadableValue::get_scoped(&root, 10usize).is_none());
}

// ==================== visit_scoped ====================

#[test]
fn test_immutable_value_visit_scoped_byte() {
    let data = create_byte_nbt(42);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    let root = owned.get("root").unwrap();
    
    let result = ScopedReadableValue::visit_scoped(&root, |v| {
        match v {
            na_nbt::ValueScoped::Byte(b) => b,
            _ => panic!("Expected Byte"),
        }
    });
    assert_eq!(result, 42);
}

#[test]
fn test_immutable_value_visit_scoped_string() {
    let data = create_string_nbt("hello");
    let owned = read_owned::<BE, BE>(&data).unwrap();
    let root = owned.get("root").unwrap();
    
    let result = ScopedReadableValue::visit_scoped(&root, |v| {
        match v {
            na_nbt::ValueScoped::String(s) => s.decode().to_string(),
            _ => panic!("Expected String"),
        }
    });
    assert_eq!(result, "hello");
}

#[test]
fn test_immutable_value_visit_scoped_list() {
    let data = create_int_list_nbt();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    let root = owned.get("root").unwrap();
    
    let result = ScopedReadableValue::visit_scoped(&root, |v| {
        match v {
            na_nbt::ValueScoped::List(list) => ScopedReadableList::len(&list),
            _ => panic!("Expected List"),
        }
    });
    assert_eq!(result, 3);
}

#[test]
fn test_immutable_value_visit_scoped_compound() {
    let data = create_compound_nbt();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    let root = owned.get("root").unwrap();
    
    let result = ScopedReadableValue::visit_scoped(&root, |v| {
        match v {
            na_nbt::ValueScoped::Compound(comp) => {
                let x = ScopedReadableCompound::get_scoped(&comp, "x").unwrap();
                ScopedReadableValue::as_int(&x).unwrap()
            }
            _ => panic!("Expected Compound"),
        }
    });
    assert_eq!(result, 42);
}

// ==================== LittleEndian Tests ====================

#[test]
fn test_immutable_from_owned_le_byte() {
    // LE source, LE target
    let mut data = vec![0x0A];
    data.extend_from_slice(&0u16.to_le_bytes());
    data.push(0x01);
    data.extend_from_slice(&4u16.to_le_bytes());
    data.extend_from_slice(b"root");
    data.push(99);
    data.push(0x00);
    
    let owned = read_owned::<LE, LE>(&data).unwrap();
    let root = owned.get("root").unwrap();
    assert_eq!(ScopedReadableValue::as_byte(&root), Some(99));
}

#[test]
fn test_immutable_from_owned_le_int() {
    let mut data = vec![0x0A];
    data.extend_from_slice(&0u16.to_le_bytes());
    data.push(0x03);
    data.extend_from_slice(&4u16.to_le_bytes());
    data.extend_from_slice(b"root");
    data.extend_from_slice(&12345i32.to_le_bytes());
    data.push(0x00);
    
    let owned = read_owned::<LE, LE>(&data).unwrap();
    let root = owned.get("root").unwrap();
    assert_eq!(ScopedReadableValue::as_int(&root), Some(12345));
}
