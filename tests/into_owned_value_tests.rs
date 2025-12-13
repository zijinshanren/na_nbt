//! Tests for IntoOwnedValue trait implementations - targeting into_owned_value.rs coverage

use na_nbt::{read_owned, OwnedList, OwnedCompound, OwnedValue};
use zerocopy::byteorder::BigEndian as BE;
use zerocopy::byteorder;

// ==================== Helper Functions ====================

fn create_empty_list_of_type(tag: u8) -> Vec<u8> {
    // List with tag type, 0 elements
    vec![0x09, 0x00, 0x00, tag, 0, 0, 0, 0]
}

fn create_empty_compound() -> Vec<u8> {
    vec![0x0A, 0x00, 0x00, 0x00]
}

fn create_int_list() -> Vec<u8> {
    // List of 3 ints
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x03); // Int tag
    data.extend_from_slice(&3u32.to_be_bytes()); // 3 elements
    data.extend_from_slice(&1i32.to_be_bytes());
    data.extend_from_slice(&2i32.to_be_bytes());
    data.extend_from_slice(&3i32.to_be_bytes());
    data
}

fn create_byte_list() -> Vec<u8> {
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x01); // Byte tag
    data.extend_from_slice(&3u32.to_be_bytes());
    data.push(1);
    data.push(2);
    data.push(3);
    data
}

fn create_short_list() -> Vec<u8> {
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x02); // Short tag
    data.extend_from_slice(&3u32.to_be_bytes());
    data.extend_from_slice(&1i16.to_be_bytes());
    data.extend_from_slice(&2i16.to_be_bytes());
    data.extend_from_slice(&3i16.to_be_bytes());
    data
}

fn create_long_list() -> Vec<u8> {
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x04); // Long tag
    data.extend_from_slice(&2u32.to_be_bytes());
    data.extend_from_slice(&100i64.to_be_bytes());
    data.extend_from_slice(&200i64.to_be_bytes());
    data
}

fn create_float_list() -> Vec<u8> {
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x05); // Float tag
    data.extend_from_slice(&2u32.to_be_bytes());
    data.extend_from_slice(&1.5f32.to_be_bytes());
    data.extend_from_slice(&2.5f32.to_be_bytes());
    data
}

fn create_double_list() -> Vec<u8> {
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x06); // Double tag
    data.extend_from_slice(&2u32.to_be_bytes());
    data.extend_from_slice(&1.5f64.to_be_bytes());
    data.extend_from_slice(&2.5f64.to_be_bytes());
    data
}

fn create_string_list() -> Vec<u8> {
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x08); // String tag
    data.extend_from_slice(&2u32.to_be_bytes());
    // "ab"
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"ab");
    // "cd"
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"cd");
    data
}

fn create_byte_array_list() -> Vec<u8> {
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x07); // ByteArray tag
    data.extend_from_slice(&2u32.to_be_bytes());
    // First array [1, 2]
    data.extend_from_slice(&2u32.to_be_bytes());
    data.push(1);
    data.push(2);
    // Second array [3, 4, 5]
    data.extend_from_slice(&3u32.to_be_bytes());
    data.push(3);
    data.push(4);
    data.push(5);
    data
}

fn create_int_array_list() -> Vec<u8> {
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x0B); // IntArray tag
    data.extend_from_slice(&2u32.to_be_bytes());
    // First array [10, 20]
    data.extend_from_slice(&2u32.to_be_bytes());
    data.extend_from_slice(&10i32.to_be_bytes());
    data.extend_from_slice(&20i32.to_be_bytes());
    // Second array [30]
    data.extend_from_slice(&1u32.to_be_bytes());
    data.extend_from_slice(&30i32.to_be_bytes());
    data
}

fn create_long_array_list() -> Vec<u8> {
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x0C); // LongArray tag
    data.extend_from_slice(&2u32.to_be_bytes());
    // First array [100]
    data.extend_from_slice(&1u32.to_be_bytes());
    data.extend_from_slice(&100i64.to_be_bytes());
    // Second array [200, 300]
    data.extend_from_slice(&2u32.to_be_bytes());
    data.extend_from_slice(&200i64.to_be_bytes());
    data.extend_from_slice(&300i64.to_be_bytes());
    data
}

// ==================== List Push Tests ====================

