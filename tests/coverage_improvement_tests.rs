//! Tests to improve coverage for visit, visit_scoped, LittleEndian, SharedDocument, and Writable traits

use bytes::Bytes;
use na_nbt::{
    OwnedCompound, OwnedList, OwnedValue, ReadableCompound, ReadableList, ReadableValue,
    ScopedReadableCompound, ScopedReadableList, ScopedReadableValue, ScopedWritableCompound,
    ScopedWritableList, ScopedWritableValue, Tag, Value, ValueScoped, WritableValue, read_borrowed,
    read_owned, read_shared, write_value_to_vec, write_value_to_writer,
};
use zerocopy::byteorder::{BigEndian as BE, I32, I64, LittleEndian as LE};

// ============ Visit and visit_scoped tests for various value types ============

#[test]
fn test_visit_end_borrowed() {
    let data = vec![0x00]; // End tag
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let res = ReadableValue::visit(&root, |v| matches!(v, Value::End));
    assert!(res);
}

#[test]
fn test_visit_short_borrowed() {
    let mut data = vec![0x02, 0x00, 0x00]; // Short tag
    data.extend_from_slice(&1234i16.to_be_bytes());
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let res = ReadableValue::visit(&root, |v| if let Value::Short(s) = v { s } else { 0 });
    assert_eq!(res, 1234);
}

#[test]
fn test_visit_long_borrowed() {
    let mut data = vec![0x04, 0x00, 0x00]; // Long tag
    data.extend_from_slice(&12345678i64.to_be_bytes());
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let res = ReadableValue::visit(&root, |v| if let Value::Long(l) = v { l } else { 0 });
    assert_eq!(res, 12345678);
}

#[test]
fn test_visit_float_borrowed() {
    let mut data = vec![0x05, 0x00, 0x00]; // Float tag
    data.extend_from_slice(&std::f32::consts::PI.to_be_bytes());
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let res = ReadableValue::visit(&root, |v| if let Value::Float(f) = v { f } else { 0.0 });
    assert!((res - std::f32::consts::PI).abs() < 0.01);
}

#[test]
fn test_visit_double_borrowed() {
    let mut data = vec![0x06, 0x00, 0x00]; // Double tag
    data.extend_from_slice(&std::f64::consts::E.to_be_bytes());
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let res = ReadableValue::visit(&root, |v| if let Value::Double(d) = v { d } else { 0.0 });
    assert!((res - std::f64::consts::E).abs() < 0.001);
}

#[test]
fn test_visit_byte_array_borrowed() {
    let mut data = vec![0x07, 0x00, 0x00]; // ByteArray tag
    data.extend_from_slice(&3u32.to_be_bytes()); // length
    data.extend_from_slice(&[1, 2, 3]);
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let res = ReadableValue::visit(&root, |v| {
        if let Value::ByteArray(arr) = v {
            arr.len()
        } else {
            0
        }
    });
    assert_eq!(res, 3);
}

#[test]
fn test_visit_int_array_borrowed() {
    let mut data = vec![0x0B, 0x00, 0x00]; // IntArray tag
    data.extend_from_slice(&2u32.to_be_bytes());
    data.extend_from_slice(&100i32.to_be_bytes());
    data.extend_from_slice(&200i32.to_be_bytes());
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let res = ReadableValue::visit(&root, |v| {
        if let Value::IntArray(arr) = v {
            arr.len()
        } else {
            0
        }
    });
    assert_eq!(res, 2);
}

#[test]
fn test_visit_long_array_borrowed() {
    let mut data = vec![0x0C, 0x00, 0x00]; // LongArray tag
    data.extend_from_slice(&1u32.to_be_bytes());
    data.extend_from_slice(&42i64.to_be_bytes());
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let res = ReadableValue::visit(&root, |v| {
        if let Value::LongArray(arr) = v {
            arr[0].get()
        } else {
            0
        }
    });
    assert_eq!(res, 42);
}

#[test]
fn test_visit_string_borrowed() {
    let mut data = vec![0x08, 0x00, 0x00]; // String tag
    data.extend_from_slice(&5u16.to_be_bytes());
    data.extend_from_slice(b"hello");
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let res = ReadableValue::visit(&root, |v| {
        if let Value::String(s) = v {
            s.decode().len()
        } else {
            0
        }
    });
    assert_eq!(res, 5);
}

#[test]
fn test_visit_list_borrowed() {
    let mut data = vec![0x09, 0x00, 0x00]; // List tag
    data.push(0x01); // Byte list
    data.extend_from_slice(&2u32.to_be_bytes()); // count
    data.extend_from_slice(&[10, 20]);
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let res = ReadableValue::visit(&root, |v| if let Value::List(l) = v { l.len() } else { 0 });
    assert_eq!(res, 2);
}

#[test]
fn test_visit_compound_borrowed() {
    let mut data = vec![0x0A, 0x00, 0x00]; // Compound tag
    // Byte "x" = 5
    data.push(0x01);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'x');
    data.push(5);
    data.push(0x00); // End
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let res = ReadableValue::visit(&root, |v| {
        if let Value::Compound(c) = v {
            c.get("x").is_some()
        } else {
            false
        }
    });
    assert!(res);
}

// ============ visit_scoped tests ============

#[test]
fn test_visit_scoped_short() {
    let mut data = vec![0x02, 0x00, 0x00];
    data.extend_from_slice(&1234i16.to_be_bytes());
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let res =
        ScopedReadableValue::visit_scoped(
            &root,
            |v| {
                if let ValueScoped::Short(s) = v { s } else { 0 }
            },
        );
    assert_eq!(res, 1234);
}

#[test]
fn test_visit_scoped_long() {
    let mut data = vec![0x04, 0x00, 0x00];
    data.extend_from_slice(&999999i64.to_be_bytes());
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let res =
        ScopedReadableValue::visit_scoped(
            &root,
            |v| {
                if let ValueScoped::Long(l) = v { l } else { 0 }
            },
        );
    assert_eq!(res, 999999);
}

#[test]
fn test_visit_scoped_float() {
    let mut data = vec![0x05, 0x00, 0x00];
    data.extend_from_slice(&1.5f32.to_be_bytes());
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let res = ScopedReadableValue::visit_scoped(&root, |v| {
        if let ValueScoped::Float(f) = v {
            f
        } else {
            0.0
        }
    });
    assert_eq!(res, 1.5);
}

#[test]
fn test_visit_scoped_double() {
    let mut data = vec![0x06, 0x00, 0x00];
    data.extend_from_slice(&2.5f64.to_be_bytes());
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let res = ScopedReadableValue::visit_scoped(&root, |v| {
        if let ValueScoped::Double(d) = v {
            d
        } else {
            0.0
        }
    });
    assert_eq!(res, 2.5);
}

#[test]
fn test_visit_scoped_byte_array() {
    let mut data = vec![0x07, 0x00, 0x00];
    data.extend_from_slice(&2u32.to_be_bytes());
    data.extend_from_slice(&[10, 20]);
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let res = ScopedReadableValue::visit_scoped(&root, |v| {
        if let ValueScoped::ByteArray(arr) = v {
            arr.len()
        } else {
            0
        }
    });
    assert_eq!(res, 2);
}

#[test]
fn test_visit_scoped_int_array() {
    let mut data = vec![0x0B, 0x00, 0x00];
    data.extend_from_slice(&1u32.to_be_bytes());
    data.extend_from_slice(&42i32.to_be_bytes());
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let res = ScopedReadableValue::visit_scoped(&root, |v| {
        if let ValueScoped::IntArray(arr) = v {
            arr[0].get()
        } else {
            0
        }
    });
    assert_eq!(res, 42);
}

#[test]
fn test_visit_scoped_long_array() {
    let mut data = vec![0x0C, 0x00, 0x00];
    data.extend_from_slice(&1u32.to_be_bytes());
    data.extend_from_slice(&123i64.to_be_bytes());
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let res = ScopedReadableValue::visit_scoped(&root, |v| {
        if let ValueScoped::LongArray(arr) = v {
            arr[0].get()
        } else {
            0
        }
    });
    assert_eq!(res, 123);
}

#[test]
fn test_visit_scoped_string() {
    let mut data = vec![0x08, 0x00, 0x00];
    data.extend_from_slice(&4u16.to_be_bytes());
    data.extend_from_slice(b"test");
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let res = ScopedReadableValue::visit_scoped(&root, |v| {
        if let ValueScoped::String(s) = v {
            s.decode().len()
        } else {
            0
        }
    });
    assert_eq!(res, 4);
}

#[test]
fn test_visit_scoped_list() {
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x01); // Byte list
    data.extend_from_slice(&3u32.to_be_bytes());
    data.extend_from_slice(&[1, 2, 3]);
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let res = ScopedReadableValue::visit_scoped(&root, |v| {
        if let ValueScoped::List(l) = v {
            l.len()
        } else {
            0
        }
    });
    assert_eq!(res, 3);
}

#[test]
fn test_visit_scoped_compound() {
    let mut data = vec![0x0A, 0x00, 0x00]; // Compound tag
    data.push(0x00); // End
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let res = ScopedReadableValue::visit_scoped(&root, |v| matches!(v, ValueScoped::Compound(_)));
    assert!(res);
}

// ============ LittleEndian tests ============

#[test]
fn test_read_borrowed_le_int() {
    let mut data = vec![0x03, 0x00, 0x00]; // Int tag, empty name (LE)
    data.extend_from_slice(&42i32.to_le_bytes());
    let doc = read_borrowed::<LE>(&data).unwrap();
    let root = doc.root();
    assert_eq!(root.as_int(), Some(42));
}

#[test]
fn test_read_borrowed_le_short() {
    let mut data = vec![0x02, 0x00, 0x00];
    data.extend_from_slice(&1234i16.to_le_bytes());
    let doc = read_borrowed::<LE>(&data).unwrap();
    let root = doc.root();
    assert_eq!(root.as_short(), Some(1234));
}

#[test]
fn test_read_borrowed_le_long() {
    let mut data = vec![0x04, 0x00, 0x00];
    data.extend_from_slice(&9999i64.to_le_bytes());
    let doc = read_borrowed::<LE>(&data).unwrap();
    let root = doc.root();
    assert_eq!(root.as_long(), Some(9999));
}

#[test]
fn test_read_borrowed_le_float() {
    let mut data = vec![0x05, 0x00, 0x00];
    data.extend_from_slice(&1.5f32.to_le_bytes());
    let doc = read_borrowed::<LE>(&data).unwrap();
    let root = doc.root();
    assert_eq!(root.as_float(), Some(1.5));
}

#[test]
fn test_read_borrowed_le_double() {
    let mut data = vec![0x06, 0x00, 0x00];
    data.extend_from_slice(&2.5f64.to_le_bytes());
    let doc = read_borrowed::<LE>(&data).unwrap();
    let root = doc.root();
    assert_eq!(root.as_double(), Some(2.5));
}

#[test]
fn test_read_borrowed_le_string() {
    let mut data = vec![0x08, 0x00, 0x00];
    data.extend_from_slice(&4u16.to_le_bytes());
    data.extend_from_slice(b"test");
    let doc = read_borrowed::<LE>(&data).unwrap();
    let root = doc.root();
    assert_eq!(root.as_string().unwrap().decode(), "test");
}

#[test]
fn test_read_borrowed_le_byte_array() {
    let mut data = vec![0x07, 0x00, 0x00];
    data.extend_from_slice(&3u32.to_le_bytes());
    data.extend_from_slice(&[1, 2, 3]);
    let doc = read_borrowed::<LE>(&data).unwrap();
    let root = doc.root();
    assert_eq!(root.as_byte_array().unwrap().len(), 3);
}

#[test]
fn test_read_borrowed_le_int_array() {
    let mut data = vec![0x0B, 0x00, 0x00];
    data.extend_from_slice(&2u32.to_le_bytes());
    data.extend_from_slice(&100i32.to_le_bytes());
    data.extend_from_slice(&200i32.to_le_bytes());
    let doc = read_borrowed::<LE>(&data).unwrap();
    let root = doc.root();
    let arr = root.as_int_array().unwrap();
    assert_eq!(arr[0].get(), 100);
    assert_eq!(arr[1].get(), 200);
}

#[test]
fn test_read_borrowed_le_long_array() {
    let mut data = vec![0x0C, 0x00, 0x00];
    data.extend_from_slice(&1u32.to_le_bytes());
    data.extend_from_slice(&42i64.to_le_bytes());
    let doc = read_borrowed::<LE>(&data).unwrap();
    let root = doc.root();
    assert_eq!(root.as_long_array().unwrap()[0].get(), 42);
}

#[test]
fn test_read_borrowed_le_list() {
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x03); // Int list
    data.extend_from_slice(&2u32.to_le_bytes());
    data.extend_from_slice(&10i32.to_le_bytes());
    data.extend_from_slice(&20i32.to_le_bytes());
    let doc = read_borrowed::<LE>(&data).unwrap();
    let root = doc.root();
    let list = root.as_list().unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list.get(0).unwrap().as_int(), Some(10));
}

#[test]
fn test_read_borrowed_le_compound() {
    let mut data = vec![0x0A, 0x00, 0x00]; // Compound
    // Int "val" = 42 (LE)
    data.push(0x03);
    data.extend_from_slice(&3u16.to_le_bytes());
    data.extend_from_slice(b"val");
    data.extend_from_slice(&42i32.to_le_bytes());
    data.push(0x00);
    let doc = read_borrowed::<LE>(&data).unwrap();
    let root = doc.root();
    assert_eq!(
        root.as_compound().unwrap().get("val").unwrap().as_int(),
        Some(42)
    );
}

#[test]
fn test_read_owned_le() {
    let mut data = vec![0x03, 0x00, 0x00];
    data.extend_from_slice(&42i32.to_le_bytes());
    let owned = read_owned::<LE, LE>(&data).unwrap();
    assert_eq!(owned.as_int(), Some(42));
}

#[test]
fn test_write_le_to_be() {
    let mut data = vec![0x03, 0x00, 0x00];
    data.extend_from_slice(&0x12345678i32.to_le_bytes());
    let doc = read_borrowed::<LE>(&data).unwrap();
    let root = doc.root();
    let written = write_value_to_vec::<_, LE, BE>(&root).unwrap();
    let doc_be = read_borrowed::<BE>(&written).unwrap();
    assert_eq!(doc_be.root().as_int(), Some(0x12345678));
}

#[test]
fn test_write_be_to_le_int_array() {
    let mut data = vec![0x0B, 0x00, 0x00];
    data.extend_from_slice(&2u32.to_be_bytes());
    data.extend_from_slice(&0x11223344i32.to_be_bytes());
    data.extend_from_slice(&0x55667788i32.to_be_bytes());
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let written = write_value_to_vec::<_, BE, LE>(&root).unwrap();
    let doc_le = read_borrowed::<LE>(&written).unwrap();
    let root_le = doc_le.root();
    let arr = root_le.as_int_array().unwrap();
    assert_eq!(arr[0].get(), 0x11223344);
    assert_eq!(arr[1].get(), 0x55667788);
}

#[test]
fn test_write_be_to_le_long_array() {
    let mut data = vec![0x0C, 0x00, 0x00];
    data.extend_from_slice(&1u32.to_be_bytes());
    data.extend_from_slice(&0x1122334455667788i64.to_be_bytes());
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let written = write_value_to_vec::<_, BE, LE>(&root).unwrap();
    let doc_le = read_borrowed::<LE>(&written).unwrap();
    let root_le = doc_le.root();
    let arr = root_le.as_long_array().unwrap();
    assert_eq!(arr[0].get(), 0x1122334455667788);
}

// ============ SharedDocument tests ============

#[test]
fn test_shared_document_root_end_tag() {
    let data = vec![0x00]; // End tag root
    let bytes = Bytes::from(data);
    let root = read_shared::<BE>(bytes).unwrap();
    assert!(root.is_end());
}

#[test]
fn test_shared_document_int() {
    let mut data = vec![0x03, 0x00, 0x00];
    data.extend_from_slice(&42i32.to_be_bytes());
    let bytes = Bytes::from(data);
    let root = read_shared::<BE>(bytes).unwrap();
    assert_eq!(root.as_int(), Some(42));
}

#[test]
fn test_shared_document_compound() {
    let mut data = vec![0x0A, 0x00, 0x00]; // Compound
    // Int "val" = 42
    data.push(0x03);
    data.extend_from_slice(&3u16.to_be_bytes());
    data.extend_from_slice(b"val");
    data.extend_from_slice(&42i32.to_be_bytes());
    data.push(0x00);
    let bytes = Bytes::from(data);
    let root = read_shared::<BE>(bytes).unwrap();
    if let Some(c) = root.as_compound() {
        assert_eq!(c.get("val").unwrap().as_int(), Some(42));
    } else {
        panic!("Expected compound");
    }
}

