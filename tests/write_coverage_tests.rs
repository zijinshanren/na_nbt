//! Tests for write functions with endianness conversion - targeting write.rs coverage

use na_nbt::{read_borrowed, read_owned, write_value_to_vec, write_value_to_writer};
use zerocopy::byteorder::BigEndian as BE;
use zerocopy::byteorder::LittleEndian as LE;
use std::io::Cursor;

// ==================== Helper Functions ====================

fn create_byte_nbt_be(value: i8) -> Vec<u8> {
    vec![0x01, 0x00, 0x00, value as u8]
}

fn create_short_nbt_be(value: i16) -> Vec<u8> {
    let mut data = vec![0x02, 0x00, 0x00];
    data.extend_from_slice(&value.to_be_bytes());
    data
}

fn create_int_nbt_be(value: i32) -> Vec<u8> {
    let mut data = vec![0x03, 0x00, 0x00];
    data.extend_from_slice(&value.to_be_bytes());
    data
}

fn create_long_nbt_be(value: i64) -> Vec<u8> {
    let mut data = vec![0x04, 0x00, 0x00];
    data.extend_from_slice(&value.to_be_bytes());
    data
}

fn create_float_nbt_be(value: f32) -> Vec<u8> {
    let mut data = vec![0x05, 0x00, 0x00];
    data.extend_from_slice(&value.to_be_bytes());
    data
}

fn create_double_nbt_be(value: f64) -> Vec<u8> {
    let mut data = vec![0x06, 0x00, 0x00];
    data.extend_from_slice(&value.to_be_bytes());
    data
}

fn create_string_nbt_be(s: &str) -> Vec<u8> {
    let bytes = s.as_bytes();
    let mut data = vec![0x08, 0x00, 0x00];
    data.extend_from_slice(&(bytes.len() as u16).to_be_bytes());
    data.extend_from_slice(bytes);
    data
}

fn create_byte_array_nbt_be(bytes: &[i8]) -> Vec<u8> {
    let mut data = vec![0x07, 0x00, 0x00];
    data.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    for b in bytes {
        data.push(*b as u8);
    }
    data
}

fn create_int_array_nbt_be(ints: &[i32]) -> Vec<u8> {
    let mut data = vec![0x0B, 0x00, 0x00];
    data.extend_from_slice(&(ints.len() as u32).to_be_bytes());
    for i in ints {
        data.extend_from_slice(&i.to_be_bytes());
    }
    data
}

fn create_long_array_nbt_be(longs: &[i64]) -> Vec<u8> {
    let mut data = vec![0x0C, 0x00, 0x00];
    data.extend_from_slice(&(longs.len() as u32).to_be_bytes());
    for l in longs {
        data.extend_from_slice(&l.to_be_bytes());
    }
    data
}

fn create_int_list_be(ints: &[i32]) -> Vec<u8> {
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x03);
    data.extend_from_slice(&(ints.len() as u32).to_be_bytes());
    for i in ints {
        data.extend_from_slice(&i.to_be_bytes());
    }
    data
}

fn create_short_list_be(shorts: &[i16]) -> Vec<u8> {
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x02);
    data.extend_from_slice(&(shorts.len() as u32).to_be_bytes());
    for s in shorts {
        data.extend_from_slice(&s.to_be_bytes());
    }
    data
}

fn create_long_list_be(longs: &[i64]) -> Vec<u8> {
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x04);
    data.extend_from_slice(&(longs.len() as u32).to_be_bytes());
    for l in longs {
        data.extend_from_slice(&l.to_be_bytes());
    }
    data
}

fn create_float_list_be(floats: &[f32]) -> Vec<u8> {
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x05);
    data.extend_from_slice(&(floats.len() as u32).to_be_bytes());
    for f in floats {
        data.extend_from_slice(&f.to_be_bytes());
    }
    data
}

fn create_double_list_be(doubles: &[f64]) -> Vec<u8> {
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x06);
    data.extend_from_slice(&(doubles.len() as u32).to_be_bytes());
    for d in doubles {
        data.extend_from_slice(&d.to_be_bytes());
    }
    data
}

