use std::collections::HashMap;

use na_nbt::{
    CompoundRef, ListBase, ListRef, ValueRef, VisitRef, from_slice_be, read_borrowed, to_vec_be,
};
use serde::{Deserialize, Serialize};
use zerocopy::BigEndian;

#[derive(Serialize, Deserialize, Debug)]
struct Player {
    name: String,
    health: f32,
    inventory: Vec<Item>,
    map: HashMap<String, i32>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Item {
    name: String,
    count: u8,
}

#[derive(Serialize, Deserialize, Debug)]
enum Data {
    // Unit variant: deserialized from Int (variant index)
    Empty,
    // Newtype variant: Compound { "Value": <inner> }
    Value(i32),
    // Tuple variant: Compound { "Point": List[Compound] }
    Point(i32, i32),
    // Struct variant: Compound { "Player": Compound { fields... } }
    Player {
        name: String,
        health: i32,
    },
    #[serde(with = "na_nbt::byte_array")]
    ByteArray(Vec<i8>),
    #[serde(with = "na_nbt::int_array")]
    IntArray(Vec<i32>),
    #[serde(with = "na_nbt::long_array")]
    LongArray(Vec<i64>),
}

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
        VisitRef::ByteArray(v) => format!("{pad}ByteArray({} bytes)", v.len()),
        VisitRef::String(v) => format!("{pad}String({:?})", v.decode()),
        VisitRef::IntArray(v) => format!("{pad}IntArray({} ints)", v.len()),
        VisitRef::LongArray(v) => format!("{pad}LongArray({} longs)", v.len()),
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

fn main() {
    let player = Player {
        name: "John".to_string(),
        health: 100.0,
        inventory: vec![
            Item {
                name: "Apple".to_string(),
                count: 1,
            },
            Item {
                name: "Banana".to_string(),
                count: 2,
            },
            Item {
                name: "Cherry".to_string(),
                count: 3,
            },
        ],
        map: HashMap::from([("apple".to_string(), 1), ("banana".to_string(), 2)]),
    };
    let serialized = to_vec_be(&player).unwrap();
    let doc = read_borrowed::<BigEndian>(&serialized).unwrap();
    let root = doc.root();

    let deserialized: Player = from_slice_be(&serialized).unwrap();
    println!("{}", dump(&root));
    println!("{:#?}", deserialized);

    let data = Data::Player {
        name: "John".to_string(),
        health: 100,
    };
    let serialized = to_vec_be(&data).unwrap();
    let doc = read_borrowed::<BigEndian>(&serialized).unwrap();
    let root = doc.root();
    let deserialized: Data = from_slice_be(&serialized).unwrap();
    println!("{}", dump(&root));
    println!("{:#?}", deserialized);

    let empty = Data::Empty;
    let serialized = to_vec_be(&empty).unwrap();
    let doc = read_borrowed::<BigEndian>(&serialized).unwrap();
    let root = doc.root();
    let deserialized: Data = from_slice_be(&serialized).unwrap();
    println!("{}", dump(&root));
    println!("{:#?}", deserialized);

    let value = Data::Value(100);
    let serialized = to_vec_be(&value).unwrap();
    let doc = read_borrowed::<BigEndian>(&serialized).unwrap();
    let root = doc.root();
    let deserialized: Data = from_slice_be(&serialized).unwrap();
    println!("{}", dump(&root));
    println!("{:#?}", deserialized);

    let point = Data::Point(100, 200);
    let serialized = to_vec_be(&point).unwrap();
    let doc = read_borrowed::<BigEndian>(&serialized).unwrap();
    let root = doc.root();
    let deserialized: Data = from_slice_be(&serialized).unwrap();
    println!("{}", dump(&root));
    println!("{:#?}", deserialized);

    let byte_array = Data::ByteArray(vec![1, 2, 3, 4, 5]);
    let serialized = to_vec_be(&byte_array).unwrap();
    let doc = read_borrowed::<BigEndian>(&serialized).unwrap();
    let root = doc.root();
    let deserialized: Data = from_slice_be(&serialized).unwrap();
    println!("{}", dump(&root));
    println!("{:#?}", deserialized);

    let int_array = Data::IntArray(vec![1, 2, 3, 4, 5]);
    let serialized = to_vec_be(&int_array).unwrap();
    let doc = read_borrowed::<BigEndian>(&serialized).unwrap();
    let root = doc.root();
    let deserialized: Data = from_slice_be(&serialized).unwrap();
    println!("{}", dump(&root));
    println!("{:#?}", deserialized);

    let long_array = Data::LongArray(vec![1, 2, 3, 4, 5]);
    let serialized = to_vec_be(&long_array).unwrap();
    let doc = read_borrowed::<BigEndian>(&serialized).unwrap();
    let root = doc.root();
    let deserialized: Data = from_slice_be(&serialized).unwrap();
    println!("{}", dump(&root));
    println!("{:#?}", deserialized);

    let serialized = to_vec_be(&()).unwrap();
    let doc = read_borrowed::<BigEndian>(&serialized).unwrap();
    let root = doc.root();
    let deserialized: () = from_slice_be(&serialized).unwrap();
    println!("{}", dump(&root));
    println!("{:#?}", deserialized);
}
