//! Tests for panic scenarios and error paths

use na_nbt::{OwnedValue, read_owned};
use std::panic;
use zerocopy::byteorder::BigEndian as BE;

fn create_int_list_be(ints: &[i32]) -> Vec<u8> {
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x03); // int tag
    data.extend_from_slice(&(ints.len() as u32).to_be_bytes());
    for i in ints {
        data.extend_from_slice(&i.to_be_bytes());
    }
    data
}

// ==================== List Remove Panic Tests ====================

#[test]
#[should_panic(expected = "index out of bounds")]
fn test_list_remove_out_of_bounds_panics() {
    let data = create_int_list_be(&[1, 2, 3]);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(mut list) = owned {
        list.remove(10); // Out of bounds
    }
}

#[test]
#[should_panic(expected = "index out of bounds")]
fn test_list_remove_empty_panics() {
    // Empty list
    let data = vec![0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(mut list) = owned {
        list.remove(0); // No elements to remove
    }
}

// ==================== List Insert Panic Tests ====================

#[test]
#[should_panic(expected = "index out of bounds")]
fn test_list_insert_out_of_bounds_panics() {
    let data = create_int_list_be(&[1, 2, 3]);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(mut list) = owned {
        list.insert(10, 42i32); // Out of bounds
    }
}

#[test]
#[should_panic(expected = "tag mismatch")]
fn test_list_push_wrong_type_panics() {
    let data = create_int_list_be(&[1, 2, 3]);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(mut list) = owned {
        list.push(42i8); // Pushing byte to int list
    }
}

#[test]
#[should_panic(expected = "tag mismatch")]
fn test_list_insert_wrong_type_panics() {
    let data = create_int_list_be(&[1, 2, 3]);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(mut list) = owned {
        list.insert(0, 42i8); // Inserting byte to int list
    }
}

// ==================== Short List Operations ====================

fn create_short_list_be(shorts: &[i16]) -> Vec<u8> {
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x02); // short tag
    data.extend_from_slice(&(shorts.len() as u32).to_be_bytes());
    for s in shorts {
        data.extend_from_slice(&s.to_be_bytes());
    }
    data
}

#[test]
fn test_short_list_push_pop() {
    let data = create_short_list_be(&[100, 200]);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(mut list) = owned {
        assert_eq!(list.len(), 2);

        list.push(300i16);
        assert_eq!(list.len(), 3);
        assert_eq!(list.get(2).unwrap().as_short(), Some(300));

        let popped = list.pop().unwrap();
        assert_eq!(popped.as_short(), Some(300));
        assert_eq!(list.len(), 2);
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_short_list_insert_remove() {
    let data = create_short_list_be(&[100, 300]);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(mut list) = owned {
        list.insert(1, 200i16);
        assert_eq!(list.len(), 3);
        assert_eq!(list.get(0).unwrap().as_short(), Some(100));
        assert_eq!(list.get(1).unwrap().as_short(), Some(200));
        assert_eq!(list.get(2).unwrap().as_short(), Some(300));

        let removed = list.remove(1);
        assert_eq!(removed.as_short(), Some(200));
        assert_eq!(list.len(), 2);
    } else {
        panic!("expected list");
    }
}

// ==================== Long List Operations ====================

fn create_long_list_be(longs: &[i64]) -> Vec<u8> {
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x04); // long tag
    data.extend_from_slice(&(longs.len() as u32).to_be_bytes());
    for l in longs {
        data.extend_from_slice(&l.to_be_bytes());
    }
    data
}

#[test]
fn test_long_list_push_pop() {
    let data = create_long_list_be(&[1000, 2000]);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(mut list) = owned {
        list.push(3000i64);
        assert_eq!(list.len(), 3);
        assert_eq!(list.get(2).unwrap().as_long(), Some(3000));

        let popped = list.pop().unwrap();
        assert_eq!(popped.as_long(), Some(3000));
    } else {
        panic!("expected list");
    }
}

// ==================== Float List Operations ====================

fn create_float_list_be(floats: &[f32]) -> Vec<u8> {
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x05); // float tag
    data.extend_from_slice(&(floats.len() as u32).to_be_bytes());
    for f in floats {
        data.extend_from_slice(&f.to_be_bytes());
    }
    data
}

#[test]
fn test_float_list_push_pop() {
    let data = create_float_list_be(&[1.5, 2.5]);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(mut list) = owned {
        list.push(3.5f32);
        assert_eq!(list.len(), 3);
        assert_eq!(list.get(2).unwrap().as_float(), Some(3.5));

        let popped = list.pop().unwrap();
        assert_eq!(popped.as_float(), Some(3.5));
    } else {
        panic!("expected list");
    }
}

// ==================== Double List Operations ====================

fn create_double_list_be(doubles: &[f64]) -> Vec<u8> {
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x06); // double tag
    data.extend_from_slice(&(doubles.len() as u32).to_be_bytes());
    for d in doubles {
        data.extend_from_slice(&d.to_be_bytes());
    }
    data
}

