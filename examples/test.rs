use bytes::Bytes;
use na_nbt::{
    BigEndian, LittleEndian, Writable, read_borrowed, read_owned, read_owned_from_reader,
    read_shared,
};
use na_nbt::{from_slice_be, from_slice_le, to_vec_be, to_vec_le};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
enum TestEnum {
    #[default]
    Unit,
    Newtype(i32),
    Tuple(i32, String),
    Struct {
        x: i32,
        y: i32,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct WithEnum {
    #[serde(default)]
    mode: TestEnum,
    #[serde(default)]
    value: i32,
}

pub fn test_serde(data: &[u8]) {
    if let Ok(val) = from_slice_be::<SimpleCompound>(data) {
        if let Ok(vec) = to_vec_be(&val) {
            let _ = from_slice_be::<SimpleCompound>(&vec).unwrap();
        }
        if let Ok(vec) = to_vec_le(&val) {
            let _ = from_slice_le::<SimpleCompound>(&vec).unwrap();
        }
    }

    if let Ok(val) = from_slice_le::<SimpleCompound>(data) {
        if let Ok(vec) = to_vec_le(&val) {
            let _ = from_slice_le::<SimpleCompound>(&vec).unwrap();
        }
        if let Ok(vec) = to_vec_be(&val) {
            let _ = from_slice_be::<SimpleCompound>(&vec).unwrap();
        }
    }

    if let Ok(val) = from_slice_be::<TestCompound>(data) {
        if let Ok(vec) = to_vec_be(&val) {
            let _ = from_slice_be::<TestCompound>(&vec).unwrap();
        }
        if let Ok(vec) = to_vec_le(&val) {
            let _ = from_slice_le::<TestCompound>(&vec).unwrap();
        }
    }

    if let Ok(val) = from_slice_le::<TestCompound>(data) {
        if let Ok(vec) = to_vec_le(&val) {
            let _ = from_slice_le::<TestCompound>(&vec).unwrap();
        }
        if let Ok(vec) = to_vec_be(&val) {
            let _ = from_slice_be::<TestCompound>(&vec).unwrap();
        }
    }

    if let Ok(val) = from_slice_be::<WithEnum>(data) {
        if let Ok(vec) = to_vec_be(&val) {
            let _ = from_slice_be::<WithEnum>(&vec).unwrap();
        }
        if let Ok(vec) = to_vec_le(&val) {
            let _ = from_slice_le::<WithEnum>(&vec).unwrap();
        }
    }

    if let Ok(val) = from_slice_le::<WithEnum>(data) {
        if let Ok(vec) = to_vec_le(&val) {
            let _ = from_slice_le::<WithEnum>(&vec).unwrap();
        }
        if let Ok(vec) = to_vec_be(&val) {
            let _ = from_slice_be::<WithEnum>(&vec).unwrap();
        }
    }

    if let Ok(val) = from_slice_be::<HashMap<String, i32>>(data) {
        if let Ok(vec) = to_vec_be(&val) {
            let _ = from_slice_be::<HashMap<String, i32>>(&vec).unwrap();
        }
        if let Ok(vec) = to_vec_le(&val) {
            let _ = from_slice_le::<HashMap<String, i32>>(&vec).unwrap();
        }
    }

    if let Ok(val) = from_slice_le::<HashMap<String, i32>>(data) {
        if let Ok(vec) = to_vec_le(&val) {
            let _ = from_slice_le::<HashMap<String, i32>>(&vec).unwrap();
        }
        if let Ok(vec) = to_vec_be(&val) {
            let _ = from_slice_be::<HashMap<String, i32>>(&vec).unwrap();
        }
    }

    if let Ok(val) = from_slice_be::<Vec<i32>>(data) {
        if let Ok(vec) = to_vec_be(&val) {
            let _ = from_slice_be::<Vec<i32>>(&vec).unwrap();
        }
        if let Ok(vec) = to_vec_le(&val) {
            let _ = from_slice_le::<Vec<i32>>(&vec).unwrap();
        }
    }

    if let Ok(val) = from_slice_le::<Vec<i32>>(data) {
        if let Ok(vec) = to_vec_le(&val) {
            let _ = from_slice_le::<Vec<i32>>(&vec).unwrap();
        }
        if let Ok(vec) = to_vec_be(&val) {
            let _ = from_slice_be::<Vec<i32>>(&vec).unwrap();
        }
    }
}

pub fn test_direct(data: &[u8]) {
    if let Ok(doc) = read_borrowed::<BigEndian>(data) {
        let vec = doc.root().write_to_vec::<BigEndian>();
        assert!(read_borrowed::<BigEndian>(&vec).is_ok());
        let vec = doc.root().write_to_vec::<LittleEndian>();
        assert!(read_borrowed::<LittleEndian>(&vec).is_ok());
    }
    if let Ok(doc) = read_borrowed::<LittleEndian>(data) {
        let vec = doc.root().write_to_vec::<LittleEndian>();
        assert!(read_borrowed::<LittleEndian>(&vec).is_ok());
        let vec = doc.root().write_to_vec::<BigEndian>();
        assert!(read_borrowed::<BigEndian>(&vec).is_ok());
    }

    let bytes = Bytes::copy_from_slice(data);
    if let Ok(root) = read_shared::<BigEndian>(bytes.clone()) {
        let vec = Bytes::from(root.write_to_vec::<BigEndian>());
        assert!(read_shared::<BigEndian>(vec).is_ok());
        let vec = Bytes::from(root.write_to_vec::<LittleEndian>());
        assert!(read_shared::<LittleEndian>(vec).is_ok());
    }
    if let Ok(root) = read_shared::<LittleEndian>(bytes.clone()) {
        let vec = Bytes::from(root.write_to_vec::<LittleEndian>());
        assert!(read_shared::<LittleEndian>(vec).is_ok());
        let vec = Bytes::from(root.write_to_vec::<BigEndian>());
        assert!(read_shared::<BigEndian>(vec).is_ok());
    }

    if let Ok(root) = read_owned::<LittleEndian, LittleEndian>(data) {
        let vec = root.write_to_vec::<LittleEndian>();
        assert!(read_owned::<LittleEndian, LittleEndian>(&vec).is_ok());
        assert!(read_owned::<LittleEndian, BigEndian>(&vec).is_ok());
        let vec = root.write_to_vec::<BigEndian>();
        assert!(read_owned::<BigEndian, LittleEndian>(&vec).is_ok());
        assert!(read_owned::<BigEndian, BigEndian>(&vec).is_ok());
    }
    if let Ok(root) = read_owned_from_reader::<LittleEndian, LittleEndian>(data) {
        let mut vec = Vec::new();
        if root.write_to_writer::<LittleEndian>(&mut vec).is_ok() {
            assert!(read_owned_from_reader::<LittleEndian, LittleEndian>(vec.as_slice()).is_ok());
            assert!(read_owned_from_reader::<LittleEndian, BigEndian>(vec.as_slice()).is_ok());
        }
        let mut vec = Vec::new();
        if root.write_to_writer::<BigEndian>(&mut vec).is_ok() {
            assert!(read_owned_from_reader::<BigEndian, LittleEndian>(vec.as_slice()).is_ok());
            assert!(read_owned_from_reader::<BigEndian, BigEndian>(vec.as_slice()).is_ok());
        }
    }
    if let Ok(root) = read_owned::<LittleEndian, BigEndian>(data) {
        let vec = root.write_to_vec::<LittleEndian>();
        assert!(read_owned::<LittleEndian, LittleEndian>(&vec).is_ok());
        assert!(read_owned::<LittleEndian, BigEndian>(&vec).is_ok());
        let vec = root.write_to_vec::<BigEndian>();
        assert!(read_owned::<BigEndian, LittleEndian>(&vec).is_ok());
        assert!(read_owned::<BigEndian, BigEndian>(&vec).is_ok());
    }
    if let Ok(root) = read_owned_from_reader::<LittleEndian, BigEndian>(data) {
        let mut vec = Vec::new();
        if root.write_to_writer::<LittleEndian>(&mut vec).is_ok() {
            assert!(read_owned_from_reader::<LittleEndian, LittleEndian>(vec.as_slice()).is_ok());
            assert!(read_owned_from_reader::<LittleEndian, BigEndian>(vec.as_slice()).is_ok());
        }
        let mut vec = Vec::new();
        if root.write_to_writer::<BigEndian>(&mut vec).is_ok() {
            assert!(read_owned_from_reader::<BigEndian, LittleEndian>(vec.as_slice()).is_ok());
            assert!(read_owned_from_reader::<BigEndian, BigEndian>(vec.as_slice()).is_ok());
        }
    }
    if let Ok(root) = read_owned::<BigEndian, LittleEndian>(data) {
        let vec = root.write_to_vec::<LittleEndian>();
        assert!(read_owned::<LittleEndian, LittleEndian>(&vec).is_ok());
        assert!(read_owned::<LittleEndian, BigEndian>(&vec).is_ok());
        let vec = root.write_to_vec::<BigEndian>();
        assert!(read_owned::<BigEndian, LittleEndian>(&vec).is_ok());
        assert!(read_owned::<BigEndian, BigEndian>(&vec).is_ok());
    }
    if let Ok(root) = read_owned_from_reader::<BigEndian, LittleEndian>(data) {
        let mut vec = Vec::new();
        if root.write_to_writer::<BigEndian>(&mut vec).is_ok() {
            assert!(read_owned_from_reader::<BigEndian, LittleEndian>(vec.as_slice()).is_ok());
            assert!(read_owned_from_reader::<BigEndian, BigEndian>(vec.as_slice()).is_ok());
        }
        let mut vec = Vec::new();
        if root.write_to_writer::<LittleEndian>(&mut vec).is_ok() {
            assert!(read_owned_from_reader::<LittleEndian, LittleEndian>(vec.as_slice()).is_ok());
            assert!(read_owned_from_reader::<LittleEndian, BigEndian>(vec.as_slice()).is_ok());
        }
    }
    if let Ok(root) = read_owned::<BigEndian, BigEndian>(data) {
        let vec = root.write_to_vec::<LittleEndian>();
        assert!(read_owned::<LittleEndian, LittleEndian>(&vec).is_ok());
        assert!(read_owned::<LittleEndian, BigEndian>(&vec).is_ok());
        let vec = root.write_to_vec::<BigEndian>();
        assert!(read_owned::<BigEndian, LittleEndian>(&vec).is_ok());
        assert!(read_owned::<BigEndian, BigEndian>(&vec).is_ok());
    }
    if let Ok(root) = read_owned_from_reader::<BigEndian, BigEndian>(data) {
        let mut vec = Vec::new();
        if root.write_to_writer::<BigEndian>(&mut vec).is_ok() {
            assert!(read_owned_from_reader::<BigEndian, LittleEndian>(vec.as_slice()).is_ok());
            assert!(read_owned_from_reader::<BigEndian, BigEndian>(vec.as_slice()).is_ok());
        }
        let mut vec = Vec::new();
        if root.write_to_writer::<LittleEndian>(&mut vec).is_ok() {
            assert!(read_owned_from_reader::<LittleEndian, LittleEndian>(vec.as_slice()).is_ok());
            assert!(read_owned_from_reader::<LittleEndian, BigEndian>(vec.as_slice()).is_ok());
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = include_bytes!(
        r"D:\cmp\na_nbt\fuzz\hfuzz_workspace\hongg\input\001aa16f5e6b102f778a7dfbf96ba661.00001000.honggfuzz.cov"
    );
    test_serde(data);
    test_direct(data);

    let dir = r"D:\cmp\na_nbt\fuzz\hfuzz_workspace\hongg\input";

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            println!("Testing: {}", path.display());
            let data = fs::read(path)?;
            test_serde(&data);
            test_direct(&data);
        }
    }

    // test_direct(data);
    // test_serde(data);
    Ok(())
}