#[test]
fn test_shared_document_le() {
    let mut data = vec![0x0A, 0x00, 0x00]; // Compound
    // Int "val" = 42 (LE)
    data.push(0x03);
    data.extend_from_slice(&3u16.to_le_bytes());
    data.extend_from_slice(b"val");
    data.extend_from_slice(&42i32.to_le_bytes());
    data.push(0x00);
    let bytes = Bytes::from(data);
    let root = read_shared::<LE>(bytes).unwrap();
    if let Some(c) = root.as_compound() {
        assert_eq!(c.get("val").unwrap().as_int(), Some(42));
    } else {
        panic!("Expected compound");
    }
}

// ============ write_value_to_writer tests ============

#[test]
fn test_write_to_writer_end() {
    let data = vec![0x00];
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let mut cursor = std::io::Cursor::new(Vec::new());
    write_value_to_writer::<_, BE, BE, _>(&mut cursor, &root).unwrap();
    assert_eq!(cursor.into_inner(), vec![0x00]);
}

#[test]
fn test_write_to_writer_byte() {
    let data = vec![0x01, 0x00, 0x00, 42];
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let mut cursor = std::io::Cursor::new(Vec::new());
    write_value_to_writer::<_, BE, BE, _>(&mut cursor, &root).unwrap();
    let written = cursor.into_inner();
    assert_eq!(written[3], 42);
}

#[test]
fn test_write_to_writer_short_be_to_le() {
    let mut data = vec![0x02, 0x00, 0x00];
    data.extend_from_slice(&1234i16.to_be_bytes());
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let mut cursor = std::io::Cursor::new(Vec::new());
    write_value_to_writer::<_, BE, LE, _>(&mut cursor, &root).unwrap();
    let written = cursor.into_inner();
    let doc_le = read_borrowed::<LE>(&written).unwrap();
    assert_eq!(doc_le.root().as_short(), Some(1234));
}

#[test]
fn test_write_to_writer_list_be_to_le() {
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x03); // Int list
    data.extend_from_slice(&2u32.to_be_bytes());
    data.extend_from_slice(&100i32.to_be_bytes());
    data.extend_from_slice(&200i32.to_be_bytes());
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let mut cursor = std::io::Cursor::new(Vec::new());
    write_value_to_writer::<_, BE, LE, _>(&mut cursor, &root).unwrap();
    let written = cursor.into_inner();
    let doc_le = read_borrowed::<LE>(&written).unwrap();
    let root_le = doc_le.root();
    let list = root_le.as_list().unwrap();
    assert_eq!(list.get(0).unwrap().as_int(), Some(100));
    assert_eq!(list.get(1).unwrap().as_int(), Some(200));
}

#[test]
fn test_write_to_writer_compound_be_to_le() {
    let mut data = vec![0x0A, 0x00, 0x00]; // Compound
    // Int "x" = 42
    data.push(0x03);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'x');
    data.extend_from_slice(&42i32.to_be_bytes());
    data.push(0x00);
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let mut cursor = std::io::Cursor::new(Vec::new());
    write_value_to_writer::<_, BE, LE, _>(&mut cursor, &root).unwrap();
    let written = cursor.into_inner();
    let doc_le = read_borrowed::<LE>(&written).unwrap();
    assert_eq!(
        doc_le
            .root()
            .as_compound()
            .unwrap()
            .get("x")
            .unwrap()
            .as_int(),
        Some(42)
    );
}

#[test]
fn test_write_to_writer_int_array_be_to_le() {
    let mut data = vec![0x0B, 0x00, 0x00];
    data.extend_from_slice(&2u32.to_be_bytes());
    data.extend_from_slice(&100i32.to_be_bytes());
    data.extend_from_slice(&200i32.to_be_bytes());
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let mut cursor = std::io::Cursor::new(Vec::new());
    write_value_to_writer::<_, BE, LE, _>(&mut cursor, &root).unwrap();
    let written = cursor.into_inner();
    let doc_le = read_borrowed::<LE>(&written).unwrap();
    let root_le = doc_le.root();
    let arr = root_le.as_int_array().unwrap();
    assert_eq!(arr[0].get(), 100);
    assert_eq!(arr[1].get(), 200);
}

#[test]
fn test_write_to_writer_long_array_be_to_le() {
    let mut data = vec![0x0C, 0x00, 0x00];
    data.extend_from_slice(&1u32.to_be_bytes());
    data.extend_from_slice(&999i64.to_be_bytes());
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let mut cursor = std::io::Cursor::new(Vec::new());
    write_value_to_writer::<_, BE, LE, _>(&mut cursor, &root).unwrap();
    let written = cursor.into_inner();
    let doc_le = read_borrowed::<LE>(&written).unwrap();
    assert_eq!(doc_le.root().as_long_array().unwrap()[0].get(), 999);
}

// ============ Empty list tests (End type) ============

#[test]
fn test_empty_list_be() {
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x00); // End type
    data.extend_from_slice(&0u32.to_be_bytes()); // count = 0
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let list = root.as_list().unwrap();
    assert!(list.is_empty());
    assert_eq!(list.tag_id(), Tag::End);
}

#[test]
fn test_write_empty_list() {
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x00); // End type
    data.extend_from_slice(&0u32.to_be_bytes());
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let mut cursor = std::io::Cursor::new(Vec::new());
    write_value_to_writer::<_, BE, LE, _>(&mut cursor, &root).unwrap();
    let written = cursor.into_inner();
    let doc_le = read_borrowed::<LE>(&written).unwrap();
    assert!(doc_le.root().as_list().unwrap().is_empty());
}

// ============ OwnedValue visit tests ============

#[test]
fn test_owned_value_visit_scoped_end() {
    let owned = OwnedValue::<BE>::End;
    let res = ScopedReadableValue::visit_scoped(&owned, |v| matches!(v, ValueScoped::End));
    assert!(res);
}

#[test]
fn test_owned_value_visit_scoped_byte_array() {
    let owned = OwnedValue::<BE>::from(vec![1i8, 2, 3]);
    let res = ScopedReadableValue::visit_scoped(&owned, |v| {
        if let ValueScoped::ByteArray(arr) = v {
            arr.len()
        } else {
            0
        }
    });
    assert_eq!(res, 3);
}

#[test]
fn test_owned_value_visit_scoped_int_array() {
    let owned = OwnedValue::<BE>::from(vec![I32::<BE>::new(1), I32::<BE>::new(2)]);
    let res = ScopedReadableValue::visit_scoped(&owned, |v| {
        if let ValueScoped::IntArray(arr) = v {
            arr.len()
        } else {
            0
        }
    });
    assert_eq!(res, 2);
}

#[test]
fn test_owned_value_visit_scoped_long_array() {
    let owned = OwnedValue::<BE>::from(vec![I64::<BE>::new(42)]);
    let res = ScopedReadableValue::visit_scoped(&owned, |v| {
        if let ValueScoped::LongArray(arr) = v {
            arr[0].get()
        } else {
            0
        }
    });
    assert_eq!(res, 42);
}

// ============ Writable trait tests ============

#[test]
fn test_writable_list_push_via_trait() {
    let mut list = OwnedList::<BE>::default();
    // WritableList is a marker trait; ScopedWritableList has the methods
    <OwnedList<BE> as ScopedWritableList>::push(&mut list, 1i32);
    <OwnedList<BE> as ScopedWritableList>::push(&mut list, 2i32);
    assert_eq!(list.len(), 2);
}

#[test]
fn test_writable_compound_insert_via_trait() {
    let mut comp = OwnedCompound::<BE>::default();
    // WritableCompound is a marker trait; ScopedWritableCompound has the methods
    <OwnedCompound<BE> as ScopedWritableCompound>::insert(&mut comp, "key", 42i32);
    assert_eq!(comp.get("key").unwrap().as_int(), Some(42));
}

#[test]
fn test_scoped_writable_list_push() {
    let mut list = OwnedList::<BE>::default();
    ScopedWritableList::push(&mut list, 10i32);
    ScopedWritableList::push(&mut list, 20i32);
    assert_eq!(ScopedReadableList::len(&list), 2);
    assert!(!ScopedReadableList::is_empty(&list));
    assert_eq!(ScopedReadableList::tag_id(&list), Tag::Int);
}

#[test]
fn test_scoped_writable_compound_insert() {
    let mut comp = OwnedCompound::<BE>::default();
    ScopedWritableCompound::insert(&mut comp, "a", 1i32);
    ScopedWritableCompound::insert(&mut comp, "b", 2i32);
    let v = ScopedReadableCompound::get_scoped(&comp, "a").unwrap();
    assert_eq!(v.as_int(), Some(1));
}

#[test]
fn test_owned_list_iter_scoped() {
    let mut list = OwnedList::<BE>::default();
    list.push(1i32);
    list.push(2i32);
    list.push(3i32);
    let mut sum = 0;
    for v in ScopedReadableList::iter_scoped(&list) {
        sum += v.as_int().unwrap();
    }
    assert_eq!(sum, 6);
}

#[test]
fn test_owned_compound_iter_scoped() {
    let mut comp = OwnedCompound::<BE>::default();
    comp.insert("a", 1i32);
    comp.insert("b", 2i32);
    let mut keys = Vec::new();
    for (k, _v) in ScopedReadableCompound::iter_scoped(&comp) {
        keys.push(k.decode().to_string());
    }
    keys.sort();
    assert_eq!(keys, vec!["a", "b"]);
}

// ============ Borrowed Document root End tag ============

#[test]
fn test_borrowed_document_root_end() {
    let data = vec![0x00]; // End tag
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    assert!(root.is_end());
}

// ============ ImmutableValue visit from mutable module ============

#[test]
fn test_mutable_immutable_value_visit() {
    let mut data = vec![0x0A, 0x00, 0x00]; // Compound
    // Byte 'b' = 1
    data.push(0x01);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'b');
    data.push(1);
    // Short 's' = 2
    data.push(0x02);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b's');
    data.extend_from_slice(&2i16.to_be_bytes());
    // Long 'l' = 4
    data.push(0x04);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'l');
    data.extend_from_slice(&4i64.to_be_bytes());
    // Float 'f' = 5.0
    data.push(0x05);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'f');
    data.extend_from_slice(&5.0f32.to_be_bytes());
    // Double 'd' = 6.0
    data.push(0x06);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'd');
    data.extend_from_slice(&6.0f64.to_be_bytes());
    data.push(0x00); // End

    let owned = read_owned::<BE, BE>(&data).unwrap();

    // Test visit on Byte
    let v = owned.get("b").unwrap();
    let res = ReadableValue::visit(&v, |val| if let Value::Byte(b) = val { b } else { 0 });
    assert_eq!(res, 1);

    // Test visit on Short
    let v = owned.get("s").unwrap();
    let res = ReadableValue::visit(&v, |val| if let Value::Short(s) = val { s } else { 0 });
    assert_eq!(res, 2);

    // Test visit on Long
    let v = owned.get("l").unwrap();
    let res = ReadableValue::visit(&v, |val| if let Value::Long(l) = val { l } else { 0 });
    assert_eq!(res, 4);

    // Test visit on Float
    let v = owned.get("f").unwrap();
    let res = ReadableValue::visit(&v, |val| if let Value::Float(f) = val { f } else { 0.0 });
    assert_eq!(res, 5.0);

    // Test visit on Double
    let v = owned.get("d").unwrap();
    let res = ReadableValue::visit(&v, |val| if let Value::Double(d) = val { d } else { 0.0 });
    assert_eq!(res, 6.0);
}

#[test]
fn test_mutable_immutable_value_visit_arrays() {
    let mut data = vec![0x0A, 0x00, 0x00]; // Compound
    // ByteArray "ba"
    data.push(0x07);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"ba");
    data.extend_from_slice(&2u32.to_be_bytes());
    data.extend_from_slice(&[1, 2]);
    // IntArray "ia"
    data.push(0x0B);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"ia");
    data.extend_from_slice(&1u32.to_be_bytes());
    data.extend_from_slice(&42i32.to_be_bytes());
    // LongArray "la"
    data.push(0x0C);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"la");
    data.extend_from_slice(&1u32.to_be_bytes());
    data.extend_from_slice(&99i64.to_be_bytes());
    data.push(0x00);

    let owned = read_owned::<BE, BE>(&data).unwrap();

    // ByteArray via visit
    let v = owned.get("ba").unwrap();
    let res = ReadableValue::visit(&v, |val| {
        if let Value::ByteArray(arr) = val {
            arr.len()
        } else {
            0
        }
    });
    assert_eq!(res, 2);

    // IntArray via visit
    let v = owned.get("ia").unwrap();
    let res = ReadableValue::visit(&v, |val| {
        if let Value::IntArray(arr) = val {
            arr[0].get()
        } else {
            0
        }
    });
    assert_eq!(res, 42);

    // LongArray via visit
    let v = owned.get("la").unwrap();
    let res = ReadableValue::visit(&v, |val| {
        if let Value::LongArray(arr) = val {
            arr[0].get()
        } else {
            0
        }
    });
    assert_eq!(res, 99);
}

#[test]
fn test_mutable_immutable_value_visit_scoped_all() {
    let mut data = vec![0x0A, 0x00, 0x00]; // Compound
    // String "st"
    data.push(0x08);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"st");
    data.extend_from_slice(&4u16.to_be_bytes());
    data.extend_from_slice(b"test");
    // List "li"
    data.push(0x09);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"li");
    data.push(0x01); // Byte list
    data.extend_from_slice(&2u32.to_be_bytes());
    data.extend_from_slice(&[1, 2]);
    // Compound "cp"
    data.push(0x0A);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"cp");
    data.push(0x00); // End (empty compound)
    data.push(0x00);

    let owned = read_owned::<BE, BE>(&data).unwrap();

    // String via visit_scoped
    let v = owned.get("st").unwrap();
    let res = ScopedReadableValue::visit_scoped(&v, |val| {
        if let ValueScoped::String(s) = val {
            s.decode().len()
        } else {
            0
        }
    });
    assert_eq!(res, 4);

    // List via visit_scoped
    let v = owned.get("li").unwrap();
    let res = ScopedReadableValue::visit_scoped(&v, |val| {
        if let ValueScoped::List(l) = val {
            l.len()
        } else {
            0
        }
    });
    assert_eq!(res, 2);

    // Compound via visit_scoped
    let v = owned.get("cp").unwrap();
    let res = ScopedReadableValue::visit_scoped(&v, |val| matches!(val, ValueScoped::Compound(_)));
    assert!(res);
}

// ============ WritableValue visit_mut tests ============

#[test]
fn test_writable_value_visit_mut() {
    let mut data = vec![0x0A, 0x00, 0x00]; // Compound
    // Int "x" = 10
    data.push(0x03);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'x');
    data.extend_from_slice(&10i32.to_be_bytes());
    data.push(0x00);

    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    {
        let mut comp = owned.as_compound_mut().unwrap();
        if let Some(mut mv) = comp.get_mut("x") {
            WritableValue::visit_mut(&mut mv, |v| {
                if let na_nbt::ValueMut::Int(i) = v {
                    i.set(i.get() + 5);
                }
            });
        }
    }
    assert_eq!(owned.get("x").unwrap().as_int(), Some(15));
}
// ============ Additional trait coverage tests ============

