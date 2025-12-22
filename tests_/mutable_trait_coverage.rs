use na_nbt::{
    ScopedWritableValue, WritableValue, ScopedReadableValue,
    read_owned, read_borrowed, ReadableValue, ReadableString, TagID
};
use zerocopy::byteorder::BigEndian as BE;

#[test]
fn test_immutable_string_trait() {
    let mut data = vec![0x08, 0x00, 0x00];
    let s = "hello";
    data.extend_from_slice(&(s.len() as u16).to_be_bytes());
    data.extend_from_slice(s.as_bytes());
    
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let str_val = ReadableValue::as_string(&root).unwrap();
    
    // Test ReadableString trait methods explicitly
    assert_eq!(ReadableString::raw_bytes(str_val), s.as_bytes());
    assert_eq!(ReadableString::decode(str_val), s);
}

#[test]
fn test_mutable_value_scoped_writable_traits() {
    // Create compound with array types and string
    let mut data = vec![0x0A, 0x00, 0x00];
    
    // ByteArray "ba" [1, 2]
    data.push(0x07);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"ba");
    data.extend_from_slice(&2u32.to_be_bytes());
    data.push(1); data.push(2);
    
    // String "st" "foo"
    data.push(0x08);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"st");
    data.extend_from_slice(&3u16.to_be_bytes());
    data.extend_from_slice(b"foo");
    
    // IntArray "ia" [10]
    data.push(0x0B);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"ia");
    data.extend_from_slice(&1u32.to_be_bytes());
    data.extend_from_slice(&10i32.to_be_bytes());
    
    // LongArray "la" [100]
    data.push(0x0C);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"la");
    data.extend_from_slice(&1u32.to_be_bytes());
    data.extend_from_slice(&100i64.to_be_bytes());
    
    data.push(0x00); // End
    
    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    let mut comp = owned.as_compound_mut().unwrap();
    
    // 1. ByteArray
    if let Some(mut mv) = comp.get_mut("ba") {
        if let Some(mut view) = ScopedWritableValue::as_byte_array_mut_scoped(&mut mv) {
            view[0] = 3;
        }
        assert_eq!(ScopedReadableValue::as_byte_array_scoped(&mv).unwrap()[0], 3);
    } else { panic!("missing ba") }
    
    // 2. String
    if let Some(mut mv) = comp.get_mut("st") {
        if let Some(mut view) = ScopedWritableValue::as_string_mut_scoped(&mut mv) {
            view.push_str("bar");
        }
        assert_eq!(ScopedReadableValue::as_string_scoped(&mv).unwrap().decode(), "foobar");
    } else { panic!("missing st") }
    
    // 3. IntArray
    if let Some(mut mv) = comp.get_mut("ia") {
        if let Some(mut view) = ScopedWritableValue::as_int_array_mut_scoped(&mut mv) {
            view[0].set(20);
        }
        assert_eq!(ScopedReadableValue::as_int_array_scoped(&mv).unwrap()[0].get(), 20);
    } else { panic!("missing ia") }
    
    // 4. LongArray
    if let Some(mut mv) = comp.get_mut("la") {
        if let Some(mut view) = ScopedWritableValue::as_long_array_mut_scoped(&mut mv) {
            view[0].set(200);
        }
        assert_eq!(ScopedReadableValue::as_long_array_scoped(&mv).unwrap()[0].get(), 200);
    } else { panic!("missing la") }
}

#[test]
fn test_mutable_value_writable_value_traits() {
    let mut data = vec![0x0A, 0x00, 0x00];
    // List "li"
    data.push(0x09);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"li");
    data.push(0x01); // Byte
    data.extend_from_slice(&0u32.to_be_bytes()); // empty
    // Compound "co"
    data.push(0x0A);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"co");
    data.push(0x00); // empty
    data.push(0x00); // End root
    
    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    let mut comp = owned.as_compound_mut().unwrap();
    
    // List
    if let Some(mut mv) = comp.get_mut("li") {
        assert!(WritableValue::as_list_mut(&mut mv).is_some());
    } else { panic!("missing li") }
    
    // Compound
    if let Some(mut mv) = comp.get_mut("co") {
        assert!(WritableValue::as_compound_mut(&mut mv).is_some());
    } else { panic!("missing co") }
}

#[test]
fn test_immutable_value_trait_wrappers() {
    let mut data = vec![0x0A, 0x00, 0x00];
    // Byte 'b'
    data.push(0x01); data.extend_from_slice(&0x0001u16.to_be_bytes()); data.push(b'b'); data.push(10i8 as u8);
    // Short 's'
    data.push(0x02); data.extend_from_slice(&0x0001u16.to_be_bytes()); data.push(b's'); data.extend_from_slice(&10i16.to_be_bytes());
    // Int 'i'
    data.push(0x03); data.extend_from_slice(&0x0001u16.to_be_bytes()); data.push(b'i'); data.extend_from_slice(&10i32.to_be_bytes());
    // Long 'l'
    data.push(0x04); data.extend_from_slice(&0x0001u16.to_be_bytes()); data.push(b'l'); data.extend_from_slice(&10i64.to_be_bytes());
    // Float 'f'
    data.push(0x05); data.extend_from_slice(&0x0001u16.to_be_bytes()); data.push(b'f'); data.extend_from_slice(&10.5f32.to_be_bytes());
    // Double 'd'
    data.push(0x06); data.extend_from_slice(&0x0001u16.to_be_bytes()); data.push(b'd'); data.extend_from_slice(&10.5f64.to_be_bytes());
    data.push(0x00); // end

    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let comp = root.as_compound().unwrap();

    // 1. Byte
    let v = comp.get("b").unwrap();
    assert_eq!(ScopedReadableValue::tag_id(&v), TagID::Byte);
    assert!(ScopedReadableValue::is_byte(&v));
    assert_eq!(ScopedReadableValue::as_byte(&v), Some(10));
    assert!(!ScopedReadableValue::is_short(&v));

    // 2. Short
    let v = comp.get("s").unwrap();
    assert!(ScopedReadableValue::is_short(&v));
    assert_eq!(ScopedReadableValue::as_short(&v), Some(10));

    // 3. Int
    let v = comp.get("i").unwrap();
    assert!(ScopedReadableValue::is_int(&v));
    assert_eq!(ScopedReadableValue::as_int(&v), Some(10));

    // 4. Long
    let v = comp.get("l").unwrap();
    assert!(ScopedReadableValue::is_long(&v));
    assert_eq!(ScopedReadableValue::as_long(&v), Some(10));

    // 5. Float
    let v = comp.get("f").unwrap();
    assert!(ScopedReadableValue::is_float(&v));
    assert_eq!(ScopedReadableValue::as_float(&v), Some(10.5));

    // 6. Double
    let v = comp.get("d").unwrap();
    assert!(ScopedReadableValue::is_double(&v));
    assert_eq!(ScopedReadableValue::as_double(&v), Some(10.5));
}
