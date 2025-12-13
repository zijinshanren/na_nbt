use na_nbt::{read_owned, OwnedValue};
use zerocopy::byteorder::BigEndian as BE;

fn create_compound_mixed_be() -> Vec<u8> {
    // Root compound header
    let mut result = vec![0x0A, 0x00, 0x00];

    // Int 'a' = 10
    result.push(0x03); // Tag Int
    result.extend_from_slice(&0x0001u16.to_be_bytes()); // name len 1
    result.push(b'a');
    result.extend_from_slice(&10i32.to_be_bytes());

    // String 's' = "foo"
    result.push(0x08); // Tag String
    result.extend_from_slice(&0x0001u16.to_be_bytes());
    result.push(b's');
    result.extend_from_slice(&0x0003u16.to_be_bytes());
    result.extend_from_slice(b"foo");

    // ByteArray 'ba' = [1,2,3]
    result.push(0x07); // Tag ByteArray
    result.extend_from_slice(&0x0002u16.to_be_bytes());
    result.extend_from_slice(b"ba");
    result.extend_from_slice(&0x00000003u32.to_be_bytes());
    result.extend_from_slice(&[1u8, 2u8, 3u8]);

    // List 'li' of ints [1,2]
    result.push(0x09); // Tag List
    result.extend_from_slice(&0x0002u16.to_be_bytes());
    result.extend_from_slice(b"li");
    result.push(0x03); // element type = Int
    result.extend_from_slice(&0x00000002u32.to_be_bytes());
    result.extend_from_slice(&1i32.to_be_bytes());
    result.extend_from_slice(&2i32.to_be_bytes());
    
    // short sh = 6
    result.push(0x02);
    result.extend_from_slice(&0x0002u16.to_be_bytes());
    result.extend_from_slice(b"sh");
    result.extend_from_slice(&6i16.to_be_bytes());

    // float f = 3.5
    result.push(0x05);
    result.extend_from_slice(&0x0001u16.to_be_bytes());
    result.push(b'f');
    result.extend_from_slice(&3.5f32.to_be_bytes());

    // double d = 1.234
    result.push(0x06);
    result.extend_from_slice(&0x0001u16.to_be_bytes());
    result.push(b'd');
    result.extend_from_slice(&1.234f64.to_be_bytes());

    // End compound
    result.push(0x00);
    result
}

#[test]
fn owned_compound_mutations_and_visits() {
    let data = create_compound_mixed_be();
    let mut owned = read_owned::<BE, BE>(&data).unwrap();

    use na_nbt::ScopedReadableValue;
    use na_nbt::ValueScoped;

    if let OwnedValue::Compound(mut c) = owned {
        assert!(c.get("a").is_some());
        assert_eq!(c.get("a").unwrap().as_int(), Some(10));

        // Test set and update via get_mut (call methods on MutableValue)
        if let Some(mut mv) = c.get_mut("a") {
            assert!(mv.set_int(20));
            assert!(mv.update_int(|x| x + 5));
        } else {
            panic!("expected existing key 'a'");
        }

        assert_eq!(c.get("a").unwrap().as_int(), Some(25));

        // Mutate string via direct StringViewMut
        if let Some(mut mv) = c.get_mut("s") {
            if let na_nbt::MutableValue::String(ref mut sview) = mv {
                sview.push_str("bar");
            }
        }

        assert_eq!(c.get("s").unwrap().as_string().unwrap().decode().to_string(), "foobar");

        // Mutate byte array
        if let Some(mut mv) = c.get_mut("ba") {
            if let na_nbt::MutableValue::ByteArray(ref mut bv) = mv {
                let slice = bv.as_mut_slice();
                slice[0] = 9i8; // set first element
            }
        }

        assert_eq!(c.get("ba").unwrap().as_byte_array().unwrap()[0], 9i8);

        // Mutate list by pushing an int
        if let Some(mut mv) = c.get_mut("li") {
            if let na_nbt::MutableValue::List(ref mut l) = mv {
                l.push(42i32);
                assert_eq!(l.get(2).unwrap().as_int(), Some(42));
            }
        }

        // Test visit scoped on ImmutableValue obtained from get()
        let got = c.get("a").unwrap().visit_scoped(|v| match v {
            ValueScoped::Int(i) => i,
            _ => panic!("unexpected scoped value"),
        });
        assert_eq!(got, 25);
    } else {
        panic!("expected compound");
    }
}