#[test]
fn test_scoped_readable_value_all_is_methods_owned() {
    // Test all is_* methods via ScopedReadableValue trait on OwnedValue
    let byte_val = OwnedValue::<BE>::from(1i8);
    assert!(ScopedReadableValue::is_byte(&byte_val));
    assert!(!ScopedReadableValue::is_short(&byte_val));
    assert!(!ScopedReadableValue::is_int(&byte_val));
    assert!(!ScopedReadableValue::is_long(&byte_val));
    assert!(!ScopedReadableValue::is_float(&byte_val));
    assert!(!ScopedReadableValue::is_double(&byte_val));
    assert!(!ScopedReadableValue::is_byte_array(&byte_val));
    assert!(!ScopedReadableValue::is_string(&byte_val));
    assert!(!ScopedReadableValue::is_list(&byte_val));
    assert!(!ScopedReadableValue::is_compound(&byte_val));
    assert!(!ScopedReadableValue::is_int_array(&byte_val));
    assert!(!ScopedReadableValue::is_long_array(&byte_val));
    assert!(!ScopedReadableValue::is_end(&byte_val));

    let short_val = OwnedValue::<BE>::from(100i16);
    assert!(ScopedReadableValue::is_short(&short_val));

    let int_val = OwnedValue::<BE>::from(1000i32);
    assert!(ScopedReadableValue::is_int(&int_val));

    let long_val = OwnedValue::<BE>::from(10000i64);
    assert!(ScopedReadableValue::is_long(&long_val));

    let float_val = OwnedValue::<BE>::from(1.5f32);
    assert!(ScopedReadableValue::is_float(&float_val));

    let double_val = OwnedValue::<BE>::from(2.5f64);
    assert!(ScopedReadableValue::is_double(&double_val));

    let ba_val = OwnedValue::<BE>::from(vec![1i8, 2, 3]);
    assert!(ScopedReadableValue::is_byte_array(&ba_val));

    let str_val = OwnedValue::<BE>::from("hello");
    assert!(ScopedReadableValue::is_string(&str_val));

    let list_val = OwnedValue::<BE>::from(OwnedList::<BE>::default());
    assert!(ScopedReadableValue::is_list(&list_val));

    let comp_val = OwnedValue::<BE>::from(OwnedCompound::<BE>::default());
    assert!(ScopedReadableValue::is_compound(&comp_val));

    let ia_val = OwnedValue::<BE>::from(vec![I32::<BE>::new(1)]);
    assert!(ScopedReadableValue::is_int_array(&ia_val));

    let la_val = OwnedValue::<BE>::from(vec![I64::<BE>::new(1)]);
    assert!(ScopedReadableValue::is_long_array(&la_val));
}

#[test]
fn test_scoped_readable_value_as_methods_owned() {
    // Test all as_* methods via ScopedReadableValue trait on OwnedValue
    let byte_val = OwnedValue::<BE>::from(42i8);
    assert_eq!(ScopedReadableValue::as_byte(&byte_val), Some(42));
    assert_eq!(ScopedReadableValue::as_short(&byte_val), None);

    let short_val = OwnedValue::<BE>::from(1000i16);
    assert_eq!(ScopedReadableValue::as_short(&short_val), Some(1000));

    let int_val = OwnedValue::<BE>::from(50000i32);
    assert_eq!(ScopedReadableValue::as_int(&int_val), Some(50000));

    let long_val = OwnedValue::<BE>::from(999999i64);
    assert_eq!(ScopedReadableValue::as_long(&long_val), Some(999999));

    let float_val = OwnedValue::<BE>::from(std::f32::consts::PI);
    assert!(
        (ScopedReadableValue::as_float(&float_val).unwrap() - std::f32::consts::PI).abs() < 0.01
    );

    let double_val = OwnedValue::<BE>::from(std::f64::consts::E);
    assert!(
        (ScopedReadableValue::as_double(&double_val).unwrap() - std::f64::consts::E).abs() < 0.001
    );

    let ba_val = OwnedValue::<BE>::from(vec![1i8, 2, 3]);
    assert_eq!(
        ScopedReadableValue::as_byte_array(&ba_val).map(|a| a.len()),
        Some(3)
    );

    let ia_val = OwnedValue::<BE>::from(vec![I32::<BE>::new(42)]);
    assert_eq!(
        ScopedReadableValue::as_int_array(&ia_val).map(|a| a[0].get()),
        Some(42)
    );

    let la_val = OwnedValue::<BE>::from(vec![I64::<BE>::new(99)]);
    assert_eq!(
        ScopedReadableValue::as_long_array(&la_val).map(|a| a[0].get()),
        Some(99)
    );
}

#[test]
fn test_scoped_writable_value_set_methods() {
    // Test set_* methods via ScopedWritableValue trait
    let mut val = OwnedValue::<BE>::from(1i8);
    assert!(ScopedWritableValue::set_byte(&mut val, 42));
    assert_eq!(val.as_byte(), Some(42));

    let mut val = OwnedValue::<BE>::from(1i16);
    assert!(ScopedWritableValue::set_short(&mut val, 1000));
    assert_eq!(val.as_short(), Some(1000));

    let mut val = OwnedValue::<BE>::from(1i32);
    assert!(ScopedWritableValue::set_int(&mut val, 50000));
    assert_eq!(val.as_int(), Some(50000));

    let mut val = OwnedValue::<BE>::from(1i64);
    assert!(ScopedWritableValue::set_long(&mut val, 999999));
    assert_eq!(val.as_long(), Some(999999));

    let mut val = OwnedValue::<BE>::from(1.0f32);
    assert!(ScopedWritableValue::set_float(
        &mut val,
        std::f32::consts::PI
    ));
    assert!((val.as_float().unwrap() - std::f32::consts::PI).abs() < 0.01);

    let mut val = OwnedValue::<BE>::from(1.0f64);
    assert!(ScopedWritableValue::set_double(
        &mut val,
        std::f64::consts::E
    ));
    assert!((val.as_double().unwrap() - std::f64::consts::E).abs() < 0.001);
}

#[test]
fn test_scoped_writable_value_update_methods() {
    // Test update_* methods via ScopedWritableValue trait
    let mut val = OwnedValue::<BE>::from(10i8);
    assert!(ScopedWritableValue::update_byte(&mut val, |x| x + 5));
    assert_eq!(val.as_byte(), Some(15));

    let mut val = OwnedValue::<BE>::from(100i16);
    assert!(ScopedWritableValue::update_short(&mut val, |x| x * 2));
    assert_eq!(val.as_short(), Some(200));

    let mut val = OwnedValue::<BE>::from(1000i32);
    assert!(ScopedWritableValue::update_int(&mut val, |x| x + 234));
    assert_eq!(val.as_int(), Some(1234));

    let mut val = OwnedValue::<BE>::from(10000i64);
    assert!(ScopedWritableValue::update_long(&mut val, |x| x - 1));
    assert_eq!(val.as_long(), Some(9999));

    let mut val = OwnedValue::<BE>::from(1.0f32);
    assert!(ScopedWritableValue::update_float(&mut val, |x| x * std::f32::consts::PI));
    assert!((val.as_float().unwrap() - std::f32::consts::PI).abs() < 0.01);

    let mut val = OwnedValue::<BE>::from(1.0f64);
    assert!(ScopedWritableValue::update_double(&mut val, |x| x * std::f64::consts::E));
    assert!((val.as_double().unwrap() - std::f64::consts::E).abs() < 0.001);
}

#[test]
fn test_scoped_writable_value_as_mut_scoped() {
    // Test as_*_mut_scoped methods
    let mut val = OwnedValue::<BE>::from(10i8);
    if let Some(b) = ScopedWritableValue::as_byte_mut(&mut val) {
        *b = 20;
    }
    assert_eq!(val.as_byte(), Some(20));

    let mut val = OwnedValue::<BE>::from(vec![1i8, 2, 3]);
    if let Some(mut arr) = ScopedWritableValue::as_byte_array_mut_scoped(&mut val) {
        arr.push(4);
    }
    assert_eq!(val.as_byte_array().map(|a| a.len()), Some(4));

    let mut val = OwnedValue::<BE>::from("hello");
    if let Some(mut s) = ScopedWritableValue::as_string_mut_scoped(&mut val) {
        s.push_str(" world");
    }
    assert!(val.as_string().is_some());

    let mut list_val = OwnedValue::<BE>::from(OwnedList::<BE>::default());
    if let Some(mut l) = ScopedWritableValue::as_list_mut_scoped(&mut list_val) {
        ScopedWritableList::push(&mut l, 1i8);
        ScopedWritableList::push(&mut l, 2i8);
        ScopedWritableList::push(&mut l, 3i8);
    }
    assert_eq!(list_val.as_list().map(|l| l.len()), Some(3));

    let mut comp_val = OwnedValue::<BE>::from(OwnedCompound::<BE>::default());
    if let Some(mut c) = ScopedWritableValue::as_compound_mut_scoped(&mut comp_val) {
        ScopedWritableCompound::insert(&mut c, "key", 42i32);
    }
    assert!(comp_val.as_compound().is_some());

    let mut ia_val = OwnedValue::<BE>::from(vec![I32::<BE>::new(1)]);
    if let Some(mut arr) = ScopedWritableValue::as_int_array_mut_scoped(&mut ia_val) {
        arr.push(I32::<BE>::new(2));
    }
    assert_eq!(ia_val.as_int_array().map(|a| a.len()), Some(2));

    let mut la_val = OwnedValue::<BE>::from(vec![I64::<BE>::new(1)]);
    if let Some(mut arr) = ScopedWritableValue::as_long_array_mut_scoped(&mut la_val) {
        arr.push(I64::<BE>::new(2));
    }
    assert_eq!(la_val.as_long_array().map(|a| a.len()), Some(2));
}

#[test]
fn test_scoped_readable_list_methods() {
    let mut list = OwnedList::<BE>::default();
    list.push(10i32);
    list.push(20i32);
    list.push(30i32);
    assert_eq!(ScopedReadableList::len(&list), 3);
    assert!(!ScopedReadableList::is_empty(&list));
    assert_eq!(ScopedReadableList::tag_id(&list), Tag::Int);

    let item = ScopedReadableList::get_scoped(&list, 1);
    assert!(item.is_some());
    assert_eq!(item.unwrap().as_int(), Some(20));

    let mut count = 0;
    for _ in ScopedReadableList::iter_scoped(&list) {
        count += 1;
    }
    assert_eq!(count, 3);
}

#[test]
fn test_scoped_writable_list_methods() {
    let mut list = OwnedList::<BE>::default();
    list.push(10i32);
    list.push(20i32);

    // push
    ScopedWritableList::push(&mut list, 30i32);
    assert_eq!(list.len(), 3);

    // insert
    ScopedWritableList::insert(&mut list, 0, 5i32);
    assert_eq!(list.len(), 4);
    assert_eq!(list.get(0).unwrap().as_int(), Some(5));

    // get_mut
    if let Some(mut mv) = ScopedWritableList::get_mut(&mut list, 1) {
        mv.set_int(100);
    }
    assert_eq!(list.get(1).unwrap().as_int(), Some(100));

    // pop
    let popped = ScopedWritableList::pop(&mut list);
    assert!(popped.is_some());
    assert_eq!(list.len(), 3);

    // remove
    let removed = ScopedWritableList::remove(&mut list, 0);
    assert_eq!(removed.as_int(), Some(5));
    assert_eq!(list.len(), 2);

    // iter_mut
    for mut mv in ScopedWritableList::iter_mut(&mut list) {
        if let Some(i) = mv.as_int() {
            mv.set_int(i + 1);
        }
    }
}

#[test]
fn test_scoped_readable_compound_methods() {
    let mut comp = OwnedCompound::<BE>::default();
    comp.insert("a", 1i32);
    comp.insert("b", 2i32);

    let val = ScopedReadableCompound::get_scoped(&comp, "a");
    assert!(val.is_some());
    assert_eq!(val.unwrap().as_int(), Some(1));

    let mut count = 0;
    for (key, _) in ScopedReadableCompound::iter_scoped(&comp) {
        assert!(key.decode().len() == 1);
        count += 1;
    }
    assert_eq!(count, 2);
}

#[test]
fn test_scoped_writable_compound_methods() {
    let mut comp = OwnedCompound::<BE>::default();

    // insert
    let old = ScopedWritableCompound::insert(&mut comp, "x", 10i32);
    assert!(old.is_none());

    // insert again (replace)
    let old = ScopedWritableCompound::insert(&mut comp, "x", 20i32);
    assert!(old.is_some());
    assert_eq!(old.unwrap().as_int(), Some(10));

    // get_mut
    if let Some(mut mv) = ScopedWritableCompound::get_mut(&mut comp, "x") {
        mv.set_int(30);
    }
    assert_eq!(comp.get("x").unwrap().as_int(), Some(30));

    // remove
    let removed = ScopedWritableCompound::remove(&mut comp, "x");
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().as_int(), Some(30));

    // iter_mut
    comp.insert("y", 100i32);
    for (_, mut mv) in ScopedWritableCompound::iter_mut(&mut comp) {
        if let Some(i) = mv.as_int() {
            mv.set_int(i * 2);
        }
    }
    assert_eq!(comp.get("y").unwrap().as_int(), Some(200));
}

// ============ MutableValue trait tests ============

#[test]
fn test_mutable_value_trait_methods() {
    // Create data with various types
    let mut data = vec![0x0A, 0x00, 0x00]; // Compound

    // Byte "b" = 10
    data.push(0x01);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'b');
    data.push(10u8);

    // Short "s" = 100
    data.push(0x02);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b's');
    data.extend_from_slice(&100i16.to_be_bytes());

    // Int "i" = 1000
    data.push(0x03);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'i');
    data.extend_from_slice(&1000i32.to_be_bytes());

    // Long "l" = 10000
    data.push(0x04);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'l');
    data.extend_from_slice(&10000i64.to_be_bytes());

    // Float "f" = 1.5
    data.push(0x05);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'f');
    data.extend_from_slice(&1.5f32.to_be_bytes());

    // Double "d" = 2.5
    data.push(0x06);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'd');
    data.extend_from_slice(&2.5f64.to_be_bytes());

    data.push(0x00); // End compound

    let mut owned = read_owned::<BE, BE>(&data).unwrap();

    // Test via MutableValue methods accessed through get_mut
    {
        let mut comp = owned.as_compound_mut().unwrap();

        // Test is_* methods on mutable values
        let mv = comp.get_mut("b").unwrap();
        assert!(ScopedReadableValue::is_byte(&mv));

        let mv = comp.get_mut("s").unwrap();
        assert!(ScopedReadableValue::is_short(&mv));

        let mv = comp.get_mut("i").unwrap();
        assert!(ScopedReadableValue::is_int(&mv));

        let mv = comp.get_mut("l").unwrap();
        assert!(ScopedReadableValue::is_long(&mv));

        let mv = comp.get_mut("f").unwrap();
        assert!(ScopedReadableValue::is_float(&mv));

        let mv = comp.get_mut("d").unwrap();
        assert!(ScopedReadableValue::is_double(&mv));
    }

    // Test update methods on MutableValue
    {
        let mut comp = owned.as_compound_mut().unwrap();

        if let Some(mut mv) = comp.get_mut("b") {
            ScopedWritableValue::update_byte(&mut mv, |x| x + 5);
        }

        if let Some(mut mv) = comp.get_mut("s") {
            ScopedWritableValue::update_short(&mut mv, |x| x * 2);
        }

        if let Some(mut mv) = comp.get_mut("i") {
            ScopedWritableValue::update_int(&mut mv, |x| x + 234);
        }

        if let Some(mut mv) = comp.get_mut("l") {
            ScopedWritableValue::update_long(&mut mv, |x| x - 1);
        }

        if let Some(mut mv) = comp.get_mut("f") {
            ScopedWritableValue::update_float(&mut mv, |x| x * 2.0);
        }

        if let Some(mut mv) = comp.get_mut("d") {
            ScopedWritableValue::update_double(&mut mv, |x| x + 0.5);
        }
    }

    assert_eq!(owned.get("b").unwrap().as_byte(), Some(15));
    assert_eq!(owned.get("s").unwrap().as_short(), Some(200));
    assert_eq!(owned.get("i").unwrap().as_int(), Some(1234));
    assert_eq!(owned.get("l").unwrap().as_long(), Some(9999));
}

// ============ Test write fallback paths (BE to LE conversion) ============

#[test]
fn test_write_be_to_le_conversion_with_all_types() {
    // Create a complex structure with all types in BE, write to LE
    let mut data = vec![0x0A, 0x00, 0x00]; // Compound root

    // Short in compound
    data.push(0x02);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"sh");
    data.extend_from_slice(&1234i16.to_be_bytes());

    // Int in compound
    data.push(0x03);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"it");
    data.extend_from_slice(&56789i32.to_be_bytes());

    // Long in compound
    data.push(0x04);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"lg");
    data.extend_from_slice(&123456789i64.to_be_bytes());

    // Float in compound
    data.push(0x05);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"ft");
    data.extend_from_slice(&std::f32::consts::PI.to_be_bytes());

    // Double in compound
    data.push(0x06);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"db");
    data.extend_from_slice(&std::f64::consts::E.to_be_bytes());

    // ByteArray in compound
    data.push(0x07);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"ba");
    data.extend_from_slice(&3u32.to_be_bytes());
    data.extend_from_slice(&[1, 2, 3]);

    // String in compound
    data.push(0x08);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"st");
    data.extend_from_slice(&5u16.to_be_bytes());
    data.extend_from_slice(b"hello");

    // IntArray in compound
    data.push(0x0B);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"ia");
    data.extend_from_slice(&2u32.to_be_bytes());
    data.extend_from_slice(&10i32.to_be_bytes());
    data.extend_from_slice(&20i32.to_be_bytes());

    // LongArray in compound
    data.push(0x0C);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"la");
    data.extend_from_slice(&2u32.to_be_bytes());
    data.extend_from_slice(&100i64.to_be_bytes());
    data.extend_from_slice(&200i64.to_be_bytes());

    data.push(0x00); // End compound

    // Read as BE, write as LE
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let written = write_value_to_vec::<_, BE, LE>(&root).unwrap();

    // Read back as LE and verify values
    let doc2 = read_borrowed::<LE>(&written).unwrap();
    let root = doc2.root();

    assert_eq!(root.get("sh").unwrap().as_short(), Some(1234));
    assert_eq!(root.get("it").unwrap().as_int(), Some(56789));
    assert_eq!(root.get("lg").unwrap().as_long(), Some(123456789));
    assert!((root.get("ft").unwrap().as_float().unwrap() - std::f32::consts::PI).abs() < 0.01);
    assert!((root.get("db").unwrap().as_double().unwrap() - std::f64::consts::E).abs() < 0.001);
    assert_eq!(
        root.get("ba").unwrap().as_byte_array().map(|a| a.len()),
        Some(3)
    );
    assert_eq!(
        root.get("ia").unwrap().as_int_array().map(|a| a[1].get()),
        Some(20)
    );
    assert_eq!(
        root.get("la").unwrap().as_long_array().map(|a| a[1].get()),
        Some(200)
    );
}

