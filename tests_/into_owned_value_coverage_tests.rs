use na_nbt::{OwnedCompound, OwnedList, OwnedValue, TagID};
use zerocopy::byteorder::BigEndian as BE;
use zerocopy::byteorder::{F32, F64, I16, I32, I64};

macro_rules! test_into_owned {
    ($name:ident, $val:expr, $tag:expr) => {
        #[test]
        fn $name() {
            // compound_insert
            let mut c = OwnedCompound::<BE>::default();
            c.insert("k", $val);
            assert!(c.get("k").is_some());
            assert_eq!(c.get("k").unwrap().tag_id(), $tag);

            // list_push
            let mut l = OwnedList::<BE>::default();
            l.push($val);
            assert_eq!(l.len(), 1);
            assert_eq!(l.tag_id(), $tag);

            // list_insert
            let mut l = OwnedList::<BE>::default();
            l.push($val); // set type
            l.insert(0, $val);
            assert_eq!(l.len(), 2);

            // list_push_unchecked
            let mut l = OwnedList::<BE>::default();
            l.push($val); // set type
            unsafe { l.push_unchecked($val) };
            assert_eq!(l.len(), 2);

            // list_insert_unchecked
            let mut l = OwnedList::<BE>::default();
            l.push($val); // set type
            unsafe { l.insert_unchecked(0, $val) };
            assert_eq!(l.len(), 2);
        }
    };
}

// test_into_owned!(test_unit, (), Tag::End); // Panics

#[test]
#[should_panic(expected = "cannot insert TAG_END")]
fn test_unit_compound_insert() {
    let mut c = OwnedCompound::<BE>::default();
    c.insert("k", ());
}

#[test]
fn test_unit_list_push() {
    let mut l = OwnedList::<BE>::default();
    l.push(());
}

test_into_owned!(test_i8, 10i8, TagID::Byte);
test_into_owned!(test_i16_native, 10i16, TagID::Short);
test_into_owned!(test_i16_wrapped, I16::<BE>::new(10), TagID::Short);
test_into_owned!(test_i32_native, 10i32, TagID::Int);
test_into_owned!(test_i32_wrapped, I32::<BE>::new(10), TagID::Int);
test_into_owned!(test_i64_native, 10i64, TagID::Long);
test_into_owned!(test_i64_wrapped, I64::<BE>::new(10), TagID::Long);
test_into_owned!(test_f32_native, 10.0f32, TagID::Float);
test_into_owned!(test_f32_wrapped, F32::<BE>::new(10.0), TagID::Float);
test_into_owned!(test_f64_native, 10.0f64, TagID::Double);
test_into_owned!(test_f64_wrapped, F64::<BE>::new(10.0), TagID::Double);

// Arrays/Vectors need clone or recreation in macro if passed as expr, or handle separately
#[test]
fn test_byte_arrays() {
    let v = vec![1i8, 2, 3];
    let s = v.as_slice();
    let a = [1i8, 2, 3];

    // Vec<i8>
    let mut c = OwnedCompound::<BE>::default();
    c.insert("k", v.clone());
    assert_eq!(c.get("k").unwrap().tag_id(), TagID::ByteArray);

    // &[i8]
    let mut c = OwnedCompound::<BE>::default();
    c.insert("k", s);
    assert_eq!(c.get("k").unwrap().tag_id(), TagID::ByteArray);

    // [i8; 3]
    let mut c = OwnedCompound::<BE>::default();
    c.insert("k", a);
    assert_eq!(c.get("k").unwrap().tag_id(), TagID::ByteArray);

    // Test list ops for Vec<i8>
    let mut l = OwnedList::<BE>::default();
    l.push(v.clone());
    l.insert(0, v.clone());
    unsafe { l.push_unchecked(v.clone()) };
    unsafe { l.insert_unchecked(0, v.clone()) };
    assert_eq!(l.len(), 4);

    // Test list ops for &[i8]
    let mut l = OwnedList::<BE>::default();
    l.push(s);
    l.insert(0, s);
    unsafe { l.push_unchecked(s) };
    unsafe { l.insert_unchecked(0, s) };
    assert_eq!(l.len(), 4);

    // Test list ops for [i8; 3]
    let mut l = OwnedList::<BE>::default();
    l.push(a);
    l.insert(0, a);
    unsafe { l.push_unchecked(a) };
    unsafe { l.insert_unchecked(0, a) };
    assert_eq!(l.len(), 4);
}

#[test]
fn test_strings() {
    let s_owned = String::from("foo");
    let s_borrowed = "bar";

    // String
    let mut c = OwnedCompound::<BE>::default();
    c.insert("k", s_owned.clone());
    assert_eq!(c.get("k").unwrap().tag_id(), TagID::String);

    let mut l = OwnedList::<BE>::default();
    l.push(s_owned.clone());
    l.insert(0, s_owned.clone());
    unsafe { l.push_unchecked(s_owned.clone()) };
    unsafe { l.insert_unchecked(0, s_owned.clone()) };
    assert_eq!(l.len(), 4);

    // &str
    let mut c = OwnedCompound::<BE>::default();
    c.insert("k", s_borrowed);
    assert_eq!(c.get("k").unwrap().tag_id(), TagID::String);

    let mut l = OwnedList::<BE>::default();
    l.push(s_borrowed);
    l.insert(0, s_borrowed);
    unsafe { l.push_unchecked(s_borrowed) };
    unsafe { l.insert_unchecked(0, s_borrowed) };
    assert_eq!(l.len(), 4);
}

