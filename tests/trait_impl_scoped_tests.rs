use na_nbt::{read_borrowed, read_owned, OwnedValue, ScopedReadableValue, ScopedWritableValue, ValueScoped};
use zerocopy::byteorder::BigEndian as BE;

fn create_all_primitives_be() -> Vec<u8> {
    // Root compound with all primitive types
    let mut data = vec![0x0A, 0x00, 0x00];
    // byte b = 1
    data.push(0x01);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'b');
    data.push(1u8);
    // short s = 2
    data.push(0x02);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b's');
    data.extend_from_slice(&2i16.to_be_bytes());
    // int i = 3
    data.push(0x03);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'i');
    data.extend_from_slice(&3i32.to_be_bytes());
    // long l = 4
    data.push(0x04);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'l');
    data.extend_from_slice(&4i64.to_be_bytes());
    // float f = 5.0
    data.push(0x05);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'f');
    data.extend_from_slice(&5.0f32.to_be_bytes());
    // double d = 6.0
    data.push(0x06);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'd');
    data.extend_from_slice(&6.0f64.to_be_bytes());
    data.push(0x00); // end
    data
}

fn create_arrays_be() -> Vec<u8> {
    // Root compound with array types
    let mut data = vec![0x0A, 0x00, 0x00];
    // ByteArray ba = [1, 2]
    data.push(0x07);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"ba");
    data.extend_from_slice(&2u32.to_be_bytes());
    data.push(1u8);
    data.push(2u8);
    // IntArray ia = [3, 4]
    data.push(0x0B);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"ia");
    data.extend_from_slice(&2u32.to_be_bytes());
    data.extend_from_slice(&3i32.to_be_bytes());
    data.extend_from_slice(&4i32.to_be_bytes());
    // LongArray la = [5, 6]
    data.push(0x0C);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"la");
    data.extend_from_slice(&2u32.to_be_bytes());
    data.extend_from_slice(&5i64.to_be_bytes());
    data.extend_from_slice(&6i64.to_be_bytes());
    data.push(0x00); // end
    data
}

#[test]
fn scoped_readable_primitives_immutable() {
    let data = create_all_primitives_be();
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let comp = root.as_compound().unwrap();

    // Use ScopedReadableValue trait methods
    let b_val = comp.get("b").unwrap();
    assert_eq!(ScopedReadableValue::as_byte(&b_val), Some(1));
    assert!(ScopedReadableValue::is_byte(&b_val));

    let s_val = comp.get("s").unwrap();
    assert_eq!(ScopedReadableValue::as_short(&s_val), Some(2));
    assert!(ScopedReadableValue::is_short(&s_val));

    let i_val = comp.get("i").unwrap();
    assert_eq!(ScopedReadableValue::as_int(&i_val), Some(3));
    assert!(ScopedReadableValue::is_int(&i_val));

    let l_val = comp.get("l").unwrap();
    assert_eq!(ScopedReadableValue::as_long(&l_val), Some(4));
    assert!(ScopedReadableValue::is_long(&l_val));

    let f_val = comp.get("f").unwrap();
    assert_eq!(ScopedReadableValue::as_float(&f_val), Some(5.0));
    assert!(ScopedReadableValue::is_float(&f_val));

    let d_val = comp.get("d").unwrap();
    assert_eq!(ScopedReadableValue::as_double(&d_val), Some(6.0));
    assert!(ScopedReadableValue::is_double(&d_val));
}

#[test]
fn scoped_readable_arrays_immutable() {
    let data = create_arrays_be();
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let comp = root.as_compound().unwrap();

    let ba = comp.get("ba").unwrap();
    assert!(ScopedReadableValue::is_byte_array(&ba));
    assert_eq!(ScopedReadableValue::as_byte_array(&ba).unwrap(), &[1i8, 2i8]);

    let ia = comp.get("ia").unwrap();
    assert!(ScopedReadableValue::is_int_array(&ia));
    let ia_slice = ScopedReadableValue::as_int_array(&ia).unwrap();
    assert_eq!(ia_slice.len(), 2);
    assert_eq!(ia_slice[0].get(), 3);
    assert_eq!(ia_slice[1].get(), 4);

    let la = comp.get("la").unwrap();
    assert!(ScopedReadableValue::is_long_array(&la));
    let la_slice = ScopedReadableValue::as_long_array(&la).unwrap();
    assert_eq!(la_slice.len(), 2);
    assert_eq!(la_slice[0].get(), 5);
    assert_eq!(la_slice[1].get(), 6);
}