#[test]
fn test_write_be_to_le_with_nested_list() {
    // Test list with various element types
    let mut data = vec![0x0A, 0x00, 0x00]; // Compound root

    // List of shorts
    data.push(0x09);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"ls");
    data.push(0x02); // Short list
    data.extend_from_slice(&3u32.to_be_bytes());
    data.extend_from_slice(&10i16.to_be_bytes());
    data.extend_from_slice(&20i16.to_be_bytes());
    data.extend_from_slice(&30i16.to_be_bytes());

    // List of ints
    data.push(0x09);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"li");
    data.push(0x03); // Int list
    data.extend_from_slice(&2u32.to_be_bytes());
    data.extend_from_slice(&100i32.to_be_bytes());
    data.extend_from_slice(&200i32.to_be_bytes());

    // List of longs
    data.push(0x09);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"ll");
    data.push(0x04); // Long list
    data.extend_from_slice(&2u32.to_be_bytes());
    data.extend_from_slice(&1000i64.to_be_bytes());
    data.extend_from_slice(&2000i64.to_be_bytes());

    // List of floats
    data.push(0x09);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"lf");
    data.push(0x05); // Float list
    data.extend_from_slice(&2u32.to_be_bytes());
    data.extend_from_slice(&1.5f32.to_be_bytes());
    data.extend_from_slice(&2.5f32.to_be_bytes());

    // List of doubles
    data.push(0x09);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"ld");
    data.push(0x06); // Double list
    data.extend_from_slice(&2u32.to_be_bytes());
    data.extend_from_slice(&1.1f64.to_be_bytes());
    data.extend_from_slice(&2.2f64.to_be_bytes());

    data.push(0x00); // End compound

    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let written = write_value_to_vec::<_, BE, LE>(&root).unwrap();

    let doc2 = read_borrowed::<LE>(&written).unwrap();
    let root2 = doc2.root();

    let ls_val = root2.get("ls").unwrap();
    let ls = ls_val.as_list().unwrap();
    assert_eq!(ls.get(2).unwrap().as_short(), Some(30));

    let li_val = root2.get("li").unwrap();
    let li = li_val.as_list().unwrap();
    assert_eq!(li.get(1).unwrap().as_int(), Some(200));

    let ll_val = root2.get("ll").unwrap();
    let ll = ll_val.as_list().unwrap();
    assert_eq!(ll.get(0).unwrap().as_long(), Some(1000));
}

#[test]
fn test_write_be_to_le_with_list_of_byte_arrays() {
    let mut data = vec![0x0A, 0x00, 0x00]; // Compound

    // List of byte arrays
    data.push(0x09);
    data.extend_from_slice(&3u16.to_be_bytes());
    data.extend_from_slice(b"lba");
    data.push(0x07); // ByteArray list
    data.extend_from_slice(&2u32.to_be_bytes());
    // First byte array
    data.extend_from_slice(&2u32.to_be_bytes());
    data.extend_from_slice(&[1, 2]);
    // Second byte array
    data.extend_from_slice(&3u32.to_be_bytes());
    data.extend_from_slice(&[3, 4, 5]);

    data.push(0x00);

    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let written = write_value_to_vec::<_, BE, LE>(&root).unwrap();

    let doc2 = read_borrowed::<LE>(&written).unwrap();
    let root2 = doc2.root();
    let list_val = root2.get("lba").unwrap();
    let list = list_val.as_list().unwrap();
    assert_eq!(list.len(), 2);
}

#[test]
fn test_write_be_to_le_with_list_of_strings() {
    let mut data = vec![0x0A, 0x00, 0x00]; // Compound

    // List of strings
    data.push(0x09);
    data.extend_from_slice(&3u16.to_be_bytes());
    data.extend_from_slice(b"lst");
    data.push(0x08); // String list
    data.extend_from_slice(&2u32.to_be_bytes());
    // First string
    data.extend_from_slice(&3u16.to_be_bytes());
    data.extend_from_slice(b"abc");
    // Second string
    data.extend_from_slice(&3u16.to_be_bytes());
    data.extend_from_slice(b"def");

    data.push(0x00);

    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let written = write_value_to_vec::<_, BE, LE>(&root).unwrap();

    let doc2 = read_borrowed::<LE>(&written).unwrap();
    let root2 = doc2.root();
    let list_val = root2.get("lst").unwrap();
    let list = list_val.as_list().unwrap();
    assert_eq!(list.len(), 2);
}

#[test]
fn test_write_be_to_le_with_list_of_lists() {
    let mut data = vec![0x0A, 0x00, 0x00]; // Compound

    // List of lists (of bytes)
    data.push(0x09);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"ll");
    data.push(0x09); // List list
    data.extend_from_slice(&2u32.to_be_bytes());
    // First inner list (bytes)
    data.push(0x01);
    data.extend_from_slice(&2u32.to_be_bytes());
    data.extend_from_slice(&[1, 2]);
    // Second inner list (bytes)
    data.push(0x01);
    data.extend_from_slice(&3u32.to_be_bytes());
    data.extend_from_slice(&[3, 4, 5]);

    data.push(0x00);

    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let written = write_value_to_vec::<_, BE, LE>(&root).unwrap();

    let doc2 = read_borrowed::<LE>(&written).unwrap();
    let root2 = doc2.root();
    let list_val = root2.get("ll").unwrap();
    let list = list_val.as_list().unwrap();
    assert_eq!(list.len(), 2);
}

#[test]
fn test_write_be_to_le_with_list_of_compounds() {
    let mut data = vec![0x0A, 0x00, 0x00]; // Compound

    // List of compounds
    data.push(0x09);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"lc");
    data.push(0x0A); // Compound list
    data.extend_from_slice(&2u32.to_be_bytes());
    // First compound
    data.push(0x01); // Byte
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'x');
    data.push(1);
    data.push(0x00); // End
    // Second compound
    data.push(0x01); // Byte
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'y');
    data.push(2);
    data.push(0x00); // End

    data.push(0x00);

    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let written = write_value_to_vec::<_, BE, LE>(&root).unwrap();

    let doc2 = read_borrowed::<LE>(&written).unwrap();
    let root2 = doc2.root();
    let list_val = root2.get("lc").unwrap();
    let list = list_val.as_list().unwrap();
    assert_eq!(list.len(), 2);
}

#[test]
fn test_write_be_to_le_with_list_of_int_arrays() {
    let mut data = vec![0x0A, 0x00, 0x00]; // Compound

    // List of int arrays
    data.push(0x09);
    data.extend_from_slice(&3u16.to_be_bytes());
    data.extend_from_slice(b"lia");
    data.push(0x0B); // IntArray list
    data.extend_from_slice(&2u32.to_be_bytes());
    // First int array
    data.extend_from_slice(&2u32.to_be_bytes());
    data.extend_from_slice(&10i32.to_be_bytes());
    data.extend_from_slice(&20i32.to_be_bytes());
    // Second int array
    data.extend_from_slice(&1u32.to_be_bytes());
    data.extend_from_slice(&30i32.to_be_bytes());

    data.push(0x00);

    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let written = write_value_to_vec::<_, BE, LE>(&root).unwrap();

    let doc2 = read_borrowed::<LE>(&written).unwrap();
    let root2 = doc2.root();
    let list_val = root2.get("lia").unwrap();
    let list = list_val.as_list().unwrap();
    assert_eq!(list.len(), 2);
}

#[test]
fn test_write_be_to_le_with_list_of_long_arrays() {
    let mut data = vec![0x0A, 0x00, 0x00]; // Compound

    // List of long arrays
    data.push(0x09);
    data.extend_from_slice(&3u16.to_be_bytes());
    data.extend_from_slice(b"lla");
    data.push(0x0C); // LongArray list
    data.extend_from_slice(&2u32.to_be_bytes());
    // First long array
    data.extend_from_slice(&2u32.to_be_bytes());
    data.extend_from_slice(&100i64.to_be_bytes());
    data.extend_from_slice(&200i64.to_be_bytes());
    // Second long array
    data.extend_from_slice(&1u32.to_be_bytes());
    data.extend_from_slice(&300i64.to_be_bytes());

    data.push(0x00);

    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let written = write_value_to_vec::<_, BE, LE>(&root).unwrap();

    let doc2 = read_borrowed::<LE>(&written).unwrap();
    let root2 = doc2.root();
    let list_val = root2.get("lla").unwrap();
    let list = list_val.as_list().unwrap();
    assert_eq!(list.len(), 2);
}

#[test]
fn test_write_to_writer_be_to_le() {
    // Test write_value_to_writer with endianness conversion
    let mut data = vec![0x0A, 0x00, 0x00]; // Compound

    // Short
    data.push(0x02);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'x');
    data.extend_from_slice(&1234i16.to_be_bytes());

    // Int
    data.push(0x03);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'y');
    data.extend_from_slice(&5678i32.to_be_bytes());

    // Long
    data.push(0x04);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'z');
    data.extend_from_slice(&91011i64.to_be_bytes());

    data.push(0x00);

    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let mut buffer = Vec::new();
    write_value_to_writer::<_, BE, LE, _>(&mut buffer, &root).unwrap();

    let doc2 = read_borrowed::<LE>(&buffer).unwrap();
    let root2 = doc2.root();
    assert_eq!(root2.get("x").unwrap().as_short(), Some(1234));
    assert_eq!(root2.get("y").unwrap().as_int(), Some(5678));
    assert_eq!(root2.get("z").unwrap().as_long(), Some(91011));
}

#[test]
fn test_write_to_writer_with_nested_compounds() {
    let mut data = vec![0x0A, 0x00, 0x00]; // Compound

    // Nested compound
    data.push(0x0A);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'n');
    // Inner compound content
    data.push(0x03); // Int
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'v');
    data.extend_from_slice(&42i32.to_be_bytes());
    data.push(0x00); // End inner

    data.push(0x00); // End outer

    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let mut buffer = Vec::new();
    write_value_to_writer::<_, BE, LE, _>(&mut buffer, &root).unwrap();

    let doc2 = read_borrowed::<LE>(&buffer).unwrap();
    let root2 = doc2.root();
    let nested_val = root2.get("n").unwrap();
    let nested = nested_val.as_compound().unwrap();
    assert_eq!(nested.get("v").unwrap().as_int(), Some(42));
}

#[test]
fn test_write_to_writer_with_list_conversions() {
    let mut data = vec![0x0A, 0x00, 0x00]; // Compound

    // List of ints
    data.push(0x09);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'l');
    data.push(0x03); // Int list
    data.extend_from_slice(&3u32.to_be_bytes());
    data.extend_from_slice(&10i32.to_be_bytes());
    data.extend_from_slice(&20i32.to_be_bytes());
    data.extend_from_slice(&30i32.to_be_bytes());

    data.push(0x00);

    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let mut buffer = Vec::new();
    write_value_to_writer::<_, BE, LE, _>(&mut buffer, &root).unwrap();

    let doc2 = read_borrowed::<LE>(&buffer).unwrap();
    let root2 = doc2.root();
    let list_val = root2.get("l").unwrap();
    let list = list_val.as_list().unwrap();
    assert_eq!(list.get(2).unwrap().as_int(), Some(30));
}

// ============ Tests for ImmutableValue trait impl methods (borrowed) ============

fn create_all_types_be() -> Vec<u8> {
    let mut data = vec![0x0A, 0x00, 0x00]; // Compound root

    // Byte
    data.push(0x01);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'b');
    data.push(0x7F);

    // Short
    data.push(0x02);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b's');
    data.extend_from_slice(&1234i16.to_be_bytes());

    // Int
    data.push(0x03);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'i');
    data.extend_from_slice(&12345678i32.to_be_bytes());

    // Long
    data.push(0x04);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'l');
    data.extend_from_slice(&123456789012345i64.to_be_bytes());

    // Float
    data.push(0x05);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'f');
    data.extend_from_slice(&std::f32::consts::PI.to_be_bytes());

    // Double
    data.push(0x06);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'd');
    data.extend_from_slice(&std::f64::consts::PI.to_be_bytes());

    // ByteArray
    data.push(0x07);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"ba");
    data.extend_from_slice(&3u32.to_be_bytes());
    data.extend_from_slice(&[1u8, 2u8, 3u8]);

    // String
    data.push(0x08);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"st");
    data.extend_from_slice(&5u16.to_be_bytes());
    data.extend_from_slice(b"hello");

    // List of bytes
    data.push(0x09);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"lb");
    data.push(0x01);
    data.extend_from_slice(&2u32.to_be_bytes());
    data.extend_from_slice(&[10u8, 20u8]);

    // Compound
    data.push(0x0A);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'c');
    data.push(0x01);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'x');
    data.push(42);
    data.push(0x00);

    // IntArray
    data.push(0x0B);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"ia");
    data.extend_from_slice(&2u32.to_be_bytes());
    data.extend_from_slice(&100i32.to_be_bytes());
    data.extend_from_slice(&200i32.to_be_bytes());

    // LongArray
    data.push(0x0C);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"la");
    data.extend_from_slice(&2u32.to_be_bytes());
    data.extend_from_slice(&1000i64.to_be_bytes());
    data.extend_from_slice(&2000i64.to_be_bytes());

    data.push(0x00);
    data
}

#[test]
fn test_immutable_value_trait_all_is_methods() {
    let data = create_all_types_be();
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();

    // Test is_* methods via trait
    let byte_val = ScopedReadableValue::get_scoped(&root, "b").unwrap();
    assert!(ScopedReadableValue::is_byte(&byte_val));
    assert!(!ScopedReadableValue::is_short(&byte_val));
    assert!(!ScopedReadableValue::is_int(&byte_val));
    assert!(!ScopedReadableValue::is_long(&byte_val));
    assert!(!ScopedReadableValue::is_float(&byte_val));
    assert!(!ScopedReadableValue::is_double(&byte_val));
    assert!(!ScopedReadableValue::is_byte_array(&byte_val));
    assert!(!ScopedReadableValue::is_string(&byte_val));
    assert!(!ScopedReadableValue::is_list(&byte_val));
    assert!(!ScopedReadableValue::is_compound(&byte_val));
    assert!(!ScopedReadableValue::is_int_array(&byte_val));
    assert!(!ScopedReadableValue::is_long_array(&byte_val));
    assert!(!ScopedReadableValue::is_end(&byte_val));

    let short_val = ScopedReadableValue::get_scoped(&root, "s").unwrap();
    assert!(ScopedReadableValue::is_short(&short_val));

    let int_val = ScopedReadableValue::get_scoped(&root, "i").unwrap();
    assert!(ScopedReadableValue::is_int(&int_val));

    let long_val = ScopedReadableValue::get_scoped(&root, "l").unwrap();
    assert!(ScopedReadableValue::is_long(&long_val));

    let float_val = ScopedReadableValue::get_scoped(&root, "f").unwrap();
    assert!(ScopedReadableValue::is_float(&float_val));

    let double_val = ScopedReadableValue::get_scoped(&root, "d").unwrap();
    assert!(ScopedReadableValue::is_double(&double_val));

    let ba_val = ScopedReadableValue::get_scoped(&root, "ba").unwrap();
    assert!(ScopedReadableValue::is_byte_array(&ba_val));

    let st_val = ScopedReadableValue::get_scoped(&root, "st").unwrap();
    assert!(ScopedReadableValue::is_string(&st_val));

    let lb_val = ScopedReadableValue::get_scoped(&root, "lb").unwrap();
    assert!(ScopedReadableValue::is_list(&lb_val));

    let c_val = ScopedReadableValue::get_scoped(&root, "c").unwrap();
    assert!(ScopedReadableValue::is_compound(&c_val));

    let ia_val = ScopedReadableValue::get_scoped(&root, "ia").unwrap();
    assert!(ScopedReadableValue::is_int_array(&ia_val));

    let la_val = ScopedReadableValue::get_scoped(&root, "la").unwrap();
    assert!(ScopedReadableValue::is_long_array(&la_val));
}

