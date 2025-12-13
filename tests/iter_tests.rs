use na_nbt::{read_borrowed, read_owned, OwnedValue};
use zerocopy::byteorder::BigEndian as BE;

fn create_int_list_be() -> Vec<u8> {
    // Root list of ints [1, 2, 3]
    let mut data = vec![0x09, 0x00, 0x00]; // list tag, empty name
    data.push(0x03); // element type = Int
    data.extend_from_slice(&3u32.to_be_bytes()); // length 3
    data.extend_from_slice(&1i32.to_be_bytes());
    data.extend_from_slice(&2i32.to_be_bytes());
    data.extend_from_slice(&3i32.to_be_bytes());
    data
}

fn create_string_list_be() -> Vec<u8> {
    // Root list of strings ["a", "bb"]
    let mut data = vec![0x09, 0x00, 0x00]; // list tag, empty name
    data.push(0x08); // element type = String
    data.extend_from_slice(&2u32.to_be_bytes()); // length 2
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'a');
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"bb");
    data
}

fn create_compound_multi_be() -> Vec<u8> {
    // Root compound { a: 1, b: 2, c: 3 }
    let mut data = vec![0x0A, 0x00, 0x00]; // compound tag, empty name
    // Int a = 1
    data.push(0x03);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'a');
    data.extend_from_slice(&1i32.to_be_bytes());
    // Int b = 2
    data.push(0x03);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'b');
    data.extend_from_slice(&2i32.to_be_bytes());
    // Int c = 3
    data.push(0x03);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'c');
    data.extend_from_slice(&3i32.to_be_bytes());
    data.push(0x00); // end
    data
}

#[test]
fn immutable_list_iter_size_hint() {
    let data = create_int_list_be();
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let list = root.as_list().unwrap();
    let iter = list.iter();
    assert_eq!(iter.size_hint(), (3, Some(3)));
    assert_eq!(iter.len(), 3);
}

#[test]
fn immutable_list_iter_clone() {
    let data = create_int_list_be();
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let list = root.as_list().unwrap();
    let iter1 = list.iter();
    let iter2 = iter1.clone();
    assert_eq!(iter1.len(), iter2.len());
}

#[test]
fn immutable_compound_iter_all() {
    let data = create_compound_multi_be();
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let comp = root.as_compound().unwrap();
    let items: Vec<_> = comp.iter().map(|(k, v)| (k.decode().to_string(), v.as_int().unwrap())).collect();
    assert_eq!(items.len(), 3);
    assert!(items.iter().any(|(k, v)| k == "a" && *v == 1));
    assert!(items.iter().any(|(k, v)| k == "b" && *v == 2));
    assert!(items.iter().any(|(k, v)| k == "c" && *v == 3));
}

#[test]
fn mutable_list_iter_size_hint_and_values() {
    let data = create_int_list_be();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(mut list) = owned {
        let iter = list.iter_mut();
        assert_eq!(iter.size_hint(), (3, Some(3)));
        assert_eq!(iter.len(), 3);
        let vals: Vec<_> = list.iter_mut().map(|v| v.as_int().unwrap()).collect();
        assert_eq!(vals, vec![1, 2, 3]);
    } else {
        panic!("expected list");
    }
}

#[test]
fn mutable_compound_iter_all() {
    let data = create_compound_multi_be();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::Compound(mut comp) = owned {
        let items: Vec<_> = comp.iter_mut().map(|(k, v)| (k.decode().to_string(), v.as_int().unwrap())).collect();
        assert_eq!(items.len(), 3);
        assert!(items.iter().any(|(k, v)| k == "a" && *v == 1));
        assert!(items.iter().any(|(k, v)| k == "b" && *v == 2));
        assert!(items.iter().any(|(k, v)| k == "c" && *v == 3));
    } else {
        panic!("expected compound");
    }
}

#[test]
fn owned_list_into_iter() {
    let data = create_int_list_be();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(list) = owned {
        let vals: Vec<_> = list.into_iter().map(|v| {
            if let OwnedValue::Int(i) = v { i.get() } else { panic!("not int") }
        }).collect();
        assert_eq!(vals, vec![1, 2, 3]);
    } else {
        panic!("expected list");
    }
}

#[test]
fn owned_list_into_iter_partial_drop() {
    // Test that drop is called correctly when iterator is not fully consumed
    let data = create_string_list_be();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(list) = owned {
        let mut iter = list.into_iter();
        // Consume first element only
        let first = iter.next().unwrap();
        assert!(first.as_string().is_some());
        // Drop iter with remaining elements - should not leak
        drop(iter);
    } else {
        panic!("expected list");
    }
}

#[test]
fn owned_compound_into_iter() {
    let data = create_compound_multi_be();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::Compound(comp) = owned {
        let items: Vec<_> = comp.into_iter().map(|(k, v)| {
            let val = if let OwnedValue::Int(i) = v { i.get() } else { panic!("not int") };
            (k, val)
        }).collect();
        assert_eq!(items.len(), 3);
    } else {
        panic!("expected compound");
    }
}

#[test]
fn owned_compound_into_iter_partial_drop() {
    // Test drop when iterator not fully consumed
    let data = create_compound_multi_be();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::Compound(comp) = owned {
        let mut iter = comp.into_iter();
        let _first = iter.next().unwrap();
        // Drop with remaining elements
        drop(iter);
    } else {
        panic!("expected compound");
    }
}

fn create_nested_list_be() -> Vec<u8> {
    // Root list of lists [[1], [2,3]]
    let mut data = vec![0x09, 0x00, 0x00]; // list tag, empty name
    data.push(0x09); // element type = List
    data.extend_from_slice(&2u32.to_be_bytes()); // length 2
    // First inner list [1]
    data.push(0x03); // inner element type = Int
    data.extend_from_slice(&1u32.to_be_bytes()); // length 1
    data.extend_from_slice(&1i32.to_be_bytes());
    // Second inner list [2,3]
    data.push(0x03);
    data.extend_from_slice(&2u32.to_be_bytes());
    data.extend_from_slice(&2i32.to_be_bytes());
    data.extend_from_slice(&3i32.to_be_bytes());
    data
}

#[test]
fn owned_nested_list_into_iter_drop() {
    // Nested lists - partial consume and drop
    let data = create_nested_list_be();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::List(list) = owned {
        let mut iter = list.into_iter();
        let first = iter.next().unwrap();
        // first is itself a list
        if let OwnedValue::List(inner) = first {
            assert_eq!(inner.len(), 1);
        } else {
            panic!("expected inner list");
        }
        // Drop iter with second list remaining
        drop(iter);
    } else {
        panic!("expected list");
    }
}