#[test]
fn test_list_push_byte() {
    let data = create_byte_list();
    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(ref mut list) = owned {
        list.push(4i8);
        assert_eq!(list.len(), 4);
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_list_push_short() {
    let data = create_short_list();
    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(ref mut list) = owned {
        list.push(4i16);
        assert_eq!(list.len(), 4);
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_list_push_int() {
    let data = create_int_list();
    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(ref mut list) = owned {
        list.push(4i32);
        assert_eq!(list.len(), 4);
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_list_push_long() {
    let data = create_long_list();
    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(ref mut list) = owned {
        list.push(300i64);
        assert_eq!(list.len(), 3);
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_list_push_float() {
    let data = create_float_list();
    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(ref mut list) = owned {
        list.push(3.5f32);
        assert_eq!(list.len(), 3);
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_list_push_double() {
    let data = create_double_list();
    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(ref mut list) = owned {
        list.push(3.5f64);
        assert_eq!(list.len(), 3);
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_list_push_string() {
    let data = create_string_list();
    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(ref mut list) = owned {
        list.push("ef");
        assert_eq!(list.len(), 3);
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_list_push_byte_array() {
    let data = create_byte_array_list();
    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(ref mut list) = owned {
        list.push(vec![6i8, 7i8]);
        assert_eq!(list.len(), 3);
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_list_push_int_array() {
    let data = create_int_array_list();
    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(ref mut list) = owned {
        list.push(vec![byteorder::I32::<BE>::new(40)]);
        assert_eq!(list.len(), 3);
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_list_push_long_array() {
    let data = create_long_array_list();
    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(ref mut list) = owned {
        list.push(vec![byteorder::I64::<BE>::new(400)]);
        assert_eq!(list.len(), 3);
    } else {
        panic!("expected list");
    }
}

// ==================== List Insert Tests ====================

#[test]
fn test_list_insert_byte_at_start() {
    let data = create_byte_list();
    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(ref mut list) = owned {
        list.insert(0, 0i8);
        assert_eq!(list.len(), 4);
        assert_eq!(list.get(0).unwrap().as_byte(), Some(0));
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_list_insert_short_at_middle() {
    let data = create_short_list();
    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(ref mut list) = owned {
        list.insert(1, 100i16);
        assert_eq!(list.len(), 4);
        assert_eq!(list.get(1).unwrap().as_short(), Some(100));
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_list_insert_long_at_end() {
    let data = create_long_list();
    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(ref mut list) = owned {
        list.insert(2, 300i64);
        assert_eq!(list.len(), 3);
        assert_eq!(list.get(2).unwrap().as_long(), Some(300));
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_list_insert_float() {
    let data = create_float_list();
    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(ref mut list) = owned {
        list.insert(0, 0.5f32);
        assert_eq!(list.len(), 3);
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_list_insert_double() {
    let data = create_double_list();
    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(ref mut list) = owned {
        list.insert(0, 0.5f64);
        assert_eq!(list.len(), 3);
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_list_insert_string() {
    let data = create_string_list();
    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(ref mut list) = owned {
        list.insert(1, "xy");
        assert_eq!(list.len(), 3);
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_list_insert_byte_array() {
    let data = create_byte_array_list();
    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(ref mut list) = owned {
        list.insert(1, vec![10i8, 11i8]);
        assert_eq!(list.len(), 3);
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_list_insert_int_array() {
    let data = create_int_array_list();
    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(ref mut list) = owned {
        list.insert(1, vec![byteorder::I32::<BE>::new(25)]);
        assert_eq!(list.len(), 3);
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_list_insert_long_array() {
    let data = create_long_array_list();
    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(ref mut list) = owned {
        list.insert(1, vec![byteorder::I64::<BE>::new(250)]);
        assert_eq!(list.len(), 3);
    } else {
        panic!("expected list");
    }
}

// ==================== Compound Insert Tests ====================

#[test]
fn test_compound_insert_long() {
    let data = create_empty_compound();
    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::Compound(ref mut comp) = owned {
        comp.insert("l", 12345678901234i64);
        assert_eq!(comp.get("l").unwrap().as_long(), Some(12345678901234));
    } else {
        panic!("expected compound");
    }
}

#[test]
fn test_compound_insert_byte_array_slice() {
    let data = create_empty_compound();
    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::Compound(ref mut comp) = owned {
        comp.insert("ba", &[1i8, 2i8, 3i8][..]);
        assert!(comp.get("ba").is_some());
    } else {
        panic!("expected compound");
    }
}

#[test]
fn test_compound_insert_byte_array_fixed() {
    let data = create_empty_compound();
    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::Compound(ref mut comp) = owned {
        comp.insert("ba", [4i8, 5i8]);
        assert!(comp.get("ba").is_some());
    } else {
        panic!("expected compound");
    }
}

#[test]
fn test_compound_insert_int_array() {
    let data = create_empty_compound();
    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::Compound(ref mut comp) = owned {
        comp.insert("ia", vec![byteorder::I32::<BE>::new(10), byteorder::I32::<BE>::new(20)]);
        assert!(comp.get("ia").is_some());
    } else {
        panic!("expected compound");
    }
}

#[test]
fn test_compound_insert_long_array() {
    let data = create_empty_compound();
    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::Compound(ref mut comp) = owned {
        comp.insert("la", vec![byteorder::I64::<BE>::new(100), byteorder::I64::<BE>::new(200)]);
        assert!(comp.get("la").is_some());
    } else {
        panic!("expected compound");
    }
}

#[test]
fn test_compound_insert_string_ref() {
    let data = create_empty_compound();
    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::Compound(ref mut comp) = owned {
        comp.insert("s", "hello world");
        assert!(comp.get("s").is_some());
    } else {
        panic!("expected compound");
    }
}

#[test]
fn test_compound_insert_owned_value() {
    let data = create_empty_compound();
    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::Compound(ref mut comp) = owned {
        let v = OwnedValue::<BE>::Int(byteorder::I32::<BE>::new(999));
        comp.insert("v", v);
        assert_eq!(comp.get("v").unwrap().as_int(), Some(999));
    } else {
        panic!("expected compound");
    }
}

#[test]
fn test_compound_insert_owned_list() {
    let list_data = create_int_list();
    let list_owned = read_owned::<BE, BE>(&list_data).unwrap();
    
    let data = create_empty_compound();
    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::Compound(ref mut comp) = owned {
        if let OwnedValue::List(list) = list_owned {
            comp.insert("list", list);
            assert!(comp.get("list").is_some());
        }
    } else {
        panic!("expected compound");
    }
}

#[test]
fn test_compound_insert_owned_compound() {
    let inner_data = create_empty_compound();
    let inner_owned = read_owned::<BE, BE>(&inner_data).unwrap();
    
    let data = create_empty_compound();
    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::Compound(ref mut comp) = owned {
        if let OwnedValue::Compound(inner) = inner_owned {
            comp.insert("inner", inner);
            assert!(comp.get("inner").is_some());
        }
    } else {
        panic!("expected compound");
    }
}

// ==================== List Push with OwnedValue ====================

#[test]
fn test_list_push_owned_value() {
    let data = create_int_list();
    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(ref mut list) = owned {
        let v = OwnedValue::<BE>::Int(byteorder::I32::<BE>::new(999));
        list.push(v);
        assert_eq!(list.len(), 4);
    } else {
        panic!("expected list");
    }
}

// ==================== Nested List Insert/Push ====================

fn create_list_of_lists() -> Vec<u8> {
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x09); // List tag
    data.extend_from_slice(&2u32.to_be_bytes()); // 2 inner lists
    
    // First inner list [1, 2]
    data.push(0x03); // Int
    data.extend_from_slice(&2u32.to_be_bytes());
    data.extend_from_slice(&1i32.to_be_bytes());
    data.extend_from_slice(&2i32.to_be_bytes());
    
    // Second inner list [3]
    data.push(0x03); // Int
    data.extend_from_slice(&1u32.to_be_bytes());
    data.extend_from_slice(&3i32.to_be_bytes());
    
    data
}

#[test]
fn test_list_push_owned_list() {
    let data = create_list_of_lists();
    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    
    // Create a new inner list
    let inner_data = create_int_list();
    let inner_owned = read_owned::<BE, BE>(&inner_data).unwrap();
    
    if let OwnedValue::List(ref mut list) = owned {
        if let OwnedValue::List(inner_list) = inner_owned {
            list.push(inner_list);
            assert_eq!(list.len(), 3);
        }
    } else {
        panic!("expected list");
    }
}

fn create_list_of_compounds() -> Vec<u8> {
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x0A); // Compound tag
    data.extend_from_slice(&1u32.to_be_bytes()); // 1 compound
    
    // Compound { "x": 1 }
    data.push(0x03);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'x');
    data.extend_from_slice(&1i32.to_be_bytes());
    data.push(0x00);
    
    data
}

#[test]
fn test_list_push_owned_compound() {
    let data = create_list_of_compounds();
    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    
    // Create a new inner compound
    let inner_data = create_empty_compound();
    let inner_owned = read_owned::<BE, BE>(&inner_data).unwrap();
    
    if let OwnedValue::List(ref mut list) = owned {
        if let OwnedValue::Compound(inner_comp) = inner_owned {
            list.push(inner_comp);
            assert_eq!(list.len(), 2);
        }
    } else {
        panic!("expected list");
    }
}

// ==================== Empty List Type Conversion ====================

#[test]
fn test_empty_list_push_establishes_type() {
    // Create an empty End list
    let data = create_empty_list_of_type(0); // End tag
    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(ref mut list) = owned {
        assert_eq!(list.len(), 0);
        // Pushing an int should work on empty list
        list.push(42i32);
        assert_eq!(list.len(), 1);
    } else {
        panic!("expected list");
    }
}