fn create_byte_list_be(bytes: &[i8]) -> Vec<u8> {
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x01);
    data.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    for b in bytes {
        data.push(*b as u8);
    }
    data
}

fn create_string_list_be(strings: &[&str]) -> Vec<u8> {
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x08);
    data.extend_from_slice(&(strings.len() as u32).to_be_bytes());
    for s in strings {
        let bytes = s.as_bytes();
        data.extend_from_slice(&(bytes.len() as u16).to_be_bytes());
        data.extend_from_slice(bytes);
    }
    data
}

fn create_byte_array_list_be(arrays: &[&[i8]]) -> Vec<u8> {
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x07);
    data.extend_from_slice(&(arrays.len() as u32).to_be_bytes());
    for arr in arrays {
        data.extend_from_slice(&(arr.len() as u32).to_be_bytes());
        for b in *arr {
            data.push(*b as u8);
        }
    }
    data
}

fn create_int_array_list_be(arrays: &[&[i32]]) -> Vec<u8> {
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x0B);
    data.extend_from_slice(&(arrays.len() as u32).to_be_bytes());
    for arr in arrays {
        data.extend_from_slice(&(arr.len() as u32).to_be_bytes());
        for i in *arr {
            data.extend_from_slice(&i.to_be_bytes());
        }
    }
    data
}

fn create_long_array_list_be(arrays: &[&[i64]]) -> Vec<u8> {
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x0C);
    data.extend_from_slice(&(arrays.len() as u32).to_be_bytes());
    for arr in arrays {
        data.extend_from_slice(&(arr.len() as u32).to_be_bytes());
        for l in *arr {
            data.extend_from_slice(&l.to_be_bytes());
        }
    }
    data
}

fn create_compound_with_all_types_be() -> Vec<u8> {
    let mut data = vec![0x0A, 0x00, 0x00];
    
    // Byte
    data.push(0x01);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'b');
    data.push(42);
    
    // Short
    data.push(0x02);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b's');
    data.extend_from_slice(&1000i16.to_be_bytes());
    
    // Int
    data.push(0x03);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'i');
    data.extend_from_slice(&100000i32.to_be_bytes());
    
    // Long
    data.push(0x04);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'l');
    data.extend_from_slice(&1000000000i64.to_be_bytes());
    
    // Float
    data.push(0x05);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'f');
    data.extend_from_slice(&3.14f32.to_be_bytes());
    
    // Double
    data.push(0x06);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'd');
    data.extend_from_slice(&2.718f64.to_be_bytes());
    
    // ByteArray
    data.push(0x07);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"ba");
    data.extend_from_slice(&3u32.to_be_bytes());
    data.extend_from_slice(&[1u8, 2, 3]);
    
    // String
    data.push(0x08);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"st");
    data.extend_from_slice(&4u16.to_be_bytes());
    data.extend_from_slice(b"test");
    
    // IntArray
    data.push(0x0B);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"ia");
    data.extend_from_slice(&2u32.to_be_bytes());
    data.extend_from_slice(&10i32.to_be_bytes());
    data.extend_from_slice(&20i32.to_be_bytes());
    
    // LongArray
    data.push(0x0C);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"la");
    data.extend_from_slice(&2u32.to_be_bytes());
    data.extend_from_slice(&100i64.to_be_bytes());
    data.extend_from_slice(&200i64.to_be_bytes());
    
    // End
    data.push(0x00);
    
    data
}

fn create_nested_list_be() -> Vec<u8> {
    // List of lists of ints
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x09); // List tag
    data.extend_from_slice(&2u32.to_be_bytes()); // 2 inner lists
    
    // First inner list [1, 2]
    data.push(0x03); // Int
    data.extend_from_slice(&2u32.to_be_bytes());
    data.extend_from_slice(&1i32.to_be_bytes());
    data.extend_from_slice(&2i32.to_be_bytes());
    
    // Second inner list [3, 4, 5]
    data.push(0x03); // Int
    data.extend_from_slice(&3u32.to_be_bytes());
    data.extend_from_slice(&3i32.to_be_bytes());
    data.extend_from_slice(&4i32.to_be_bytes());
    data.extend_from_slice(&5i32.to_be_bytes());
    
    data
}