#[test]
fn test_immutable_value_trait_all_as_methods() {
    let data = create_all_types_be();
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();

    // Test as_* methods via trait
    let byte_val = ScopedReadableValue::get_scoped(&root, "b").unwrap();
    assert_eq!(ScopedReadableValue::as_byte(&byte_val), Some(0x7F));
    assert_eq!(ScopedReadableValue::as_short(&byte_val), None);
    assert_eq!(ScopedReadableValue::as_end(&byte_val), None);

    let short_val = ScopedReadableValue::get_scoped(&root, "s").unwrap();
    assert_eq!(ScopedReadableValue::as_short(&short_val), Some(1234));

    let int_val = ScopedReadableValue::get_scoped(&root, "i").unwrap();
    assert_eq!(ScopedReadableValue::as_int(&int_val), Some(12345678));

    let long_val = ScopedReadableValue::get_scoped(&root, "l").unwrap();
    assert_eq!(
        ScopedReadableValue::as_long(&long_val),
        Some(123456789012345)
    );

    let float_val = ScopedReadableValue::get_scoped(&root, "f").unwrap();
    let f = ScopedReadableValue::as_float(&float_val).unwrap();
    assert!((f - std::f32::consts::PI).abs() < 0.001);

    let double_val = ScopedReadableValue::get_scoped(&root, "d").unwrap();
    let d = ScopedReadableValue::as_double(&double_val).unwrap();
    assert!((d - std::f64::consts::PI).abs() < 0.000001);

    let ba_val = ScopedReadableValue::get_scoped(&root, "ba").unwrap();
    let ba: &[i8] = ScopedReadableValue::as_byte_array(&ba_val).unwrap();
    assert_eq!(ba.len(), 3);

    let st_val = ScopedReadableValue::get_scoped(&root, "st").unwrap();
    let _st = ScopedReadableValue::as_string_scoped(&st_val).unwrap();

    let lb_val = ScopedReadableValue::get_scoped(&root, "lb").unwrap();
    let _lb = ScopedReadableValue::as_list_scoped(&lb_val).unwrap();

    let c_val = ScopedReadableValue::get_scoped(&root, "c").unwrap();
    let _c = ScopedReadableValue::as_compound_scoped(&c_val).unwrap();

    let ia_val = ScopedReadableValue::get_scoped(&root, "ia").unwrap();
    let ia = ScopedReadableValue::as_int_array(&ia_val).unwrap();
    assert_eq!(ia.len(), 2);

    let la_val = ScopedReadableValue::get_scoped(&root, "la").unwrap();
    let la = ScopedReadableValue::as_long_array(&la_val).unwrap();
    assert_eq!(la.len(), 2);
}

#[test]
fn test_immutable_value_trait_tag_id() {
    let data = create_all_types_be();
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();

    let byte_val = ScopedReadableValue::get_scoped(&root, "b").unwrap();
    assert_eq!(ScopedReadableValue::tag_id(&byte_val), Tag::Byte);

    let short_val = ScopedReadableValue::get_scoped(&root, "s").unwrap();
    assert_eq!(ScopedReadableValue::tag_id(&short_val), Tag::Short);

    let int_val = ScopedReadableValue::get_scoped(&root, "i").unwrap();
    assert_eq!(ScopedReadableValue::tag_id(&int_val), Tag::Int);

    let long_val = ScopedReadableValue::get_scoped(&root, "l").unwrap();
    assert_eq!(ScopedReadableValue::tag_id(&long_val), Tag::Long);

    let float_val = ScopedReadableValue::get_scoped(&root, "f").unwrap();
    assert_eq!(ScopedReadableValue::tag_id(&float_val), Tag::Float);

    let double_val = ScopedReadableValue::get_scoped(&root, "d").unwrap();
    assert_eq!(ScopedReadableValue::tag_id(&double_val), Tag::Double);
}

#[test]
fn test_immutable_value_trait_visit_scoped_all_types() {
    let data = create_all_types_be();
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();

    let byte_val = ScopedReadableValue::get_scoped(&root, "b").unwrap();
    ScopedReadableValue::visit_scoped(&byte_val, |v| {
        assert!(matches!(v, ValueScoped::Byte(0x7F)));
    });

    let short_val = ScopedReadableValue::get_scoped(&root, "s").unwrap();
    ScopedReadableValue::visit_scoped(&short_val, |v| {
        assert!(matches!(v, ValueScoped::Short(1234)));
    });

    let int_val = ScopedReadableValue::get_scoped(&root, "i").unwrap();
    ScopedReadableValue::visit_scoped(&int_val, |v| {
        assert!(matches!(v, ValueScoped::Int(12345678)));
    });

    let long_val = ScopedReadableValue::get_scoped(&root, "l").unwrap();
    ScopedReadableValue::visit_scoped(&long_val, |v| {
        assert!(matches!(v, ValueScoped::Long(123456789012345)));
    });

    let float_val = ScopedReadableValue::get_scoped(&root, "f").unwrap();
    ScopedReadableValue::visit_scoped(&float_val, |v| {
        assert!(matches!(v, ValueScoped::Float(_)));
    });

    let double_val = ScopedReadableValue::get_scoped(&root, "d").unwrap();
    ScopedReadableValue::visit_scoped(&double_val, |v| {
        assert!(matches!(v, ValueScoped::Double(_)));
    });

    let ba_val = ScopedReadableValue::get_scoped(&root, "ba").unwrap();
    ScopedReadableValue::visit_scoped(&ba_val, |v| {
        assert!(matches!(v, ValueScoped::ByteArray(_)));
    });

    let st_val = ScopedReadableValue::get_scoped(&root, "st").unwrap();
    ScopedReadableValue::visit_scoped(&st_val, |v| {
        assert!(matches!(v, ValueScoped::String(_)));
    });

    let lb_val = ScopedReadableValue::get_scoped(&root, "lb").unwrap();
    ScopedReadableValue::visit_scoped(&lb_val, |v| {
        assert!(matches!(v, ValueScoped::List(_)));
    });

    let c_val = ScopedReadableValue::get_scoped(&root, "c").unwrap();
    ScopedReadableValue::visit_scoped(&c_val, |v| {
        assert!(matches!(v, ValueScoped::Compound(_)));
    });

    let ia_val = ScopedReadableValue::get_scoped(&root, "ia").unwrap();
    ScopedReadableValue::visit_scoped(&ia_val, |v| {
        assert!(matches!(v, ValueScoped::IntArray(_)));
    });

    let la_val = ScopedReadableValue::get_scoped(&root, "la").unwrap();
    ScopedReadableValue::visit_scoped(&la_val, |v| {
        assert!(matches!(v, ValueScoped::LongArray(_)));
    });
}

#[test]
fn test_immutable_value_readable_trait_visit() {
    let data = create_all_types_be();
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();

    let byte_val = ReadableValue::get(&root, "b").unwrap();
    ReadableValue::visit(&byte_val, |v| {
        assert!(matches!(v, Value::Byte(0x7F)));
    });

    let short_val = ReadableValue::get(&root, "s").unwrap();
    ReadableValue::visit(&short_val, |v| {
        assert!(matches!(v, Value::Short(1234)));
    });

    let int_val = ReadableValue::get(&root, "i").unwrap();
    ReadableValue::visit(&int_val, |v| {
        assert!(matches!(v, Value::Int(12345678)));
    });

    let st_val = ReadableValue::get(&root, "st").unwrap();
    ReadableValue::visit(&st_val, |v| {
        assert!(matches!(v, Value::String(_)));
    });

    let lb_val = ReadableValue::get(&root, "lb").unwrap();
    ReadableValue::visit(&lb_val, |v| {
        assert!(matches!(v, Value::List(_)));
    });

    let c_val = ReadableValue::get(&root, "c").unwrap();
    ReadableValue::visit(&c_val, |v| {
        assert!(matches!(v, Value::Compound(_)));
    });
}

#[test]
fn test_immutable_list_trait_methods() {
    let data = create_all_types_be();
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();

    let lb_val = ScopedReadableValue::get_scoped(&root, "lb").unwrap();
    let list = ScopedReadableValue::as_list_scoped(&lb_val).unwrap();

    // Test ScopedReadableList methods
    assert_eq!(ScopedReadableList::tag_id(&list), Tag::Byte);
    assert_eq!(ScopedReadableList::len(&list), 2);
    assert!(!ScopedReadableList::is_empty(&list));

    let item = ScopedReadableList::get_scoped(&list, 0).unwrap();
    assert_eq!(ScopedReadableValue::as_byte(&item), Some(10));

    let mut count = 0;
    for _item in ScopedReadableList::iter_scoped(&list) {
        count += 1;
    }
    assert_eq!(count, 2);

    // Test ReadableList methods
    let item2 = ReadableList::get(&list, 1).unwrap();
    assert_eq!(ScopedReadableValue::as_byte(&item2), Some(20));

    count = 0;
    for _item in ReadableList::iter(&list) {
        count += 1;
    }
    assert_eq!(count, 2);
}

#[test]
fn test_immutable_compound_trait_methods() {
    let data = create_all_types_be();
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();

    let c_val = ScopedReadableValue::get_scoped(&root, "c").unwrap();
    let compound = ScopedReadableValue::as_compound_scoped(&c_val).unwrap();

    // Test ScopedReadableCompound methods
    let x = ScopedReadableCompound::get_scoped(&compound, "x").unwrap();
    assert_eq!(ScopedReadableValue::as_byte(&x), Some(42));

    let mut count = 0;
    for (_name, _val) in ScopedReadableCompound::iter_scoped(&compound) {
        count += 1;
    }
    assert_eq!(count, 1);

    // Test ReadableCompound methods
    let x2 = ReadableCompound::get(&compound, "x").unwrap();
    assert_eq!(ScopedReadableValue::as_byte(&x2), Some(42));

    count = 0;
    for (_name, _val) in ReadableCompound::iter(&compound) {
        count += 1;
    }
    assert_eq!(count, 1);
}

#[test]
fn test_immutable_readable_value_as_methods() {
    let data = create_all_types_be();
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();

    // Test ReadableValue::as_* methods (non-scoped)
    let st_val = ReadableValue::get(&root, "st").unwrap();
    let st = ReadableValue::as_string(&st_val).unwrap();
    assert_eq!(st.decode(), "hello");

    let lb_val = ReadableValue::get(&root, "lb").unwrap();
    let _lb = ReadableValue::as_list(&lb_val).unwrap();

    let c_val = ReadableValue::get(&root, "c").unwrap();
    let _c = ReadableValue::as_compound(&c_val).unwrap();
}

#[test]
fn test_immutable_string_trait_methods() {
    use na_nbt::ReadableString;

    let data = create_all_types_be();
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();

    let st_val = ReadableValue::get(&root, "st").unwrap();
    let st = ReadableValue::as_string(&st_val).unwrap();

    let raw = ReadableString::raw_bytes(st);
    assert_eq!(raw, b"hello");

    let decoded = ReadableString::decode(st);
    assert_eq!(&*decoded, "hello");
}

#[test]
fn test_immutable_end_via_trait() {
    let data = vec![0x00]; // End tag
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();

    assert!(ScopedReadableValue::is_end(&root));
    assert_eq!(ScopedReadableValue::as_end(&root), Some(()));
    assert_eq!(ScopedReadableValue::tag_id(&root), Tag::End);
}

// ============ Tests for mutable module's ImmutableValue trait (via OwnedValue::get) ============

#[test]
fn test_mutable_immutable_value_trait_is_methods() {
    let data = create_all_types_be();
    let doc = read_owned::<BE, BE>(&data).unwrap();

    // When we call get() on OwnedValue, we get mutable::ImmutableValue
    let byte_val = doc.get("b").unwrap();
    assert!(ScopedReadableValue::is_byte(&byte_val));
    assert!(!ScopedReadableValue::is_short(&byte_val));
    assert!(!ScopedReadableValue::is_int(&byte_val));
    assert!(!ScopedReadableValue::is_long(&byte_val));
    assert!(!ScopedReadableValue::is_float(&byte_val));
    assert!(!ScopedReadableValue::is_double(&byte_val));
    assert!(!ScopedReadableValue::is_byte_array(&byte_val));
    assert!(!ScopedReadableValue::is_string(&byte_val));
    assert!(!ScopedReadableValue::is_list(&byte_val));
    assert!(!ScopedReadableValue::is_compound(&byte_val));
    assert!(!ScopedReadableValue::is_int_array(&byte_val));
    assert!(!ScopedReadableValue::is_long_array(&byte_val));
    assert!(!ScopedReadableValue::is_end(&byte_val));

    let short_val = doc.get("s").unwrap();
    assert!(ScopedReadableValue::is_short(&short_val));

    let int_val = doc.get("i").unwrap();
    assert!(ScopedReadableValue::is_int(&int_val));

    let long_val = doc.get("l").unwrap();
    assert!(ScopedReadableValue::is_long(&long_val));

    let float_val = doc.get("f").unwrap();
    assert!(ScopedReadableValue::is_float(&float_val));

    let double_val = doc.get("d").unwrap();
    assert!(ScopedReadableValue::is_double(&double_val));

    let ba_val = doc.get("ba").unwrap();
    assert!(ScopedReadableValue::is_byte_array(&ba_val));

    let st_val = doc.get("st").unwrap();
    assert!(ScopedReadableValue::is_string(&st_val));

    let lb_val = doc.get("lb").unwrap();
    assert!(ScopedReadableValue::is_list(&lb_val));

    let c_val = doc.get("c").unwrap();
    assert!(ScopedReadableValue::is_compound(&c_val));

    let ia_val = doc.get("ia").unwrap();
    assert!(ScopedReadableValue::is_int_array(&ia_val));

    let la_val = doc.get("la").unwrap();
    assert!(ScopedReadableValue::is_long_array(&la_val));
}

#[test]
fn test_mutable_immutable_value_trait_as_methods() {
    let data = create_all_types_be();
    let doc = read_owned::<BE, BE>(&data).unwrap();

    let byte_val = doc.get("b").unwrap();
    assert_eq!(ScopedReadableValue::as_byte(&byte_val), Some(0x7F));
    assert_eq!(ScopedReadableValue::as_short(&byte_val), None);
    assert_eq!(ScopedReadableValue::as_end(&byte_val), None);

    let short_val = doc.get("s").unwrap();
    assert_eq!(ScopedReadableValue::as_short(&short_val), Some(1234));

    let int_val = doc.get("i").unwrap();
    assert_eq!(ScopedReadableValue::as_int(&int_val), Some(12345678));

    let long_val = doc.get("l").unwrap();
    assert_eq!(
        ScopedReadableValue::as_long(&long_val),
        Some(123456789012345)
    );

    let float_val = doc.get("f").unwrap();
    let f = ScopedReadableValue::as_float(&float_val).unwrap();
    assert!((f - std::f32::consts::PI).abs() < 0.001);

    let double_val = doc.get("d").unwrap();
    let d = ScopedReadableValue::as_double(&double_val).unwrap();
    assert!((d - std::f64::consts::PI).abs() < 0.000001);

    let ba_val = doc.get("ba").unwrap();
    let ba: &[i8] = ScopedReadableValue::as_byte_array(&ba_val).unwrap();
    assert_eq!(ba.len(), 3);

    let st_val = doc.get("st").unwrap();
    let _st = ScopedReadableValue::as_string_scoped(&st_val).unwrap();

    let lb_val = doc.get("lb").unwrap();
    let _lb = ScopedReadableValue::as_list_scoped(&lb_val).unwrap();

    let c_val = doc.get("c").unwrap();
    let _c = ScopedReadableValue::as_compound_scoped(&c_val).unwrap();

    let ia_val = doc.get("ia").unwrap();
    let ia = ScopedReadableValue::as_int_array(&ia_val).unwrap();
    assert_eq!(ia.len(), 2);

    let la_val = doc.get("la").unwrap();
    let la = ScopedReadableValue::as_long_array(&la_val).unwrap();
    assert_eq!(la.len(), 2);
}

