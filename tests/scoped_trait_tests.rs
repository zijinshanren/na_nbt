//! Tests for ScopedReadableValue trait implementations - targeting trait_impl.rs and trait_impl_own.rs

use na_nbt::{
    read_borrowed, read_owned, ScopedReadableValue, ScopedReadableList, ScopedReadableCompound,
    OwnedValue, Tag,
};
use zerocopy::byteorder::BigEndian as BE;

// ==================== Helper Functions ====================

fn create_byte_nbt(v: i8) -> Vec<u8> {
    vec![0x01, 0x00, 0x00, v as u8]
}

fn create_short_nbt(v: i16) -> Vec<u8> {
    let mut data = vec![0x02, 0x00, 0x00];
    data.extend_from_slice(&v.to_be_bytes());
    data
}

fn create_int_nbt(v: i32) -> Vec<u8> {
    let mut data = vec![0x03, 0x00, 0x00];
    data.extend_from_slice(&v.to_be_bytes());
    data
}

fn create_long_nbt(v: i64) -> Vec<u8> {
    let mut data = vec![0x04, 0x00, 0x00];
    data.extend_from_slice(&v.to_be_bytes());
    data
}

fn create_float_nbt(v: f32) -> Vec<u8> {
    let mut data = vec![0x05, 0x00, 0x00];
    data.extend_from_slice(&v.to_be_bytes());
    data
}

fn create_double_nbt(v: f64) -> Vec<u8> {
    let mut data = vec![0x06, 0x00, 0x00];
    data.extend_from_slice(&v.to_be_bytes());
    data
}

fn create_byte_array_nbt(bytes: &[i8]) -> Vec<u8> {
    let mut data = vec![0x07, 0x00, 0x00];
    data.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    for b in bytes {
        data.push(*b as u8);
    }
    data
}

fn create_string_nbt(s: &str) -> Vec<u8> {
    let bytes = s.as_bytes();
    let mut data = vec![0x08, 0x00, 0x00];
    data.extend_from_slice(&(bytes.len() as u16).to_be_bytes());
    data.extend_from_slice(bytes);
    data
}

fn create_int_array_nbt(ints: &[i32]) -> Vec<u8> {
    let mut data = vec![0x0B, 0x00, 0x00];
    data.extend_from_slice(&(ints.len() as u32).to_be_bytes());
    for i in ints {
        data.extend_from_slice(&i.to_be_bytes());
    }
    data
}

fn create_long_array_nbt(longs: &[i64]) -> Vec<u8> {
    let mut data = vec![0x0C, 0x00, 0x00];
    data.extend_from_slice(&(longs.len() as u32).to_be_bytes());
    for l in longs {
        data.extend_from_slice(&l.to_be_bytes());
    }
    data
}

fn create_int_list_nbt() -> Vec<u8> {
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x03); // Int tag
    data.extend_from_slice(&3u32.to_be_bytes());
    data.extend_from_slice(&1i32.to_be_bytes());
    data.extend_from_slice(&2i32.to_be_bytes());
    data.extend_from_slice(&3i32.to_be_bytes());
    data
}

fn create_compound_nbt() -> Vec<u8> {
    let mut data = vec![0x0A, 0x00, 0x00];
    // Int field "x"
    data.push(0x03);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'x');
    data.extend_from_slice(&42i32.to_be_bytes());
    // End
    data.push(0x00);
    data
}

fn create_end_nbt() -> Vec<u8> {
    vec![0x00]
}

// ==================== ImmutableValue ScopedReadableValue Tests ====================

#[test]
fn test_immutable_scoped_tag_id() {
    let data = create_byte_nbt(42);
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    assert_eq!(ScopedReadableValue::tag_id(&root), Tag::Byte);
}

#[test]
fn test_immutable_scoped_as_byte() {
    let data = create_byte_nbt(42);
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    assert_eq!(ScopedReadableValue::as_byte(&root), Some(42));
    assert!(ScopedReadableValue::is_byte(&root));
}

#[test]
fn test_immutable_scoped_as_short() {
    let data = create_short_nbt(1234);
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    assert_eq!(ScopedReadableValue::as_short(&root), Some(1234));
    assert!(ScopedReadableValue::is_short(&root));
}

#[test]
fn test_immutable_scoped_as_int() {
    let data = create_int_nbt(123456);
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    assert_eq!(ScopedReadableValue::as_int(&root), Some(123456));
    assert!(ScopedReadableValue::is_int(&root));
}

#[test]
fn test_immutable_scoped_as_long() {
    let data = create_long_nbt(123456789012345i64);
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    assert_eq!(ScopedReadableValue::as_long(&root), Some(123456789012345i64));
    assert!(ScopedReadableValue::is_long(&root));
}