#[test]
fn test_owned_list_recursive() {
    let _child_l = OwnedList::<BE>::default();

    let mut c = OwnedCompound::<BE>::default();
    // Move child_l into insert
    c.insert("k", OwnedList::<BE>::default());
    assert_eq!(c.get("k").unwrap().tag_id(), TagID::List);

    let mut l = OwnedList::<BE>::default();
    l.push(OwnedList::<BE>::default());
    l.insert(0, OwnedList::<BE>::default());
    unsafe { l.push_unchecked(OwnedList::<BE>::default()) };
    unsafe { l.insert_unchecked(0, OwnedList::<BE>::default()) };
    assert_eq!(l.len(), 4);
}

#[test]
fn test_owned_compound_recursive() {
    let mut c = OwnedCompound::<BE>::default();
    c.insert("k", OwnedCompound::<BE>::default());
    assert_eq!(c.get("k").unwrap().tag_id(), TagID::Compound);

    let mut l = OwnedList::<BE>::default();
    l.push(OwnedCompound::<BE>::default());
    l.insert(0, OwnedCompound::<BE>::default());
    unsafe { l.push_unchecked(OwnedCompound::<BE>::default()) };
    unsafe { l.insert_unchecked(0, OwnedCompound::<BE>::default()) };
    assert_eq!(l.len(), 4);
}

#[test]
fn test_int_arrays() {
    let v = vec![I32::<BE>::new(1)];
    let s = v.as_slice();
    let a = [I32::<BE>::new(1)];

    // Vec
    let mut c = OwnedCompound::<BE>::default();
    c.insert("k", v.clone());
    assert_eq!(c.get("k").unwrap().tag_id(), TagID::IntArray);

    let mut l = OwnedList::<BE>::default();
    l.push(v.clone());
    l.insert(0, v.clone());
    unsafe { l.push_unchecked(v.clone()) };
    unsafe { l.insert_unchecked(0, v.clone()) };
    assert_eq!(l.len(), 4);

    // Slice
    let mut c = OwnedCompound::<BE>::default();
    c.insert("k", s);
    assert_eq!(c.get("k").unwrap().tag_id(), TagID::IntArray);

    let mut l = OwnedList::<BE>::default();
    l.push(s);
    l.insert(0, s);
    unsafe { l.push_unchecked(s) };
    unsafe { l.insert_unchecked(0, s) };
    assert_eq!(l.len(), 4);

    // Array
    let mut c = OwnedCompound::<BE>::default();
    c.insert("k", a);
    assert_eq!(c.get("k").unwrap().tag_id(), TagID::IntArray);

    let mut l = OwnedList::<BE>::default();
    l.push(a);
    l.insert(0, a);
    unsafe { l.push_unchecked(a) };
    unsafe { l.insert_unchecked(0, a) };
    assert_eq!(l.len(), 4);
}

#[test]
fn test_long_arrays() {
    let v = vec![I64::<BE>::new(1)];
    let s = v.as_slice();
    let a = [I64::<BE>::new(1)];

    // Vec
    let mut c = OwnedCompound::<BE>::default();
    c.insert("k", v.clone());
    assert_eq!(c.get("k").unwrap().tag_id(), TagID::LongArray);

    let mut l = OwnedList::<BE>::default();
    l.push(v.clone());
    l.insert(0, v.clone());
    unsafe { l.push_unchecked(v.clone()) };
    unsafe { l.insert_unchecked(0, v.clone()) };
    assert_eq!(l.len(), 4);

    // Slice
    let mut c = OwnedCompound::<BE>::default();
    c.insert("k", s);
    assert_eq!(c.get("k").unwrap().tag_id(), TagID::LongArray);

    let mut l = OwnedList::<BE>::default();
    l.push(s);
    l.insert(0, s);
    unsafe { l.push_unchecked(s) };
    unsafe { l.insert_unchecked(0, s) };
    assert_eq!(l.len(), 4);

    // Array
    let mut c = OwnedCompound::<BE>::default();
    c.insert("k", a);
    assert_eq!(c.get("k").unwrap().tag_id(), TagID::LongArray);

    let mut l = OwnedList::<BE>::default();
    l.push(a);
    l.insert(0, a);
    unsafe { l.push_unchecked(a) };
    unsafe { l.insert_unchecked(0, a) };
    assert_eq!(l.len(), 4);
}

#[test]
fn test_owned_value() {
    let _val = OwnedValue::<BE>::from(10i32);

    let mut c = OwnedCompound::<BE>::default();
    // We need to clone val because it is consumed
    // OwnedValue doesn't implement Clone in test unless I missed it?
    // It should implement Clone if T does? No, OwnedValue is an enum wrapping types.
    // Let's create new ones.

    c.insert("k", OwnedValue::<BE>::from(10i32));
    assert_eq!(c.get("k").unwrap().tag_id(), TagID::Int);

    let mut l = OwnedList::<BE>::default();
    l.push(OwnedValue::<BE>::from(10i32));
    l.insert(0, OwnedValue::<BE>::from(10i32));
    unsafe { l.push_unchecked(OwnedValue::<BE>::from(10i32)) };
    unsafe { l.insert_unchecked(0, OwnedValue::<BE>::from(10i32)) };
    assert_eq!(l.len(), 4);
}
