//! Tests for mutable::write functions

use na_nbt::{read_owned, OwnedValue, ImmutableValue};
use zerocopy::byteorder::{BigEndian as BE, LittleEndian as LE};

// ==================== Helper Functions ====================

fn create_compound_int_be(name: &str, value: i32) -> Vec<u8> {
    let name_len = name.len() as u16;
    let mut data = vec![0x0A, 0x00, 0x00]; // Compound, empty root name
    data.push(0x03); // Tag::Int
    data.extend_from_slice(&name_len.to_be_bytes());
    data.extend_from_slice(name.as_bytes());
    data.extend_from_slice(&value.to_be_bytes());
    data.push(0x00); // End compound
    data
}

fn create_compound_string_be(name: &str, value: &str) -> Vec<u8> {
    let name_len = name.len() as u16;
    let value_len = value.len() as u16;
    let mut data = vec![0x0A, 0x00, 0x00]; // Compound, empty root name
    data.push(0x08); // Tag::String
    data.extend_from_slice(&name_len.to_be_bytes());
    data.extend_from_slice(name.as_bytes());
    data.extend_from_slice(&value_len.to_be_bytes());
    data.extend_from_slice(value.as_bytes());
    data.push(0x00); // End compound
    data
}

fn create_compound_byte_array_be(name: &str, values: &[i8]) -> Vec<u8> {
    let name_len = name.len() as u16;
    let mut data = vec![0x0A, 0x00, 0x00]; // Compound, empty root name
    data.push(0x07); // Tag::ByteArray
    data.extend_from_slice(&name_len.to_be_bytes());
    data.extend_from_slice(name.as_bytes());
    data.extend_from_slice(&(values.len() as u32).to_be_bytes());
    for &v in values {
        data.push(v as u8);
    }
    data.push(0x00); // End compound
    data
}

fn create_compound_int_array_be(name: &str, values: &[i32]) -> Vec<u8> {
    let name_len = name.len() as u16;
    let mut data = vec![0x0A, 0x00, 0x00]; // Compound, empty root name
    data.push(0x0B); // Tag::IntArray
    data.extend_from_slice(&name_len.to_be_bytes());
    data.extend_from_slice(name.as_bytes());
    data.extend_from_slice(&(values.len() as u32).to_be_bytes());
    for &v in values {
        data.extend_from_slice(&v.to_be_bytes());
    }
    data.push(0x00); // End compound
    data
}

fn create_compound_long_array_be(name: &str, values: &[i64]) -> Vec<u8> {
    let name_len = name.len() as u16;
    let mut data = vec![0x0A, 0x00, 0x00]; // Compound, empty root name
    data.push(0x0C); // Tag::LongArray
    data.extend_from_slice(&name_len.to_be_bytes());
    data.extend_from_slice(name.as_bytes());
    data.extend_from_slice(&(values.len() as u32).to_be_bytes());
    for &v in values {
        data.extend_from_slice(&v.to_be_bytes());
    }
    data.push(0x00); // End compound
    data
}

fn create_compound_with_list_be(name: &str, ints: &[i32]) -> Vec<u8> {
    let name_len = name.len() as u16;
    let mut data = vec![0x0A, 0x00, 0x00]; // Compound, empty root name
    data.push(0x09); // Tag::List
    data.extend_from_slice(&name_len.to_be_bytes());
    data.extend_from_slice(name.as_bytes());
    data.push(0x03); // Element type = Int
    data.extend_from_slice(&(ints.len() as u32).to_be_bytes());
    for &v in ints {
        data.extend_from_slice(&v.to_be_bytes());
    }
    data.push(0x00); // End compound
    data
}

fn create_nested_compound_be() -> Vec<u8> {
    let mut data = vec![0x0A, 0x00, 0x00]; // Outer compound
    
    // Inner compound entry with name "inner"
    data.push(0x0A); // Tag::Compound
    data.extend_from_slice(&5u16.to_be_bytes()); // name length = 5
    data.extend_from_slice(b"inner");
    
    // Inner compound content: { "val": 42 }
    data.push(0x03); // Tag::Int
    data.extend_from_slice(&3u16.to_be_bytes());
    data.extend_from_slice(b"val");
    data.extend_from_slice(&42i32.to_be_bytes());
    data.push(0x00); // End inner compound
    
    data.push(0x00); // End outer compound
    data
}