#[test]
fn test_immutable_scoped_as_float() {
    let data = create_float_nbt(3.14);
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let f = ScopedReadableValue::as_float(&root).unwrap();
    assert!((f - 3.14).abs() < 0.001);
    assert!(ScopedReadableValue::is_float(&root));
}

#[test]
fn test_immutable_scoped_as_double() {
    let data = create_double_nbt(2.718281828);
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let d = ScopedReadableValue::as_double(&root).unwrap();
    assert!((d - 2.718281828).abs() < 0.0001);
    assert!(ScopedReadableValue::is_double(&root));
}

#[test]
fn test_immutable_scoped_as_byte_array() {
    let data = create_byte_array_nbt(&[1, 2, 3, 4]);
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let arr = ScopedReadableValue::as_byte_array(&root).unwrap();
    assert_eq!(arr, &[1, 2, 3, 4]);
    assert!(ScopedReadableValue::is_byte_array(&root));
}

#[test]
fn test_immutable_scoped_as_string() {
    let data = create_string_nbt("hello");
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let s = ScopedReadableValue::as_string_scoped(&root).unwrap();
    assert_eq!(s.decode(), "hello");
    assert!(ScopedReadableValue::is_string(&root));
}

#[test]
fn test_immutable_scoped_as_list() {
    let data = create_int_list_nbt();
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let list = ScopedReadableValue::as_list_scoped(&root).unwrap();
    assert_eq!(ScopedReadableList::len(&list), 3);
    assert!(ScopedReadableValue::is_list(&root));
}

#[test]
fn test_immutable_scoped_as_compound() {
    let data = create_compound_nbt();
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let comp = ScopedReadableValue::as_compound_scoped(&root).unwrap();
    let x = ScopedReadableCompound::get_scoped(&comp, "x").unwrap();
    assert_eq!(ScopedReadableValue::as_int(&x), Some(42));
    assert!(ScopedReadableValue::is_compound(&root));
}

#[test]
fn test_immutable_scoped_as_int_array() {
    let data = create_int_array_nbt(&[10, 20, 30]);
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let arr = ScopedReadableValue::as_int_array(&root).unwrap();
    assert_eq!(arr.len(), 3);
    assert!(ScopedReadableValue::is_int_array(&root));
}

#[test]
fn test_immutable_scoped_as_long_array() {
    let data = create_long_array_nbt(&[100, 200, 300]);
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let arr = ScopedReadableValue::as_long_array(&root).unwrap();
    assert_eq!(arr.len(), 3);
    assert!(ScopedReadableValue::is_long_array(&root));
}

#[test]
fn test_immutable_scoped_get_scoped_int() {
    let data = create_int_list_nbt();
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let v = ScopedReadableValue::get_scoped(&root, 0usize).unwrap();
    assert_eq!(ScopedReadableValue::as_int(&v), Some(1));
}

#[test]
fn test_immutable_scoped_get_scoped_str() {
    let data = create_compound_nbt();
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let v = ScopedReadableValue::get_scoped(&root, "x").unwrap();
    assert_eq!(ScopedReadableValue::as_int(&v), Some(42));
}

// ==================== OwnedValue ScopedReadableValue Tests ====================

#[test]
fn test_owned_scoped_tag_id() {
    let data = create_byte_nbt(42);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    assert_eq!(ScopedReadableValue::tag_id(&owned), Tag::Byte);
}

#[test]
fn test_owned_scoped_as_byte() {
    let data = create_byte_nbt(42);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    assert_eq!(ScopedReadableValue::as_byte(&owned), Some(42));
    assert!(ScopedReadableValue::is_byte(&owned));
}

#[test]
fn test_owned_scoped_as_short() {
    let data = create_short_nbt(1234);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    assert_eq!(ScopedReadableValue::as_short(&owned), Some(1234));
    assert!(ScopedReadableValue::is_short(&owned));
}

#[test]
fn test_owned_scoped_as_int() {
    let data = create_int_nbt(123456);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    assert_eq!(ScopedReadableValue::as_int(&owned), Some(123456));
    assert!(ScopedReadableValue::is_int(&owned));
}

#[test]
fn test_owned_scoped_as_long() {
    let data = create_long_nbt(123456789012345i64);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    assert_eq!(ScopedReadableValue::as_long(&owned), Some(123456789012345i64));
    assert!(ScopedReadableValue::is_long(&owned));
}

#[test]
fn test_owned_scoped_as_float() {
    let data = create_float_nbt(3.14);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    let f = ScopedReadableValue::as_float(&owned).unwrap();
    assert!((f - 3.14).abs() < 0.001);
    assert!(ScopedReadableValue::is_float(&owned));
}

