use bytes::Bytes;
use na_nbt::{
    BigEndian, LittleEndian, ReadableString as _, ScopedReadableList as _, ScopedReadableValue,
    ValueScoped, read_borrowed, read_owned, read_shared,
};

use na_nbt::{from_slice_be, from_slice_le, to_vec_be, to_vec_le};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn dump<'doc>(value: &impl ScopedReadableValue<'doc>) -> String {
    dump_inner(value, 0)
}

fn dump_inner<'doc>(value: &impl ScopedReadableValue<'doc>, indent: usize) -> String {
    let pad = "  ".repeat(indent);
    value.visit_scoped(|v| match v {
        ValueScoped::End => format!("{pad}End"),
        ValueScoped::Byte(v) => format!("{pad}Byte({v})"),
        ValueScoped::Short(v) => format!("{pad}Short({v})"),
        ValueScoped::Int(v) => format!("{pad}Int({v})"),
        ValueScoped::Long(v) => format!("{pad}Long({v})"),
        ValueScoped::Float(v) => format!("{pad}Float({v})"),
        ValueScoped::Double(v) => format!("{pad}Double({v})"),
        ValueScoped::ByteArray(v) => format!("{pad}ByteArray({} bytes)", v.len()),
        ValueScoped::String(v) => format!("{pad}String({:?})", v.decode()),
        ValueScoped::IntArray(v) => format!("{pad}IntArray({} ints)", v.len()),
        ValueScoped::LongArray(v) => format!("{pad}LongArray({} longs)", v.len()),
        ValueScoped::List(list) => {
            let mut out = format!("{pad}List[{}] {{\n", list.len());
            for item in list {
                out.push_str(&dump_inner(&item, indent + 1));
                out.push('\n');
            }
            out.push_str(&format!("{pad}}}"));
            out
        }
        ValueScoped::Compound(compound) => {
            let mut out = format!("{pad}Compound {{\n");
            for (key, val) in compound {
                let nested = dump_inner(&val, indent + 1);
                out.push_str(&format!(
                    "{}  {:?}: {}\n",
                    pad,
                    key.decode(),
                    nested.trim_start()
                ));
            }
            out.push_str(&format!("{pad}}}"));
            out
        }
    })
}

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = include_bytes!(r"D:\cmp\na_nbt\fuzz\out\default\crashes\id000000,sig06,src000000,time5439,execs7241,oparith16,pos1,valbe-6");

    test_serde(data);
    test_direct(data);
    Ok(())
}