#[test]
fn owned_list_push_various_types() {
    // Create a list of strings: ["one"]
    let mut data = vec![0x09, 0x00, 0x00, 0x08]; // list tag, empty name, element type = string
    data.extend_from_slice(&0x00000001u32.to_be_bytes());
    data.extend_from_slice(&0x0003u16.to_be_bytes());
    data.extend_from_slice(b"one");

    let mut owned = read_owned::<BE, BE>(&data).unwrap();

    if let OwnedValue::List(mut l) = owned {
        assert_eq!(l.len(), 1);
        l.push("two");
        assert_eq!(l.len(), 2);
        assert_eq!(l.get(1).unwrap().as_string().unwrap().decode().to_string(), "two");
    } else {
        panic!("expected list");
    }

    // Test calling trait methods on OwnedValue (wrapper trait impls)
    use na_nbt::ScopedWritableValue;
    use na_nbt::ScopedReadableValue;

    // OwnedValue root Int: set via trait and read back via trait
    let mut owned_val = read_owned::<BE, BE>(&vec![0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07]).unwrap();
    assert!(na_nbt::ScopedWritableValue::set_int(&mut owned_val, 12));
    assert_eq!(na_nbt::ScopedReadableValue::as_int(&owned_val), Some(12));

    // Also cover trait methods on MutableValue via the ScopedWritableValue/ScopedReadableValue impls
    let mut c2 = read_owned::<BE, BE>(&create_compound_mixed_be()).unwrap();
    if let OwnedValue::Compound(mut comp2) = c2 {
        if let Some(mut mv) = comp2.get_mut("a") {
            assert!(na_nbt::ScopedWritableValue::set_int(&mut mv, 99));
            let read_back = na_nbt::ScopedReadableValue::as_int(&mv);
            assert_eq!(read_back, Some(99));
        } else {
            panic!("expected to find key 'a'");
        }
    } else {
        panic!("expected compound");
    }
}

#[test]
fn owned_value_visit_owned_and_mutable() {
    // Root: Int 7
    let data = vec![0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07];
    let owned = read_owned::<BE, BE>(&data).unwrap();
    use na_nbt::ScopedReadableValue;
    use na_nbt::ValueScoped;

    if let OwnedValue::Int(_) = owned {
        let val = owned.visit_scoped(|v| match v {
            ValueScoped::Int(x) => x,
            _ => panic!("unexpected"),
        });
        assert_eq!(val, 7);
    } else {
        panic!("expected int");
    }
}

#[test]
fn into_owned_value_insert_various_types() {
    // empty root compound
    let data = vec![0x0A, 0x00, 0x00, 0x00];
    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::Compound(mut c) = owned {
        c.insert("d", 3.14f64);
        assert!((c.get("d").unwrap().as_double().unwrap() - 3.14).abs() < 0.0001);

        c.insert("ba1", vec![1i8, 2i8, 3i8]);
        assert_eq!(c.get("ba1").unwrap().as_byte_array().unwrap(), &[1i8, 2i8, 3i8]);

        use zerocopy::byteorder;
        c.insert("ia", vec![byteorder::I32::<BE>::new(0), byteorder::I32::<BE>::new(1), byteorder::I32::<BE>::new(2)]);
        assert_eq!(c.get("ia").unwrap().as_int_array().unwrap()[1].get(), 1);

        c.insert("la", vec![byteorder::I64::<BE>::new(0), byteorder::I64::<BE>::new(1)]);
        assert_eq!(c.get("la").unwrap().as_long_array().unwrap()[1].get(), 1);

        // Test inserting array by slice and by [i8;N]
        c.insert("ba2", &[9i8, 8i8][..]);
        assert_eq!(c.get("ba2").unwrap().as_byte_array().unwrap()[0], 9i8);

        c.insert("ba3", [7i8, 6i8]);
        assert_eq!(c.get("ba3").unwrap().as_byte_array().unwrap()[1], 6i8);
    } else {
        panic!("expected compound");
    }
}