fn create_list_of_compounds_be() -> Vec<u8> {
    let mut data = vec![0x09, 0x00, 0x00]; // List, empty root name
    data.push(0x0A); // Element type = Compound
    data.extend_from_slice(&2u32.to_be_bytes()); // 2 elements
    
    // First compound { "a": 1 }
    data.push(0x03);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'a');
    data.extend_from_slice(&1i32.to_be_bytes());
    data.push(0x00);
    
    // Second compound { "b": 2 }
    data.push(0x03);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'b');
    data.extend_from_slice(&2i32.to_be_bytes());
    data.push(0x00);
    
    data
}

fn create_list_of_lists_be() -> Vec<u8> {
    let mut data = vec![0x09, 0x00, 0x00]; // List, empty root name
    data.push(0x09); // Element type = List
    data.extend_from_slice(&2u32.to_be_bytes()); // 2 lists
    
    // First inner list: [1, 2] (ints)
    data.push(0x03); // Int
    data.extend_from_slice(&2u32.to_be_bytes());
    data.extend_from_slice(&1i32.to_be_bytes());
    data.extend_from_slice(&2i32.to_be_bytes());
    
    // Second inner list: [3, 4, 5] (ints)
    data.push(0x03); // Int
    data.extend_from_slice(&3u32.to_be_bytes());
    data.extend_from_slice(&3i32.to_be_bytes());
    data.extend_from_slice(&4i32.to_be_bytes());
    data.extend_from_slice(&5i32.to_be_bytes());
    
    data
}

fn create_list_of_strings_be(strings: &[&str]) -> Vec<u8> {
    let mut data = vec![0x09, 0x00, 0x00]; // List, empty root name
    data.push(0x08); // Element type = String
    data.extend_from_slice(&(strings.len() as u32).to_be_bytes());
    for s in strings {
        data.extend_from_slice(&(s.len() as u16).to_be_bytes());
        data.extend_from_slice(s.as_bytes());
    }
    data
}

fn create_list_of_byte_arrays_be(arrays: &[&[i8]]) -> Vec<u8> {
    let mut data = vec![0x09, 0x00, 0x00]; // List, empty root name
    data.push(0x07); // Element type = ByteArray
    data.extend_from_slice(&(arrays.len() as u32).to_be_bytes());
    for arr in arrays {
        data.extend_from_slice(&(arr.len() as u32).to_be_bytes());
        for &b in *arr {
            data.push(b as u8);
        }
    }
    data
}

fn create_list_of_int_arrays_be(arrays: &[&[i32]]) -> Vec<u8> {
    let mut data = vec![0x09, 0x00, 0x00]; // List, empty root name
    data.push(0x0B); // Element type = IntArray
    data.extend_from_slice(&(arrays.len() as u32).to_be_bytes());
    for arr in arrays {
        data.extend_from_slice(&(arr.len() as u32).to_be_bytes());
        for &i in *arr {
            data.extend_from_slice(&i.to_be_bytes());
        }
    }
    data
}

fn create_list_of_long_arrays_be(arrays: &[&[i64]]) -> Vec<u8> {
    let mut data = vec![0x09, 0x00, 0x00]; // List, empty root name
    data.push(0x0C); // Element type = LongArray
    data.extend_from_slice(&(arrays.len() as u32).to_be_bytes());
    for arr in arrays {
        data.extend_from_slice(&(arr.len() as u32).to_be_bytes());
        for &l in *arr {
            data.extend_from_slice(&l.to_be_bytes());
        }
    }
    data
}

// ==================== Compound Write Tests (Same Endian) ====================

#[test]
fn test_write_compound_same_endian() {
    let data = create_compound_int_be("test", 12345);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    
    let output = owned.write_to_vec::<BE>().unwrap();
    let re_read = read_owned::<BE, BE>(&output).unwrap();
    
    if let OwnedValue::Compound(c) = re_read {
        let val = c.get("test").unwrap();
        if let ImmutableValue::Int(i) = val {
            assert_eq!(i, 12345);
        } else {
            panic!("expected int");
        }
    } else {
        panic!("expected compound");
    }
}

#[test]
fn test_write_compound_with_string_same_endian() {
    let data = create_compound_string_be("msg", "hello world");
    let owned = read_owned::<BE, BE>(&data).unwrap();
    
    let output = owned.write_to_vec::<BE>().unwrap();
    let re_read = read_owned::<BE, BE>(&output).unwrap();
    
    if let OwnedValue::Compound(c) = re_read {
        let val = c.get("msg").unwrap();
        if let ImmutableValue::String(s) = val {
            assert_eq!(s.decode(), "hello world");
        } else {
            panic!("expected string");
        }
    } else {
        panic!("expected compound");
    }
}

