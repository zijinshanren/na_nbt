use na_nbt::{
    read_owned, OwnedValue, ScopedReadableValue, ScopedWritableValue, ValueScoped, ValueMutScoped,
    OwnedList, OwnedCompound, read_borrowed, ReadableValue, Value,
};
use zerocopy::byteorder::BigEndian as BE;
use zerocopy::byteorder::{I32, I64};

#[test]
fn test_visit_scoped_mutable_value() {
    let mut data = vec![0x0A, 0x00, 0x00];
    // Int 'i' = 10
    data.push(0x03); data.extend_from_slice(&1u16.to_be_bytes()); data.push(b'i'); data.extend_from_slice(&10i32.to_be_bytes());
    // End
    data.push(0x00);

    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    let mut comp = owned.as_compound_mut().unwrap();
    let mut mv = comp.get_mut("i").unwrap();

    // Test visit_scoped on MutableValue
    let res = ScopedReadableValue::visit_scoped(&mv, |v| match v {
        ValueScoped::Int(i) => i,
        _ => -1,
    });
    assert_eq!(res, 10);

    // Test visit_mut_scoped on MutableValue
    ScopedWritableValue::visit_mut_scoped(&mut mv, |v| match v {
        ValueMutScoped::Int(val) => { *val = I32::new(20); },
        _ => panic!("expected int"),
    });
    
    assert_eq!(mv.as_int(), Some(20));
}

#[test]
fn test_visit_scoped_owned_value() {
    let mut owned = OwnedValue::<BE>::from(10i32);
    
    // ScopedReadableValue::visit_scoped for OwnedValue
    let res = ScopedReadableValue::visit_scoped(&owned, |v| match v {
        ValueScoped::Int(i) => i,
        _ => -1,
    });
    assert_eq!(res, 10);

    // ScopedWritableValue::visit_mut for OwnedValue
    ScopedWritableValue::visit_mut_scoped(&mut owned, |v| match v {
        ValueMutScoped::Int(val) => { *val = I32::new(20); },
        _ => panic!("expected int"),
    });
    
    assert_eq!(owned.as_int(), Some(20));
}

#[test]
fn test_visit_all_types_mutable() {
    // Create compound with all types
    let mut c = OwnedCompound::<BE>::default();
    c.insert("b", 1i8);
    c.insert("s", 2i16);
    c.insert("i", 3i32);
    c.insert("l", 4i64);
    c.insert("f", 5.0f32);
    c.insert("d", 6.0f64);
    c.insert("ba", vec![7i8]);
    c.insert("st", "str");
    c.insert("li", OwnedList::<BE>::default());
    c.insert("co", OwnedCompound::<BE>::default());
    c.insert("ia", vec![I32::new(8)]);
    c.insert("la", vec![I64::new(9)]);
    
    // We can iterate the compound mutably
    for (_, mut v) in c.iter_mut() {
        ScopedWritableValue::visit_mut_scoped(&mut v, |val| match val {
            ValueMutScoped::Byte(x) => *x += 1,
            ValueMutScoped::Short(x) => x.set(x.get() + 1),
            ValueMutScoped::Int(x) => x.set(x.get() + 1),
            ValueMutScoped::Long(x) => x.set(x.get() + 1),
            ValueMutScoped::Float(x) => x.set(x.get() + 1.0),
            ValueMutScoped::Double(x) => x.set(x.get() + 1.0),
            ValueMutScoped::ByteArray(mut x) => x[0] += 1,
            ValueMutScoped::String(mut x) => x.push_str("ing"),
            ValueMutScoped::List(mut x) => x.push(10i8),
            ValueMutScoped::Compound(mut x) => { x.insert("new", 11i8); },
            ValueMutScoped::IntArray(mut x) => { let val = x[0].get(); x[0].set(val + 1); },
            ValueMutScoped::LongArray(mut x) => { let val = x[0].get(); x[0].set(val + 1); },
            ValueMutScoped::End => {},
        });
    }
    
    // Verify
    assert_eq!(c.get("b").unwrap().as_byte(), Some(2));
    assert_eq!(c.get("s").unwrap().as_short(), Some(3));
    assert_eq!(c.get("i").unwrap().as_int(), Some(4));
    assert_eq!(c.get("l").unwrap().as_long(), Some(5));
    assert_eq!(c.get("f").unwrap().as_float(), Some(6.0));
    assert_eq!(c.get("d").unwrap().as_double(), Some(7.0));
    assert_eq!(c.get("ba").unwrap().as_byte_array().unwrap()[0], 8);
    assert_eq!(c.get("st").unwrap().as_string().unwrap().decode(), "string");
    assert_eq!(c.get("li").unwrap().as_list().unwrap().len(), 1);
    assert!(c.get("co").unwrap().as_compound().unwrap().get("new").is_some());
    assert_eq!(c.get("ia").unwrap().as_int_array().unwrap()[0].get(), 9);
    assert_eq!(c.get("la").unwrap().as_long_array().unwrap()[0].get(), 10);
}

