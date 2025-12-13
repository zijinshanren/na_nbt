//! Additional tests for OwnedList and OwnedCompound variants

use na_nbt::{read_owned, OwnedValue};
use zerocopy::byteorder::BigEndian as BE;

fn create_string_list_nbt_be(values: &[&str]) -> Vec<u8> {
    let len = values.len() as u32;
    let len_bytes = len.to_be_bytes();
    let mut result = vec![
        0x09,
        0x00,
        0x00,
        0x08, // element type = String
        len_bytes[0],
        len_bytes[1],
        len_bytes[2],
        len_bytes[3],
    ];
    for s in values {
        let str_len = s.len() as u16;
        result.extend_from_slice(&str_len.to_be_bytes());
        result.extend_from_slice(s.as_bytes());
    }
    result
}

fn create_byte_array_list_nbt_be(vectors: &[&[i8]]) -> Vec<u8> {
    let len = vectors.len() as u32;
    let len_bytes = len.to_be_bytes();
    let mut result = vec![
        0x09,
        0x00,
        0x00,
        0x07, // element type = ByteArray
        len_bytes[0],
        len_bytes[1],
        len_bytes[2],
        len_bytes[3],
    ];
    for vec in vectors {
        let l = vec.len() as u32;
        result.extend_from_slice(&l.to_be_bytes());
        for &b in *vec {
            result.push(b as u8);
        }
    }
    result
}

#[test]
fn owned_list_string_push_and_get() {
    let data = create_string_list_nbt_be(&["a", "b"]);
    let mut owned = read_owned::<BE, BE>(&data).unwrap();

    if let OwnedValue::List(mut list) = owned {
        assert_eq!(list.len(), 2);
        list.push("c");
        assert_eq!(list.len(), 3);
        assert_eq!(list.get(2).unwrap().as_string().unwrap().decode().to_string(), "c");
    } else {
        panic!("expected list");
    }
}

#[test]
fn owned_list_byte_array_push_and_get() {
    let data = create_byte_array_list_nbt_be(&[&[1i8, 2], &[3i8]]);
    let mut owned = read_owned::<BE, BE>(&data).unwrap();

    if let OwnedValue::List(mut list) = owned {
        assert_eq!(list.len(), 2);
        list.push(vec![4i8, 5]);
        assert_eq!(list.len(), 3);
        let val = list.get(2).unwrap();
        let arr = val.as_byte_array().unwrap();
        assert_eq!(arr, &[4, 5]);
    } else {
        panic!("expected list");
    }
}

#[test]
fn owned_compound_insert_replace_arr_int() {
    use zerocopy::byteorder::I32 as I32BE;
    let data = vec![0x0A, 0x00, 0x00, 0x00];
    let mut owned = read_owned::<BE, BE>(&data).unwrap();

    if let OwnedValue::Compound(mut compound) = owned {
        compound.insert("x", 42i32);
        assert_eq!(compound.get("x").unwrap().as_int(), Some(42));

        // Replace value
        let old = compound.insert("x", 43i32);
        if let Some(OwnedValue::Int(i)) = old {
            assert_eq!(i.get(), 42);
        } else {
            panic!("expected previous int");
        }

        // Insert an int array via Vec<I32BE>
        let arr: Vec<I32BE<BE>> = vec![I32BE::new(1), I32BE::new(2)];
        compound.insert("arr", arr);
        let val = compound.get("arr").unwrap();
        if let na_nbt::ImmutableValue::IntArray(a) = val {
            let v: Vec<i32> = a.iter().map(|x| x.get()).collect();
            assert_eq!(v, vec![1, 2]);
        } else {
            panic!("expected int array");
        }
    } else {
        panic!("expected compound");
    }
}