#[test]
fn test_write_compound_with_byte_array_same_endian() {
    let data = create_compound_byte_array_be("arr", &[1, 2, 3, 4, 5]);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    
    let output = owned.write_to_vec::<BE>().unwrap();
    let re_read = read_owned::<BE, BE>(&output).unwrap();
    
    if let OwnedValue::Compound(c) = re_read {
        let val = c.get("arr").unwrap();
        if let ImmutableValue::ByteArray(arr) = val {
            assert_eq!(arr.len(), 5);
        } else {
            panic!("expected byte array");
        }
    } else {
        panic!("expected compound");
    }
}

#[test]
fn test_write_compound_with_int_array_same_endian() {
    let data = create_compound_int_array_be("ints", &[10, 20, 30]);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    
    let output = owned.write_to_vec::<BE>().unwrap();
    let re_read = read_owned::<BE, BE>(&output).unwrap();
    
    if let OwnedValue::Compound(c) = re_read {
        let val = c.get("ints").unwrap();
        if let ImmutableValue::IntArray(arr) = val {
            assert_eq!(arr.len(), 3);
        } else {
            panic!("expected int array");
        }
    } else {
        panic!("expected compound");
    }
}

#[test]
fn test_write_compound_with_long_array_same_endian() {
    let data = create_compound_long_array_be("longs", &[100, 200, 300]);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    
    let output = owned.write_to_vec::<BE>().unwrap();
    let re_read = read_owned::<BE, BE>(&output).unwrap();
    
    if let OwnedValue::Compound(c) = re_read {
        let val = c.get("longs").unwrap();
        if let ImmutableValue::LongArray(arr) = val {
            assert_eq!(arr.len(), 3);
        } else {
            panic!("expected long array");
        }
    } else {
        panic!("expected compound");
    }
}

#[test]
fn test_write_compound_with_list_same_endian() {
    let data = create_compound_with_list_be("nums", &[1, 2, 3]);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    
    let output = owned.write_to_vec::<BE>().unwrap();
    let re_read = read_owned::<BE, BE>(&output).unwrap();
    
    if let OwnedValue::Compound(c) = re_read {
        let val = c.get("nums").unwrap();
        if let ImmutableValue::List(list) = val {
            assert_eq!(list.len(), 3);
        } else {
            panic!("expected list");
        }
    } else {
        panic!("expected compound");
    }
}

#[test]
fn test_write_nested_compound_same_endian() {
    let data = create_nested_compound_be();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    
    let output = owned.write_to_vec::<BE>().unwrap();
    let re_read = read_owned::<BE, BE>(&output).unwrap();
    
    if let OwnedValue::Compound(outer) = re_read {
        let inner = outer.get("inner").unwrap();
        if let ImmutableValue::Compound(inner_c) = inner {
            let val = inner_c.get("val").unwrap();
            if let ImmutableValue::Int(i) = val {
                assert_eq!(i, 42);
            } else {
                panic!("expected int");
            }
        } else {
            panic!("expected inner compound");
        }
    } else {
        panic!("expected outer compound");
    }
}

// ==================== List Write Tests (Same Endian) ====================