#[test]
fn trait_impls_explicit_calls() {
    use na_nbt::{ScopedWritableValue, ScopedReadableValue};
    use zerocopy::byteorder;

    // Build a compound with many types
    let mut data = vec![0x0A, 0x00, 0x00];
    // byte b = 5
    data.push(0x01);
    data.extend_from_slice(&0x0001u16.to_be_bytes());
    data.push(b'b');
    data.push(5u8);
    // short sh = 6
    data.push(0x02);
    data.extend_from_slice(&0x0002u16.to_be_bytes());
    data.extend_from_slice(b"sh");
    data.extend_from_slice(&6i16.to_be_bytes());
    // float f = 3.5
    data.push(0x05);
    data.extend_from_slice(&0x0001u16.to_be_bytes());
    data.push(b'f');
    data.extend_from_slice(&3.5f32.to_be_bytes());
    // double d = 1.234
    data.push(0x06);
    data.extend_from_slice(&0x0001u16.to_be_bytes());
    data.push(b'd');
    data.extend_from_slice(&1.234f64.to_be_bytes());
    data.push(0x00); // end

    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    if let OwnedValue::Compound(mut comp) = owned {
        // Set byte via ScopedWritableValue on MutableValue
        if let Some(mut mv) = comp.get_mut("b") {
            assert!(ScopedWritableValue::set_byte(&mut mv, 42));
            assert_eq!(ScopedReadableValue::as_byte(&mv), Some(42));
            assert!(ScopedWritableValue::update_byte(&mut mv, |x| x + 1));
            assert_eq!(ScopedReadableValue::as_byte(&mv), Some(43));
        }

        // Set short via trait
        if let Some(mut mv) = comp.get_mut("sh") {
            assert!(ScopedWritableValue::set_short(&mut mv, 123));
            assert_eq!(ScopedReadableValue::as_short(&mv), Some(123));
            assert!(ScopedWritableValue::update_short(&mut mv, |x| x - 1));
            assert_eq!(ScopedReadableValue::as_short(&mv), Some(122));
        }

        // Set float/double via trait
        if let Some(mut mv) = comp.get_mut("f") {
            assert!(ScopedWritableValue::set_float(&mut mv, 7.25));
            assert_eq!(ScopedReadableValue::as_float(&mv), Some(7.25));
        }
        if let Some(mut mv) = comp.get_mut("d") {
            assert!(ScopedWritableValue::set_double(&mut mv, 6.28));
            assert_eq!(ScopedReadableValue::as_double(&mv), Some(6.28));
        }
    } else {
        panic!("expected compound");
    }
}

#[test]
fn trait_impl_more_mut_calls() {
    let mut owned = read_owned::<BE, BE>(&create_compound_mixed_be()).unwrap();
    if let OwnedValue::Compound(mut c) = owned {
        // short
        if let Some(mut mv) = c.get_mut("sh") {
            assert!(mv.set_short(321));
            assert!(mv.update_short(|x| x - 1));
            assert_eq!(mv.as_short(), Some(320));
        }
        // float
        if let Some(mut mv) = c.get_mut("f") {
            assert!(mv.set_float(1.5));
            assert!(mv.update_float(|x| x + 0.5));
            assert_eq!(mv.as_float(), Some(2.0));
        }
        // double
        if let Some(mut mv) = c.get_mut("d") {
            assert!(mv.set_double(2.0));
            assert!(mv.update_double(|x| x + 0.5));
            assert_eq!(mv.as_double(), Some(2.5));
        }

        // Byte array mutation via as_byte_array_mut
        if let Some(mut mv) = c.get_mut("ba") {
            if let na_nbt::MutableValue::ByteArray(ref mut bv) = mv {
                let s = bv.as_mut_slice();
                if !s.is_empty() {
                    s[0] = 99i8;
                }
            }
        }

        // String mutation via as_string_mut
        if let Some(mut mv) = c.get_mut("s") {
            if let na_nbt::MutableValue::String(ref mut sv) = mv {
                sv.push_str("baz");
            }
        }

        // Verify changes
        assert_eq!(c.get("sh").unwrap().as_short(), Some(320));
        assert_eq!(c.get("f").unwrap().as_float().unwrap(), 2.0);
        assert_eq!(c.get("d").unwrap().as_double().unwrap(), 2.5);
        assert!(c.get("s").unwrap().as_string().unwrap().decode().to_string().contains("baz"));
    } else {
        panic!("expected compound");
    }
}
