use na_nbt::{OwnedValue, read_owned};
use zerocopy::byteorder::{BigEndian, LittleEndian};

fn create_primitive_list_be(tag: u8, element_size: usize, data: &[u8]) -> Vec<u8> {
    let count = data.len() / element_size;
    let mut res = vec![0x09]; // List
    res.extend_from_slice(&0u16.to_be_bytes()); // Name
    res.push(tag);
    res.extend_from_slice(&(count as u32).to_be_bytes());
    res.extend_from_slice(data);
    res
}

#[test]
fn test_read_fallback_primitives() {
    // Short (Tag 2)
    let val_short: i16 = 0x1234;
    let data = create_primitive_list_be(2, 2, &val_short.to_be_bytes());
    let owned = read_owned::<BigEndian, LittleEndian>(&data).unwrap();
    if let OwnedValue::List(l) = owned {
        assert_eq!(l.get(0).unwrap().as_short(), Some(0x1234));
    } else {
        panic!("Expected list");
    }

    // Long (Tag 4)
    let val_long: i64 = 0x1234567890ABCDEF;
    let data = create_primitive_list_be(4, 8, &val_long.to_be_bytes());
    let owned = read_owned::<BigEndian, LittleEndian>(&data).unwrap();
    if let OwnedValue::List(l) = owned {
        assert_eq!(l.get(0).unwrap().as_long(), Some(0x1234567890ABCDEF));
    } else {
        panic!("Expected list");
    }

    // Float (Tag 5)
    let val_float: f32 = 1.234;
    let data = create_primitive_list_be(5, 4, &val_float.to_be_bytes());
    let owned = read_owned::<BigEndian, LittleEndian>(&data).unwrap();
    if let OwnedValue::List(l) = owned {
        // Strict equality might fail slightly due to float ops, but strict read should preserve bits?
        // Wait, f32 to_be_bytes then read as LE means the *memory* will be LE representation of the float.
        // The value should remain the same.
        assert_eq!(l.get(0).unwrap().as_float(), Some(1.234));
    } else {
        panic!("Expected list");
    }

    // Double (Tag 6)
    let val_double: f64 = 123.456789;
    let data = create_primitive_list_be(6, 8, &val_double.to_be_bytes());
    let owned = read_owned::<BigEndian, LittleEndian>(&data).unwrap();
    if let OwnedValue::List(l) = owned {
        assert_eq!(l.get(0).unwrap().as_double(), Some(123.456789));
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_read_fallback_arrays() {
    // ByteArray (Tag 7) - No endianness swap needed for u8/i8, but goes through fallback path logic
    let bytes = vec![1i8, 2, 3];
    let mut data = vec![0x0A, 0x00, 0x00]; // Compound
    data.push(0x07); // ByteArray
    data.extend_from_slice(&1u16.to_be_bytes()); // Name len
    data.push(b'b'); // Name
    data.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    for b in &bytes {
        data.push(*b as u8);
    }
    data.push(0x00); // End

    let owned = read_owned::<BigEndian, LittleEndian>(&data).unwrap();
    if let OwnedValue::Compound(c) = owned {
        assert_eq!(
            c.get("b").unwrap().as_byte_array().cloned(),
            Some(bytes.as_slice())
        );
    } else {
        panic!("Expected compound");
    }

    // IntArray (Tag 11) - Needs swap
    let ints: Vec<i32> = vec![0x12345678, 0x0ABCDEF0];
    let mut data = vec![0x0A, 0x00, 0x00];
    data.push(0x0B); // IntArray
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'i');
    data.extend_from_slice(&(ints.len() as u32).to_be_bytes());
    for i in &ints {
        data.extend_from_slice(&(*i).to_be_bytes());
    }
    data.push(0x00);

    let owned = read_owned::<BigEndian, LittleEndian>(&data).unwrap();
    if let OwnedValue::Compound(c) = owned {
        let val = c.get("i").unwrap();
        let arr = val.as_int_array().unwrap();
        assert_eq!(arr[0].get(), 0x12345678);
        assert_eq!(arr[1].get(), 0x0ABCDEF0);
    } else {
        panic!("Expected compound");
    }

    // LongArray (Tag 12) - Needs swap
    let longs: Vec<i64> = vec![0x1234567890ABCDEF, 0x0FEDCBA098765432];
    let mut data = vec![0x0A, 0x00, 0x00];
    data.push(0x0C); // LongArray
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'l');
    data.extend_from_slice(&(longs.len() as u32).to_be_bytes());
    for l in &longs {
        data.extend_from_slice(&(*l).to_be_bytes());
    }
    data.push(0x00);

    let owned = read_owned::<BigEndian, LittleEndian>(&data).unwrap();
    if let OwnedValue::Compound(c) = owned {
        let val = c.get("l").unwrap();
        let arr = val.as_long_array().unwrap();
        assert_eq!(arr[0].get(), 0x1234567890ABCDEF);
        assert_eq!(arr[1].get(), 0x0FEDCBA098765432);
    } else {
        panic!("Expected compound");
    }
}

#[test]
fn test_read_fallback_string() {
    // String (Tag 8) - No endian swap for content, but length prefix is swapped (handled by reader)
    let s = "Hello World";
    let mut data = vec![0x0A, 0x00, 0x00];
    data.push(0x08);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b's');
    data.extend_from_slice(&(s.len() as u16).to_be_bytes());
    data.extend_from_slice(s.as_bytes());
    data.push(0x00);

    let owned = read_owned::<BigEndian, LittleEndian>(&data).unwrap();
    if let OwnedValue::Compound(c) = owned {
        assert_eq!(c.get("s").unwrap().as_string().unwrap().decode(), s);
    } else {
        panic!("Expected compound");
    }
}
