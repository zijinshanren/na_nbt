use na_nbt::{read_borrowed, read_owned};
use zerocopy::byteorder::BigEndian as BE;

fn create_compound_with_list() -> Vec<u8> {
    // Root compound with list "li" [1,2,3]
    let mut data = vec![0x0A, 0x00, 0x00];
    // list li
    data.push(0x09);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"li");
    data.push(0x03); // int
    data.extend_from_slice(&3u32.to_be_bytes());
    data.extend_from_slice(&1i32.to_be_bytes());
    data.extend_from_slice(&2i32.to_be_bytes());
    data.extend_from_slice(&3i32.to_be_bytes());
    // int value "num" = 42
    data.push(0x03);
    data.extend_from_slice(&3u16.to_be_bytes());
    data.extend_from_slice(b"num");
    data.extend_from_slice(&42i32.to_be_bytes());
    data.push(0x00);
    data
}

#[test]
fn index_usize_on_owned_value() {
    // Create root list [1, 2, 3]
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x03); // int
    data.extend_from_slice(&3u32.to_be_bytes());
    data.extend_from_slice(&1i32.to_be_bytes());
    data.extend_from_slice(&2i32.to_be_bytes());
    data.extend_from_slice(&3i32.to_be_bytes());

    let owned = read_owned::<BE, BE>(&data).unwrap();
    // Use the Index trait via OwnedValue::get()
    assert_eq!(owned.get(0_usize).unwrap().as_int(), Some(1));
    assert_eq!(owned.get(1_usize).unwrap().as_int(), Some(2));
    assert_eq!(owned.get(2_usize).unwrap().as_int(), Some(3));
    assert!(owned.get(3_usize).is_none());
}

#[test]
fn index_str_on_owned_value() {
    let data = create_compound_with_list();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    // Use the Index trait via OwnedValue::get() with &str
    assert!(owned.get("li").is_some());
    assert!(owned.get("num").is_some());
    assert_eq!(owned.get("num").unwrap().as_int(), Some(42));
    assert!(owned.get("nonexistent").is_none());
}

#[test]
fn index_string_on_owned_value() {
    let data = create_compound_with_list();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    let key = String::from("li");
    // Use the Index trait via OwnedValue::get() with String
    assert!(owned.get(&key).is_some());
    let missing = String::from("missing");
    assert!(owned.get(&missing).is_none());
}

#[test]
fn index_ref_str_on_owned_value() {
    let data = create_compound_with_list();
    let owned = read_owned::<BE, BE>(&data).unwrap();
    let key: &str = "li";
    let key_ref: &&str = &key;
    // Use &&str via Index impl for &T
    assert!(owned.get(*key_ref).is_some());
}

#[test]
fn index_usize_on_immutable_value() {
    // Create root list [1, 2, 3]
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x03); // int
    data.extend_from_slice(&3u32.to_be_bytes());
    data.extend_from_slice(&1i32.to_be_bytes());
    data.extend_from_slice(&2i32.to_be_bytes());
    data.extend_from_slice(&3i32.to_be_bytes());

    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    // Use the Index trait via ImmutableValue::get()
    assert_eq!(root.get(0_usize).unwrap().as_int(), Some(1));
    assert_eq!(root.get(1_usize).unwrap().as_int(), Some(2));
    assert_eq!(root.get(2_usize).unwrap().as_int(), Some(3));
    assert!(root.get(3_usize).is_none());
}

#[test]
fn index_str_on_immutable_value() {
    let data = create_compound_with_list();
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    // Use the Index trait via ImmutableValue::get() with &str
    assert!(root.get("li").is_some());
    assert!(root.get("num").is_some());
    assert_eq!(root.get("num").unwrap().as_int(), Some(42));
    assert!(root.get("nonexistent").is_none());
}