#[test]
fn test_mutable_immutable_value_trait_tag_id() {
    let data = create_all_types_be();
    let doc = read_owned::<BE, BE>(&data).unwrap();

    let byte_val = doc.get("b").unwrap();
    assert_eq!(ScopedReadableValue::tag_id(&byte_val), Tag::Byte);

    let short_val = doc.get("s").unwrap();
    assert_eq!(ScopedReadableValue::tag_id(&short_val), Tag::Short);

    let int_val = doc.get("i").unwrap();
    assert_eq!(ScopedReadableValue::tag_id(&int_val), Tag::Int);

    let long_val = doc.get("l").unwrap();
    assert_eq!(ScopedReadableValue::tag_id(&long_val), Tag::Long);

    let float_val = doc.get("f").unwrap();
    assert_eq!(ScopedReadableValue::tag_id(&float_val), Tag::Float);

    let double_val = doc.get("d").unwrap();
    assert_eq!(ScopedReadableValue::tag_id(&double_val), Tag::Double);
}

#[test]
fn test_mutable_immutable_value_trait_visit_scoped() {
    let data = create_all_types_be();
    let doc = read_owned::<BE, BE>(&data).unwrap();

    let byte_val = doc.get("b").unwrap();
    ScopedReadableValue::visit_scoped(&byte_val, |v| {
        assert!(matches!(v, ValueScoped::Byte(0x7F)));
    });

    let short_val = doc.get("s").unwrap();
    ScopedReadableValue::visit_scoped(&short_val, |v| {
        assert!(matches!(v, ValueScoped::Short(1234)));
    });

    let int_val = doc.get("i").unwrap();
    ScopedReadableValue::visit_scoped(&int_val, |v| {
        assert!(matches!(v, ValueScoped::Int(12345678)));
    });

    let long_val = doc.get("l").unwrap();
    ScopedReadableValue::visit_scoped(&long_val, |v| {
        assert!(matches!(v, ValueScoped::Long(123456789012345)));
    });

    let float_val = doc.get("f").unwrap();
    ScopedReadableValue::visit_scoped(&float_val, |v| {
        assert!(matches!(v, ValueScoped::Float(_)));
    });

    let double_val = doc.get("d").unwrap();
    ScopedReadableValue::visit_scoped(&double_val, |v| {
        assert!(matches!(v, ValueScoped::Double(_)));
    });

    let ba_val = doc.get("ba").unwrap();
    ScopedReadableValue::visit_scoped(&ba_val, |v| {
        assert!(matches!(v, ValueScoped::ByteArray(_)));
    });

    let st_val = doc.get("st").unwrap();
    ScopedReadableValue::visit_scoped(&st_val, |v| {
        assert!(matches!(v, ValueScoped::String(_)));
    });

    let lb_val = doc.get("lb").unwrap();
    ScopedReadableValue::visit_scoped(&lb_val, |v| {
        assert!(matches!(v, ValueScoped::List(_)));
    });

    let c_val = doc.get("c").unwrap();
    ScopedReadableValue::visit_scoped(&c_val, |v| {
        assert!(matches!(v, ValueScoped::Compound(_)));
    });

    let ia_val = doc.get("ia").unwrap();
    ScopedReadableValue::visit_scoped(&ia_val, |v| {
        assert!(matches!(v, ValueScoped::IntArray(_)));
    });

    let la_val = doc.get("la").unwrap();
    ScopedReadableValue::visit_scoped(&la_val, |v| {
        assert!(matches!(v, ValueScoped::LongArray(_)));
    });
}

#[test]
fn test_mutable_immutable_value_readable_visit() {
    let data = create_all_types_be();
    let doc = read_owned::<BE, BE>(&data).unwrap();

    let byte_val = doc.get("b").unwrap();
    ReadableValue::visit(&byte_val, |v| {
        assert!(matches!(v, Value::Byte(0x7F)));
    });

    let short_val = doc.get("s").unwrap();
    ReadableValue::visit(&short_val, |v| {
        assert!(matches!(v, Value::Short(1234)));
    });

    let st_val = doc.get("st").unwrap();
    ReadableValue::visit(&st_val, |v| {
        assert!(matches!(v, Value::String(_)));
    });

    let lb_val = doc.get("lb").unwrap();
    ReadableValue::visit(&lb_val, |v| {
        assert!(matches!(v, Value::List(_)));
    });

    let c_val = doc.get("c").unwrap();
    ReadableValue::visit(&c_val, |v| {
        assert!(matches!(v, Value::Compound(_)));
    });
}

#[test]
fn test_mutable_immutable_list_trait_methods() {
    let data = create_all_types_be();
    let doc = read_owned::<BE, BE>(&data).unwrap();

    let lb_val = doc.get("lb").unwrap();
    let list = ScopedReadableValue::as_list_scoped(&lb_val).unwrap();

    // Test ScopedReadableList methods
    assert_eq!(ScopedReadableList::tag_id(&list), Tag::Byte);
    assert_eq!(ScopedReadableList::len(&list), 2);
    assert!(!ScopedReadableList::is_empty(&list));

    let item = ScopedReadableList::get_scoped(&list, 0).unwrap();
    assert_eq!(ScopedReadableValue::as_byte(&item), Some(10));

    let mut count = 0;
    for _item in ScopedReadableList::iter_scoped(&list) {
        count += 1;
    }
    assert_eq!(count, 2);

    // Test ReadableList methods
    let item2 = ReadableList::get(&list, 1).unwrap();
    assert_eq!(ScopedReadableValue::as_byte(&item2), Some(20));

    count = 0;
    for _item in ReadableList::iter(&list) {
        count += 1;
    }
    assert_eq!(count, 2);
}

#[test]
fn test_mutable_immutable_compound_trait_methods() {
    let data = create_all_types_be();
    let doc = read_owned::<BE, BE>(&data).unwrap();

    let c_val = doc.get("c").unwrap();
    let compound = ScopedReadableValue::as_compound_scoped(&c_val).unwrap();

    // Test ScopedReadableCompound methods
    let x = ScopedReadableCompound::get_scoped(&compound, "x").unwrap();
    assert_eq!(ScopedReadableValue::as_byte(&x), Some(42));

    let mut count = 0;
    for (_name, _val) in ScopedReadableCompound::iter_scoped(&compound) {
        count += 1;
    }
    assert_eq!(count, 1);

    // Test ReadableCompound methods
    let x2 = ReadableCompound::get(&compound, "x").unwrap();
    assert_eq!(ScopedReadableValue::as_byte(&x2), Some(42));

    count = 0;
    for (_name, _val) in ReadableCompound::iter(&compound) {
        count += 1;
    }
    assert_eq!(count, 1);
}

#[test]
fn test_mutable_immutable_readable_value_as() {
    let data = create_all_types_be();
    let doc = read_owned::<BE, BE>(&data).unwrap();

    let st_val = doc.get("st").unwrap();
    let st = ReadableValue::as_string(&st_val).unwrap();
    assert_eq!(st.decode(), "hello");

    let lb_val = doc.get("lb").unwrap();
    let _lb = ReadableValue::as_list(&lb_val).unwrap();

    let c_val = doc.get("c").unwrap();
    let _c = ReadableValue::as_compound(&c_val).unwrap();
}

#[test]
fn test_mutable_immutable_string_trait() {
    use na_nbt::ReadableString;

    let data = create_all_types_be();
    let doc = read_owned::<BE, BE>(&data).unwrap();

    let st_val = doc.get("st").unwrap();
    let st = ReadableValue::as_string(&st_val).unwrap();

    let raw = ReadableString::raw_bytes(st);
    assert_eq!(raw, b"hello");

    let decoded = ReadableString::decode(st);
    assert_eq!(&*decoded, "hello");
}

#[test]
fn test_mutable_immutable_get_scoped() {
    let data = create_all_types_be();
    let doc = read_owned::<BE, BE>(&data).unwrap();

    // Use get_scoped trait method
    let byte_val = ScopedReadableValue::get_scoped(&doc, "b").unwrap();
    assert_eq!(ScopedReadableValue::as_byte(&byte_val), Some(0x7F));

    // Test nested access via compound
    let c_val = doc.get("c").unwrap();
    let nested = ScopedReadableValue::get_scoped(&c_val, "x").unwrap();
    assert_eq!(ScopedReadableValue::as_byte(&nested), Some(42));
}

// ============ Tests for MutableValue trait implementations (trait_impl_mut.rs) ============

#[test]
fn test_mutable_value_trait_all_is_methods() {
    let data = create_all_types_be();
    let mut doc = read_owned::<BE, BE>(&data).unwrap();

    // get_mut returns MutableValue
    let byte_mv = doc.get_mut("b").unwrap();
    assert!(ScopedReadableValue::is_byte(&byte_mv));
    assert!(!ScopedReadableValue::is_short(&byte_mv));
    assert!(!ScopedReadableValue::is_int(&byte_mv));
    assert!(!ScopedReadableValue::is_long(&byte_mv));
    assert!(!ScopedReadableValue::is_float(&byte_mv));
    assert!(!ScopedReadableValue::is_double(&byte_mv));
    assert!(!ScopedReadableValue::is_byte_array(&byte_mv));
    assert!(!ScopedReadableValue::is_string(&byte_mv));
    assert!(!ScopedReadableValue::is_list(&byte_mv));
    assert!(!ScopedReadableValue::is_compound(&byte_mv));
    assert!(!ScopedReadableValue::is_int_array(&byte_mv));
    assert!(!ScopedReadableValue::is_long_array(&byte_mv));
    assert!(!ScopedReadableValue::is_end(&byte_mv));

    let short_mv = doc.get_mut("s").unwrap();
    assert!(ScopedReadableValue::is_short(&short_mv));

    let int_mv = doc.get_mut("i").unwrap();
    assert!(ScopedReadableValue::is_int(&int_mv));

    let long_mv = doc.get_mut("l").unwrap();
    assert!(ScopedReadableValue::is_long(&long_mv));

    let float_mv = doc.get_mut("f").unwrap();
    assert!(ScopedReadableValue::is_float(&float_mv));

    let double_mv = doc.get_mut("d").unwrap();
    assert!(ScopedReadableValue::is_double(&double_mv));

    let ba_mv = doc.get_mut("ba").unwrap();
    assert!(ScopedReadableValue::is_byte_array(&ba_mv));

    let st_mv = doc.get_mut("st").unwrap();
    assert!(ScopedReadableValue::is_string(&st_mv));

    let lb_mv = doc.get_mut("lb").unwrap();
    assert!(ScopedReadableValue::is_list(&lb_mv));

    let c_mv = doc.get_mut("c").unwrap();
    assert!(ScopedReadableValue::is_compound(&c_mv));

    let ia_mv = doc.get_mut("ia").unwrap();
    assert!(ScopedReadableValue::is_int_array(&ia_mv));

    let la_mv = doc.get_mut("la").unwrap();
    assert!(ScopedReadableValue::is_long_array(&la_mv));
}

#[test]
fn test_mutable_value_trait_all_as_methods() {
    let data = create_all_types_be();
    let mut doc = read_owned::<BE, BE>(&data).unwrap();

    let byte_mv = doc.get_mut("b").unwrap();
    assert_eq!(ScopedReadableValue::as_byte(&byte_mv), Some(0x7F));
    assert_eq!(ScopedReadableValue::as_short(&byte_mv), None);
    assert_eq!(ScopedReadableValue::as_end(&byte_mv), None);

    let short_mv = doc.get_mut("s").unwrap();
    assert_eq!(ScopedReadableValue::as_short(&short_mv), Some(1234));

    let int_mv = doc.get_mut("i").unwrap();
    assert_eq!(ScopedReadableValue::as_int(&int_mv), Some(12345678));

    let long_mv = doc.get_mut("l").unwrap();
    assert_eq!(
        ScopedReadableValue::as_long(&long_mv),
        Some(123456789012345)
    );

    let float_mv = doc.get_mut("f").unwrap();
    let f = ScopedReadableValue::as_float(&float_mv).unwrap();
    assert!((f - std::f32::consts::PI).abs() < 0.001);

    let double_mv = doc.get_mut("d").unwrap();
    let d = ScopedReadableValue::as_double(&double_mv).unwrap();
    assert!((d - std::f64::consts::PI).abs() < 0.000001);

    let ba_mv = doc.get_mut("ba").unwrap();
    let ba: &[i8] = ScopedReadableValue::as_byte_array(&ba_mv).unwrap();
    assert_eq!(ba.len(), 3);

    let st_mv = doc.get_mut("st").unwrap();
    let _st = ScopedReadableValue::as_string_scoped(&st_mv).unwrap();

    let lb_mv = doc.get_mut("lb").unwrap();
    let _lb = ScopedReadableValue::as_list_scoped(&lb_mv).unwrap();

    let c_mv = doc.get_mut("c").unwrap();
    let _c = ScopedReadableValue::as_compound_scoped(&c_mv).unwrap();

    let ia_mv = doc.get_mut("ia").unwrap();
    let ia = ScopedReadableValue::as_int_array(&ia_mv).unwrap();
    assert_eq!(ia.len(), 2);

    let la_mv = doc.get_mut("la").unwrap();
    let la = ScopedReadableValue::as_long_array(&la_mv).unwrap();
    assert_eq!(la.len(), 2);
}

#[test]
fn test_mutable_value_trait_tag_id() {
    let data = create_all_types_be();
    let mut doc = read_owned::<BE, BE>(&data).unwrap();

    let byte_mv = doc.get_mut("b").unwrap();
    assert_eq!(ScopedReadableValue::tag_id(&byte_mv), Tag::Byte);

    let short_mv = doc.get_mut("s").unwrap();
    assert_eq!(ScopedReadableValue::tag_id(&short_mv), Tag::Short);

    let int_mv = doc.get_mut("i").unwrap();
    assert_eq!(ScopedReadableValue::tag_id(&int_mv), Tag::Int);

    let long_mv = doc.get_mut("l").unwrap();
    assert_eq!(ScopedReadableValue::tag_id(&long_mv), Tag::Long);

    let float_mv = doc.get_mut("f").unwrap();
    assert_eq!(ScopedReadableValue::tag_id(&float_mv), Tag::Float);

    let double_mv = doc.get_mut("d").unwrap();
    assert_eq!(ScopedReadableValue::tag_id(&double_mv), Tag::Double);
}

#[test]
fn test_mutable_value_trait_visit_scoped() {
    let data = create_all_types_be();
    let mut doc = read_owned::<BE, BE>(&data).unwrap();

    let byte_mv = doc.get_mut("b").unwrap();
    ScopedReadableValue::visit_scoped(&byte_mv, |v| {
        assert!(matches!(v, ValueScoped::Byte(0x7F)));
    });

    let short_mv = doc.get_mut("s").unwrap();
    ScopedReadableValue::visit_scoped(&short_mv, |v| {
        assert!(matches!(v, ValueScoped::Short(1234)));
    });

    let int_mv = doc.get_mut("i").unwrap();
    ScopedReadableValue::visit_scoped(&int_mv, |v| {
        assert!(matches!(v, ValueScoped::Int(12345678)));
    });

    let long_mv = doc.get_mut("l").unwrap();
    ScopedReadableValue::visit_scoped(&long_mv, |v| {
        assert!(matches!(v, ValueScoped::Long(123456789012345)));
    });

    let float_mv = doc.get_mut("f").unwrap();
    ScopedReadableValue::visit_scoped(&float_mv, |v| {
        assert!(matches!(v, ValueScoped::Float(_)));
    });

    let double_mv = doc.get_mut("d").unwrap();
    ScopedReadableValue::visit_scoped(&double_mv, |v| {
        assert!(matches!(v, ValueScoped::Double(_)));
    });

    let ba_mv = doc.get_mut("ba").unwrap();
    ScopedReadableValue::visit_scoped(&ba_mv, |v| {
        assert!(matches!(v, ValueScoped::ByteArray(_)));
    });

    let st_mv = doc.get_mut("st").unwrap();
    ScopedReadableValue::visit_scoped(&st_mv, |v| {
        assert!(matches!(v, ValueScoped::String(_)));
    });

    let lb_mv = doc.get_mut("lb").unwrap();
    ScopedReadableValue::visit_scoped(&lb_mv, |v| {
        assert!(matches!(v, ValueScoped::List(_)));
    });

    let c_mv = doc.get_mut("c").unwrap();
    ScopedReadableValue::visit_scoped(&c_mv, |v| {
        assert!(matches!(v, ValueScoped::Compound(_)));
    });

    let ia_mv = doc.get_mut("ia").unwrap();
    ScopedReadableValue::visit_scoped(&ia_mv, |v| {
        assert!(matches!(v, ValueScoped::IntArray(_)));
    });

    let la_mv = doc.get_mut("la").unwrap();
    ScopedReadableValue::visit_scoped(&la_mv, |v| {
        assert!(matches!(v, ValueScoped::LongArray(_)));
    });
}

