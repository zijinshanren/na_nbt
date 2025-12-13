use na_nbt::Tag;
use zerocopy::byteorder::BigEndian as BE;
use zerocopy::byteorder::{I32, I64};

#[test]
fn test_mutable_value_accessors_and_mutators() {
    let mut data = vec![0x0A, 0x00, 0x00]; // Root compound

    // 1. Byte
    data.push(0x01);
    data.extend_from_slice(&0x0001u16.to_be_bytes()); // name "b"
    data.push(b'b');
    data.push(10i8 as u8);

    // 2. Short
    data.push(0x02);
    data.extend_from_slice(&0x0001u16.to_be_bytes()); // name "s"
    data.push(b's');
    data.extend_from_slice(&20i16.to_be_bytes());

    // 3. Int
    data.push(0x03);
    data.extend_from_slice(&0x0001u16.to_be_bytes()); // name "i"
    data.push(b'i');
    data.extend_from_slice(&30i32.to_be_bytes());

    // 4. Long
    data.push(0x04);
    data.extend_from_slice(&0x0001u16.to_be_bytes()); // name "l"
    data.push(b'l');
    data.extend_from_slice(&40i64.to_be_bytes());

    // 5. Float
    data.push(0x05);
    data.extend_from_slice(&0x0001u16.to_be_bytes()); // name "f"
    data.push(b'f');
    data.extend_from_slice(&50.5f32.to_be_bytes());

    // 6. Double
    data.push(0x06);
    data.extend_from_slice(&0x0001u16.to_be_bytes()); // name "d"
    data.push(b'd');
    data.extend_from_slice(&60.6f64.to_be_bytes());

    // 7. ByteArray
    data.push(0x07);
    data.extend_from_slice(&0x0002u16.to_be_bytes()); // name "ba"
    data.extend_from_slice(b"ba");
    data.extend_from_slice(&2u32.to_be_bytes()); // len 2
    data.push(1);
    data.push(2);

    // 8. String
    data.push(0x08);
    data.extend_from_slice(&0x0002u16.to_be_bytes()); // name "st"
    data.extend_from_slice(b"st");
    data.extend_from_slice(&3u16.to_be_bytes()); // len 3
    data.extend_from_slice(b"str");

    // 9. List
    data.push(0x09);
    data.extend_from_slice(&0x0002u16.to_be_bytes()); // name "li"
    data.extend_from_slice(b"li");
    data.push(0x01); // type byte
    data.extend_from_slice(&1u32.to_be_bytes()); // len 1
    data.push(5);

    // 10. Compound (nested)
    data.push(0x0A);
    data.extend_from_slice(&0x0002u16.to_be_bytes()); // name "co"
    data.extend_from_slice(b"co");
    data.push(0x00); // end

    // 11. IntArray
    data.push(0x0B);
    data.extend_from_slice(&0x0002u16.to_be_bytes()); // name "ia"
    data.extend_from_slice(b"ia");
    data.extend_from_slice(&2u32.to_be_bytes()); // len 2
    data.extend_from_slice(&100i32.to_be_bytes());
    data.extend_from_slice(&200i32.to_be_bytes());

    // 12. LongArray
    data.push(0x0C);
    data.extend_from_slice(&0x0002u16.to_be_bytes()); // name "la"
    data.extend_from_slice(b"la");
    data.extend_from_slice(&2u32.to_be_bytes()); // len 2
    data.extend_from_slice(&1000i64.to_be_bytes());
    data.extend_from_slice(&2000i64.to_be_bytes());

    data.push(0x00); // End root

    use na_nbt::read_owned;
    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    let mut comp = owned.as_compound_mut().unwrap();

    // 1. Byte
    if let Some(mut mv) = comp.get_mut("b") {
        assert!(mv.is_byte());
        assert_eq!(mv.tag_id(), Tag::Byte);
        assert_eq!(mv.as_byte(), Some(10));
        assert_eq!(mv.as_byte_mut(), Some(&mut 10));
        assert!(mv.set_byte(11));
        assert_eq!(mv.as_byte(), Some(11));
        assert!(mv.update_byte(|b| b + 1));
        assert_eq!(mv.as_byte(), Some(12));

        // Negative tests
        assert!(!mv.is_short());
        assert!(mv.as_short().is_none());
        assert!(mv.as_short_mut().is_none());
        assert!(!mv.set_short(1));
        assert!(!mv.update_short(|s| s));
    } else {
        panic!("Missing b");
    }

    // 2. Short
    if let Some(mut mv) = comp.get_mut("s") {
        assert!(mv.is_short());
        assert_eq!(mv.tag_id(), Tag::Short);
        assert_eq!(mv.as_short(), Some(20));
        assert!(mv.as_short_mut().is_some());
        assert!(mv.set_short(21));
        assert_eq!(mv.as_short(), Some(21));
        assert!(mv.update_short(|s| s + 1));
        assert_eq!(mv.as_short(), Some(22));

        assert!(!mv.is_byte());
        assert!(mv.as_byte().is_none());
    } else {
        panic!("Missing s");
    }

    // 3. Int
    if let Some(mut mv) = comp.get_mut("i") {
        assert!(mv.is_int());
        assert_eq!(mv.tag_id(), Tag::Int);
        assert_eq!(mv.as_int(), Some(30));
        assert!(mv.as_int_mut().is_some());
        assert!(mv.set_int(31));
        assert_eq!(mv.as_int(), Some(31));
        assert!(mv.update_int(|i| i + 1));
        assert_eq!(mv.as_int(), Some(32));
    } else {
        panic!("Missing i");
    }

    // 4. Long
    if let Some(mut mv) = comp.get_mut("l") {
        assert!(mv.is_long());
        assert_eq!(mv.tag_id(), Tag::Long);
        assert_eq!(mv.as_long(), Some(40));
        assert!(mv.as_long_mut().is_some());
        assert!(mv.set_long(41));
        assert_eq!(mv.as_long(), Some(41));
        assert!(mv.update_long(|l| l + 1));
        assert_eq!(mv.as_long(), Some(42));
    } else {
        panic!("Missing l");
    }

    // 5. Float
    if let Some(mut mv) = comp.get_mut("f") {
        assert!(mv.is_float());
        assert_eq!(mv.tag_id(), Tag::Float);
        assert_eq!(mv.as_float(), Some(50.5));
        assert!(mv.as_float_mut().is_some());
        assert!(mv.set_float(51.5));
        assert_eq!(mv.as_float(), Some(51.5));
        assert!(mv.update_float(|f| f + 1.0));
        assert_eq!(mv.as_float(), Some(52.5));
    } else {
        panic!("Missing f");
    }

    // 6. Double
    if let Some(mut mv) = comp.get_mut("d") {
        assert!(mv.is_double());
        assert_eq!(mv.tag_id(), Tag::Double);
        assert_eq!(mv.as_double(), Some(60.6));
        assert!(mv.as_double_mut().is_some());
        assert!(mv.set_double(61.6));
        assert_eq!(mv.as_double(), Some(61.6));
        assert!(mv.update_double(|d| d + 1.0));
        assert_eq!(mv.as_double(), Some(62.6));
    } else {
        panic!("Missing d");
    }

    // 7. ByteArray
    if let Some(mut mv) = comp.get_mut("ba") {
        assert!(mv.is_byte_array());
        assert_eq!(mv.tag_id(), Tag::ByteArray);
        assert!(mv.as_byte_array().is_some());
        assert!(mv.as_byte_array_mut().is_some());
        
        let arr = mv.as_byte_array_mut().unwrap();
        arr[0] = 99;
        assert_eq!(mv.as_byte_array().unwrap()[0], 99);
    } else {
        panic!("Missing ba");
    }

    // 8. String
    if let Some(mut mv) = comp.get_mut("st") {
        assert!(mv.is_string());
        assert_eq!(mv.tag_id(), Tag::String);
        assert!(mv.as_string().is_some());
        assert!(mv.as_string_mut().is_some());

        let s = mv.as_string_mut().unwrap();
        s.push_str("ing");
        assert_eq!(mv.as_string().unwrap().decode(), "string");
    } else {
        panic!("Missing st");
    }

    // 9. List
    if let Some(mut mv) = comp.get_mut("li") {
        assert!(mv.is_list());
        assert_eq!(mv.tag_id(), Tag::List);
        assert!(mv.as_list().is_some());
        assert!(mv.as_list_mut().is_some());
        
        let l = mv.as_list_mut().unwrap();
        l.push(6i8);
        assert_eq!(l.len(), 2);
    } else {
        panic!("Missing li");
    }

    // 10. Compound
    if let Some(mut mv) = comp.get_mut("co") {
        assert!(mv.is_compound());
        assert_eq!(mv.tag_id(), Tag::Compound);
        assert!(mv.as_compound().is_some());
        assert!(mv.as_compound_mut().is_some());
        
        let c = mv.as_compound_mut().unwrap();
        c.insert("new", 1i8);
        assert!(c.get("new").is_some());
    } else {
        panic!("Missing co");
    }

    // 11. IntArray
    if let Some(mut mv) = comp.get_mut("ia") {
        assert!(mv.is_int_array());
        assert_eq!(mv.tag_id(), Tag::IntArray);
        assert!(mv.as_int_array().is_some());
        assert!(mv.as_int_array_mut().is_some());
        
        let arr = mv.as_int_array_mut().unwrap();
        arr[0] = I32::new(101);
        assert_eq!(mv.as_int_array().unwrap()[0].get(), 101);
    } else {
        panic!("Missing ia");
    }

    // 12. LongArray
    if let Some(mut mv) = comp.get_mut("la") {
        assert!(mv.is_long_array());
        assert_eq!(mv.tag_id(), Tag::LongArray);
        assert!(mv.as_long_array().is_some());
        assert!(mv.as_long_array_mut().is_some());

        let arr = mv.as_long_array_mut().unwrap();
        arr[0] = I64::new(1001);
        assert_eq!(mv.as_long_array().unwrap()[0].get(), 1001);
    } else {
        panic!("Missing la");
    }
}