#[test]
fn test_owned_scoped_as_double() {
    let data = create_double_nbt(2.718281828);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    let d = ScopedReadableValue::as_double(&owned).unwrap();
    assert!((d - 2.718281828).abs() < 0.0001);
    assert!(ScopedReadableValue::is_double(&owned));
}

#[test]
fn test_owned_scoped_as_byte_array() {
    let data = create_byte_array_nbt(&[1, 2, 3, 4]);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    let arr = ScopedReadableValue::as_byte_array(&owned).unwrap();
    assert_eq!(arr, &[1, 2, 3, 4]);
    assert!(ScopedReadableValue::is_byte_array(&owned));
}

#[test]
fn test_owned_scoped_as_string() {
    let data = create_string_nbt("hello");
    let owned = read_owned::<BE, BE>(&data).unwrap();
    let s = ScopedReadableValue::as_string_scoped(&owned).unwrap();
    assert_eq!(s.decode(), "hello");
    assert!(ScopedReadableValue::is_string(&owned));
}

#[test]
fn test_owned_scoped_as_list() {
    let data = create_int_list_nbt();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    let list = ScopedReadableValue::as_list_scoped(&owned).unwrap();
    assert_eq!(ScopedReadableList::len(&list), 3);
    assert!(ScopedReadableValue::is_list(&owned));
}

#[test]
fn test_owned_scoped_as_compound() {
    let data = create_compound_nbt();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    let comp = ScopedReadableValue::as_compound_scoped(&owned).unwrap();
    let x = ScopedReadableCompound::get_scoped(&comp, "x").unwrap();
    assert_eq!(ScopedReadableValue::as_int(&x), Some(42));
    assert!(ScopedReadableValue::is_compound(&owned));
}

#[test]
fn test_owned_scoped_as_int_array() {
    let data = create_int_array_nbt(&[10, 20, 30]);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    let arr = ScopedReadableValue::as_int_array(&owned).unwrap();
    assert_eq!(arr.len(), 3);
    assert!(ScopedReadableValue::is_int_array(&owned));
}

#[test]
fn test_owned_scoped_as_long_array() {
    let data = create_long_array_nbt(&[100, 200, 300]);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    let arr = ScopedReadableValue::as_long_array(&owned).unwrap();
    assert_eq!(arr.len(), 3);
    assert!(ScopedReadableValue::is_long_array(&owned));
}

#[test]
fn test_owned_scoped_get_scoped_int() {
    let data = create_int_list_nbt();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    let v = ScopedReadableValue::get_scoped(&owned, 0usize).unwrap();
    assert_eq!(ScopedReadableValue::as_int(&v), Some(1));
}

#[test]
fn test_owned_scoped_get_scoped_str() {
    let data = create_compound_nbt();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    let v = ScopedReadableValue::get_scoped(&owned, "x").unwrap();
    assert_eq!(ScopedReadableValue::as_int(&v), Some(42));
}

// ==================== End Value Tests ====================

#[test]
fn test_immutable_scoped_as_end() {
    let data = create_end_nbt();
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    assert_eq!(ScopedReadableValue::as_end(&root), Some(()));
    assert!(ScopedReadableValue::is_end(&root));
}

#[test]
fn test_owned_scoped_as_end() {
    let owned = OwnedValue::<BE>::End;
    assert_eq!(ScopedReadableValue::as_end(&owned), Some(()));
    assert!(ScopedReadableValue::is_end(&owned));
}

// ==================== Type Mismatch Tests ====================

#[test]
fn test_immutable_type_mismatch_returns_none() {
    let data = create_byte_nbt(42);
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    
    assert_eq!(ScopedReadableValue::as_short(&root), None);
    assert_eq!(ScopedReadableValue::as_int(&root), None);
    assert_eq!(ScopedReadableValue::as_long(&root), None);
    assert_eq!(ScopedReadableValue::as_float(&root), None);
    assert_eq!(ScopedReadableValue::as_double(&root), None);
    assert!(ScopedReadableValue::as_byte_array(&root).is_none());
    assert!(ScopedReadableValue::as_string_scoped(&root).is_none());
    assert!(ScopedReadableValue::as_list_scoped(&root).is_none());
    assert!(ScopedReadableValue::as_compound_scoped(&root).is_none());
    assert!(ScopedReadableValue::as_int_array(&root).is_none());
    assert!(ScopedReadableValue::as_long_array(&root).is_none());
    assert_eq!(ScopedReadableValue::as_end(&root), None);
    
    assert!(!ScopedReadableValue::is_short(&root));
    assert!(!ScopedReadableValue::is_int(&root));
    assert!(!ScopedReadableValue::is_long(&root));
    assert!(!ScopedReadableValue::is_float(&root));
    assert!(!ScopedReadableValue::is_double(&root));
    assert!(!ScopedReadableValue::is_byte_array(&root));
    assert!(!ScopedReadableValue::is_string(&root));
    assert!(!ScopedReadableValue::is_list(&root));
    assert!(!ScopedReadableValue::is_compound(&root));
    assert!(!ScopedReadableValue::is_int_array(&root));
    assert!(!ScopedReadableValue::is_long_array(&root));
    assert!(!ScopedReadableValue::is_end(&root));
}

