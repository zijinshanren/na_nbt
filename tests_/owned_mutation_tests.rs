//! Tests for OwnedList and OwnedCompound mutation APIs

use na_nbt::{OwnedValue, read_owned};
use zerocopy::byteorder::BigEndian as BE;

fn create_int_list_nbt_be(values: &[i32]) -> Vec<u8> {
    let len = values.len() as u32;
    let len_bytes = len.to_be_bytes();
    let mut result = vec![
        0x09, // Tag::List
        0x00,
        0x00, // empty name
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
fn owned_list_push_pop_remove() {
    let data = create_int_list_nbt_be(&[10, 20, 30]);
    let owned = read_owned::<BE, BE>(&data).unwrap();

    if let OwnedValue::List(mut list) = owned {
        assert_eq!(list.len(), 3);
        list.push(40i32);
        assert_eq!(list.len(), 4);
        assert_eq!(list.get(3).unwrap().as_int(), Some(40));

        // Test pushing multiple types into separate lists
        list.push(50i32);
        assert_eq!(list.get(4).unwrap().as_int(), Some(50));

        let removed = list.remove(1);
        if let OwnedValue::Int(i) = removed {
            assert_eq!(i.get(), 20);
        } else {
            panic!("expected int");
        }
        assert_eq!(list.len(), 4);

        let popped = list.pop().unwrap();
        if let OwnedValue::Int(i) = popped {
            assert_eq!(i.get(), 50);
        } else {
            panic!("expected int");
        }
    } else {
        panic!("expected list");
    }
}

#[test]
fn owned_compound_insert_remove_get_mut() {
    let data = create_compound_int_entry_be("a", 100);
    let owned = read_owned::<BE, BE>(&data).unwrap();

    if let OwnedValue::Compound(mut compound) = owned {
        assert!(compound.get("a").is_some());
        compound.insert("b", 200i32);
        assert_eq!(compound.get("b").unwrap().as_int(), Some(200));

        // Insert various types
        compound.insert("by", 2i8);
        assert_eq!(compound.get("by").unwrap().as_byte(), Some(2));

        compound.insert("sh", 3i16);
        assert_eq!(compound.get("sh").unwrap().as_short(), Some(3));

        compound.insert("f", 3.5f32);
        assert!((compound.get("f").unwrap().as_float().unwrap() - 3.5).abs() < 0.0001);

        compound.insert("s", "hello");
        assert_eq!(
            compound
                .get("s")
                .unwrap()
                .as_string()
                .unwrap()
                .decode()
                .to_string(),
            "hello"
        );

        // Remove 'b'
        let removed = compound.remove("b").unwrap();
        if let OwnedValue::Int(i) = removed {
            assert_eq!(i.get(), 200);
        } else {
            panic!("expected int");
        }

        // Test get_mut on existing value
        if let Some(v) = compound.get_mut("a")
            && let na_nbt::MutableValue::Int(val) = v
        {
            *val = 999i32.into();
        }

        assert_eq!(compound.get("a").unwrap().as_int(), Some(999));
    } else {
        panic!("expected compound");
    }
}
