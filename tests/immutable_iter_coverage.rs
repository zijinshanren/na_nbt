use na_nbt::{read_borrowed, Tag, ReadableValue, ReadableCompound, ReadableList, ScopedReadableList, ScopedReadableValue};
use zerocopy::byteorder::BigEndian;

fn create_all_types_compound() -> Vec<u8> {
    let mut buf = vec![0x0A, 0x00, 0x00];
    
    // Byte
    buf.push(Tag::Byte as u8);
    buf.extend_from_slice(&1u16.to_be_bytes()); buf.push(b'b');
    buf.push(1);

    // Short
    buf.push(Tag::Short as u8);
    buf.extend_from_slice(&1u16.to_be_bytes()); buf.push(b's');
    buf.extend_from_slice(&2u16.to_be_bytes());

    // Int
    buf.push(Tag::Int as u8);
    buf.extend_from_slice(&1u16.to_be_bytes()); buf.push(b'i');
    buf.extend_from_slice(&3u32.to_be_bytes());
    
    // Long
    buf.push(Tag::Long as u8);
    buf.extend_from_slice(&1u16.to_be_bytes()); buf.push(b'l');
    buf.extend_from_slice(&4u64.to_be_bytes());
    
    // Float
    buf.push(Tag::Float as u8);
    buf.extend_from_slice(&1u16.to_be_bytes()); buf.push(b'f');
    buf.extend_from_slice(&5.0f32.to_be_bytes());
    
    // Double
    buf.push(Tag::Double as u8);
    buf.extend_from_slice(&1u16.to_be_bytes()); buf.push(b'd');
    buf.extend_from_slice(&6.0f64.to_be_bytes());
    
    // ByteArray
    buf.push(Tag::ByteArray as u8);
    buf.extend_from_slice(&2u16.to_be_bytes()); buf.extend_from_slice(b"ba");
    buf.extend_from_slice(&2u32.to_be_bytes()); buf.extend_from_slice(&[7, 8]);
    
    // String
    buf.push(Tag::String as u8);
    buf.extend_from_slice(&2u16.to_be_bytes()); buf.extend_from_slice(b"st");
    buf.extend_from_slice(&3u16.to_be_bytes()); buf.extend_from_slice(b"str");
    
    // List (of bytes)
    buf.push(Tag::List as u8);
    buf.extend_from_slice(&2u16.to_be_bytes()); buf.extend_from_slice(b"li");
    buf.push(Tag::Byte as u8); buf.extend_from_slice(&1u32.to_be_bytes());
    buf.push(10);
    
    // Compound
    buf.push(Tag::Compound as u8);
    buf.extend_from_slice(&2u16.to_be_bytes()); buf.extend_from_slice(b"co");
    buf.push(Tag::End as u8); // Empty compound
    
    // IntArray
    buf.push(Tag::IntArray as u8);
    buf.extend_from_slice(&2u16.to_be_bytes()); buf.extend_from_slice(b"ia");
    buf.extend_from_slice(&1u32.to_be_bytes()); buf.extend_from_slice(&11u32.to_be_bytes());
    
    // LongArray
    buf.push(Tag::LongArray as u8);
    buf.extend_from_slice(&2u16.to_be_bytes()); buf.extend_from_slice(b"la");
    buf.extend_from_slice(&1u32.to_be_bytes()); buf.extend_from_slice(&12u64.to_be_bytes());
    
    buf.push(0x00); // End
    buf
}

#[test]
fn test_immutable_iter_all_types() {
    let data = create_all_types_compound();
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();
    let comp = root.as_compound().unwrap();
    
    let mut count = 0;
    // Iterating over the compound will call tag_size to find the next element
    for (name, val) in comp.iter() {
        count += 1;
        match name.decode().as_ref() {
            "b" => assert!(val.as_byte().is_some()),
            "s" => assert!(val.as_short().is_some()),
            "i" => assert!(val.as_int().is_some()),
            "l" => assert!(val.as_long().is_some()),
            "f" => assert!(val.as_float().is_some()),
            "d" => assert!(val.as_double().is_some()),
            "ba" => assert!(val.as_byte_array().is_some()),
            "st" => assert!(val.as_string().is_some()),
            "li" => assert!(val.as_list().is_some()),
            "co" => assert!(val.as_compound().is_some()),
            "ia" => assert!(val.as_int_array().is_some()),
            "la" => assert!(val.as_long_array().is_some()),
            _ => panic!("Unknown tag {}", name.decode()),
        }
    }
    assert_eq!(count, 12);
}

#[test]
fn test_immutable_get_random_access() {
    // This tests skipping tags using tag_size via get()
    let data = create_all_types_compound();
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();
    let comp = root.as_compound().unwrap();
    
    // Access in reverse order to force full scans if optimization isn't there (but we just want correctness)
    // Actually get() scans linearly from start each time? 
    // "get" in ImmutableCompound loops from data start.
    // So accessing "la" (last element) forces skipping all previous ones.
    
    assert!(comp.get("la").unwrap().as_long_array().is_some());
    assert!(comp.get("ia").unwrap().as_int_array().is_some());
    assert!(comp.get("co").unwrap().as_compound().is_some());
    assert!(comp.get("li").unwrap().as_list().is_some());
    assert!(comp.get("st").unwrap().as_string().is_some());
    assert!(comp.get("ba").unwrap().as_byte_array().is_some());
    assert!(comp.get("d").unwrap().as_double().is_some());
    assert!(comp.get("f").unwrap().as_float().is_some());
    assert!(comp.get("l").unwrap().as_long().is_some());
    assert!(comp.get("i").unwrap().as_int().is_some());
    assert!(comp.get("s").unwrap().as_short().is_some());
    assert!(comp.get("b").unwrap().as_byte().is_some());
}