#[test]
fn test_mutable_value_scoped_writable_set_methods() {
    let data = create_all_types_be();
    let mut doc = read_owned::<BE, BE>(&data).unwrap();

    // Test ScopedWritableValue set methods on MutableValue
    let mut byte_mv = doc.get_mut("b").unwrap();
    assert!(ScopedWritableValue::set_byte(&mut byte_mv, 100));
    assert_eq!(ScopedReadableValue::as_byte(&byte_mv), Some(100));

    let mut short_mv = doc.get_mut("s").unwrap();
    assert!(ScopedWritableValue::set_short(&mut short_mv, 999));
    assert_eq!(ScopedReadableValue::as_short(&short_mv), Some(999));

    let mut int_mv = doc.get_mut("i").unwrap();
    assert!(ScopedWritableValue::set_int(&mut int_mv, 888));
    assert_eq!(ScopedReadableValue::as_int(&int_mv), Some(888));

    let mut long_mv = doc.get_mut("l").unwrap();
    assert!(ScopedWritableValue::set_long(&mut long_mv, 777));
    assert_eq!(ScopedReadableValue::as_long(&long_mv), Some(777));

    let mut float_mv = doc.get_mut("f").unwrap();
    assert!(ScopedWritableValue::set_float(&mut float_mv, 1.5));
    let f = ScopedReadableValue::as_float(&float_mv).unwrap();
    assert!((f - 1.5).abs() < 0.001);

    let mut double_mv = doc.get_mut("d").unwrap();
    assert!(ScopedWritableValue::set_double(&mut double_mv, 2.5));
    let d = ScopedReadableValue::as_double(&double_mv).unwrap();
    assert!((d - 2.5).abs() < 0.001);
}

#[test]
fn test_mutable_value_scoped_writable_set_wrong_type() {
    let data = create_all_types_be();
    let mut doc = read_owned::<BE, BE>(&data).unwrap();

    // Setting wrong type should return false
    let mut byte_mv = doc.get_mut("b").unwrap();
    assert!(!ScopedWritableValue::set_short(&mut byte_mv, 100));
    assert!(!ScopedWritableValue::set_int(&mut byte_mv, 100));
    assert!(!ScopedWritableValue::set_long(&mut byte_mv, 100));
    assert!(!ScopedWritableValue::set_float(&mut byte_mv, 1.0));
    assert!(!ScopedWritableValue::set_double(&mut byte_mv, 1.0));

    let mut short_mv = doc.get_mut("s").unwrap();
    assert!(!ScopedWritableValue::set_byte(&mut short_mv, 1));
}

#[test]
fn test_mutable_value_scoped_writable_update_methods() {
    let data = create_all_types_be();
    let mut doc = read_owned::<BE, BE>(&data).unwrap();

    let mut byte_mv = doc.get_mut("b").unwrap();
    ScopedWritableValue::update_byte(&mut byte_mv, |b| b.wrapping_add(1));
    assert_eq!(ScopedReadableValue::as_byte(&byte_mv), Some(-128)); // 127 + 1 wraps to -128

    let mut short_mv = doc.get_mut("s").unwrap();
    ScopedWritableValue::update_short(&mut short_mv, |s| s + 1);
    assert_eq!(ScopedReadableValue::as_short(&short_mv), Some(1235));

    let mut int_mv = doc.get_mut("i").unwrap();
    ScopedWritableValue::update_int(&mut int_mv, |i| i + 1);
    assert_eq!(ScopedReadableValue::as_int(&int_mv), Some(12345679));

    let mut long_mv = doc.get_mut("l").unwrap();
    ScopedWritableValue::update_long(&mut long_mv, |l| l + 1);
    assert_eq!(
        ScopedReadableValue::as_long(&long_mv),
        Some(123456789012346)
    );

    let mut float_mv = doc.get_mut("f").unwrap();
    ScopedWritableValue::update_float(&mut float_mv, |f| f + 1.0);

    let mut double_mv = doc.get_mut("d").unwrap();
    ScopedWritableValue::update_double(&mut double_mv, |d| d + 1.0);
}

#[test]
fn test_mutable_value_scoped_writable_as_mut_scoped() {
    let data = create_all_types_be();
    let mut doc = read_owned::<BE, BE>(&data).unwrap();

    // Test as_*_mut_scoped methods on MutableValue
    let mut st_mv = doc.get_mut("st").unwrap();
    let mut st_mut = ScopedWritableValue::as_string_mut_scoped(&mut st_mv).unwrap();
    st_mut.push_str("!");

    let mut lb_mv = doc.get_mut("lb").unwrap();
    let _lb_mut = ScopedWritableValue::as_list_mut_scoped(&mut lb_mv).unwrap();

    let mut c_mv = doc.get_mut("c").unwrap();
    let _c_mut = ScopedWritableValue::as_compound_mut_scoped(&mut c_mv).unwrap();

    let mut ba_mv = doc.get_mut("ba").unwrap();
    let ba_mut = ba_mv.as_byte_array_mut().unwrap();
    ba_mut[0] = 99;

    let mut ia_mv = doc.get_mut("ia").unwrap();
    let ia_mut = ia_mv.as_int_array_mut().unwrap();
    ia_mut[0].set(999);

    let mut la_mv = doc.get_mut("la").unwrap();
    let la_mut = la_mv.as_long_array_mut().unwrap();
    la_mut[0].set(9999);
}

#[test]
fn test_mutable_list_trait_methods() {
    let data = create_all_types_be();
    let mut doc = read_owned::<BE, BE>(&data).unwrap();

    let mut lb_mv = doc.get_mut("lb").unwrap();
    let mut list = ScopedWritableValue::as_list_mut_scoped(&mut lb_mv).unwrap();

    // Test ScopedReadableList methods on MutableList
    assert_eq!(ScopedReadableList::tag_id(&list), Tag::Byte);
    assert_eq!(ScopedReadableList::len(&list), 2);
    assert!(!ScopedReadableList::is_empty(&list));

    let item = ScopedReadableList::get_scoped(&list, 0).unwrap();
    assert_eq!(ScopedReadableValue::as_byte(&item), Some(10));

    // Test ScopedWritableList methods
    let mut item_mut = list.get_mut(0).unwrap();
    ScopedWritableValue::set_byte(&mut item_mut, 99);
    let item_check = ScopedReadableList::get_scoped(&list, 0).unwrap();
    assert_eq!(ScopedReadableValue::as_byte(&item_check), Some(99));
}

#[test]
fn test_mutable_compound_trait_methods() {
    let data = create_all_types_be();
    let mut doc = read_owned::<BE, BE>(&data).unwrap();

    let mut c_mv = doc.get_mut("c").unwrap();
    let mut compound = ScopedWritableValue::as_compound_mut_scoped(&mut c_mv).unwrap();

    // Test ScopedReadableCompound methods on MutableCompound
    let x = ScopedReadableCompound::get_scoped(&compound, "x").unwrap();
    assert_eq!(ScopedReadableValue::as_byte(&x), Some(42));

    // Test ScopedWritableCompound methods
    let mut x_mut = compound.get_mut("x").unwrap();
    ScopedWritableValue::set_byte(&mut x_mut, 99);
    let x_check = ScopedReadableCompound::get_scoped(&compound, "x").unwrap();
    assert_eq!(ScopedReadableValue::as_byte(&x_check), Some(99));
}

#[test]
fn test_mutable_value_get_scoped() {
    let data = create_all_types_be();
    let mut doc = read_owned::<BE, BE>(&data).unwrap();

    // get_scoped on MutableValue
    let c_mv = doc.get_mut("c").unwrap();
    let x = ScopedReadableValue::get_scoped(&c_mv, "x").unwrap();
    assert_eq!(ScopedReadableValue::as_byte(&x), Some(42));
}

// ============ OwnedValue trait implementation tests ============

/// Test ScopedReadableValue trait on OwnedValue
#[test]
fn test_owned_value_scoped_readable_trait_is_methods() {
    let data = create_all_types_be();
    let doc = read_owned::<BE, BE>(&data).unwrap();

    // First get the compound root to access typed values
    let root_compound = doc.as_compound().unwrap();

    // Get individual OwnedValues by extracting from list (which we can iterate)
    // For direct OwnedValue trait testing, we need to test on values we own
    // But doc.get() returns ImmutableValue, not &OwnedValue
    // So we test using the ImmutableValue (from mutable module) instead
    // which already has ScopedReadableValue impl

    // Let's create individual OwnedValue values to test trait on them
    let byte_val = OwnedValue::<BE>::Byte(0x7F);
    let short_val = OwnedValue::<BE>::Short(zerocopy::byteorder::I16::<BE>::new(0x1234));
    let int_val = OwnedValue::<BE>::Int(zerocopy::byteorder::I32::<BE>::new(0x12345678));
    let long_val = OwnedValue::<BE>::Long(zerocopy::byteorder::I64::<BE>::new(
        0x123456789ABCDEF0u64 as i64,
    ));
    let float_val = OwnedValue::<BE>::Float(zerocopy::byteorder::F32::<BE>::new(1.5));
    let double_val = OwnedValue::<BE>::Double(zerocopy::byteorder::F64::<BE>::new(2.5));
    let end_val = OwnedValue::<BE>::End;

    // Test each is_* method via trait on OwnedValue directly
    assert!(ScopedReadableValue::is_byte(&byte_val));
    assert!(ScopedReadableValue::is_short(&short_val));
    assert!(ScopedReadableValue::is_int(&int_val));
    assert!(ScopedReadableValue::is_long(&long_val));
    assert!(ScopedReadableValue::is_float(&float_val));
    assert!(ScopedReadableValue::is_double(&double_val));
    assert!(ScopedReadableValue::is_end(&end_val));

    // Also test is_* returning false
    assert!(!ScopedReadableValue::is_short(&byte_val));
    assert!(!ScopedReadableValue::is_int(&byte_val));
}

#[test]
fn test_owned_value_scoped_readable_trait_as_methods() {
    // Create OwnedValue instances directly to test trait methods on OwnedValue type
    let byte_val = OwnedValue::<BE>::Byte(0x7F);
    let short_val = OwnedValue::<BE>::Short(zerocopy::byteorder::I16::<BE>::new(0x1234));
    let int_val = OwnedValue::<BE>::Int(zerocopy::byteorder::I32::<BE>::new(0x12345678));
    let long_val = OwnedValue::<BE>::Long(zerocopy::byteorder::I64::<BE>::new(
        0x123456789ABCDEF0u64 as i64,
    ));
    let float_val = OwnedValue::<BE>::Float(zerocopy::byteorder::F32::<BE>::new(1.5));
    let double_val = OwnedValue::<BE>::Double(zerocopy::byteorder::F64::<BE>::new(2.5));

    // Test each as_* method via trait on OwnedValue
    assert_eq!(ScopedReadableValue::as_byte(&byte_val), Some(0x7F));
    assert_eq!(ScopedReadableValue::as_short(&short_val), Some(0x1234));
    assert_eq!(ScopedReadableValue::as_int(&int_val), Some(0x12345678));
    assert_eq!(
        ScopedReadableValue::as_long(&long_val),
        Some(0x123456789ABCDEF0u64 as i64)
    );
    assert!(ScopedReadableValue::as_float(&float_val).is_some());
    assert!(ScopedReadableValue::as_double(&double_val).is_some());

    // Test as_end on End
    let end_val = OwnedValue::<BE>::End;
    assert_eq!(ScopedReadableValue::as_end(&end_val), Some(()));

    // Test type mismatch returns None
    assert!(ScopedReadableValue::as_short(&byte_val).is_none());
}

#[test]
fn test_owned_value_scoped_readable_trait_tag_id() {
    // Create OwnedValue instances directly
    let byte_val = OwnedValue::<BE>::Byte(0x7F);
    let short_val = OwnedValue::<BE>::Short(zerocopy::byteorder::I16::<BE>::new(0x1234));
    let int_val = OwnedValue::<BE>::Int(zerocopy::byteorder::I32::<BE>::new(0x12345678));
    let long_val = OwnedValue::<BE>::Long(zerocopy::byteorder::I64::<BE>::new(
        0x123456789ABCDEF0u64 as i64,
    ));
    let float_val = OwnedValue::<BE>::Float(zerocopy::byteorder::F32::<BE>::new(1.5));
    let double_val = OwnedValue::<BE>::Double(zerocopy::byteorder::F64::<BE>::new(2.5));
    let end_val = OwnedValue::<BE>::End;

    assert_eq!(ScopedReadableValue::tag_id(&byte_val), Tag::Byte);
    assert_eq!(ScopedReadableValue::tag_id(&short_val), Tag::Short);
    assert_eq!(ScopedReadableValue::tag_id(&int_val), Tag::Int);
    assert_eq!(ScopedReadableValue::tag_id(&long_val), Tag::Long);
    assert_eq!(ScopedReadableValue::tag_id(&float_val), Tag::Float);
    assert_eq!(ScopedReadableValue::tag_id(&double_val), Tag::Double);
    assert_eq!(ScopedReadableValue::tag_id(&end_val), Tag::End);
}

#[test]
fn test_owned_value_scoped_readable_trait_visit_scoped() {
    // Create OwnedValue instances directly to test trait on OwnedValue type
    let byte_val = OwnedValue::<BE>::Byte(0x7F);
    let res =
        ScopedReadableValue::visit_scoped(&byte_val, |v| matches!(v, ValueScoped::Byte(0x7F)));
    assert!(res);

    let short_val = OwnedValue::<BE>::Short(zerocopy::byteorder::I16::<BE>::new(0x1234));
    let res =
        ScopedReadableValue::visit_scoped(&short_val, |v| matches!(v, ValueScoped::Short(0x1234)));
    assert!(res);

    let int_val = OwnedValue::<BE>::Int(zerocopy::byteorder::I32::<BE>::new(0x12345678));
    let res =
        ScopedReadableValue::visit_scoped(&int_val, |v| matches!(v, ValueScoped::Int(0x12345678)));
    assert!(res);

    let long_val = OwnedValue::<BE>::Long(zerocopy::byteorder::I64::<BE>::new(
        0x123456789ABCDEF0u64 as i64,
    ));
    let res = ScopedReadableValue::visit_scoped(&long_val, |v| matches!(v, ValueScoped::Long(_)));
    assert!(res);

    let float_val = OwnedValue::<BE>::Float(zerocopy::byteorder::F32::<BE>::new(1.5));
    let res = ScopedReadableValue::visit_scoped(&float_val, |v| matches!(v, ValueScoped::Float(_)));
    assert!(res);

    let double_val = OwnedValue::<BE>::Double(zerocopy::byteorder::F64::<BE>::new(2.5));
    let res =
        ScopedReadableValue::visit_scoped(&double_val, |v| matches!(v, ValueScoped::Double(_)));
    assert!(res);

    let end_val = OwnedValue::<BE>::End;
    let res = ScopedReadableValue::visit_scoped(&end_val, |v| matches!(v, ValueScoped::End));
    assert!(res);
}

#[test]
fn test_owned_value_scoped_readable_trait_get_scoped() {
    // Test get_scoped on OwnedValue directly (when it's a compound or list)
    let data = create_all_types_be();
    let doc = read_owned::<BE, BE>(&data).unwrap(); // doc is OwnedValue

    // get_scoped by string key on OwnedValue (compound root)
    let x = ScopedReadableValue::get_scoped(&doc, "b").unwrap();
    assert_eq!(x.as_byte(), Some(0x7F));

    // Also test on list within compound
    let list = doc.get("lb").unwrap().as_list().unwrap();
    // OwnedList::get is tested via its trait impl
}

