use bytes::Bytes;
use na_nbt::{
    BigEndian, CompoundRef, ListBase, ListRef, LittleEndian, ValueRef, VisitRef, Writable,
    read_borrowed, read_owned, read_shared,
};
use na_nbt::{from_slice_be, from_slice_le, to_vec_be, to_vec_le};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use zerocopy::byteorder;

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

/// Pretty-print any NBT value recursively
fn dump<'doc>(value: &impl ValueRef<'doc>) -> String {
    dump_inner(value, 0)
}

fn dump_inner<'doc>(value: &impl ValueRef<'doc>, indent: usize) -> String {
    let pad = "  ".repeat(indent);
    value.visit(|v| match v {
        VisitRef::End(_) => format!("{pad}End"),
        VisitRef::Byte(v) => format!("{pad}Byte({v})"),
        VisitRef::Short(v) => format!("{pad}Short({v})"),
        VisitRef::Int(v) => format!("{pad}Int({v})"),
        VisitRef::Long(v) => format!("{pad}Long({v})"),
        VisitRef::Float(v) => format!("{pad}Float({v})"),
        VisitRef::Double(v) => format!("{pad}Double({v})"),
        VisitRef::ByteArray(v) => {
            let elems = v
                .iter()
                .map(|b| b.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            format!("{pad}ByteArray[{}] {{ {} }}", v.len(), elems)
        }
        VisitRef::String(v) => format!("{pad}String({:?})", v.decode()),
        VisitRef::IntArray(v) => {
            let elems = v
                .iter()
                .map(|b| b.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            format!("{pad}IntArray[{}] {{ {} }}", v.len(), elems)
        }
        VisitRef::LongArray(v) => {
            let elems = v
                .iter()
                .map(|b| b.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            format!("{pad}LongArray[{}] {{ {} }}", v.len(), elems)
        }
        VisitRef::List(list) => {
            let mut out = format!("{pad}List[{}] {{\n", list.len());
            for item in list.iter() {
                out.push_str(&dump_inner(&item, indent + 1));
                out.push('\n');
            }
            out.push_str(&format!("{pad}}}"));
            out
        }
        VisitRef::Compound(compound) => {
            let mut out = format!("{pad}Compound {{\n");
            for (key, val) in compound.iter() {
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

fn test_serde(data: &[u8]) {
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

fn test_round<'doc>(value: &impl ValueRef<'doc>) {
    let a = dump(value);
    let vec_little = value.write_to_vec::<LittleEndian>();
    let vec_big = value.write_to_vec::<BigEndian>();
    {
        let mut vec = Vec::new();
        value.write_to_writer::<BigEndian>(&mut vec).unwrap();
        assert_eq!(vec, vec_big);
    }
    {
        let mut vec = Vec::new();
        value.write_to_writer::<LittleEndian>(&mut vec).unwrap();
        assert_eq!(vec, vec_little);
    }
    {
        let vec = vec_big;
        {
            let rev = read_borrowed::<BigEndian>(&vec).unwrap();
            let b = dump(&rev.root());
            assert_eq!(a, b);
        }
        {
            let rev = read_owned::<BigEndian, BigEndian>(&vec).unwrap();
            let b = dump(&rev.to_ref());
            assert_eq!(a, b);
        }
        {
            let rev = read_owned::<BigEndian, LittleEndian>(&vec).unwrap();
            let b = dump(&rev.to_ref());
            assert_eq!(a, b);
        }
        {
            let bytes = Bytes::from(vec);
            let rev = read_shared::<BigEndian>(bytes).unwrap();
            let b = dump(&rev);
            assert_eq!(a, b);
        }
    }
    {
        let vec = vec_little;
        {
            let rev = read_borrowed::<LittleEndian>(&vec).unwrap();
            let b = dump(&rev.root());
            assert_eq!(a, b);
        }
        {
            let rev = read_owned::<LittleEndian, BigEndian>(&vec).unwrap();
            let b = dump(&rev.to_ref());
            assert_eq!(a, b);
        }
        {
            let rev = read_owned::<LittleEndian, LittleEndian>(&vec).unwrap();
            let b = dump(&rev.to_ref());
            assert_eq!(a, b);
        }
        {
            let bytes = Bytes::from(vec);
            let rev = read_shared::<LittleEndian>(bytes).unwrap();
            let b = dump(&rev);
            assert_eq!(a, b);
        }
    }
}

fn test_direct(data: &[u8]) {
    if let Ok(doc) = read_borrowed::<BigEndian>(data) {
        test_round(&doc.root());
        let a = dump(&doc.root());
        let bytes = Bytes::copy_from_slice(data);
        let doc = read_shared::<BigEndian>(bytes.clone()).unwrap();
        let b = dump(&doc);
        assert_eq!(a, b);
        let doc = read_owned::<BigEndian, BigEndian>(&bytes).unwrap();
        let b = dump(&doc.to_ref());
        assert_eq!(a, b);
        let doc = read_owned::<BigEndian, LittleEndian>(&bytes).unwrap();
        let b = dump(&doc.to_ref());
        assert_eq!(a, b);
    }
    if let Ok(doc) = read_borrowed::<LittleEndian>(data) {
        test_round(&doc.root());
        let a = dump(&doc.root());
        let bytes = Bytes::copy_from_slice(data);
        let doc = read_shared::<LittleEndian>(bytes.clone()).unwrap();
        let b = dump(&doc);
        assert_eq!(a, b);
        let doc = read_owned::<LittleEndian, BigEndian>(&bytes).unwrap();
        let b = dump(&doc.to_ref());
        assert_eq!(a, b);
        let doc = read_owned::<LittleEndian, LittleEndian>(&bytes).unwrap();
        let b = dump(&doc.to_ref());
        assert_eq!(a, b);
    }
}

#[test]
fn read_write() {
    let dir = r".\fuzz\in";
    let mut count = 0;
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            count += 1;
            let data = fs::read(path).unwrap();
            test_serde(&data);
            test_direct(&data);
        }
    }
    assert!(count > 0);
}

fn test_writable<V: Writable + ?Sized>(value: &V) {
    test_direct(&value.write_to_vec::<BigEndian>());
    test_direct(&value.write_to_vec::<LittleEndian>());
}

#[test]
fn construct_read_write() {
    test_writable(&());
    test_writable(&42_i8);
    test_writable(&13413_i16);
    test_writable(&114514_i32);
    test_writable(&1145141919810_i64);
    test_writable(&std::f32::consts::PI);
    test_writable(&std::f64::consts::E);
    test_writable(&[1_i8, 2, 3, 4, 5]);
    test_writable(&[
        byteorder::I32::<BigEndian>::new(1),
        2.into(),
        3.into(),
        4.into(),
        5.into(),
    ]);
    test_writable(&[
        byteorder::I32::<LittleEndian>::new(1),
        2.into(),
        3.into(),
        4.into(),
        5.into(),
    ]);
    test_writable(&[
        byteorder::I64::<BigEndian>::new(1),
        2.into(),
        3.into(),
        4.into(),
        5.into(),
    ]);
    test_writable(&[
        byteorder::I64::<LittleEndian>::new(1),
        2.into(),
        3.into(),
        4.into(),
        5.into(),
    ]);
}