#[test]
fn test_visit_all_types_owned() {
    // Create compound with all types
    let mut c = OwnedCompound::<BE>::default();
    c.insert("b", 1i8);
    c.insert("s", 2i16);
    c.insert("i", 3i32);
    c.insert("l", 4i64);
    c.insert("f", 5.0f32);
    c.insert("d", 6.0f64);
    c.insert("ba", vec![7i8]);
    c.insert("st", "str");
    c.insert("li", OwnedList::<BE>::default());
    c.insert("co", OwnedCompound::<BE>::default());
    c.insert("ia", vec![I32::new(8)]);
    c.insert("la", vec![I64::new(9)]);

    // We can iterate the compound mutably, getting MutableValues.
    // To test OwnedValue::visit_mut_scoped, we need actual OwnedValues.
    // We can remove them from compound (getting OwnedValue), visit them, and insert back?
    // Or just create them standalone.
    
    let mut v = OwnedValue::<BE>::from(1i8);
    ScopedWritableValue::visit_mut_scoped(&mut v, |val| if let ValueMutScoped::Byte(x) = val { *x += 1 });
    assert_eq!(v.as_byte(), Some(2));
    
    let mut v = OwnedValue::<BE>::from(2i16);
    ScopedWritableValue::visit_mut_scoped(&mut v, |val| if let ValueMutScoped::Short(x) = val { x.set(x.get() + 1) });
    assert_eq!(v.as_short(), Some(3));

    let mut v = OwnedValue::<BE>::from(3i32);
    ScopedWritableValue::visit_mut_scoped(&mut v, |val| if let ValueMutScoped::Int(x) = val { x.set(x.get() + 1) });
    assert_eq!(v.as_int(), Some(4));

    let mut v = OwnedValue::<BE>::from(4i64);
    ScopedWritableValue::visit_mut_scoped(&mut v, |val| if let ValueMutScoped::Long(x) = val { x.set(x.get() + 1) });
    assert_eq!(v.as_long(), Some(5));

    let mut v = OwnedValue::<BE>::from(5.0f32);
    ScopedWritableValue::visit_mut_scoped(&mut v, |val| if let ValueMutScoped::Float(x) = val { x.set(x.get() + 1.0) });
    assert_eq!(v.as_float(), Some(6.0));

    let mut v = OwnedValue::<BE>::from(6.0f64);
    ScopedWritableValue::visit_mut_scoped(&mut v, |val| if let ValueMutScoped::Double(x) = val { x.set(x.get() + 1.0) });
    assert_eq!(v.as_double(), Some(7.0));

    let mut v = OwnedValue::<BE>::from(vec![7i8]);
    ScopedWritableValue::visit_mut_scoped(&mut v, |val| if let ValueMutScoped::ByteArray(mut x) = val { x[0] += 1 });
    assert_eq!(v.as_byte_array().unwrap()[0], 8);

    let mut v = OwnedValue::<BE>::from("str");
    ScopedWritableValue::visit_mut_scoped(&mut v, |val| if let ValueMutScoped::String(mut x) = val { x.push_str("ing") });
    assert_eq!(v.as_string().unwrap().decode(), "string");

    let mut v = OwnedValue::<BE>::from(OwnedList::<BE>::default());
    ScopedWritableValue::visit_mut_scoped(&mut v, |val| if let ValueMutScoped::List(mut x) = val { x.push(10i8) });
    assert_eq!(v.as_list().unwrap().len(), 1);

    let mut v = OwnedValue::<BE>::from(OwnedCompound::<BE>::default());
    ScopedWritableValue::visit_mut_scoped(&mut v, |val| if let ValueMutScoped::Compound(mut x) = val { x.insert("new", 11i8); });
    assert!(v.as_compound().unwrap().get("new").is_some());
    
    let mut v = OwnedValue::<BE>::from(vec![I32::new(8)]);
    ScopedWritableValue::visit_mut_scoped(&mut v, |val| if let ValueMutScoped::IntArray(mut x) = val { let val = x[0].get(); x[0].set(val + 1); });
    assert_eq!(v.as_int_array().unwrap()[0].get(), 9);

    let mut v = OwnedValue::<BE>::from(vec![I64::new(9)]);
    ScopedWritableValue::visit_mut_scoped(&mut v, |val| if let ValueMutScoped::LongArray(mut x) = val { let val = x[0].get(); x[0].set(val + 1); });
    assert_eq!(v.as_long_array().unwrap()[0].get(), 10);
}

#[test]
fn test_visit_immutable_value() {
    let mut data = vec![0x0A, 0x00, 0x00];
    // Int 'i' = 10
    data.push(0x03); data.extend_from_slice(&1u16.to_be_bytes()); data.push(b'i'); data.extend_from_slice(&10i32.to_be_bytes());
    // End
    data.push(0x00);

    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let comp = root.as_compound().unwrap();
    let val = comp.get("i").unwrap();

    // ScopedReadableValue::visit_scoped
    let res = ScopedReadableValue::visit_scoped(&val, |v| match v {
        ValueScoped::Int(i) => i,
        _ => -1,
    });
    assert_eq!(res, 10);

    // ReadableValue::visit
    let res = ReadableValue::visit(&val, |v| match v {
        Value::Int(i) => i,
        _ => -1,
    });
    assert_eq!(res, 10);
}
