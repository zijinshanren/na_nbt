use bytes::Bytes;
use honggfuzz::fuzz;
use na_nbt::{BigEndian, LittleEndian, read_borrowed, read_owned, read_shared};

fn main() {
    loop {
        fuzz!(|data: &[u8]| {
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
            let _ = read_owned::<BigEndian, BigEndian>(data);
            if let Ok(root) = read_owned::<BigEndian, BigEndian>(data) {
                let _ = root.write_to_vec::<LittleEndian>();
                let _ = root.write_to_vec::<BigEndian>();
            }
        });
    }
}
