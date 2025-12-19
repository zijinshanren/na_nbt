use bytes::Bytes;
use na_nbt::{
    BigEndian, LittleEndian, ReadableString as _, ScopedReadableList as _, ScopedReadableValue,
    ValueScoped, read_borrowed, read_owned, read_shared,
};

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = include_bytes!();

    if let Ok(doc) = read_borrowed::<BigEndian>(data) {
        println!("{}", dump(&doc.root()));
        let _ = doc.root().write_to_vec::<BigEndian>();
        let _ = doc.root().write_to_vec::<LittleEndian>();
    }
    if let Ok(doc) = read_borrowed::<LittleEndian>(data) {
        println!("{}", dump(&doc.root()));
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
    let _ = read_owned::<BigEndian, BigEndian>(data);
    if let Ok(root) = read_owned::<BigEndian, BigEndian>(data) {
        let _ = root.write_to_vec::<LittleEndian>();
        let _ = root.write_to_vec::<BigEndian>();
    }
    Ok(())
}