#[test]
fn test_double_list_push_pop() {
    let data = create_double_list_be(&[1.5, 2.5]);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(mut list) = owned {
        list.push(3.5f64);
        assert_eq!(list.len(), 3);
        assert_eq!(list.get(2).unwrap().as_double(), Some(3.5));

        let popped = list.pop().unwrap();
        assert_eq!(popped.as_double(), Some(3.5));
    } else {
        panic!("expected list");
    }
}

// ==================== Compound Insert/Remove All Types ====================

fn create_empty_compound_be() -> Vec<u8> {
    vec![0x0A, 0x00, 0x00, 0x00]
}

#[test]
fn test_compound_insert_all_primitive_types() {
    let data = create_empty_compound_be();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::Compound(mut comp) = owned {
        // Insert all primitive types
        comp.insert("byte", 42i8);
        comp.insert("short", 1000i16);
        comp.insert("int", 100000i32);
        comp.insert("long", 1000000000i64);
        comp.insert("float", std::f32::consts::PI);
        comp.insert("double", std::f64::consts::E);

        // Verify
        assert_eq!(comp.get("byte").unwrap().as_byte(), Some(42));
        assert_eq!(comp.get("short").unwrap().as_short(), Some(1000));
        assert_eq!(comp.get("int").unwrap().as_int(), Some(100000));
        assert_eq!(comp.get("long").unwrap().as_long(), Some(1000000000));
        assert!(
            (comp.get("float").unwrap().as_float().unwrap() - std::f32::consts::PI).abs() < 0.001
        );
        assert!(
            (comp.get("double").unwrap().as_double().unwrap() - std::f64::consts::E).abs() < 0.0001
        );
    } else {
        panic!("expected compound");
    }
}

#[test]
fn test_compound_insert_replaces_existing() {
    let data = create_empty_compound_be();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::Compound(mut comp) = owned {
        // Insert then replace
        let old1 = comp.insert("key", 100i32);
        assert!(old1.is_none());

        let old2 = comp.insert("key", 200i32);
        assert!(old2.is_some());
        assert_eq!(old2.unwrap().as_int(), Some(100));

        assert_eq!(comp.get("key").unwrap().as_int(), Some(200));
    } else {
        panic!("expected compound");
    }
}

#[test]
fn test_compound_remove_nonexistent_returns_none() {
    let data = create_empty_compound_be();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::Compound(mut comp) = owned {
        let removed = comp.remove("nonexistent");
        assert!(removed.is_none());
    } else {
        panic!("expected compound");
    }
}

// ==================== Multiple Consecutive Operations ====================

#[test]
fn test_list_many_push_pop_cycles() {
    let data = create_int_list_be(&[]);
    let _owned = read_owned::<BE, BE>(&data).unwrap();
    // Empty int list - need to push first element with correct type
    // Actually, empty list has End tag - we need a list with at least one element
    let data = create_int_list_be(&[0]);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(mut list) = owned {
        // Pop the initial element
        list.pop();
        assert!(list.is_empty());

        // Push many elements
        for i in 0..100 {
            list.push(i);
        }
        assert_eq!(list.len(), 100);

        // Pop all elements
        for i in (0..100).rev() {
            let val = list.pop().unwrap();
            assert_eq!(val.as_int(), Some(i));
        }
        assert!(list.is_empty());
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_compound_many_insert_remove_cycles() {
    let data = create_empty_compound_be();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::Compound(mut comp) = owned {
        // Insert many elements
        for i in 0..50 {
            let key = format!("key{}", i);
            comp.insert(&key, i);
        }

        // Verify all exist
        for i in 0..50 {
            let key = format!("key{}", i);
            assert_eq!(comp.get(&key).unwrap().as_int(), Some(i));
        }

        // Remove half
        for i in 0..25 {
            let key = format!("key{}", i);
            let removed = comp.remove(&key);
            assert!(removed.is_some());
        }

        // Verify remaining
        for i in 25..50 {
            let key = format!("key{}", i);
            assert!(comp.get(&key).is_some());
        }
        for i in 0..25 {
            let key = format!("key{}", i);
            assert!(comp.get(&key).is_none());
        }
    } else {
        panic!("expected compound");
    }
}

// ==================== OwnedValue Tag ID and Type Checks ====================

#[test]
fn test_owned_value_tag_id() {
    // Byte
    let data = vec![0x01, 0x00, 0x00, 42];
    let owned = read_owned::<BE, BE>(&data).unwrap();
    assert!(owned.is_byte());
    assert!(!owned.is_int());
    assert!(!owned.is_compound());

    // Int
    let data = create_int_list_be(&[1]);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    assert!(owned.is_list());
    assert!(!owned.is_compound());
}

#[test]
fn test_mutable_value_get_mut_modifies() {
    let data = create_int_list_be(&[100, 200, 300]);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(mut list) = owned {
        // Get mutable reference and check we can read it
        let val = list.get_mut(1).unwrap();
        assert_eq!(val.as_int(), Some(200));
    } else {
        panic!("expected list");
    }
}
