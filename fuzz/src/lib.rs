use bytes::Bytes;
use na_nbt::{BigEndian, LittleEndian, read_borrowed, read_owned, read_shared};
use na_nbt::{from_slice_be, from_slice_le, to_vec_be, to_vec_le};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct TestCompound {
    byte_val: i8,
    short_val: i16,
    int_val: i32,
    long_val: i64,
    float_val: f32,
    double_val: f64,
    string_val: String,
    #[serde(default)]
    list_ints: Vec<i32>,
    #[serde(default)]
    list_strings: Vec<String>,
    #[serde(default)]
    nested: Option<Box<TestCompound>>,
    #[serde(default)]
    map_vals: HashMap<String, i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct SimpleCompound {
    #[serde(default)]
    value: i32,
    #[serde(default)]
    name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum TestEnum {
    Unit,
    Newtype(i32),
    Tuple(i32, String),
    Struct { x: i32, y: i32 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct WithEnum {
    #[serde(default)]
    mode: Option<TestEnum>,
    #[serde(default)]
    value: i32,
}

pub fn test_serde(data: &[u8]) {
    if let Ok(val) = from_slice_be::<SimpleCompound>(data) {
        let _ = to_vec_be(&val);
        let _ = to_vec_le(&val);
    }

    if let Ok(val) = from_slice_le::<SimpleCompound>(data) {
        let _ = to_vec_le(&val);
        let _ = to_vec_be(&val);
    }

    if let Ok(val) = from_slice_be::<TestCompound>(data) {
        let _ = to_vec_be(&val);
        let _ = to_vec_le(&val);
    }

    if let Ok(val) = from_slice_le::<TestCompound>(data) {
        let _ = to_vec_le(&val);
        let _ = to_vec_be(&val);
    }

    if let Ok(val) = from_slice_be::<WithEnum>(data) {
        let _ = to_vec_be(&val);
        let _ = to_vec_le(&val);
    }

    if let Ok(val) = from_slice_le::<WithEnum>(data) {
        let _ = to_vec_le(&val);
        let _ = to_vec_be(&val);
    }

    if let Ok(val) = from_slice_be::<HashMap<String, i32>>(data) {
        let _ = to_vec_be(&val);
        let _ = to_vec_le(&val);
    }

    if let Ok(val) = from_slice_le::<HashMap<String, i32>>(data) {
        let _ = to_vec_le(&val);
        let _ = to_vec_be(&val);
    }

    if let Ok(val) = from_slice_be::<Vec<i32>>(data) {
        let _ = to_vec_be(&val);
        let _ = to_vec_le(&val);
    }

    if let Ok(val) = from_slice_le::<Vec<i32>>(data) {
        let _ = to_vec_le(&val);
        let _ = to_vec_be(&val);
    }
}

pub fn test_direct(data: &[u8]) {
    if let Ok(doc) = read_borrowed::<BigEndian>(data) {
        let _ = doc.root().write_to_vec::<BigEndian>();
        let _ = doc.root().write_to_vec::<LittleEndian>();
    }
    if let Ok(doc) = read_borrowed::<LittleEndian>(data) {
        let _ = doc.root().write_to_vec::<LittleEndian>();
        let _ = doc.root().write_to_vec::<BigEndian>();
    }

    let bytes = Bytes::copy_from_slice(data);
    if let Ok(root) = read_shared::<BigEndian>(bytes.clone()) {
        let _ = root.write_to_vec::<BigEndian>();
        let _ = root.write_to_vec::<LittleEndian>();
    }
    if let Ok(root) = read_shared::<LittleEndian>(bytes.clone()) {
        let _ = root.write_to_vec::<LittleEndian>();
        let _ = root.write_to_vec::<BigEndian>();
    }

    if let Ok(root) = read_owned::<LittleEndian, LittleEndian>(data) {
        let _ = root.write_to_vec::<LittleEndian>();
        let _ = root.write_to_vec::<BigEndian>();
    }
    if let Ok(root) = read_owned::<LittleEndian, BigEndian>(data) {
        let _ = root.write_to_vec::<LittleEndian>();
        let _ = root.write_to_vec::<BigEndian>();
    }
    if let Ok(root) = read_owned::<BigEndian, LittleEndian>(data) {
        let _ = root.write_to_vec::<LittleEndian>();
        let _ = root.write_to_vec::<BigEndian>();
    }
    if let Ok(root) = read_owned::<BigEndian, BigEndian>(data) {
        let _ = root.write_to_vec::<LittleEndian>();
        let _ = root.write_to_vec::<BigEndian>();
    }
}

pub fn test(data: &[u8]) {
    test_serde(data);
    test_direct(data);
}
