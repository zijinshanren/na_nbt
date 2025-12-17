//! Extended edge case tests - empty containers, boundary conditions, type mismatches

use na_nbt::{OwnedValue, read_borrowed, read_owned};
use zerocopy::byteorder::BigEndian as BE;
use zerocopy::byteorder::LittleEndian as LE;

// ==================== Empty List Tests ====================

fn create_empty_list_be() -> Vec<u8> {
    // Root list with tag=End and length=0
    vec![0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
}

fn create_empty_compound_be() -> Vec<u8> {
    // Root compound with just TAG_END
    vec![0x0A, 0x00, 0x00, 0x00]
}

#[test]
fn test_empty_list_read_borrowed() {
    let data = create_empty_list_be();
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let list = root.as_list().unwrap();
    assert!(list.is_empty());
    assert_eq!(list.len(), 0);
    assert!(list.get(0).is_none());
    assert_eq!(list.iter().count(), 0);
}

#[test]
fn test_empty_list_read_owned() {
    let data = create_empty_list_be();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(mut list) = owned {
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
        assert!(list.get(0).is_none());
        assert!(list.get_mut(0).is_none());
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_empty_compound_read_borrowed() {
    let data = create_empty_compound_be();
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let comp = root.as_compound().unwrap();
    assert!(comp.get("any").is_none());
    assert_eq!(comp.iter().count(), 0);
}

#[test]
fn test_empty_compound_read_owned() {
    let data = create_empty_compound_be();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::Compound(mut comp) = owned {
        assert!(comp.get("any").is_none());
        assert!(comp.get_mut("any").is_none());
    } else {
        panic!("expected compound");
    }
}

// ==================== Empty List Push/Pop ====================

#[test]
fn test_empty_list_pop_returns_none() {
    let data = create_empty_list_be();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(mut list) = owned {
        assert!(list.pop().is_none());
        // List should still be valid after pop on empty
        assert!(list.is_empty());
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_empty_list_push_then_pop() {
    let data = create_empty_list_be();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(mut list) = owned {
        // Push a byte to empty list
        list.push(42i8);
        assert_eq!(list.len(), 1);
        assert!(!list.is_empty());

        // Pop it back
        let popped = list.pop().unwrap();
        assert_eq!(popped.as_byte(), Some(42));
        assert!(list.is_empty());
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_empty_compound_insert_then_remove() {
    let data = create_empty_compound_be();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::Compound(mut comp) = owned {
        // Insert into empty compound
        let old = comp.insert("key", 123i32);
        assert!(old.is_none()); // No previous value

        // Verify it exists
        assert!(comp.get("key").is_some());
        assert_eq!(comp.get("key").unwrap().as_int(), Some(123));

        // Remove it
        let removed = comp.remove("key");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().as_int(), Some(123));

        // Should be empty again
        assert!(comp.get("key").is_none());
    } else {
        panic!("expected compound");
    }
}

// ==================== Boundary Value Tests ====================

fn create_byte_list_be(bytes: &[i8]) -> Vec<u8> {
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x01); // byte tag
    data.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    for b in bytes {
        data.push(*b as u8);
    }
    data
}

#[test]
fn test_byte_boundary_values() {
    let bytes = [i8::MIN, -1, 0, 1, i8::MAX];
    let data = create_byte_list_be(&bytes);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(list) = owned {
        assert_eq!(list.len(), 5);
        assert_eq!(list.get(0).unwrap().as_byte(), Some(i8::MIN));
        assert_eq!(list.get(1).unwrap().as_byte(), Some(-1));
        assert_eq!(list.get(2).unwrap().as_byte(), Some(0));
        assert_eq!(list.get(3).unwrap().as_byte(), Some(1));
        assert_eq!(list.get(4).unwrap().as_byte(), Some(i8::MAX));
    } else {
        panic!("expected list");
    }
}

fn create_int_list_be(ints: &[i32]) -> Vec<u8> {
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x03); // int tag
    data.extend_from_slice(&(ints.len() as u32).to_be_bytes());
    for i in ints {
        data.extend_from_slice(&i.to_be_bytes());
    }
    data
}

#[test]
fn test_int_boundary_values() {
    let ints = [i32::MIN, -1, 0, 1, i32::MAX];
    let data = create_int_list_be(&ints);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(list) = owned {
        assert_eq!(list.len(), 5);
        assert_eq!(list.get(0).unwrap().as_int(), Some(i32::MIN));
        assert_eq!(list.get(1).unwrap().as_int(), Some(-1));
        assert_eq!(list.get(2).unwrap().as_int(), Some(0));
        assert_eq!(list.get(3).unwrap().as_int(), Some(1));
        assert_eq!(list.get(4).unwrap().as_int(), Some(i32::MAX));
    } else {
        panic!("expected list");
    }
}

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
fn test_long_boundary_values() {
    let longs = [i64::MIN, -1, 0, 1, i64::MAX];
    let data = create_long_list_be(&longs);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(list) = owned {
        assert_eq!(list.len(), 5);
        assert_eq!(list.get(0).unwrap().as_long(), Some(i64::MIN));
        assert_eq!(list.get(1).unwrap().as_long(), Some(-1));
        assert_eq!(list.get(2).unwrap().as_long(), Some(0));
        assert_eq!(list.get(3).unwrap().as_long(), Some(1));
        assert_eq!(list.get(4).unwrap().as_long(), Some(i64::MAX));
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_float_special_values() {
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x05); // float tag
    data.extend_from_slice(&5u32.to_be_bytes());
    data.extend_from_slice(&0.0f32.to_be_bytes());
    data.extend_from_slice(&(-0.0f32).to_be_bytes());
    data.extend_from_slice(&f32::INFINITY.to_be_bytes());
    data.extend_from_slice(&f32::NEG_INFINITY.to_be_bytes());
    data.extend_from_slice(&f32::NAN.to_be_bytes());

    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(list) = owned {
        assert_eq!(list.len(), 5);
        assert_eq!(list.get(0).unwrap().as_float(), Some(0.0));
        assert_eq!(list.get(2).unwrap().as_float(), Some(f32::INFINITY));
        assert_eq!(list.get(3).unwrap().as_float(), Some(f32::NEG_INFINITY));
        assert!(list.get(4).unwrap().as_float().unwrap().is_nan());
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_double_special_values() {
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x06); // double tag
    data.extend_from_slice(&5u32.to_be_bytes());
    data.extend_from_slice(&0.0f64.to_be_bytes());
    data.extend_from_slice(&(-0.0f64).to_be_bytes());
    data.extend_from_slice(&f64::INFINITY.to_be_bytes());
    data.extend_from_slice(&f64::NEG_INFINITY.to_be_bytes());
    data.extend_from_slice(&f64::NAN.to_be_bytes());

    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(list) = owned {
        assert_eq!(list.len(), 5);
        assert_eq!(list.get(0).unwrap().as_double(), Some(0.0));
        assert_eq!(list.get(2).unwrap().as_double(), Some(f64::INFINITY));
        assert_eq!(list.get(3).unwrap().as_double(), Some(f64::NEG_INFINITY));
        assert!(list.get(4).unwrap().as_double().unwrap().is_nan());
    } else {
        panic!("expected list");
    }
}

// ==================== List Insert at Boundaries ====================

#[test]
fn test_list_insert_at_start() {
    let data = create_int_list_be(&[1, 2, 3]);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(mut list) = owned {
        list.insert(0, 0i32);
        assert_eq!(list.len(), 4);
        assert_eq!(list.get(0).unwrap().as_int(), Some(0));
        assert_eq!(list.get(1).unwrap().as_int(), Some(1));
        assert_eq!(list.get(2).unwrap().as_int(), Some(2));
        assert_eq!(list.get(3).unwrap().as_int(), Some(3));
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_list_insert_at_end() {
    let data = create_int_list_be(&[1, 2, 3]);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(mut list) = owned {
        list.insert(3, 4i32); // Insert at len() position
        assert_eq!(list.len(), 4);
        assert_eq!(list.get(0).unwrap().as_int(), Some(1));
        assert_eq!(list.get(1).unwrap().as_int(), Some(2));
        assert_eq!(list.get(2).unwrap().as_int(), Some(3));
        assert_eq!(list.get(3).unwrap().as_int(), Some(4));
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_list_insert_in_middle() {
    let data = create_int_list_be(&[1, 3]);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(mut list) = owned {
        list.insert(1, 2i32);
        assert_eq!(list.len(), 3);
        assert_eq!(list.get(0).unwrap().as_int(), Some(1));
        assert_eq!(list.get(1).unwrap().as_int(), Some(2));
        assert_eq!(list.get(2).unwrap().as_int(), Some(3));
    } else {
        panic!("expected list");
    }
}

// ==================== List Remove at Boundaries ====================

#[test]
fn test_list_remove_first() {
    let data = create_int_list_be(&[1, 2, 3]);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(mut list) = owned {
        let removed = list.remove(0);
        assert_eq!(removed.as_int(), Some(1));
        assert_eq!(list.len(), 2);
        assert_eq!(list.get(0).unwrap().as_int(), Some(2));
        assert_eq!(list.get(1).unwrap().as_int(), Some(3));
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_list_remove_last() {
    let data = create_int_list_be(&[1, 2, 3]);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(mut list) = owned {
        let removed = list.remove(2);
        assert_eq!(removed.as_int(), Some(3));
        assert_eq!(list.len(), 2);
        assert_eq!(list.get(0).unwrap().as_int(), Some(1));
        assert_eq!(list.get(1).unwrap().as_int(), Some(2));
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_list_remove_until_empty() {
    let data = create_int_list_be(&[1, 2]);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(mut list) = owned {
        list.remove(0);
        assert_eq!(list.len(), 1);
        list.remove(0);
        assert_eq!(list.len(), 0);
        assert!(list.is_empty());
    } else {
        panic!("expected list");
    }
}

// ==================== Write Edge Cases ====================

#[test]
fn test_write_empty_list() {
    let data = create_empty_list_be();
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let output = root.write_to_vec::<BE>().unwrap();
    // Roundtrip should produce identical data
    let owned2 = read_owned::<BE, BE>(&output).unwrap();
    if let OwnedValue::List(list) = owned2 {
        assert!(list.is_empty());
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_write_empty_compound() {
    let data = create_empty_compound_be();
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let output = root.write_to_vec::<BE>().unwrap();
    let owned2 = read_owned::<BE, BE>(&output).unwrap();
    if let OwnedValue::Compound(comp) = owned2 {
        assert!(comp.get("any").is_none());
    } else {
        panic!("expected compound");
    }
}

// ==================== Endianness Conversion Edge Cases ====================

#[test]
fn test_write_be_to_le_boundary_values() {
    let ints = [i32::MIN, -1, 0, 1, i32::MAX];
    let data = create_int_list_be(&ints);
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();

    // Write as LE
    let le_output = root.write_to_vec::<LE>().unwrap();

    // Read back as LE
    let owned_le = read_owned::<LE, LE>(&le_output).unwrap();
    if let OwnedValue::List(list) = owned_le {
        assert_eq!(list.get(0).unwrap().as_int(), Some(i32::MIN));
        assert_eq!(list.get(1).unwrap().as_int(), Some(-1));
        assert_eq!(list.get(2).unwrap().as_int(), Some(0));
        assert_eq!(list.get(3).unwrap().as_int(), Some(1));
        assert_eq!(list.get(4).unwrap().as_int(), Some(i32::MAX));
    } else {
        panic!("expected list");
    }
}

// ==================== Nested Structure Edge Cases ====================

fn create_nested_empty_lists_be() -> Vec<u8> {
    // List of lists, where inner lists are empty
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x09); // List tag
    data.extend_from_slice(&2u32.to_be_bytes()); // 2 inner lists

    // First empty list
    data.push(0x00); // End tag
    data.extend_from_slice(&0u32.to_be_bytes());

    // Second empty list
    data.push(0x00);
    data.extend_from_slice(&0u32.to_be_bytes());

    data
}

#[test]
fn test_nested_empty_lists() {
    let data = create_nested_empty_lists_be();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(outer) = owned {
        assert_eq!(outer.len(), 2);
        let inner1_val = outer.get(0).unwrap();
        let inner2_val = outer.get(1).unwrap();
        let inner1 = inner1_val.as_list().unwrap();
        let inner2 = inner2_val.as_list().unwrap();
        assert!(inner1.is_empty());
        assert!(inner2.is_empty());
    } else {
        panic!("expected list");
    }
}

fn create_deeply_nested_compound_be() -> Vec<u8> {
    // Compound { "a": Compound { "b": Compound { "c": Int(42) } } }
    let mut data = vec![0x0A, 0x00, 0x00]; // root compound

    // Key "a" -> compound
    data.push(0x0A);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'a');

    // Key "b" -> compound
    data.push(0x0A);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'b');

    // Key "c" -> int 42
    data.push(0x03);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'c');
    data.extend_from_slice(&42i32.to_be_bytes());

    // Close innermost compound
    data.push(0x00);
    // Close middle compound
    data.push(0x00);
    // Close outer compound
    data.push(0x00);

    data
}

#[test]
fn test_deeply_nested_compound_get() {
    let data = create_deeply_nested_compound_be();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::Compound(root) = owned {
        let a_val = root.get("a").unwrap();
        let a = a_val.as_compound().unwrap();
        let b_val = a.get("b").unwrap();
        let b = b_val.as_compound().unwrap();
        let c = b.get("c").unwrap();
        assert_eq!(c.as_int(), Some(42));
    } else {
        panic!("expected compound");
    }
}

#[test]
fn test_deeply_nested_compound_write_roundtrip() {
    let data = create_deeply_nested_compound_be();
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let output = root.write_to_vec::<BE>().unwrap();
    let owned2 = read_owned::<BE, BE>(&output).unwrap();

    if let OwnedValue::Compound(root2) = owned2 {
        let a_val = root2.get("a").unwrap();
        let a = a_val.as_compound().unwrap();
        let b_val = a.get("b").unwrap();
        let b = b_val.as_compound().unwrap();
        let c = b.get("c").unwrap();
        assert_eq!(c.as_int(), Some(42));
    } else {
        panic!("expected compound");
    }
}

// ==================== String Edge Cases ====================

fn create_string_nbt_be(s: &str) -> Vec<u8> {
    let bytes = s.as_bytes();
    let mut data = vec![0x08, 0x00, 0x00];
    data.extend_from_slice(&(bytes.len() as u16).to_be_bytes());
    data.extend_from_slice(bytes);
    data
}

#[test]
fn test_empty_string() {
    let data = create_string_nbt_be("");
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::String(s) = owned {
        assert_eq!(s.decode(), "");
        assert!(s.is_empty());
    } else {
        panic!("expected string");
    }
}

#[test]
fn test_string_with_special_chars() {
    let data = create_string_nbt_be("hello\nworld\t!");
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::String(s) = owned {
        assert_eq!(s.decode(), "hello\nworld\t!");
    } else {
        panic!("expected string");
    }
}

// ==================== Array Edge Cases ====================

fn create_empty_byte_array_be() -> Vec<u8> {
    let mut data = vec![0x07, 0x00, 0x00];
    data.extend_from_slice(&0u32.to_be_bytes());
    data
}

fn create_empty_int_array_be() -> Vec<u8> {
    let mut data = vec![0x0B, 0x00, 0x00];
    data.extend_from_slice(&0u32.to_be_bytes());
    data
}

fn create_empty_long_array_be() -> Vec<u8> {
    let mut data = vec![0x0C, 0x00, 0x00];
    data.extend_from_slice(&0u32.to_be_bytes());
    data
}

#[test]
fn test_empty_byte_array() {
    let data = create_empty_byte_array_be();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::ByteArray(arr) = owned {
        assert!(arr.is_empty());
        assert_eq!(arr.len(), 0);
    } else {
        panic!("expected byte array");
    }
}

#[test]
fn test_empty_int_array() {
    let data = create_empty_int_array_be();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::IntArray(arr) = owned {
        assert!(arr.is_empty());
        assert_eq!(arr.len(), 0);
    } else {
        panic!("expected int array");
    }
}

#[test]
fn test_empty_long_array() {
    let data = create_empty_long_array_be();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::LongArray(arr) = owned {
        assert!(arr.is_empty());
        assert_eq!(arr.len(), 0);
    } else {
        panic!("expected long array");
    }
}

// ==================== Type Mismatch on Get ====================

#[test]
fn test_get_wrong_type_returns_none() {
    let data = create_int_list_be(&[1, 2, 3]);
    let owned = read_owned::<BE, BE>(&data).unwrap();

    // It's a list, not a compound - get with string key should return None
    assert!(owned.get("key").is_none());

    // Get with index should work
    assert!(owned.get(0usize).is_some());
}

#[test]
fn test_as_wrong_type_returns_none() {
    let data = create_int_list_be(&[42]);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(list) = owned {
        let val = list.get(0).unwrap();
        // It's an Int, not these other types
        assert!(val.as_byte().is_none());
        assert!(val.as_short().is_none());
        assert!(val.as_long().is_none());
        assert!(val.as_float().is_none());
        assert!(val.as_double().is_none());
        assert!(val.as_string().is_none());
        assert!(val.as_list().is_none());
        assert!(val.as_compound().is_none());
        assert!(val.as_byte_array().is_none());
        assert!(val.as_int_array().is_none());
        assert!(val.as_long_array().is_none());

        // But it is an int
        assert_eq!(val.as_int(), Some(42));
    } else {
        panic!("expected list");
    }
}