fn create_nested_compound_be() -> Vec<u8> {
    // List of compounds
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x0A); // Compound tag
    data.extend_from_slice(&2u32.to_be_bytes()); // 2 compounds
    
    // First compound { "x": 1 }
    data.push(0x03);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'x');
    data.extend_from_slice(&1i32.to_be_bytes());
    data.push(0x00);
    
    // Second compound { "y": 2 }
    data.push(0x03);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'y');
    data.extend_from_slice(&2i32.to_be_bytes());
    data.push(0x00);
    
    data
}

// ==================== Write BE to LE Conversion Tests ====================

#[test]
fn test_write_byte_be_to_le() {
    let data = create_byte_nbt_be(42);
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    
    let output = write_value_to_vec::<_, BE, LE>(&root).unwrap();
    let owned = read_owned::<LE, LE>(&output).unwrap();
    assert_eq!(owned.as_byte(), Some(42));
}

#[test]
fn test_write_short_be_to_le() {
    let data = create_short_nbt_be(1234);
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    
    let output = write_value_to_vec::<_, BE, LE>(&root).unwrap();
    let owned = read_owned::<LE, LE>(&output).unwrap();
    assert_eq!(owned.as_short(), Some(1234));
}

#[test]
fn test_write_int_be_to_le() {
    let data = create_int_nbt_be(123456);
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    
    let output = write_value_to_vec::<_, BE, LE>(&root).unwrap();
    let owned = read_owned::<LE, LE>(&output).unwrap();
    assert_eq!(owned.as_int(), Some(123456));
}

#[test]
fn test_write_long_be_to_le() {
    let data = create_long_nbt_be(123456789012345i64);
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    
    let output = write_value_to_vec::<_, BE, LE>(&root).unwrap();
    let owned = read_owned::<LE, LE>(&output).unwrap();
    assert_eq!(owned.as_long(), Some(123456789012345i64));
}

#[test]
fn test_write_float_be_to_le() {
    let data = create_float_nbt_be(3.14159);
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    
    let output = write_value_to_vec::<_, BE, LE>(&root).unwrap();
    let owned = read_owned::<LE, LE>(&output).unwrap();
    assert!((owned.as_float().unwrap() - 3.14159).abs() < 0.0001);
}

#[test]
fn test_write_double_be_to_le() {
    let data = create_double_nbt_be(2.718281828);
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    
    let output = write_value_to_vec::<_, BE, LE>(&root).unwrap();
    let owned = read_owned::<LE, LE>(&output).unwrap();
    assert!((owned.as_double().unwrap() - 2.718281828).abs() < 0.0001);
}

#[test]
fn test_write_string_be_to_le() {
    let data = create_string_nbt_be("hello world");
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    
    let output = write_value_to_vec::<_, BE, LE>(&root).unwrap();
    let owned = read_owned::<LE, LE>(&output).unwrap();
    if let na_nbt::OwnedValue::String(s) = owned {
        assert_eq!(s.decode(), "hello world");
    } else {
        panic!("expected string");
    }
}

#[test]
fn test_write_byte_array_be_to_le() {
    let data = create_byte_array_nbt_be(&[1, 2, 3, 4, 5]);
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    
    let output = write_value_to_vec::<_, BE, LE>(&root).unwrap();
    let owned = read_owned::<LE, LE>(&output).unwrap();
    if let na_nbt::OwnedValue::ByteArray(arr) = owned {
        assert_eq!(arr.as_slice(), &[1, 2, 3, 4, 5]);
    } else {
        panic!("expected byte array");
    }
}

#[test]
fn test_write_int_array_be_to_le() {
    let data = create_int_array_nbt_be(&[100, 200, 300]);
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    
    let output = write_value_to_vec::<_, BE, LE>(&root).unwrap();
    let owned = read_owned::<LE, LE>(&output).unwrap();
    if let na_nbt::OwnedValue::IntArray(arr) = owned {
        let vals: Vec<i32> = arr.iter().map(|x| x.get()).collect();
        assert_eq!(vals, vec![100, 200, 300]);
    } else {
        panic!("expected int array");
    }
}