#[test]
fn test_owned_type_mismatch_returns_none() {
    let data = create_byte_nbt(42);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    
    assert_eq!(ScopedReadableValue::as_short(&owned), None);
    assert_eq!(ScopedReadableValue::as_int(&owned), None);
    assert_eq!(ScopedReadableValue::as_long(&owned), None);
    assert_eq!(ScopedReadableValue::as_float(&owned), None);
    assert_eq!(ScopedReadableValue::as_double(&owned), None);
    assert!(ScopedReadableValue::as_byte_array(&owned).is_none());
    assert!(ScopedReadableValue::as_string_scoped(&owned).is_none());
    assert!(ScopedReadableValue::as_list_scoped(&owned).is_none());
    assert!(ScopedReadableValue::as_compound_scoped(&owned).is_none());
    assert!(ScopedReadableValue::as_int_array(&owned).is_none());
    assert!(ScopedReadableValue::as_long_array(&owned).is_none());
    assert_eq!(ScopedReadableValue::as_end(&owned), None);
    
    assert!(!ScopedReadableValue::is_short(&owned));
    assert!(!ScopedReadableValue::is_int(&owned));
    assert!(!ScopedReadableValue::is_long(&owned));
    assert!(!ScopedReadableValue::is_float(&owned));
    assert!(!ScopedReadableValue::is_double(&owned));
    assert!(!ScopedReadableValue::is_byte_array(&owned));
    assert!(!ScopedReadableValue::is_string(&owned));
    assert!(!ScopedReadableValue::is_list(&owned));
    assert!(!ScopedReadableValue::is_compound(&owned));
    assert!(!ScopedReadableValue::is_int_array(&owned));
    assert!(!ScopedReadableValue::is_long_array(&owned));
    assert!(!ScopedReadableValue::is_end(&owned));
}

// ==================== ScopedReadableList Tests ====================

#[test]
fn test_immutable_scoped_list_methods() {
    let data = create_int_list_nbt();
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let list = ScopedReadableValue::as_list_scoped(&root).unwrap();
    
    assert_eq!(list.len(), 3);
    assert!(!list.is_empty());
    assert_eq!(list.tag_id(), Tag::Int);
    
    let v0 = list.get(0).unwrap();
    assert_eq!(v0.as_int(), Some(1));
    
    let v1 = list.get(1).unwrap();
    assert_eq!(v1.as_int(), Some(2));
    
    let v2 = list.get(2).unwrap();
    assert_eq!(v2.as_int(), Some(3));
    
    assert!(list.get(3).is_none());
    
    // Test iter
    let mut count = 0;
    for v in list.iter() {
        count += 1;
        assert!(v.is_int());
    }
    assert_eq!(count, 3);
}

#[test]
fn test_owned_scoped_list_methods() {
    let data = create_int_list_nbt();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    let list = ScopedReadableValue::as_list_scoped(&owned).unwrap();
    
    assert_eq!(list.len(), 3);
    assert!(!list.is_empty());
    assert_eq!(list.tag_id(), Tag::Int);
    
    let v0 = list.get(0).unwrap();
    assert_eq!(v0.as_int(), Some(1));
}

// ==================== ScopedReadableCompound Tests ====================

#[test]
fn test_immutable_scoped_compound_methods() {
    let data = create_compound_nbt();
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let comp = ScopedReadableValue::as_compound_scoped(&root).unwrap();
    
    // Test get
    let x = ScopedReadableCompound::get_scoped(&comp, "x").unwrap();
    assert_eq!(ScopedReadableValue::as_int(&x), Some(42));
    
    assert!(ScopedReadableCompound::get_scoped(&comp, "nonexistent").is_none());
    
    // Test iter
    let mut count = 0;
    for (k, v) in ScopedReadableCompound::iter_scoped(&comp) {
        assert_eq!(k.decode(), "x");
        assert_eq!(ScopedReadableValue::as_int(&v), Some(42));
        count += 1;
    }
    assert_eq!(count, 1);
}

#[test]
fn test_owned_scoped_compound_methods() {
    let data = create_compound_nbt();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    let comp = ScopedReadableValue::as_compound_scoped(&owned).unwrap();
    
    // Test get
    let x = ScopedReadableCompound::get_scoped(&comp, "x").unwrap();
    assert_eq!(ScopedReadableValue::as_int(&x), Some(42));
}
