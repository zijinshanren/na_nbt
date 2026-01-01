use std::fs::File;

use na_nbt::{
    BE, CompoundMut, CompoundOwn, CompoundRef, ListBase, ListRef, TypedListOwn, ValueRef, VisitRef,
    read_borrowed,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = include_bytes!("../fuzz/in/level");
    let nbt = read_borrowed::<BE>(data)?;
    let str = dump(&nbt.root(), 0);
    println!("{}", str);
    let _ = an_address();
    Ok(())
}

fn example<'s>(nbt: &mut impl CompoundMut<'s>) {
    let mut comp = CompoundOwn::default();
    comp.insert("Int", 1);
    comp.insert("String", "2");
    comp.insert("ByteArray", [1i8, 2, 3]);
    nbt.insert("Compound", comp);
    let mut list = TypedListOwn::default();
    list.push(1);
    list.push(2);
    list.push(3);
    nbt.insert("List", list);
}

fn dump<'s>(value: &impl ValueRef<'s>, indent: usize) -> String {
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
        VisitRef::String(v) => format!("{pad}String({:?})", v.decode_lossy()),
        VisitRef::IntArray(v) => format!("{pad}IntArray({} ints)", v.len()),
        VisitRef::LongArray(v) => format!("{pad}LongArray({} longs)", v.len()),
        VisitRef::List(list) => {
            let mut out = format!("{pad}List[{}] {{\n", list.len());
            for item in list.iter() {
                out.push_str(&dump(&item, indent + 1));
                out.push('\n');
            }
            out.push_str(&format!("{pad}}}"));
            out
        }
        VisitRef::Compound(compound) => {
            let mut out = format!("{pad}Compound {{\n");
            for (key, val) in compound.iter() {
                let nested = dump(&val, indent + 1);
                out.push_str(&format!(
                    "{}  {:?}: {}\n",
                    pad,
                    key.decode_lossy(),
                    nested.trim_start()
                ));
            }
            out.push_str(&format!("{pad}}}"));
            out
        }
    })
}

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Address {
    street: String,
    city: String,
}

fn an_address() -> na_nbt::Result<()> {
    let address = Address {
        street: "10 Downing Street".to_owned(),
        city: "London".to_owned(),
    };

    let j = na_nbt::to_vec_be(&address)?;
    let addr: Address = na_nbt::from_slice_be(&j)?;
    assert_eq!(addr, address);

    Ok(())
}