/// Test ScopedWritableValue trait on OwnedValue directly
#[test]
fn test_owned_value_scoped_writable_trait_set_methods() {
    // Create individual OwnedValue instances to test trait on them directly
    let mut byte_val = OwnedValue::<BE>::Byte(0x7F);
    assert!(ScopedWritableValue::set_byte(&mut byte_val, 99));
    assert_eq!(byte_val.as_byte(), Some(99));

    let mut short_val = OwnedValue::<BE>::Short(zerocopy::byteorder::I16::<BE>::new(0x1234));
    assert!(ScopedWritableValue::set_short(&mut short_val, 9999));
    assert_eq!(short_val.as_short(), Some(9999));

    let mut int_val = OwnedValue::<BE>::Int(zerocopy::byteorder::I32::<BE>::new(0x12345678));
    assert!(ScopedWritableValue::set_int(&mut int_val, 123456));
    assert_eq!(int_val.as_int(), Some(123456));

    let mut long_val = OwnedValue::<BE>::Long(zerocopy::byteorder::I64::<BE>::new(
        0x123456789ABCDEF0u64 as i64,
    ));
    assert!(ScopedWritableValue::set_long(
        &mut long_val,
        999999999999i64
    ));
    assert_eq!(long_val.as_long(), Some(999999999999i64));

    let mut float_val = OwnedValue::<BE>::Float(zerocopy::byteorder::F32::<BE>::new(1.5));
    assert!(ScopedWritableValue::set_float(&mut float_val, 2.5f32));
    assert!((float_val.as_float().unwrap() - 2.5f32).abs() < 0.001);

    let mut double_val = OwnedValue::<BE>::Double(zerocopy::byteorder::F64::<BE>::new(2.5));
    assert!(ScopedWritableValue::set_double(&mut double_val, 3.5f64));
    assert!((double_val.as_double().unwrap() - 3.5f64).abs() < 0.001);

    // Test set on wrong type returns false
    let mut byte_val2 = OwnedValue::<BE>::Byte(0x7F);
    assert!(!ScopedWritableValue::set_short(&mut byte_val2, 1234));
}

#[test]
fn test_owned_value_scoped_writable_trait_update_methods() {
    // Create individual OwnedValue instances to test trait on them directly
    let mut byte_val = OwnedValue::<BE>::Byte(0x7F);
    assert!(ScopedWritableValue::update_byte(&mut byte_val, |b| b.wrapping_add(1)));
    assert_eq!(byte_val.as_byte(), Some(-128)); // 127 + 1 wraps to -128

    let mut short_val = OwnedValue::<BE>::Short(zerocopy::byteorder::I16::<BE>::new(0x1234));
    assert!(ScopedWritableValue::update_short(&mut short_val, |s| s + 1));
    assert_eq!(short_val.as_short(), Some(0x1235));

    let mut int_val = OwnedValue::<BE>::Int(zerocopy::byteorder::I32::<BE>::new(0x12345678));
    assert!(ScopedWritableValue::update_int(&mut int_val, |i| i + 1));
    assert_eq!(int_val.as_int(), Some(0x12345679));

    let mut long_val = OwnedValue::<BE>::Long(zerocopy::byteorder::I64::<BE>::new(
        0x123456789ABCDEF0u64 as i64,
    ));
    assert!(ScopedWritableValue::update_long(&mut long_val, |l| l.wrapping_add(1)));

    let mut float_val = OwnedValue::<BE>::Float(zerocopy::byteorder::F32::<BE>::new(1.5));
    assert!(ScopedWritableValue::update_float(&mut float_val, |f| f + 1.0));

    let mut double_val = OwnedValue::<BE>::Double(zerocopy::byteorder::F64::<BE>::new(2.5));
    assert!(ScopedWritableValue::update_double(&mut double_val, |d| d + 1.0));

    // Test update on wrong type returns false
    let mut byte_val2 = OwnedValue::<BE>::Byte(0x7F);
    assert!(!ScopedWritableValue::update_short(&mut byte_val2, |s| s + 1));
}

#[test]
fn test_owned_value_scoped_writable_trait_as_mut_methods() {
    // Test as_*_mut methods on OwnedValue directly
    let mut byte_val = OwnedValue::<BE>::Byte(0x7F);
    let byte_ref = ScopedWritableValue::as_byte_mut(&mut byte_val).unwrap();
    *byte_ref = 50;
    assert_eq!(byte_val.as_byte(), Some(50));

    let mut short_val = OwnedValue::<BE>::Short(zerocopy::byteorder::I16::<BE>::new(0x1234));
    let short_ref = ScopedWritableValue::as_short_mut(&mut short_val).unwrap();
    short_ref.set(5000);
    assert_eq!(short_val.as_short(), Some(5000));

    let mut int_val = OwnedValue::<BE>::Int(zerocopy::byteorder::I32::<BE>::new(0x12345678));
    let int_ref = ScopedWritableValue::as_int_mut(&mut int_val).unwrap();
    int_ref.set(50000);
    assert_eq!(int_val.as_int(), Some(50000));

    let mut long_val = OwnedValue::<BE>::Long(zerocopy::byteorder::I64::<BE>::new(
        0x123456789ABCDEF0u64 as i64,
    ));
    let long_ref = ScopedWritableValue::as_long_mut(&mut long_val).unwrap();
    long_ref.set(5000000000i64);
    assert_eq!(long_val.as_long(), Some(5000000000i64));

    let mut float_val = OwnedValue::<BE>::Float(zerocopy::byteorder::F32::<BE>::new(1.5));
    let float_ref = ScopedWritableValue::as_float_mut(&mut float_val).unwrap();
    float_ref.set(99.0f32);
    assert!((float_val.as_float().unwrap() - 99.0f32).abs() < 0.001);

    let mut double_val = OwnedValue::<BE>::Double(zerocopy::byteorder::F64::<BE>::new(2.5));
    let double_ref = ScopedWritableValue::as_double_mut(&mut double_val).unwrap();
    double_ref.set(999.0f64);
    assert!((double_val.as_double().unwrap() - 999.0f64).abs() < 0.001);

    // Test on wrong type returns None
    let mut byte_val2 = OwnedValue::<BE>::Byte(0x7F);
    assert!(ScopedWritableValue::as_short_mut(&mut byte_val2).is_none());
}

#[test]
fn test_owned_value_scoped_writable_trait_array_mut_methods() {
    let data = create_all_types_be();
    let mut doc = read_owned::<BE, BE>(&data).unwrap(); // doc is OwnedValue

    // Test get_mut via trait on OwnedValue (doc is compound)
    let mut ba_val = ScopedWritableValue::get_mut(&mut doc, "ba").unwrap();
    assert!(ba_val.is_byte_array());
    drop(ba_val);

    // Test as_compound_mut_scoped via trait on OwnedValue
    let compound_view = ScopedWritableValue::as_compound_mut_scoped(&mut doc).unwrap();
    assert!(compound_view.get("b").is_some());
    drop(compound_view);

    // For array mut methods on OwnedValue, we need OwnedValue variants directly
    // Create an OwnedValue::ByteArray to test
    // Note: OwnedByteArray is private, so we test via parsed data

    let mut lib = doc.get_mut("lb").unwrap();
    // Actually, test via parsed list
    let mut list = lib.as_list_mut().unwrap();
    assert!(ScopedReadableList::len(list) == 2);
}

#[test]
fn test_owned_value_scoped_writable_trait_get_mut() {
    let data = create_all_types_be();
    let mut doc = read_owned::<BE, BE>(&data).unwrap();

    // Test get_mut via trait on compound
    let mut compound_val = doc.get_mut("c").unwrap();
    let mut x_val = ScopedWritableValue::get_mut(&mut compound_val, "x").unwrap();
    ScopedWritableValue::set_byte(&mut x_val, 99);
    drop(x_val);
    drop(compound_val);

    // Verify
    let x_check = doc.get("c").unwrap().get("x").unwrap();
    assert_eq!(x_check.as_byte(), Some(99));

    // Test get_mut via trait on list
    let mut list_val = doc.get_mut("lb").unwrap();
    let mut item = ScopedWritableValue::get_mut(&mut list_val, 0usize).unwrap();
    ScopedWritableValue::set_byte(&mut item, 77);
    drop(item);
    drop(list_val);

    // Verify
    let item_check = doc.get("lb").unwrap().get(0).unwrap();
    assert_eq!(item_check.as_byte(), Some(77));
}

/// Test ScopedReadableList trait on OwnedList
#[test]
fn test_owned_list_scoped_readable_trait() {
    let data = create_all_types_be();
    let doc = read_owned::<BE, BE>(&data).unwrap();

    let lb = doc.get("lb").unwrap();
    let list = lb.as_list().unwrap();

    // Test all ScopedReadableList methods via trait
    assert_eq!(ScopedReadableList::tag_id(list), Tag::Byte);
    assert_eq!(ScopedReadableList::len(list), 2);
    assert!(!ScopedReadableList::is_empty(list));

    let item = ScopedReadableList::get_scoped(list, 0).unwrap();
    assert_eq!(item.as_byte(), Some(10));

    // Test iter_scoped
    let iter = ScopedReadableList::iter_scoped(list);
    assert_eq!(iter.count(), 2);
}

/// Test ScopedWritableList trait on OwnedList
#[test]
fn test_owned_list_scoped_writable_trait() {
    let data = create_all_types_be();
    let mut doc = read_owned::<BE, BE>(&data).unwrap();

    let mut lb = doc.get_mut("lb").unwrap();
    let mut list = lb.as_list_mut().unwrap();

    // Test get_mut via trait
    let mut item = ScopedWritableList::get_mut(list, 0).unwrap();
    ScopedWritableValue::set_byte(&mut item, 99);
    drop(item);

    // Verify
    let item_check = ScopedReadableList::get_scoped(list, 0).unwrap();
    assert_eq!(ScopedReadableValue::as_byte(&item_check), Some(99));

    // Test push via trait
    ScopedWritableList::push(list, 55i8);
    assert_eq!(ScopedReadableList::len(list), 3);

    // Test insert via trait
    ScopedWritableList::insert(list, 0, 11i8);
    assert_eq!(ScopedReadableList::len(list), 4);

    // Test pop via trait
    let popped = ScopedWritableList::pop(list).unwrap();
    assert_eq!(popped.as_byte(), Some(55));

    // Test remove via trait
    let removed = ScopedWritableList::remove(list, 0);
    assert_eq!(removed.as_byte(), Some(11));

    // Test iter_mut via trait
    let iter_mut = ScopedWritableList::iter_mut(list);
    assert_eq!(iter_mut.count(), 2);
}

/// Test ScopedReadableCompound trait on OwnedCompound
#[test]
fn test_owned_compound_scoped_readable_trait() {
    let data = create_all_types_be();
    let doc = read_owned::<BE, BE>(&data).unwrap();

    let c = doc.get("c").unwrap();
    let compound = c.as_compound().unwrap();

    // Test get_scoped via trait
    let x = ScopedReadableCompound::get_scoped(compound, "x").unwrap();
    assert_eq!(x.as_byte(), Some(42));

    // Test iter_scoped via trait
    let iter = ScopedReadableCompound::iter_scoped(compound);
    assert_eq!(iter.count(), 1);
}

/// Test ScopedWritableCompound trait on OwnedCompound
#[test]
fn test_owned_compound_scoped_writable_trait() {
    let data = create_all_types_be();
    let mut doc = read_owned::<BE, BE>(&data).unwrap();

    let mut c = doc.get_mut("c").unwrap();
    let mut compound = c.as_compound_mut().unwrap();

    // Test get_mut via trait
    let mut x_mut = ScopedWritableCompound::get_mut(compound, "x").unwrap();
    ScopedWritableValue::set_byte(&mut x_mut, 88);
    drop(x_mut);

    // Verify
    let x_check = ScopedReadableCompound::get_scoped(compound, "x").unwrap();
    assert_eq!(ScopedReadableValue::as_byte(&x_check), Some(88));

    // Test insert via trait
    let old = ScopedWritableCompound::insert(compound, "y", 123i8);
    assert!(old.is_none());

    // Verify insert worked
    let y = ScopedReadableCompound::get_scoped(compound, "y").unwrap();
    assert_eq!(ScopedReadableValue::as_byte(&y), Some(123));

    // Test remove via trait
    let removed = ScopedWritableCompound::remove(compound, "y").unwrap();
    assert_eq!(removed.as_byte(), Some(123));

    // Test iter_mut via trait
    let iter_mut = ScopedWritableCompound::iter_mut(compound);
    assert_eq!(iter_mut.count(), 1);
}

/// Test OwnedValue with End tag
#[test]
fn test_owned_value_end_trait() {
    // Create an End tag value
    let data = vec![0x00]; // End tag
    let doc = read_owned::<BE, BE>(&data).unwrap();

    assert!(ScopedReadableValue::is_end(&doc));
    assert_eq!(ScopedReadableValue::as_end(&doc), Some(()));
    assert_eq!(ScopedReadableValue::tag_id(&doc), Tag::End);

    // visit_scoped on End
    let res = ScopedReadableValue::visit_scoped(&doc, |v| matches!(v, ValueScoped::End));
    assert!(res);
}

/// Test visit_mut_scoped on OwnedValue
#[test]
fn test_owned_value_visit_mut_scoped() {
    use na_nbt::ValueMutScoped;

    let data = create_all_types_be();
    let mut doc = read_owned::<BE, BE>(&data).unwrap();

    // Test visit_mut_scoped on byte
    let mut byte_val = doc.get_mut("b").unwrap();
    ScopedWritableValue::visit_mut_scoped(&mut byte_val, |v| {
        if let ValueMutScoped::Byte(b) = v {
            *b = 42;
        }
    });
    drop(byte_val);
    assert_eq!(doc.get("b").unwrap().as_byte(), Some(42));

    // Test visit_mut_scoped on short
    let mut short_val = doc.get_mut("s").unwrap();
    ScopedWritableValue::visit_mut_scoped(&mut short_val, |v| {
        if let ValueMutScoped::Short(s) = v {
            s.set(4242);
        }
    });
    drop(short_val);
    assert_eq!(doc.get("s").unwrap().as_short(), Some(4242));

    // Test visit_mut_scoped on int
    let mut int_val = doc.get_mut("i").unwrap();
    ScopedWritableValue::visit_mut_scoped(&mut int_val, |v| {
        if let ValueMutScoped::Int(i) = v {
            i.set(424242);
        }
    });
    drop(int_val);
    assert_eq!(doc.get("i").unwrap().as_int(), Some(424242));

    // Test visit_mut_scoped on long
    let mut long_val = doc.get_mut("l").unwrap();
    ScopedWritableValue::visit_mut_scoped(&mut long_val, |v| {
        if let ValueMutScoped::Long(l) = v {
            l.set(42424242i64);
        }
    });
    drop(long_val);
    assert_eq!(doc.get("l").unwrap().as_long(), Some(42424242i64));

    // Test visit_mut_scoped on float
    let mut float_val = doc.get_mut("f").unwrap();
    ScopedWritableValue::visit_mut_scoped(&mut float_val, |v| {
        if let ValueMutScoped::Float(f) = v {
            f.set(42.42f32);
        }
    });
    drop(float_val);

    // Test visit_mut_scoped on double
    let mut double_val = doc.get_mut("d").unwrap();
    ScopedWritableValue::visit_mut_scoped(&mut double_val, |v| {
        if let ValueMutScoped::Double(d) = v {
            d.set(42.4242f64);
        }
    });
    drop(double_val);

    // Test visit_mut_scoped on byte_array
    let mut ba_val = doc.get_mut("ba").unwrap();
    ScopedWritableValue::visit_mut_scoped(&mut ba_val, |v| {
        if let ValueMutScoped::ByteArray(arr) = v {
            assert_eq!(arr.len(), 3);
        }
    });
    drop(ba_val);

    // Test visit_mut_scoped on string
    let mut str_val = doc.get_mut("st").unwrap();
    ScopedWritableValue::visit_mut_scoped(&mut str_val, |v| {
        if let ValueMutScoped::String(mut s) = v {
            s.push_str("!");
        }
    });
    drop(str_val);

    // Test visit_mut_scoped on list
    let mut list_val = doc.get_mut("lb").unwrap();
    ScopedWritableValue::visit_mut_scoped(&mut list_val, |v| {
        if let ValueMutScoped::List(list) = v {
            assert_eq!(list.len(), 2);
        }
    });
    drop(list_val);

    // Test visit_mut_scoped on compound
    let mut compound_val = doc.get_mut("c").unwrap();
    ScopedWritableValue::visit_mut_scoped(&mut compound_val, |v| {
        if let ValueMutScoped::Compound(compound) = v {
            let _ = compound.get("x");
        }
    });
    drop(compound_val);

    // Test visit_mut_scoped on int_array
    let mut ia_val = doc.get_mut("ia").unwrap();
    ScopedWritableValue::visit_mut_scoped(&mut ia_val, |v| {
        if let ValueMutScoped::IntArray(arr) = v {
            assert_eq!(arr.len(), 2);
        }
    });
    drop(ia_val);

    // Test visit_mut_scoped on long_array
    let mut la_val = doc.get_mut("la").unwrap();
    ScopedWritableValue::visit_mut_scoped(&mut la_val, |v| {
        if let ValueMutScoped::LongArray(arr) = v {
            assert_eq!(arr.len(), 2);
        }
    });
    drop(la_val);
}