#[test]
fn scoped_writable_primitives_mutable() {
    let data = create_all_primitives_be();
    let owned = read_owned::<BE, BE>(&data).unwrap();

    if let OwnedValue::Compound(mut comp) = owned {
        // byte
        if let Some(mut mv) = comp.get_mut("b") {
            assert!(ScopedWritableValue::set_byte(&mut mv, 10));
            assert_eq!(ScopedReadableValue::as_byte(&mv), Some(10));
            assert!(ScopedWritableValue::update_byte(&mut mv, |x| x + 1));
            assert_eq!(ScopedReadableValue::as_byte(&mv), Some(11));
        }

        // short
        if let Some(mut mv) = comp.get_mut("s") {
            assert!(ScopedWritableValue::set_short(&mut mv, 20));
            assert_eq!(ScopedReadableValue::as_short(&mv), Some(20));
            assert!(ScopedWritableValue::update_short(&mut mv, |x| x + 1));
            assert_eq!(ScopedReadableValue::as_short(&mv), Some(21));
        }

        // int
        if let Some(mut mv) = comp.get_mut("i") {
            assert!(ScopedWritableValue::set_int(&mut mv, 30));
            assert_eq!(ScopedReadableValue::as_int(&mv), Some(30));
            assert!(ScopedWritableValue::update_int(&mut mv, |x| x + 1));
            assert_eq!(ScopedReadableValue::as_int(&mv), Some(31));
        }

        // long
        if let Some(mut mv) = comp.get_mut("l") {
            assert!(ScopedWritableValue::set_long(&mut mv, 40));
            assert_eq!(ScopedReadableValue::as_long(&mv), Some(40));
            assert!(ScopedWritableValue::update_long(&mut mv, |x| x + 1));
            assert_eq!(ScopedReadableValue::as_long(&mv), Some(41));
        }

        // float
        if let Some(mut mv) = comp.get_mut("f") {
            assert!(ScopedWritableValue::set_float(&mut mv, 50.0));
            assert_eq!(ScopedReadableValue::as_float(&mv), Some(50.0));
            assert!(ScopedWritableValue::update_float(&mut mv, |x| x + 1.0));
            assert_eq!(ScopedReadableValue::as_float(&mv), Some(51.0));
        }

        // double
        if let Some(mut mv) = comp.get_mut("d") {
            assert!(ScopedWritableValue::set_double(&mut mv, 60.0));
            assert_eq!(ScopedReadableValue::as_double(&mv), Some(60.0));
            assert!(ScopedWritableValue::update_double(&mut mv, |x| x + 1.0));
            assert_eq!(ScopedReadableValue::as_double(&mv), Some(61.0));
        }
    } else {
        panic!("expected compound");
    }
}

#[test]
fn scoped_readable_visit_scoped() {
    let data = create_all_primitives_be();
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let comp = root.as_compound().unwrap();

    let i_val = comp.get("i").unwrap();
    let result = ScopedReadableValue::visit_scoped(&i_val, |v| match v {
        ValueScoped::Int(x) => x,
        _ => panic!("expected int"),
    });
    assert_eq!(result, 3);
}

#[test]
fn scoped_writable_mut_references() {
    let data = create_all_primitives_be();
    let owned = read_owned::<BE, BE>(&data).unwrap();

    if let OwnedValue::Compound(mut comp) = owned {
        // as_byte_mut
        if let Some(mut mv) = comp.get_mut("b") {
            if let Some(byte_ref) = ScopedWritableValue::as_byte_mut(&mut mv) {
                *byte_ref = 99;
            }
            assert_eq!(ScopedReadableValue::as_byte(&mv), Some(99));
        }

        // as_short_mut
        if let Some(mut mv) = comp.get_mut("s") {
            if let Some(short_ref) = ScopedWritableValue::as_short_mut(&mut mv) {
                short_ref.set(999);
            }
            assert_eq!(ScopedReadableValue::as_short(&mv), Some(999));
        }

        // as_int_mut
        if let Some(mut mv) = comp.get_mut("i") {
            if let Some(int_ref) = ScopedWritableValue::as_int_mut(&mut mv) {
                int_ref.set(9999);
            }
            assert_eq!(ScopedReadableValue::as_int(&mv), Some(9999));
        }

        // as_long_mut
        if let Some(mut mv) = comp.get_mut("l") {
            if let Some(long_ref) = ScopedWritableValue::as_long_mut(&mut mv) {
                long_ref.set(99999);
            }
            assert_eq!(ScopedReadableValue::as_long(&mv), Some(99999));
        }

        // as_float_mut
        if let Some(mut mv) = comp.get_mut("f") {
            if let Some(float_ref) = ScopedWritableValue::as_float_mut(&mut mv) {
                float_ref.set(9.9);
            }
            assert_eq!(ScopedReadableValue::as_float(&mv), Some(9.9));
        }

        // as_double_mut
        if let Some(mut mv) = comp.get_mut("d") {
            if let Some(double_ref) = ScopedWritableValue::as_double_mut(&mut mv) {
                double_ref.set(99.9);
            }
            assert_eq!(ScopedReadableValue::as_double(&mv), Some(99.9));
        }
    } else {
        panic!("expected compound");
    }
}

#[test]
fn scoped_writable_type_mismatch_returns_false() {
    let data = create_all_primitives_be();
    let owned = read_owned::<BE, BE>(&data).unwrap();

    if let OwnedValue::Compound(mut comp) = owned {
        // Try to set byte on an int value - should return false
        if let Some(mut mv) = comp.get_mut("i") {
            assert!(!ScopedWritableValue::set_byte(&mut mv, 1));
            assert!(!ScopedWritableValue::update_byte(&mut mv, |x| x + 1));
        }

        // Try to set int on a byte value - should return false
        if let Some(mut mv) = comp.get_mut("b") {
            assert!(!ScopedWritableValue::set_int(&mut mv, 1));
            assert!(!ScopedWritableValue::update_int(&mut mv, |x| x + 1));
        }
    } else {
        panic!("expected compound");
    }
}