#[test]
fn test_write_long_array_be_to_le() {
    let data = create_long_array_nbt_be(&[1000, 2000, 3000]);
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    
    let output = write_value_to_vec::<_, BE, LE>(&root).unwrap();
    let owned = read_owned::<LE, LE>(&output).unwrap();
    if let na_nbt::OwnedValue::LongArray(arr) = owned {
        let vals: Vec<i64> = arr.iter().map(|x| x.get()).collect();
        assert_eq!(vals, vec![1000, 2000, 3000]);
    } else {
        panic!("expected long array");
    }
}

// ==================== Write List BE to LE Tests ====================

#[test]
fn test_write_byte_list_be_to_le() {
    let data = create_byte_list_be(&[1, 2, 3]);
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    
    let output = write_value_to_vec::<_, BE, LE>(&root).unwrap();
    let owned = read_owned::<LE, LE>(&output).unwrap();
    if let na_nbt::OwnedValue::List(list) = owned {
        assert_eq!(list.len(), 3);
        assert_eq!(list.get(0).unwrap().as_byte(), Some(1));
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_write_short_list_be_to_le() {
    let data = create_short_list_be(&[100, 200, 300]);
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    
    let output = write_value_to_vec::<_, BE, LE>(&root).unwrap();
    let owned = read_owned::<LE, LE>(&output).unwrap();
    if let na_nbt::OwnedValue::List(list) = owned {
        assert_eq!(list.len(), 3);
        assert_eq!(list.get(0).unwrap().as_short(), Some(100));
        assert_eq!(list.get(1).unwrap().as_short(), Some(200));
        assert_eq!(list.get(2).unwrap().as_short(), Some(300));
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_write_long_list_be_to_le() {
    let data = create_long_list_be(&[1000, 2000, 3000]);
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    
    let output = write_value_to_vec::<_, BE, LE>(&root).unwrap();
    let owned = read_owned::<LE, LE>(&output).unwrap();
    if let na_nbt::OwnedValue::List(list) = owned {
        assert_eq!(list.get(0).unwrap().as_long(), Some(1000));
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_write_float_list_be_to_le() {
    let data = create_float_list_be(&[1.5, 2.5, 3.5]);
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    
    let output = write_value_to_vec::<_, BE, LE>(&root).unwrap();
    let owned = read_owned::<LE, LE>(&output).unwrap();
    if let na_nbt::OwnedValue::List(list) = owned {
        assert!((list.get(0).unwrap().as_float().unwrap() - 1.5).abs() < 0.001);
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_write_double_list_be_to_le() {
    let data = create_double_list_be(&[1.5, 2.5, 3.5]);
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    
    let output = write_value_to_vec::<_, BE, LE>(&root).unwrap();
    let owned = read_owned::<LE, LE>(&output).unwrap();
    if let na_nbt::OwnedValue::List(list) = owned {
        assert!((list.get(0).unwrap().as_double().unwrap() - 1.5).abs() < 0.001);
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_write_string_list_be_to_le() {
    let data = create_string_list_be(&["a", "bb", "ccc"]);
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    
    let output = write_value_to_vec::<_, BE, LE>(&root).unwrap();
    let owned = read_owned::<LE, LE>(&output).unwrap();
    if let na_nbt::OwnedValue::List(list) = owned {
        assert_eq!(list.len(), 3);
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_write_byte_array_list_be_to_le() {
    let arr1: &[i8] = &[1, 2];
    let arr2: &[i8] = &[3, 4, 5];
    let data = create_byte_array_list_be(&[arr1, arr2]);
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    
    let output = write_value_to_vec::<_, BE, LE>(&root).unwrap();
    let owned = read_owned::<LE, LE>(&output).unwrap();
    if let na_nbt::OwnedValue::List(list) = owned {
        assert_eq!(list.len(), 2);
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_write_int_array_list_be_to_le() {
    let arr1: &[i32] = &[1, 2];
    let arr2: &[i32] = &[3, 4, 5];
    let data = create_int_array_list_be(&[arr1, arr2]);
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    
    let output = write_value_to_vec::<_, BE, LE>(&root).unwrap();
    let owned = read_owned::<LE, LE>(&output).unwrap();
    if let na_nbt::OwnedValue::List(list) = owned {
        assert_eq!(list.len(), 2);
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_write_long_array_list_be_to_le() {
    let arr1: &[i64] = &[1, 2];
    let arr2: &[i64] = &[3, 4, 5];
    let data = create_long_array_list_be(&[arr1, arr2]);
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    
    let output = write_value_to_vec::<_, BE, LE>(&root).unwrap();
    let owned = read_owned::<LE, LE>(&output).unwrap();
    if let na_nbt::OwnedValue::List(list) = owned {
        assert_eq!(list.len(), 2);
    } else {
        panic!("expected list");
    }
}

// ==================== Nested Structure Write Tests ====================

#[test]
fn test_write_nested_list_be_to_le() {
    let data = create_nested_list_be();
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    
    let output = write_value_to_vec::<_, BE, LE>(&root).unwrap();
    let owned = read_owned::<LE, LE>(&output).unwrap();
    if let na_nbt::OwnedValue::List(outer) = owned {
        assert_eq!(outer.len(), 2);
        let val0 = outer.get(0).unwrap();
        let inner1 = val0.as_list().unwrap();
        assert_eq!(inner1.len(), 2);
        let val1 = outer.get(1).unwrap();
        let inner2 = val1.as_list().unwrap();
        assert_eq!(inner2.len(), 3);
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_write_nested_compound_list_be_to_le() {
    let data = create_nested_compound_be();
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    
    let output = write_value_to_vec::<_, BE, LE>(&root).unwrap();
    let owned = read_owned::<LE, LE>(&output).unwrap();
    if let na_nbt::OwnedValue::List(list) = owned {
        assert_eq!(list.len(), 2);
        let val0 = list.get(0).unwrap();
        let c1 = val0.as_compound().unwrap();
        assert_eq!(c1.get("x").unwrap().as_int(), Some(1));
        let val1 = list.get(1).unwrap();
        let c2 = val1.as_compound().unwrap();
        assert_eq!(c2.get("y").unwrap().as_int(), Some(2));
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_write_compound_all_types_be_to_le() {
    let data = create_compound_with_all_types_be();
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    
    let output = write_value_to_vec::<_, BE, LE>(&root).unwrap();
    let owned = read_owned::<LE, LE>(&output).unwrap();
    if let na_nbt::OwnedValue::Compound(comp) = owned {
        assert_eq!(comp.get("b").unwrap().as_byte(), Some(42));
        assert_eq!(comp.get("s").unwrap().as_short(), Some(1000));
        assert_eq!(comp.get("i").unwrap().as_int(), Some(100000));
        assert_eq!(comp.get("l").unwrap().as_long(), Some(1000000000));
    } else {
        panic!("expected compound");
    }
}

// ==================== Write to Writer Tests ====================

#[test]
fn test_write_to_writer_int_be_to_le() {
    let data = create_int_nbt_be(999);
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    
    let mut buffer = Cursor::new(Vec::new());
    write_value_to_writer::<_, BE, LE, _>(&mut buffer, &root).unwrap();
    
    let output = buffer.into_inner();
    let owned = read_owned::<LE, LE>(&output).unwrap();
    assert_eq!(owned.as_int(), Some(999));
}

#[test]
fn test_write_to_writer_list_be_to_le() {
    let data = create_int_list_be(&[1, 2, 3]);
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    
    let mut buffer = Cursor::new(Vec::new());
    write_value_to_writer::<_, BE, LE, _>(&mut buffer, &root).unwrap();
    
    let output = buffer.into_inner();
    let owned = read_owned::<LE, LE>(&output).unwrap();
    if let na_nbt::OwnedValue::List(list) = owned {
        assert_eq!(list.len(), 3);
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_write_to_writer_compound_be_to_le() {
    let data = create_compound_with_all_types_be();
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    
    let mut buffer = Cursor::new(Vec::new());
    write_value_to_writer::<_, BE, LE, _>(&mut buffer, &root).unwrap();
    
    let output = buffer.into_inner();
    let owned = read_owned::<LE, LE>(&output).unwrap();
    assert!(owned.is_compound());
}