#[test]
fn test_write_list_of_compounds_same_endian() {
    let data = create_list_of_compounds_be();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    
    let output = owned.write_to_vec::<BE>().unwrap();
    let re_read = read_owned::<BE, BE>(&output).unwrap();
    
    if let OwnedValue::List(list) = re_read {
        assert_eq!(list.len(), 2);
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_write_list_of_lists_same_endian() {
    let data = create_list_of_lists_be();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    
    let output = owned.write_to_vec::<BE>().unwrap();
    let re_read = read_owned::<BE, BE>(&output).unwrap();
    
    if let OwnedValue::List(list) = re_read {
        assert_eq!(list.len(), 2);
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_write_list_of_strings_same_endian() {
    let data = create_list_of_strings_be(&["hello", "world", "test"]);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    
    let output = owned.write_to_vec::<BE>().unwrap();
    let re_read = read_owned::<BE, BE>(&output).unwrap();
    
    if let OwnedValue::List(list) = re_read {
        assert_eq!(list.len(), 3);
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_write_list_of_byte_arrays_same_endian() {
    let data = create_list_of_byte_arrays_be(&[&[1, 2, 3], &[4, 5]]);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    
    let output = owned.write_to_vec::<BE>().unwrap();
    let re_read = read_owned::<BE, BE>(&output).unwrap();
    
    if let OwnedValue::List(list) = re_read {
        assert_eq!(list.len(), 2);
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_write_list_of_int_arrays_same_endian() {
    let data = create_list_of_int_arrays_be(&[&[1, 2], &[3, 4, 5]]);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    
    let output = owned.write_to_vec::<BE>().unwrap();
    let re_read = read_owned::<BE, BE>(&output).unwrap();
    
    if let OwnedValue::List(list) = re_read {
        assert_eq!(list.len(), 2);
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_write_list_of_long_arrays_same_endian() {
    let data = create_list_of_long_arrays_be(&[&[100, 200], &[300]]);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    
    let output = owned.write_to_vec::<BE>().unwrap();
    let re_read = read_owned::<BE, BE>(&output).unwrap();
    
    if let OwnedValue::List(list) = re_read {
        assert_eq!(list.len(), 2);
    } else {
        panic!("expected list");
    }
}

// ==================== Endian Conversion Write Tests ====================

#[test]
fn test_write_compound_be_to_le() {
    let data = create_compound_int_be("num", 0x01020304);
    let owned = read_owned::<BE, BE>(&data).unwrap();
    
    let output = owned.write_to_vec::<LE>().unwrap();
    let re_read = read_owned::<LE, LE>(&output).unwrap();
    
    if let OwnedValue::Compound(c) = re_read {
        let val = c.get("num").unwrap();
        if let ImmutableValue::Int(i) = val {
            assert_eq!(i, 0x01020304);
        } else {
            panic!("expected int");
        }
    } else {
        panic!("expected compound");
    }
}

#[test]
fn test_write_nested_compound_be_to_le() {
    let data = create_nested_compound_be();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    
    let output = owned.write_to_vec::<LE>().unwrap();
    let re_read = read_owned::<LE, LE>(&output).unwrap();
    
    if let OwnedValue::Compound(outer) = re_read {
        let inner = outer.get("inner").unwrap();
        if let ImmutableValue::Compound(inner_c) = inner {
            let val = inner_c.get("val").unwrap();
            if let ImmutableValue::Int(i) = val {
                assert_eq!(i, 42);
            } else {
                panic!("expected int");
            }
        } else {
            panic!("expected inner compound");
        }
    } else {
        panic!("expected outer compound");
    }
}

#[test]
fn test_write_list_of_compounds_be_to_le() {
    let data = create_list_of_compounds_be();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    
    let output = owned.write_to_vec::<LE>().unwrap();
    let re_read = read_owned::<LE, LE>(&output).unwrap();
    
    if let OwnedValue::List(list) = re_read {
        assert_eq!(list.len(), 2);
        
        // Check first element
        let first = list.get(0).unwrap();
        if let ImmutableValue::Compound(c) = first {
            let val = c.get("a").unwrap();
            if let ImmutableValue::Int(i) = val {
                assert_eq!(i, 1);
            } else {
                panic!("expected int");
            }
        } else {
            panic!("expected compound");
        }
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_write_list_of_lists_be_to_le() {
    let data = create_list_of_lists_be();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    
    let output = owned.write_to_vec::<LE>().unwrap();
    let re_read = read_owned::<LE, LE>(&output).unwrap();
    
    if let OwnedValue::List(list) = re_read {
        assert_eq!(list.len(), 2);
    } else {
        panic!("expected list");
    }
}

// ==================== Empty Compound Tests ====================

#[test]
fn test_write_empty_compound() {
    let data = vec![0x0A, 0x00, 0x00, 0x00]; // Compound, empty root name, end tag
    let owned = read_owned::<BE, BE>(&data).unwrap();
    
    let output = owned.write_to_vec::<BE>().unwrap();
    let re_read = read_owned::<BE, BE>(&output).unwrap();
    
    if let OwnedValue::Compound(c) = re_read {
        assert!(c.into_iter().next().is_none());
    } else {
        panic!("expected compound");
    }
}

#[test]
fn test_write_empty_list() {
    let data = vec![0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]; // List of End, length 0
    let owned = read_owned::<BE, BE>(&data).unwrap();
    
    let output = owned.write_to_vec::<BE>().unwrap();
    let re_read = read_owned::<BE, BE>(&output).unwrap();
    
    if let OwnedValue::List(list) = re_read {
        assert_eq!(list.len(), 0);
    } else {
        panic!("expected list");
    }
}
