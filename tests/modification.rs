use na_nbt::{
    CompoundMut, CompoundRef, Error, ListBase, ListMut, ListRef, MapMut, MapRef, MutValue,
    OwnCompound, OwnList, OwnTypedList, OwnValue, RefValue, TagID, TypedListBase, TypedListMut,
    TypedListRef, ValueBase, ValueMut, ValueRef, VisitRef, tag,
};
use zerocopy::byteorder::BigEndian as BE;

trait ValueHelper {
    fn as_int_val(&self) -> Option<i32>;
    fn as_string_val(&self) -> Option<String>;
    fn as_float_val(&self) -> Option<f32>;
}

impl<'a> ValueHelper for RefValue<'a, BE> {
    fn as_int_val(&self) -> Option<i32> {
        match self {
            RefValue::Byte(b) => Some(*b as i32),
            RefValue::Short(s) => Some(*s as i32),
            RefValue::Int(i) => Some(*i),
            RefValue::Long(l) => Some(*l as i32),
            _ => None,
        }
    }
    fn as_string_val(&self) -> Option<String> {
        if let RefValue::String(s) = self {
            Some(s.decode_lossy().into_owned())
        } else {
            None
        }
    }
    fn as_float_val(&self) -> Option<f32> {
        match self {
            RefValue::Float(f) => Some(*f),
            RefValue::Double(d) => Some(*d as f32),
            _ => None,
        }
    }
}

impl ValueHelper for OwnValue<BE> {
    fn as_int_val(&self) -> Option<i32> {
        match self {
            OwnValue::Byte(b) => Some(*b as i32),
            OwnValue::Short(s) => Some(s.get() as i32),
            OwnValue::Int(i) => Some(i.get()),
            OwnValue::Long(l) => Some(l.get() as i32),
            _ => None,
        }
    }
    fn as_string_val(&self) -> Option<String> {
        if let OwnValue::String(s) = self {
            Some(s.decode().into_owned())
        } else {
            None
        }
    }
    fn as_float_val(&self) -> Option<f32> {
        match self {
            OwnValue::Float(f) => Some(f.get()),
            OwnValue::Double(d) => Some(d.get() as f32),
            _ => None,
        }
    }
}

#[test]
fn test_list_primitive_ops() {
    let mut list = OwnList::<BE>::default();
    assert!(list.is_empty());
    assert_eq!(list.len(), 0);

    // Push
    list.push(10i32);
    assert_eq!(list.len(), 1);
    assert!(!list.is_empty());
    assert_eq!(list.get(0).unwrap().as_int_val(), Some(10));

    // Push mismatch type (should be ignored/handled)
    // list is now Int list. pushing float?
    list.push(1.5f32);
    // Should not change length if type mismatch
    assert_eq!(list.len(), 1);
    assert_eq!(list.get(0).unwrap().as_int_val(), Some(10));

    // Pop
    let popped = list.pop();
    assert_eq!(popped.unwrap().as_int_val(), Some(10));
    assert!(list.is_empty());

    // Pop empty
    assert!(list.pop().is_none());

    // Insert
    list.insert(0, 20i32);
    assert_eq!(list.get(0).unwrap().as_int_val(), Some(20));

    list.insert(1, 30i32);
    assert_eq!(list.get(1).unwrap().as_int_val(), Some(30));

    list.insert(1, 25i32); // [20, 25, 30]
    assert_eq!(list.get(0).unwrap().as_int_val(), Some(20));
    assert_eq!(list.get(1).unwrap().as_int_val(), Some(25));
    assert_eq!(list.get(2).unwrap().as_int_val(), Some(30));

    // Insert OOB
    list.insert(100, 40i32);
    assert_eq!(list.len(), 3);

    // Remove
    let removed = list.remove(1); // 25
    assert_eq!(removed.unwrap().as_int_val(), Some(25));
    assert_eq!(list.len(), 2);
    assert_eq!(list.get(0).unwrap().as_int_val(), Some(20));
    assert_eq!(list.get(1).unwrap().as_int_val(), Some(30));

    // Remove OOB
    assert!(list.remove(100).is_none());
}

#[test]
fn test_list_string_ops() {
    let mut list = OwnList::<BE>::default();
    list.push("hello");
    list.push("world");

    assert_eq!(list.len(), 2);
    assert_eq!(list.get(0).unwrap().as_string_val().unwrap(), "hello");

    list.insert(1, "middle");
    assert_eq!(list.get(1).unwrap().as_string_val().unwrap(), "middle");

    let removed = list.remove(0);
    assert_eq!(removed.unwrap().as_string_val().unwrap(), "hello");
    assert_eq!(list.get(0).unwrap().as_string_val().unwrap(), "middle");
}

#[test]
fn test_list_compound_ops() {
    let mut list = OwnList::<BE>::default();

    let mut c1 = OwnCompound::<BE>::default();
    c1.insert("id", 1i32);
    list.push(c1);

    let mut c2 = OwnCompound::<BE>::default();
    c2.insert("id", 2i32);
    list.push(c2);

    assert_eq!(list.len(), 2);

    let v1 = list.get(0).unwrap();
    if let RefValue::Compound(c1_ref) = v1 {
        assert_eq!(c1_ref.get_::<na_nbt::tag::Int>("id").unwrap(), 1);
    } else {
        panic!("expected compound");
    }
}

#[test]
fn test_compound_ops() {
    let mut comp = OwnCompound::<BE>::default();

    comp.insert("a", 1i32);
    comp.insert("b", "string");

    assert_eq!(comp.get("a").unwrap().as_int_val(), Some(1));
    assert_eq!(comp.get("b").unwrap().as_string_val().unwrap(), "string");

    // Replace different type
    let old = comp.insert("a", 1.5f32);
    assert_eq!(old.unwrap().as_int_val(), Some(1));
    assert_eq!(comp.get("a").unwrap().as_float_val(), Some(1.5));

    // Remove
    let removed = comp.remove("b");
    assert_eq!(removed.unwrap().as_string_val().unwrap(), "string");
    assert!(comp.get("b").is_none());

    // Remove non-existent
    assert!(comp.remove("z").is_none());
}

#[test]
fn test_compound_nested() {
    let mut comp = OwnCompound::<BE>::default();

    let mut inner = OwnCompound::<BE>::default();
    inner.insert("val", 10i32);

    comp.insert("inner", inner);

    // get inner
    let inner_val = comp.get("inner").unwrap();
    if let RefValue::Compound(inner_ref) = inner_val {
        assert_eq!(inner_ref.get_::<na_nbt::tag::Int>("val").unwrap(), 10);
    } else {
        panic!("expected compound");
    }

    // remove inner
    let removed = comp.remove("inner").unwrap();
    if let OwnValue::Compound(c) = removed {
        assert_eq!(c.get("val").unwrap().as_int_val(), Some(10));
    } else {
        panic!("expected compound");
    }
}

#[test]
fn test_typed_list() {
    let mut list = OwnList::<BE>::default();
    list.push(1i32);
    list.push(2i32);

    let mut typed = list.typed_::<na_nbt::tag::Int>().unwrap();

    typed.push(3i32);
    assert_eq!(typed.len(), 3);
    assert_eq!(typed.get(0).unwrap(), 1);

    typed.insert(0, 0i32);
    assert_eq!(typed.get(0).unwrap(), 0);

    assert_eq!(typed.pop().unwrap(), 3);
    assert_eq!(typed.remove(0).unwrap(), 0);
    assert_eq!(typed.len(), 2);

    // Into OwnValue
    let val: OwnValue<BE> = typed.into();
    if let OwnValue::List(l) = val {
        assert_eq!(l.len(), 2);
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_large_modifications() {
    let mut list = OwnList::<BE>::default();
    for i in 0..1000 {
        list.push(i);
    }
    assert_eq!(list.len(), 1000);

    for _ in 0..500 {
        list.pop();
    }
    assert_eq!(list.len(), 500);

    let mut comp = OwnCompound::<BE>::default();
    for i in 0..100 {
        comp.insert(&format!("key{}", i), i);
    }

    for i in 0..100 {
        assert_eq!(
            comp.get(&format!("key{}", i)).unwrap().as_int_val(),
            Some(i)
        );
    }

    for i in 0..50 {
        comp.remove(&format!("key{}", i));
    }

    for i in 0..50 {
        assert!(comp.get(&format!("key{}", i)).is_none());
    }
    for i in 50..100 {
        assert!(comp.get(&format!("key{}", i)).is_some());
    }
}

/// Regression test for insert into empty list bug (index >= len instead of index > len)
#[test]
fn test_insert_empty_list_regression() {
    // Test OwnList
    let mut list = OwnList::<BE>::default();
    assert!(list.is_empty());

    // This was broken before: insert at index 0 into empty list
    list.insert(0, 42i32);
    assert_eq!(list.len(), 1);
    assert_eq!(list.get(0).unwrap().as_int_val(), Some(42));

    // Insert at end (index == len)
    list.insert(1, 100i32);
    assert_eq!(list.len(), 2);
    assert_eq!(list.get(1).unwrap().as_int_val(), Some(100));

    // Test OwnTypedList
    let mut list2 = OwnList::<BE>::default();
    list2.push(1i32);
    let mut typed = list2.typed_::<na_nbt::tag::Int>().unwrap();
    typed.pop();
    assert!(typed.is_empty());

    // This was also broken: insert at index 0 into empty typed list
    typed.insert(0, 99i32);
    assert_eq!(typed.len(), 1);
    assert_eq!(typed.get(0).unwrap(), 99);
}

#[test]
fn test_list_all_primitive_types() {
    // Test byte
    let mut list = OwnList::<BE>::default();
    list.push(127i8);
    assert!(matches!(list.get(0).unwrap(), RefValue::Byte(127)));

    // Test short
    let mut list = OwnList::<BE>::default();
    list.push(1000i16);
    assert!(matches!(list.get(0).unwrap(), RefValue::Short(1000)));

    // Test long
    let mut list = OwnList::<BE>::default();
    list.push(1234567890123i64);
    assert!(matches!(
        list.get(0).unwrap(),
        RefValue::Long(1234567890123)
    ));

    // Test double
    let mut list = OwnList::<BE>::default();
    list.push(std::f64::consts::PI);
    if let RefValue::Double(d) = list.get(0).unwrap() {
        assert!((d - std::f64::consts::PI).abs() < 0.0001);
    } else {
        panic!("expected double");
    }
}

#[test]
fn test_list_oob_access() {
    let mut list = OwnList::<BE>::default();
    list.push(1i32);
    list.push(2i32);

    // Valid accesses
    assert!(list.get(0).is_some());
    assert!(list.get(1).is_some());

    // OOB accesses
    assert!(list.get(2).is_none());
    assert!(list.get(100).is_none());
    assert!(list.get(usize::MAX).is_none());

    // OOB remove
    assert!(list.remove(2).is_none());
    assert!(list.remove(100).is_none());
    assert_eq!(list.len(), 2); // unchanged
}

#[test]
fn test_compound_insert_all_types() {
    let mut comp = OwnCompound::<BE>::default();

    comp.insert("byte", 42i8);
    comp.insert("short", 1000i16);
    comp.insert("int", 100000i32);
    comp.insert("long", 1234567890123i64);
    comp.insert("float", 1.5f32);
    comp.insert("double", std::f64::consts::PI);
    comp.insert("string", "hello");

    assert!(matches!(comp.get("byte").unwrap(), RefValue::Byte(42)));
    assert!(matches!(comp.get("short").unwrap(), RefValue::Short(1000)));
    assert!(matches!(comp.get("int").unwrap(), RefValue::Int(100000)));
    assert!(matches!(
        comp.get("long").unwrap(),
        RefValue::Long(1234567890123)
    ));

    if let RefValue::Float(f) = comp.get("float").unwrap() {
        assert!((f - 1.5).abs() < 0.001);
    } else {
        panic!("expected float");
    }
}

#[test]
fn test_list_nested_list() {
    let mut outer = OwnList::<BE>::default();

    let mut inner1 = OwnList::<BE>::default();
    inner1.push(1i32);
    inner1.push(2i32);
    outer.push(inner1);

    let mut inner2 = OwnList::<BE>::default();
    inner2.push(3i32);
    outer.push(inner2);

    assert_eq!(outer.len(), 2);

    if let RefValue::List(l) = outer.get(0).unwrap() {
        assert_eq!(l.len(), 2);
    } else {
        panic!("expected list");
    }

    if let RefValue::List(l) = outer.get(1).unwrap() {
        assert_eq!(l.len(), 1);
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_typed_list_empty_ops() {
    // Create an empty typed list from typed_() on empty list
    let list = OwnList::<BE>::default();
    let mut typed = list.typed_::<na_nbt::tag::Int>().unwrap();

    assert!(typed.is_empty());
    assert_eq!(typed.len(), 0);

    // Pop on empty should return None
    assert!(typed.pop().is_none());

    // Remove on empty should return None
    assert!(typed.remove(0).is_none());

    // Get on empty should return None
    assert!(typed.get(0).is_none());

    // Insert at 0 on empty should work
    typed.insert(0, 1i32);
    assert_eq!(typed.len(), 1);
}

#[test]
fn test_insert_at_end() {
    let mut list = OwnList::<BE>::default();
    list.push(1i32);
    list.push(2i32);
    list.push(3i32);
    // [1, 2, 3]

    // Insert at end (index == len)
    list.insert(3, 4i32);
    assert_eq!(list.len(), 4);
    assert_eq!(list.get(3).unwrap().as_int_val(), Some(4));

    // Insert OOB (index > len)
    list.insert(10, 5i32);
    assert_eq!(list.len(), 4); // unchanged
}

/// Test mutable iteration over lists and compounds
#[test]
fn test_iter_mut_list() {
    // Create a list of ints
    let mut list = OwnList::<BE>::default();
    list.push(1i32);
    list.push(2i32);
    list.push(3i32);

    // Get OwnValue::List
    let mut list_value = OwnValue::<BE>::List(list);

    // Convert to MutValue to use iter_mut
    let mut_value = list_value.to_mut();

    if let MutValue::List(mut mut_list) = mut_value {
        // Use iter_mut to iterate and modify values
        let mut count = 0;
        for item in mut_list.iter_mut() {
            count += 1;
            // Verify we can read the value (I32<BE> needs .get())
            if let MutValue::Int(val) = item {
                let v = val.get();
                assert!((1..=3).contains(&v), "Value {} not in range 1..=3", v);
            }
        }
        assert_eq!(count, 3);

        // Also test into_iter
        let mut into_count = 0;
        for item in mut_list {
            into_count += 1;
            if let MutValue::Int(val) = item {
                let v = val.get();
                assert!((1..=3).contains(&v));
            }
        }
        assert_eq!(into_count, 3);
    } else {
        panic!("expected MutValue::List");
    }

    // Verify original is still valid
    if let OwnValue::List(ref list) = list_value {
        assert_eq!(list.len(), 3);
    }
}

#[test]
fn test_iter_mut_compound() {
    // Create a compound
    let mut comp = OwnCompound::<BE>::default();
    comp.insert("a", 10i32);
    comp.insert("b", 20i32);
    comp.insert("c", 30i32);

    // Get OwnValue::Compound
    let mut comp_value = OwnValue::<BE>::Compound(comp);

    // Convert to MutValue
    let mut_value = comp_value.to_mut();

    if let MutValue::Compound(mut mut_comp) = mut_value {
        // Use iter_mut to iterate over key-value pairs
        let mut count = 0;
        let mut keys = Vec::new();
        let mut values = Vec::new();

        for (key, val) in mut_comp.iter_mut() {
            count += 1;
            keys.push(key.decode_lossy().into_owned());
            if let MutValue::Int(v) = val {
                values.push(v.get());
            }
        }

        assert_eq!(count, 3);
        assert!(keys.contains(&"a".to_string()));
        assert!(keys.contains(&"b".to_string()));
        assert!(keys.contains(&"c".to_string()));
        assert!(values.contains(&10));
        assert!(values.contains(&20));
        assert!(values.contains(&30));

        // Also test into_iter
        let mut into_count = 0;
        for (_key, _val) in mut_comp {
            into_count += 1;
        }
        assert_eq!(into_count, 3);
    } else {
        panic!("expected MutValue::Compound");
    }
}

#[test]
fn test_iter_mut_typed_list() {
    use na_nbt::TypedListMut;

    // Create a typed list
    let mut list = OwnList::<BE>::default();
    list.push(100i32);
    list.push(200i32);
    list.push(300i32);

    // Get OwnValue::List
    let mut list_value = OwnValue::<BE>::List(list);

    // Convert to MutValue and then get typed list
    let mut_value = list_value.to_mut();

    if let MutValue::List(mut_list) = mut_value {
        // Convert MutList to MutTypedList
        let mut typed = mut_list.typed_::<na_nbt::tag::Int>().unwrap();

        // Test iter on typed list (should give i32 values directly)
        let mut count = 0;
        for val in typed.iter() {
            count += 1;
            // iter() returns the native type (i32)
            assert!(val == 100 || val == 200 || val == 300);
        }
        assert_eq!(count, 3);

        // Test iter_mut on typed list
        let mut mut_count = 0;
        for val in typed.iter_mut() {
            mut_count += 1;
            // iter_mut() returns &mut I32<BE>
            let v = val.get();
            assert!(v == 100 || v == 200 || v == 300);
        }
        assert_eq!(mut_count, 3);
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_iter_mut_nested() {
    // Create a compound containing a list
    let mut comp = OwnCompound::<BE>::default();

    let mut inner_list = OwnList::<BE>::default();
    inner_list.push(1i32);
    inner_list.push(2i32);
    comp.insert("numbers", inner_list);

    comp.insert("name", "test");

    // Get OwnValue::Compound
    let mut comp_value = OwnValue::<BE>::Compound(comp);

    let mut_value = comp_value.to_mut();

    if let MutValue::Compound(mut mut_comp) = mut_value {
        for (key, val) in mut_comp.iter_mut() {
            let key_str = key.decode_lossy();
            if key_str == "numbers" {
                // Should be a list
                if let MutValue::List(mut nested_list) = val {
                    let mut list_count = 0;
                    for item in nested_list.iter_mut() {
                        list_count += 1;
                        if let MutValue::Int(n) = item {
                            let v = n.get();
                            assert!(v == 1 || v == 2);
                        }
                    }
                    assert_eq!(list_count, 2);
                } else {
                    panic!("expected nested list");
                }
            } else if key_str == "name" {
                // Should be a string
                if let MutValue::String(s) = val {
                    assert_eq!(s.decode(), "test");
                } else {
                    panic!("expected string");
                }
            }
        }
    } else {
        panic!("expected compound");
    }
}

// ============== Value Indexing Tests ==============

/// Test value indexing with usize for lists
#[test]
fn test_value_indexing_list_by_usize() {
    let mut list = OwnList::<BE>::default();
    list.push(10i32);
    list.push(20i32);
    list.push(30i32);

    let value = OwnValue::<BE>::List(list);

    // Index by usize
    assert!(value.get(0usize).is_some());
    assert!(value.get(1usize).is_some());
    assert!(value.get(2usize).is_some());
    assert!(value.get(3usize).is_none()); // OOB

    // Verify values
    if let Some(RefValue::Int(v)) = value.get(0usize) {
        assert_eq!(v, 10);
    } else {
        panic!("expected int at index 0");
    }

    if let Some(RefValue::Int(v)) = value.get(2usize) {
        assert_eq!(v, 30);
    } else {
        panic!("expected int at index 2");
    }
}

/// Test value indexing with string for compounds
#[test]
fn test_value_indexing_compound_by_string() {
    let mut comp = OwnCompound::<BE>::default();
    comp.insert("name", "test");
    comp.insert("value", 42i32);
    comp.insert("flag", 1i8);

    let value = OwnValue::<BE>::Compound(comp);

    // Index by &str
    assert!(value.get("name").is_some());
    assert!(value.get("value").is_some());
    assert!(value.get("flag").is_some());
    assert!(value.get("nonexistent").is_none());

    // Index by String
    let key = String::from("name");
    assert!(value.get(&key).is_some());

    // Verify values
    if let Some(RefValue::Int(v)) = value.get("value") {
        assert_eq!(v, 42);
    } else {
        panic!("expected int for 'value'");
    }

    if let Some(RefValue::String(s)) = value.get("name") {
        assert_eq!(s.decode_lossy(), "test");
    } else {
        panic!("expected string for 'name'");
    }
}

/// Test get_mut indexing
#[test]
fn test_value_indexing_get_mut() {
    let mut list = OwnList::<BE>::default();
    list.push(100i32);
    list.push(200i32);

    let mut value = OwnValue::<BE>::List(list);

    // Get mutable reference
    if let Some(MutValue::Int(v)) = value.get_mut(0usize) {
        assert_eq!(v.get(), 100);
        // Modify the value
        *v = zerocopy::byteorder::I32::<BE>::new(999);
    } else {
        panic!("expected mutable int at index 0");
    }

    // Verify modification
    if let Some(RefValue::Int(v)) = value.get(0usize) {
        assert_eq!(v, 999);
    } else {
        panic!("expected int at index 0");
    }
}

/// Test get_mut on compound
#[test]
fn test_value_indexing_compound_get_mut() {
    let mut comp = OwnCompound::<BE>::default();
    comp.insert("counter", 0i32);

    let mut value = OwnValue::<BE>::Compound(comp);

    // Modify via get_mut
    if let Some(MutValue::Int(v)) = value.get_mut("counter") {
        *v = zerocopy::byteorder::I32::<BE>::new(42);
    }

    // Verify
    if let Some(RefValue::Int(v)) = value.get("counter") {
        assert_eq!(v, 42);
    } else {
        panic!("expected int for 'counter'");
    }
}

/// Test nested indexing
#[test]
fn test_value_indexing_nested() {
    let mut inner_list = OwnList::<BE>::default();
    inner_list.push(1i32);
    inner_list.push(2i32);

    let mut comp = OwnCompound::<BE>::default();
    comp.insert("numbers", inner_list);

    let value = OwnValue::<BE>::Compound(comp);

    // Get nested list
    if let Some(RefValue::List(list)) = value.get("numbers") {
        assert_eq!(list.len(), 2);
        // Can index into the list
        if let Some(RefValue::Int(v)) = list.get(0) {
            assert_eq!(v, 1);
        }
    } else {
        panic!("expected list for 'numbers'");
    }
}

/// Test indexing wrong type (list indexed by string, compound indexed by usize)
#[test]
fn test_value_indexing_type_mismatch() {
    // List indexed by string should return None
    let mut list = OwnList::<BE>::default();
    list.push(1i32);
    let value = OwnValue::<BE>::List(list);
    assert!(value.get("key").is_none());

    // Compound indexed by usize should return None
    let mut comp = OwnCompound::<BE>::default();
    comp.insert("a", 1i32);
    let value = OwnValue::<BE>::Compound(comp);
    assert!(value.get(0usize).is_none());

    // Primitive value indexed should return None
    let value = OwnValue::<BE>::Int(zerocopy::byteorder::I32::<BE>::new(42));
    assert!(value.get(0usize).is_none());
    assert!(value.get("key").is_none());
}

// ============== Error Tests ==============

/// Test Error Display implementation
#[test]
fn test_error_display() {
    // Test EOF
    let err = Error::EOF;
    assert_eq!(format!("{}", err), "unexpected end of input");

    // Test INVALID
    let err = Error::INVALID(0xFF);
    assert_eq!(format!("{}", err), "invalid NBT tag type: 0xff");

    // Test LEN
    let err = Error::LEN(999999999);
    assert_eq!(format!("{}", err), "list length too long: 999999999");

    // Test KEY
    let err = Error::KEY;
    assert_eq!(format!("{}", err), "map key must be a string");

    // Test MISMATCH
    let err = Error::MISMATCH {
        expected: TagID::Int,
        actual: TagID::String,
    };
    assert!(format!("{}", err).contains("mismatch"));

    // Test CHAR
    let err = Error::CHAR(0xD800);
    assert!(format!("{}", err).contains("invalid character"));

    // Test REMAIN
    let err = Error::REMAIN(100);
    assert!(format!("{}", err).contains("remaining"));

    // Test MSG
    let err = Error::MSG("custom error message".to_string());
    assert_eq!(format!("{}", err), "custom error message");
}

/// Test Error Debug implementation
#[test]
fn test_error_debug() {
    let err = Error::EOF;
    let debug_str = format!("{:?}", err);
    assert!(debug_str.contains("EOF"));

    let err = Error::INVALID(0x0F);
    let debug_str = format!("{:?}", err);
    assert!(debug_str.contains("INVALID"));

    let err = Error::MISMATCH {
        expected: TagID::Compound,
        actual: TagID::List,
    };
    let debug_str = format!("{:?}", err);
    assert!(debug_str.contains("MISMATCH"));
}

/// Test Error std::error::Error implementation
#[test]
fn test_error_std_error() {
    fn assert_std_error<E: std::error::Error>(_: &E) {}

    let err = Error::EOF;
    assert_std_error(&err);

    let err = Error::INVALID(0xFF);
    assert_std_error(&err);

    let err = Error::IO(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "file not found",
    ));
    assert_std_error(&err);
}

/// Test reading with various error conditions
#[test]
fn test_read_errors() {
    use na_nbt::read_borrowed;
    use zerocopy::byteorder::BigEndian;

    // Empty data - should error
    let result = read_borrowed::<BigEndian>(&[]);
    assert!(result.is_err());

    // Just a tag byte, incomplete - should error
    let result = read_borrowed::<BigEndian>(&[0x0a]);
    assert!(result.is_err());

    // Invalid tag type
    let result = read_borrowed::<BigEndian>(&[0xFF, 0x00, 0x00]);
    assert!(matches!(result, Err(Error::INVALID(0xFF))));

    // Another invalid tag
    let result = read_borrowed::<BigEndian>(&[0x0D, 0x00, 0x00]);
    assert!(matches!(result, Err(Error::INVALID(0x0D))));
}

/// Test TagID values
#[test]
fn test_tag_id_values() {
    assert_eq!(TagID::End as u8, 0);
    assert_eq!(TagID::Byte as u8, 1);
    assert_eq!(TagID::Short as u8, 2);
    assert_eq!(TagID::Int as u8, 3);
    assert_eq!(TagID::Long as u8, 4);
    assert_eq!(TagID::Float as u8, 5);
    assert_eq!(TagID::Double as u8, 6);
    assert_eq!(TagID::ByteArray as u8, 7);
    assert_eq!(TagID::String as u8, 8);
    assert_eq!(TagID::List as u8, 9);
    assert_eq!(TagID::Compound as u8, 10);
    assert_eq!(TagID::IntArray as u8, 11);
    assert_eq!(TagID::LongArray as u8, 12);
}

// ============== OwnTypedList Tests ==============

/// Test OwnTypedList get and get_mut for Int type
#[test]
fn test_own_typed_list_int() {
    let mut typed = OwnTypedList::<BE, tag::Int>::default();
    typed.push(100i32);
    typed.push(200i32);
    typed.push(300i32);

    // get() returns the value directly
    assert_eq!(typed.get(0).unwrap(), 100);
    assert_eq!(typed.get(1).unwrap(), 200);
    assert_eq!(typed.get(2).unwrap(), 300);
    assert!(typed.get(3).is_none());

    // get_mut() returns mutable reference
    let val = typed.get_mut(1).unwrap();
    *val = zerocopy::byteorder::I32::<BE>::new(999);

    // Verify modification
    assert_eq!(typed.get(1).unwrap(), 999);
}

/// Test OwnTypedList with different primitive types
#[test]
fn test_own_typed_list_various_types() {
    // Byte list - returns i8 directly
    {
        let mut typed = OwnTypedList::<BE, tag::Byte>::default();
        typed.push(1i8);
        typed.push(2i8);
        typed.push(3i8);
        assert_eq!(typed.get(0).unwrap(), 1);
        assert_eq!(typed.get(1).unwrap(), 2);
        assert_eq!(typed.get(2).unwrap(), 3);
    }

    // Short list - returns i16 directly
    {
        let mut typed = OwnTypedList::<BE, tag::Short>::default();
        typed.push(100i16);
        typed.push(200i16);
        assert_eq!(typed.get(0).unwrap(), 100);
        assert_eq!(typed.get(1).unwrap(), 200);
    }

    // Long list - returns i64 directly
    {
        let mut typed = OwnTypedList::<BE, tag::Long>::default();
        typed.push(1000000000000i64);
        typed.push(2000000000000i64);
        assert_eq!(typed.get(0).unwrap(), 1000000000000);
        assert_eq!(typed.get(1).unwrap(), 2000000000000);
    }

    // Float list - returns f32 directly
    {
        let mut typed = OwnTypedList::<BE, tag::Float>::default();
        typed.push(1.5f32);
        typed.push(2.5f32);
        assert!((typed.get(0).unwrap() - 1.5).abs() < 0.001);
        assert!((typed.get(1).unwrap() - 2.5).abs() < 0.001);
    }

    // Double list - returns f64 directly
    {
        let mut typed = OwnTypedList::<BE, tag::Double>::default();
        typed.push(std::f64::consts::PI);
        typed.push(std::f64::consts::E);
        assert!((typed.get(0).unwrap() - std::f64::consts::PI).abs() < 0.0001);
        assert!((typed.get(1).unwrap() - std::f64::consts::E).abs() < 0.0001);
    }
}

/// Test OwnTypedList iteration via MutTypedList
#[test]
fn test_own_typed_list_iteration() {
    // Create typed list and convert to OwnValue to use MutTypedList's iter
    let mut list = OwnList::<BE>::default();
    list.push(10i32);
    list.push(20i32);
    list.push(30i32);

    let mut list_value = OwnValue::<BE>::List(list);
    let mut_value = list_value.to_mut();

    if let MutValue::List(mut_list) = mut_value {
        let mut typed = mut_list.typed_::<tag::Int>().unwrap();

        // Collect values via iter
        let mut values = Vec::new();
        for val in typed.iter() {
            values.push(val);
        }
        assert_eq!(values, vec![10, 20, 30]);

        // Test iter_mut
        let mut sum = 0i32;
        for val in typed.iter_mut() {
            sum += val.get();
        }
        assert_eq!(sum, 60);
    }
}

// ============== OwnList get_ / get_mut_ for All Types ==============

/// Test OwnList::get_ for all primitive types
#[test]
fn test_ownlist_get_typed_all_primitives() {
    // Byte - returns i8 directly
    {
        let mut list = OwnList::<BE>::default();
        list.push(42i8);
        let val = list.get_::<tag::Byte>(0).unwrap();
        assert_eq!(val, 42i8);
    }

    // Short - returns i16 directly
    {
        let mut list = OwnList::<BE>::default();
        list.push(1000i16);
        let val = list.get_::<tag::Short>(0).unwrap();
        assert_eq!(val, 1000);
    }

    // Int - returns i32 directly
    {
        let mut list = OwnList::<BE>::default();
        list.push(100000i32);
        let val = list.get_::<tag::Int>(0).unwrap();
        assert_eq!(val, 100000);
    }

    // Long - returns i64 directly
    {
        let mut list = OwnList::<BE>::default();
        list.push(1234567890123i64);
        let val = list.get_::<tag::Long>(0).unwrap();
        assert_eq!(val, 1234567890123);
    }

    // Float - returns f32 directly
    {
        let mut list = OwnList::<BE>::default();
        list.push(std::f32::consts::PI);
        let val = list.get_::<tag::Float>(0).unwrap();
        assert!((val - std::f32::consts::PI).abs() < 0.001);
    }

    // Double - returns f64 directly
    {
        let mut list = OwnList::<BE>::default();
        list.push(std::f64::consts::E);
        let val = list.get_::<tag::Double>(0).unwrap();
        assert!((val - std::f64::consts::E).abs() < 0.0001);
    }
}

/// Test OwnList::get_ with type mismatch returns None
#[test]
fn test_ownlist_get_typed_mismatch() {
    let mut list = OwnList::<BE>::default();
    list.push(42i32);

    // Try to get as wrong type
    assert!(list.get_::<tag::Byte>(0).is_none());
    assert!(list.get_::<tag::Short>(0).is_none());
    assert!(list.get_::<tag::Long>(0).is_none());
    assert!(list.get_::<tag::Float>(0).is_none());
    assert!(list.get_::<tag::String>(0).is_none());

    // Correct type works
    assert!(list.get_::<tag::Int>(0).is_some());
}

/// Test OwnList::get_mut_ for modifying values
#[test]
fn test_ownlist_get_mut_typed() {
    let mut list = OwnList::<BE>::default();
    list.push(100i32);

    // Get mutable reference with type - returns &mut I32<BE>
    let val = list.get_mut_::<tag::Int>(0).unwrap();
    *val = zerocopy::byteorder::I32::<BE>::new(999);

    // Verify - get_::<tag::Int> returns i32 directly
    assert_eq!(list.get_::<tag::Int>(0).unwrap(), 999);
}

/// Test OwnList::get_mut for all types
#[test]
fn test_ownlist_get_mut_all_types() {
    // Test Byte with get_mut
    {
        let mut list = OwnList::<BE>::default();
        list.push(10i8);
        if let Some(MutValue::Byte(v)) = list.get_mut(0) {
            *v = 99;
        }
        assert_eq!(list.get_::<tag::Byte>(0).unwrap(), 99i8);
    }

    // Test Short with get_mut
    {
        let mut list = OwnList::<BE>::default();
        list.push(100i16);
        if let Some(MutValue::Short(v)) = list.get_mut(0) {
            *v = zerocopy::byteorder::I16::<BE>::new(999);
        }
        assert_eq!(list.get_::<tag::Short>(0).unwrap(), 999);
    }

    // Test Long with get_mut
    {
        let mut list = OwnList::<BE>::default();
        list.push(1000i64);
        if let Some(MutValue::Long(v)) = list.get_mut(0) {
            *v = zerocopy::byteorder::I64::<BE>::new(9999);
        }
        assert_eq!(list.get_::<tag::Long>(0).unwrap(), 9999);
    }

    // Test Float with get_mut
    {
        let mut list = OwnList::<BE>::default();
        list.push(1.5f32);
        if let Some(MutValue::Float(v)) = list.get_mut(0) {
            *v = zerocopy::byteorder::F32::<BE>::new(std::f32::consts::PI);
        }
        assert!((list.get_::<tag::Float>(0).unwrap() - std::f32::consts::PI).abs() < 0.001);
    }

    // Test Double with get_mut
    {
        let mut list = OwnList::<BE>::default();
        list.push(1.0f64);
        if let Some(MutValue::Double(v)) = list.get_mut(0) {
            *v = zerocopy::byteorder::F64::<BE>::new(std::f64::consts::E);
        }
        assert!((list.get_::<tag::Double>(0).unwrap() - std::f64::consts::E).abs() < 0.0001);
    }
}

// ============== Visit/Map Tests ==============

/// Test visit on RefValue
#[test]
fn test_value_visit_ref() {
    use na_nbt::ValueRef;

    let mut comp = OwnCompound::<BE>::default();
    comp.insert("int", 42i32);
    comp.insert("string", "hello");
    comp.insert("float", 1.5f32);

    let value = OwnValue::<BE>::Compound(comp);
    let ref_value = value.to_ref();

    // Visit compound
    let result = ref_value.visit(|v| match v {
        VisitRef::Compound(_) => "compound",
        _ => "other",
    });
    assert_eq!(result, "compound");

    // Visit nested values
    if let RefValue::Compound(c) = ref_value {
        let int_val = c.get("int").unwrap();
        let type_name = int_val.visit(|v| match v {
            VisitRef::Int(i) => format!("int:{}", i),
            _ => "other".to_string(),
        });
        assert_eq!(type_name, "int:42");

        let string_val = c.get("string").unwrap();
        let type_name = string_val.visit(|v| match v {
            VisitRef::String(s) => format!("string:{}", s.decode_lossy()),
            _ => "other".to_string(),
        });
        assert_eq!(type_name, "string:hello");
    }
}

/// Test visit on all primitive types
#[test]
fn test_value_visit_all_types() {
    // Test Byte
    {
        let mut list = OwnList::<BE>::default();
        list.push(42i8);
        let val = list.get(0).unwrap();
        let result = val.visit(|v| matches!(v, VisitRef::Byte(_)));
        assert!(result);
    }

    // Test Short
    {
        let mut list = OwnList::<BE>::default();
        list.push(1000i16);
        let val = list.get(0).unwrap();
        let result = val.visit(|v| matches!(v, VisitRef::Short(_)));
        assert!(result);
    }

    // Test Int
    {
        let mut list = OwnList::<BE>::default();
        list.push(100000i32);
        let val = list.get(0).unwrap();
        let result = val.visit(|v| matches!(v, VisitRef::Int(_)));
        assert!(result);
    }

    // Test Long
    {
        let mut list = OwnList::<BE>::default();
        list.push(1234567890123i64);
        let val = list.get(0).unwrap();
        let result = val.visit(|v| matches!(v, VisitRef::Long(_)));
        assert!(result);
    }

    // Test Float
    {
        let mut list = OwnList::<BE>::default();
        list.push(std::f32::consts::PI);
        let val = list.get(0).unwrap();
        let result = val.visit(|v| matches!(v, VisitRef::Float(_)));
        assert!(result);
    }

    // Test Double
    {
        let mut list = OwnList::<BE>::default();
        list.push(std::f64::consts::E);
        let val = list.get(0).unwrap();
        let result = val.visit(|v| matches!(v, VisitRef::Double(_)));
        assert!(result);
    }

    // Test String
    {
        let mut list = OwnList::<BE>::default();
        list.push("hello");
        let val = list.get(0).unwrap();
        let result = val.visit(|v| matches!(v, VisitRef::String(_)));
        assert!(result);
    }

    // Test List
    {
        let mut inner = OwnList::<BE>::default();
        inner.push(1i32);
        let mut outer = OwnList::<BE>::default();
        outer.push(inner);
        let val = outer.get(0).unwrap();
        let result = val.visit(|v| matches!(v, VisitRef::List(_)));
        assert!(result);
    }

    // Test Compound
    {
        let inner = OwnCompound::<BE>::default();
        let mut outer = OwnList::<BE>::default();
        outer.push(inner);
        let val = outer.get(0).unwrap();
        let result = val.visit(|v| matches!(v, VisitRef::Compound(_)));
        assert!(result);
    }
}

/// Test map on RefValue (consuming the value)
#[test]
fn test_value_map_ref() {
    let mut list = OwnList::<BE>::default();
    list.push(42i32);

    let val = list.get(0).unwrap();

    // Map consumes and transforms
    let result = val.map(|m| match m {
        MapRef::Int(i) => i * 2,
        _ => 0,
    });
    assert_eq!(result, 84);

    let val = list.get_mut(0).unwrap();

    // Map consumes and transforms
    let result = val.map(|m| match m {
        MapMut::Int(i) => i.get() * 2,
        _ => 0,
    });
    assert_eq!(result, 84);
}

/// Test map with extracting different types
#[test]
fn test_value_map_extract_types() {
    // Extract string
    {
        let mut list = OwnList::<BE>::default();
        list.push("hello world");
        let val = list.get(0).unwrap();
        let result = val.map(|m| match m {
            MapRef::String(s) => s.decode_lossy().to_string(),
            _ => String::new(),
        });
        assert_eq!(result, "hello world");
    }

    // Extract and process list
    {
        let mut inner = OwnList::<BE>::default();
        inner.push(1i32);
        inner.push(2i32);
        inner.push(3i32);
        let mut outer = OwnList::<BE>::default();
        outer.push(inner);

        let val = outer.get(0).unwrap();
        let sum = val.map(|m| match m {
            MapRef::List(list) => {
                let mut sum = 0i32;
                for item in list.iter() {
                    if let RefValue::Int(i) = item {
                        sum += i;
                    }
                }
                sum
            }
            _ => 0,
        });
        assert_eq!(sum, 6);
    }
}

/// Test visit/map for arrays
#[test]
fn test_value_visit_arrays() {
    use zerocopy::byteorder::{I32, I64};

    // ByteArray
    {
        let bytes: Vec<i8> = vec![1, 2, 3, 4, 5];
        let mut list = OwnList::<BE>::default();
        list.push(bytes);
        let val = list.get(0).unwrap();
        let result: usize = val.visit(|v| match v {
            VisitRef::ByteArray(arr) => arr.len(),
            _ => 0,
        });
        assert_eq!(result, 5);
    }

    // IntArray
    {
        let ints: Vec<I32<BE>> = vec![I32::new(10), I32::new(20), I32::new(30)];
        let mut list = OwnList::<BE>::default();
        list.push(ints);
        let val = list.get(0).unwrap();
        let result: usize = val.visit(|v| match v {
            VisitRef::IntArray(arr) => arr.len(),
            _ => 0,
        });
        assert_eq!(result, 3);
    }

    // LongArray
    {
        let longs: Vec<I64<BE>> = vec![I64::new(100), I64::new(200)];
        let mut list = OwnList::<BE>::default();
        list.push(longs);
        let val = list.get(0).unwrap();
        let result: usize = val.visit(|v| match v {
            VisitRef::LongArray(arr) => arr.len(),
            _ => 0,
        });
        assert_eq!(result, 2);
    }
}

// ============== Additional Coverage Tests ==============

/// Test OwnValue::tag_id() for all types
#[test]
fn test_own_value_tag_id() {
    // End
    let val = OwnValue::<BE>::End(());
    assert_eq!(val.tag_id(), TagID::End);

    // Byte
    let val = OwnValue::<BE>::Byte(42);
    assert_eq!(val.tag_id(), TagID::Byte);

    // Short
    let val = OwnValue::<BE>::Short(zerocopy::byteorder::I16::<BE>::new(100));
    assert_eq!(val.tag_id(), TagID::Short);

    // Int
    let val = OwnValue::<BE>::Int(zerocopy::byteorder::I32::<BE>::new(1000));
    assert_eq!(val.tag_id(), TagID::Int);

    // Long
    let val = OwnValue::<BE>::Long(zerocopy::byteorder::I64::<BE>::new(100000));
    assert_eq!(val.tag_id(), TagID::Long);

    // Float
    let val = OwnValue::<BE>::Float(zerocopy::byteorder::F32::<BE>::new(1.5));
    assert_eq!(val.tag_id(), TagID::Float);

    // Double
    let val = OwnValue::<BE>::Double(zerocopy::byteorder::F64::<BE>::new(2.5));
    assert_eq!(val.tag_id(), TagID::Double);

    // List
    let val = OwnValue::<BE>::List(OwnList::<BE>::default());
    assert_eq!(val.tag_id(), TagID::List);

    // Compound
    let val = OwnValue::<BE>::Compound(OwnCompound::<BE>::default());
    assert_eq!(val.tag_id(), TagID::Compound);
}

/// Test OwnValue::is_<T>() type checking
#[test]
fn test_own_value_is_type() {
    let val = OwnValue::<BE>::Int(zerocopy::byteorder::I32::<BE>::new(42));
    assert!(val.is_::<tag::Int>());
    assert!(!val.is_::<tag::Byte>());
    assert!(!val.is_::<tag::String>());
    assert!(!val.is_::<tag::List>());

    let val = OwnValue::<BE>::List(OwnList::<BE>::default());
    assert!(val.is_::<tag::List>());
    assert!(!val.is_::<tag::Int>());

    let val = OwnValue::<BE>::Compound(OwnCompound::<BE>::default());
    assert!(val.is_::<tag::Compound>());
    assert!(!val.is_::<tag::List>());
}

/// Test OwnValue::to_ref() conversion for all types
#[test]
fn test_own_value_to_ref() {
    // Byte
    {
        let val = OwnValue::<BE>::Byte(42);
        if let RefValue::Byte(b) = val.to_ref() {
            assert_eq!(b, 42);
        } else {
            panic!("expected byte");
        }
    }

    // Short
    {
        let val = OwnValue::<BE>::Short(zerocopy::byteorder::I16::<BE>::new(1000));
        if let RefValue::Short(s) = val.to_ref() {
            assert_eq!(s, 1000);
        } else {
            panic!("expected short");
        }
    }

    // Int
    {
        let val = OwnValue::<BE>::Int(zerocopy::byteorder::I32::<BE>::new(100000));
        if let RefValue::Int(i) = val.to_ref() {
            assert_eq!(i, 100000);
        } else {
            panic!("expected int");
        }
    }

    // Long
    {
        let val = OwnValue::<BE>::Long(zerocopy::byteorder::I64::<BE>::new(9999999999));
        if let RefValue::Long(l) = val.to_ref() {
            assert_eq!(l, 9999999999);
        } else {
            panic!("expected long");
        }
    }

    // Float
    {
        let val = OwnValue::<BE>::Float(zerocopy::byteorder::F32::<BE>::new(std::f32::consts::PI));
        if let RefValue::Float(f) = val.to_ref() {
            assert!((f - std::f32::consts::PI).abs() < 0.0001);
        } else {
            panic!("expected float");
        }
    }

    // Double
    {
        let val = OwnValue::<BE>::Double(zerocopy::byteorder::F64::<BE>::new(std::f64::consts::E));
        if let RefValue::Double(d) = val.to_ref() {
            assert!((d - std::f64::consts::E).abs() < 0.00001);
        } else {
            panic!("expected double");
        }
    }
}

/// Test OwnValue::to_mut() and modification
#[test]
fn test_own_value_to_mut_modification() {
    // Modify byte
    {
        let mut val = OwnValue::<BE>::Byte(10);
        if let MutValue::Byte(b) = val.to_mut() {
            *b = 99;
        }
        if let RefValue::Byte(b) = val.to_ref() {
            assert_eq!(b, 99);
        }
    }

    // Modify short
    {
        let mut val = OwnValue::<BE>::Short(zerocopy::byteorder::I16::<BE>::new(100));
        if let MutValue::Short(s) = val.to_mut() {
            *s = zerocopy::byteorder::I16::<BE>::new(999);
        }
        if let RefValue::Short(s) = val.to_ref() {
            assert_eq!(s, 999);
        }
    }

    // Modify int
    {
        let mut val = OwnValue::<BE>::Int(zerocopy::byteorder::I32::<BE>::new(1000));
        if let MutValue::Int(i) = val.to_mut() {
            *i = zerocopy::byteorder::I32::<BE>::new(9999);
        }
        if let RefValue::Int(i) = val.to_ref() {
            assert_eq!(i, 9999);
        }
    }
}

/// Test string encoding and decoding
#[test]
fn test_string_operations() {
    use na_nbt::OwnString;

    // Create string and test basic operations
    let s = OwnString::from("Hello, World!");
    assert_eq!(s.decode(), "Hello, World!");

    // Test Unicode
    let s = OwnString::from("日本語テスト");
    assert_eq!(s.decode(), "日本語テスト");

    // Test emoji
    let s = OwnString::from("Test 🎮 emoji");
    assert_eq!(s.decode(), "Test 🎮 emoji");

    // Test empty string
    let s = OwnString::from("");
    assert_eq!(s.decode(), "");

    // Test through list
    let mut list = OwnList::<BE>::default();
    list.push("first");
    list.push("second");
    list.push("third");

    if let RefValue::String(s) = list.get(0).unwrap() {
        assert_eq!(s.decode_lossy(), "first");
    }
    if let RefValue::String(s) = list.get(2).unwrap() {
        assert_eq!(s.decode_lossy(), "third");
    }
}

/// Test ByteArray operations
#[test]
fn test_byte_array_operations() {
    let bytes: Vec<i8> = vec![1, 2, 3, 4, 5, -1, -128, 127];
    let mut list = OwnList::<BE>::default();
    list.push(bytes);

    if let RefValue::ByteArray(arr) = list.get(0).unwrap() {
        assert_eq!(arr.len(), 8);
        assert_eq!(arr[0], 1);
        assert_eq!(arr[5], -1);
        assert_eq!(arr[6], -128);
        assert_eq!(arr[7], 127);
    } else {
        panic!("expected byte array");
    }

    // Test empty byte array
    let empty: Vec<i8> = vec![];
    let mut list2 = OwnList::<BE>::default();
    list2.push(empty);

    if let RefValue::ByteArray(arr) = list2.get(0).unwrap() {
        assert_eq!(arr.len(), 0);
    }
}

/// Test IntArray operations
#[test]
fn test_int_array_operations() {
    use zerocopy::byteorder::I32;

    let ints: Vec<I32<BE>> = vec![
        I32::new(i32::MIN),
        I32::new(-1),
        I32::new(0),
        I32::new(1),
        I32::new(i32::MAX),
    ];
    let mut list = OwnList::<BE>::default();
    list.push(ints);

    if let RefValue::IntArray(arr) = list.get(0).unwrap() {
        assert_eq!(arr.len(), 5);
        assert_eq!(arr[0].get(), i32::MIN);
        assert_eq!(arr[1].get(), -1);
        assert_eq!(arr[2].get(), 0);
        assert_eq!(arr[3].get(), 1);
        assert_eq!(arr[4].get(), i32::MAX);
    } else {
        panic!("expected int array");
    }
}

/// Test LongArray operations
#[test]
fn test_long_array_operations() {
    use zerocopy::byteorder::I64;

    let longs: Vec<I64<BE>> = vec![
        I64::new(i64::MIN),
        I64::new(-1),
        I64::new(0),
        I64::new(1),
        I64::new(i64::MAX),
    ];
    let mut list = OwnList::<BE>::default();
    list.push(longs);

    if let RefValue::LongArray(arr) = list.get(0).unwrap() {
        assert_eq!(arr.len(), 5);
        assert_eq!(arr[0].get(), i64::MIN);
        assert_eq!(arr[1].get(), -1);
        assert_eq!(arr[2].get(), 0);
        assert_eq!(arr[3].get(), 1);
        assert_eq!(arr[4].get(), i64::MAX);
    } else {
        panic!("expected long array");
    }
}

/// Test compound iteration
#[test]
fn test_compound_iteration() {
    let mut comp = OwnCompound::<BE>::default();
    comp.insert("a", 1i32);
    comp.insert("b", 2i32);
    comp.insert("c", 3i32);

    let value = OwnValue::<BE>::Compound(comp);

    if let RefValue::Compound(c) = value.to_ref() {
        let mut count = 0;
        let mut sum = 0i32;
        for (key, val) in c.iter() {
            count += 1;
            let key_str = key.decode_lossy();
            assert!(key_str == "a" || key_str == "b" || key_str == "c");
            if let RefValue::Int(i) = val {
                sum += i;
            }
        }
        assert_eq!(count, 3);
        assert_eq!(sum, 6);
    }
}

/// Test list iteration
#[test]
fn test_list_iteration() {
    let mut list = OwnList::<BE>::default();
    list.push(10i32);
    list.push(20i32);
    list.push(30i32);

    let value = OwnValue::<BE>::List(list);

    if let RefValue::List(l) = value.to_ref() {
        let mut count = 0;
        let mut sum = 0i32;
        for item in l.iter() {
            count += 1;
            if let RefValue::Int(i) = item {
                sum += i;
            }
        }
        assert_eq!(count, 3);
        assert_eq!(sum, 60);
    }
}

/// Test pop operations on list
#[test]
fn test_list_pop_operations() {
    let mut list = OwnList::<BE>::default();
    list.push(1i32);
    list.push(2i32);
    list.push(3i32);

    assert_eq!(list.len(), 3);

    // Pop last
    let popped = list.pop().unwrap();
    assert_eq!(popped.as_int_val(), Some(3));
    assert_eq!(list.len(), 2);

    // Pop again
    let popped = list.pop().unwrap();
    assert_eq!(popped.as_int_val(), Some(2));
    assert_eq!(list.len(), 1);

    // Pop last one
    let popped = list.pop().unwrap();
    assert_eq!(popped.as_int_val(), Some(1));
    assert_eq!(list.len(), 0);

    // Pop from empty
    assert!(list.pop().is_none());
}

/// Test remove operations on list
#[test]
fn test_list_remove_operations() {
    let mut list = OwnList::<BE>::default();
    list.push(10i32);
    list.push(20i32);
    list.push(30i32);
    list.push(40i32);

    // Remove from middle
    let removed = list.remove(1).unwrap();
    assert_eq!(removed.as_int_val(), Some(20));
    assert_eq!(list.len(), 3);

    // Verify order: [10, 30, 40]
    assert_eq!(list.get(0).unwrap().as_int_val(), Some(10));
    assert_eq!(list.get(1).unwrap().as_int_val(), Some(30));
    assert_eq!(list.get(2).unwrap().as_int_val(), Some(40));

    // Remove from beginning
    let removed = list.remove(0).unwrap();
    assert_eq!(removed.as_int_val(), Some(10));

    // Remove from end
    let removed = list.remove(1).unwrap();
    assert_eq!(removed.as_int_val(), Some(40));

    // Only [30] left
    assert_eq!(list.len(), 1);
    assert_eq!(list.get(0).unwrap().as_int_val(), Some(30));
}

/// Test typed list pop and remove
#[test]
fn test_typed_list_pop_remove() {
    let mut typed = OwnTypedList::<BE, tag::Int>::default();
    typed.push(100i32);
    typed.push(200i32);
    typed.push(300i32);

    // Pop
    let val = typed.pop().unwrap();
    assert_eq!(val.get(), 300);

    // Remove
    let val = typed.remove(0).unwrap();
    assert_eq!(val.get(), 100);

    // Only 200 left
    assert_eq!(typed.len(), 1);
    assert_eq!(typed.get(0).unwrap(), 200);
}

/// Test compound remove operations
#[test]
fn test_compound_remove_operations() {
    let mut comp = OwnCompound::<BE>::default();
    comp.insert("first", 1i32);
    comp.insert("second", 2i32);
    comp.insert("third", 3i32);

    // Remove existing
    let removed = comp.remove("second").unwrap();
    assert_eq!(removed.as_int_val(), Some(2));

    // Verify it's gone
    assert!(comp.get("second").is_none());
    assert!(comp.get("first").is_some());
    assert!(comp.get("third").is_some());

    // Remove non-existent
    assert!(comp.remove("nonexistent").is_none());
}

/// Test mixed type list operations
#[test]
fn test_list_type_mixing() {
    // Create list with ints
    let mut list = OwnList::<BE>::default();
    list.push(1i32);
    list.push(2i32);

    // Try to push different type - should be silently ignored
    // (the list already has type Int established)
    list.push("string"); // This should not add
    assert_eq!(list.len(), 2); // Still 2

    // Try insert with wrong type
    list.insert(0, std::f32::consts::PI); // Should be ignored
    assert_eq!(list.len(), 2);
}

/// Test empty list operations
#[test]
fn test_empty_list_operations() {
    let mut list = OwnList::<BE>::default();

    assert!(list.is_empty());
    assert_eq!(list.len(), 0);
    assert!(list.get(0).is_none());
    assert!(list.pop().is_none());
    assert!(list.remove(0).is_none());

    // After adding and removing
    list.push(42i32);
    assert!(!list.is_empty());
    list.pop();
    assert!(list.is_empty());
}

/// Test empty compound operations
#[test]
fn test_empty_compound_operations() {
    let comp = OwnCompound::<BE>::default();

    assert!(comp.get("any").is_none());
    assert!(comp.get("key").is_none());

    // Test iteration on empty compound
    let value = OwnValue::<BE>::Compound(comp);
    if let RefValue::Compound(c) = value.to_ref() {
        let count = c.iter().count();
        assert_eq!(count, 0);
    }
}

/// Test nested compound modification
#[test]
fn test_nested_compound_modification() {
    let mut inner = OwnCompound::<BE>::default();
    inner.insert("value", 10i32);

    let mut outer = OwnCompound::<BE>::default();
    outer.insert("nested", inner);
    outer.insert("top_level", 99i32);

    // Verify structure
    assert!(outer.get("nested").is_some());
    assert!(outer.get("top_level").is_some());

    if let RefValue::Compound(nested) = outer.get("nested").unwrap() {
        let val = nested.get("value");
        assert!(val.is_some());
        if let RefValue::Int(i) = val.unwrap() {
            assert_eq!(i, 10);
        }
    }

    // Modify via get_mut
    if let Some(MutValue::Int(i)) = outer.get_mut("top_level") {
        *i = zerocopy::byteorder::I32::<BE>::new(100);
    }

    assert_eq!(outer.get("top_level").unwrap().as_int_val(), Some(100));
}

/// Test deeply nested structure
#[test]
fn test_deeply_nested_structure() {
    // Create nested lists
    let mut level3 = OwnList::<BE>::default();
    level3.push(1i32);

    let mut level2 = OwnList::<BE>::default();
    level2.push(level3);

    let mut level1 = OwnList::<BE>::default();
    level1.push(level2);

    let mut root = OwnCompound::<BE>::default();
    root.insert("levels", level1);

    // Navigate through the levels
    let val = OwnValue::<BE>::Compound(root);

    if let RefValue::Compound(c) = val.to_ref() {
        if let Some(RefValue::List(l1)) = c.get("levels") {
            assert_eq!(l1.len(), 1);
            if let Some(RefValue::List(l2)) = l1.get(0) {
                assert_eq!(l2.len(), 1);
                if let Some(RefValue::List(l3)) = l2.get(0) {
                    assert_eq!(l3.len(), 1);
                    if let Some(RefValue::Int(i)) = l3.get(0) {
                        assert_eq!(i, 1);
                    } else {
                        panic!("expected int at level 3");
                    }
                } else {
                    panic!("expected list at level 2");
                }
            } else {
                panic!("expected list at level 1");
            }
        } else {
            panic!("expected list at root");
        }
    }
}

// Test OwnValue From implementations for all primitive types
#[test]
fn test_own_value_from_primitives() {
    use zerocopy::byteorder::{F32, F64, I16, I32, I64};

    // Test From<i8> for Byte
    let val: OwnValue<BE> = 42i8.into();
    assert_eq!(val.tag_id(), TagID::Byte);

    // Test From<I16<O>> for Short
    let val: OwnValue<BE> = I16::<BE>::new(1234).into();
    assert_eq!(val.tag_id(), TagID::Short);

    // Test From<I32<O>> for Int
    let val: OwnValue<BE> = I32::<BE>::new(123456).into();
    assert_eq!(val.tag_id(), TagID::Int);

    // Test From<I64<O>> for Long
    let val: OwnValue<BE> = I64::<BE>::new(123456789).into();
    assert_eq!(val.tag_id(), TagID::Long);

    // Test From<F32<O>> for Float
    let val: OwnValue<BE> = F32::<BE>::new(std::f32::consts::PI).into();
    assert_eq!(val.tag_id(), TagID::Float);

    // Test From<F64<O>> for Double
    let val: OwnValue<BE> = F64::<BE>::new(std::f64::consts::PI).into();
    assert_eq!(val.tag_id(), TagID::Double);
}

// Test OwnValue From implementations for array and complex types
#[test]
fn test_own_value_from_arrays() {
    use na_nbt::view::OwnString;
    use na_nbt::view::OwnVec;
    use zerocopy::byteorder::{I32, I64};

    // Test From<OwnVec<i8>> for ByteArray
    let byte_vec = OwnVec::from(vec![1i8, 2, 3]);
    let val: OwnValue<BE> = byte_vec.into();
    assert_eq!(val.tag_id(), TagID::ByteArray);

    // Test From<OwnString> for String
    let string = OwnString::from(String::from("hello"));
    let val: OwnValue<BE> = string.into();
    assert_eq!(val.tag_id(), TagID::String);

    // Test From<OwnVec<I32<O>>> for IntArray
    let int_vec = OwnVec::from(vec![I32::<BE>::new(1), I32::<BE>::new(2)]);
    let val: OwnValue<BE> = int_vec.into();
    assert_eq!(val.tag_id(), TagID::IntArray);

    // Test From<OwnVec<I64<O>>> for LongArray
    let long_vec = OwnVec::from(vec![I64::<BE>::new(1), I64::<BE>::new(2)]);
    let val: OwnValue<BE> = long_vec.into();
    assert_eq!(val.tag_id(), TagID::LongArray);
}

// Test OwnValue From implementations for List and Compound
#[test]
fn test_own_value_from_containers() {
    // Test From<OwnList<O>> for List
    let list = OwnList::<BE>::default();
    let val: OwnValue<BE> = list.into();
    assert_eq!(val.tag_id(), TagID::List);

    // Test From<OwnCompound<O>> for Compound
    let compound = OwnCompound::<BE>::default();
    let val: OwnValue<BE> = compound.into();
    assert_eq!(val.tag_id(), TagID::Compound);
}

// Test MutValue map method for all primitive types
#[test]
fn test_mut_value_map_primitives() {
    use na_nbt::value::{MapMut, ValueMut};
    use zerocopy::byteorder::{F32, F64, I16, I32, I64};

    // Create compound with all primitive types
    let mut compound = OwnCompound::<BE>::default();
    compound.insert("byte", 10i8);
    compound.insert("short", I16::<BE>::new(100));
    compound.insert("int", I32::<BE>::new(1000));
    compound.insert("long", I64::<BE>::new(10000));
    compound.insert("float", F32::<BE>::new(1.5));
    compound.insert("double", F64::<BE>::new(2.5));

    // Wrap in OwnValue to get mutable access
    let mut compound_value = OwnValue::<BE>::Compound(compound);

    if let MutValue::Compound(mut compound_mut) = compound_value.to_mut() {
        // Test Byte mapping
        if let Some(val) = compound_mut.get_mut("byte") {
            val.map(|m| match m {
                MapMut::Byte(b) => {
                    *b = 20;
                }
                _ => panic!("expected byte"),
            });
        }

        // Test Short mapping
        if let Some(val) = compound_mut.get_mut("short") {
            val.map(|m| match m {
                MapMut::Short(s) => {
                    *s = I16::<BE>::new(200);
                }
                _ => panic!("expected short"),
            });
        }

        // Test Int mapping
        if let Some(val) = compound_mut.get_mut("int") {
            val.map(|m| match m {
                MapMut::Int(i) => {
                    *i = I32::<BE>::new(2000);
                }
                _ => panic!("expected int"),
            });
        }

        // Test Long mapping
        if let Some(val) = compound_mut.get_mut("long") {
            val.map(|m| match m {
                MapMut::Long(l) => {
                    *l = I64::<BE>::new(20000);
                }
                _ => panic!("expected long"),
            });
        }

        // Test Float mapping
        if let Some(val) = compound_mut.get_mut("float") {
            val.map(|m| match m {
                MapMut::Float(f) => {
                    *f = F32::<BE>::new(3.5);
                }
                _ => panic!("expected float"),
            });
        }

        // Test Double mapping
        if let Some(val) = compound_mut.get_mut("double") {
            val.map(|m| match m {
                MapMut::Double(d) => {
                    *d = F64::<BE>::new(4.5);
                }
                _ => panic!("expected double"),
            });
        }
    }

    // Verify modifications
    if let OwnValue::Compound(comp) = compound_value {
        assert_eq!(comp.get("byte").unwrap().as_int_val(), Some(20));
        assert_eq!(comp.get("short").unwrap().as_int_val(), Some(200));
        assert_eq!(comp.get("int").unwrap().as_int_val(), Some(2000));
        assert_eq!(comp.get("long").unwrap().as_int_val(), Some(20000));
        assert_eq!(comp.get("float").unwrap().as_float_val(), Some(3.5));
        assert_eq!(comp.get("double").unwrap().as_float_val(), Some(4.5));
    }
}

// Test RefList get for various element types
#[test]
fn test_ref_list_get_all_types() {
    // Create list with Short elements
    let mut short_list = OwnList::<BE>::default();
    short_list.push(100i16);
    short_list.push(200i16);
    assert_eq!(short_list.get(0).unwrap().as_int_val(), Some(100));
    assert_eq!(short_list.get(1).unwrap().as_int_val(), Some(200));
    assert!(short_list.get(2).is_none()); // OOB check

    // Create list with Long elements
    let mut long_list = OwnList::<BE>::default();
    long_list.push(1000000i64);
    assert_eq!(long_list.get(0).unwrap().as_int_val(), Some(1000000));

    // Create list with Float elements
    let mut float_list = OwnList::<BE>::default();
    float_list.push(std::f32::consts::PI);
    assert!(
        (float_list.get(0).unwrap().as_float_val().unwrap() - std::f32::consts::PI).abs() < 0.01
    );

    // Create list with Double elements
    let mut double_list = OwnList::<BE>::default();
    double_list.push(std::f64::consts::E);
    assert!(
        (double_list.get(0).unwrap().as_float_val().unwrap() - std::f32::consts::E).abs() < 0.0001
    );
}

// Test RefList get for complex element types
#[test]
fn test_ref_list_get_complex_types() {
    use zerocopy::byteorder::{I32, I64};

    // List of byte arrays
    let mut ba_list = OwnList::<BE>::default();
    ba_list.push(vec![1i8, 2, 3]);
    ba_list.push(vec![4i8, 5, 6]);
    let first = ba_list.get(0).unwrap();
    assert_eq!(first.tag_id(), TagID::ByteArray);

    // List of strings
    let mut str_list = OwnList::<BE>::default();
    str_list.push("hello");
    str_list.push("world");
    assert_eq!(str_list.get(0).unwrap().as_string_val().unwrap(), "hello");
    assert_eq!(str_list.get(1).unwrap().as_string_val().unwrap(), "world");

    // List of int arrays
    let mut ia_list = OwnList::<BE>::default();
    ia_list.push(vec![I32::<BE>::new(10), I32::<BE>::new(20)]);
    assert_eq!(ia_list.get(0).unwrap().tag_id(), TagID::IntArray);

    // List of long arrays
    let mut la_list = OwnList::<BE>::default();
    la_list.push(vec![I64::<BE>::new(100), I64::<BE>::new(200)]);
    assert_eq!(la_list.get(0).unwrap().tag_id(), TagID::LongArray);

    // Nested list (list of lists)
    let mut outer = OwnList::<BE>::default();
    let mut inner = OwnList::<BE>::default();
    inner.push(42i32);
    outer.push(inner);
    let inner_val = outer.get(0).unwrap();
    assert_eq!(inner_val.tag_id(), TagID::List);

    // List of compounds
    let mut comp_list = OwnList::<BE>::default();
    let mut comp = OwnCompound::<BE>::default();
    comp.insert("key", 123i32);
    comp_list.push(comp);
    assert_eq!(comp_list.get(0).unwrap().tag_id(), TagID::Compound);
}

// Test typed list with various primitive types
#[test]
fn test_typed_list_all_primitives() {
    use na_nbt::tag;
    use zerocopy::byteorder::{F32, F64, I16, I64};

    // Short typed list
    let mut short_list = OwnTypedList::<BE, tag::Short>::default();
    short_list.push(I16::<BE>::new(100));
    short_list.push(I16::<BE>::new(200));
    assert_eq!(short_list.len(), 2);
    assert_eq!(short_list.get(0).unwrap(), 100);

    // Long typed list
    let mut long_list = OwnTypedList::<BE, tag::Long>::default();
    long_list.push(I64::<BE>::new(1000000));
    assert_eq!(long_list.get(0).unwrap(), 1000000);

    // Float typed list
    let mut float_list = OwnTypedList::<BE, tag::Float>::default();
    float_list.push(F32::<BE>::new(1.5));
    assert!((float_list.get(0).unwrap() - 1.5).abs() < 0.01);

    // Double typed list
    let mut double_list = OwnTypedList::<BE, tag::Double>::default();
    double_list.push(F64::<BE>::new(2.5));
    assert!((double_list.get(0).unwrap() - 2.5).abs() < 0.01);
}

// Test OwnTypedList From implementations
#[test]
fn test_typed_list_into_own_value() {
    use na_nbt::tag;

    // OwnTypedList<O, T> should convert to OwnValue::List
    let mut typed_list = OwnTypedList::<BE, tag::Int>::default();
    typed_list.push(zerocopy::byteorder::I32::<BE>::new(42));

    let own_value: OwnValue<BE> = typed_list.into();
    assert_eq!(own_value.tag_id(), TagID::List);
}

// Test list with byte type
#[test]
fn test_list_byte_operations() {
    let mut list = OwnList::<BE>::default();
    list.push(-128i8);
    list.push(0i8);
    list.push(127i8);

    assert_eq!(list.len(), 3);
    // Byte list returns RefValue::Byte, use match to check
    if let RefValue::Byte(b) = list.get(0).unwrap() {
        assert_eq!(b, -128);
    } else {
        panic!("expected byte");
    }
    if let RefValue::Byte(b) = list.get(1).unwrap() {
        assert_eq!(b, 0);
    } else {
        panic!("expected byte");
    }
    if let RefValue::Byte(b) = list.get(2).unwrap() {
        assert_eq!(b, 127);
    } else {
        panic!("expected byte");
    }
}

// Test OwnList get with various indices
#[test]
fn test_list_boundary_indices() {
    let mut list = OwnList::<BE>::default();
    for i in 0..5 {
        list.push(i);
    }

    // Valid indices
    assert!(list.get(0).is_some());
    assert!(list.get(4).is_some());

    // Invalid indices (should return None, not panic)
    assert!(list.get(5).is_none());
    assert!(list.get(100).is_none());
    assert!(list.get(usize::MAX).is_none());
}

// Test compound with various string keys
#[test]
fn test_compound_key_variations() {
    let mut compound = OwnCompound::<BE>::default();

    // Empty key
    compound.insert("", 1i32);

    // Single char key
    compound.insert("a", 2i32);

    // Long key
    let long_key = "a".repeat(100);
    compound.insert(&long_key, 3i32);

    // Key with special characters
    compound.insert("key-with-dashes", 4i32);
    compound.insert("key.with.dots", 5i32);
    compound.insert("key_with_underscores", 6i32);

    assert_eq!(compound.get("").unwrap().as_int_val(), Some(1));
    assert_eq!(compound.get("a").unwrap().as_int_val(), Some(2));
    assert_eq!(compound.get(&long_key).unwrap().as_int_val(), Some(3));
    assert_eq!(
        compound.get("key-with-dashes").unwrap().as_int_val(),
        Some(4)
    );
}

// Test MutValue::map for End type (line 98 coverage)
#[test]
fn test_mut_value_map_end() {
    use na_nbt::value::ValueMut;

    // Create an End value via OwnValue
    let mut val = OwnValue::<BE>::End(());
    let mut_val = val.to_mut();

    mut_val.map(|m| match m {
        MapMut::End(_) => { /* End type handled */ }
        _ => panic!("expected End"),
    });
}

// Test MutValue::map for ByteArray type (line 105 coverage)
#[test]
fn test_mut_value_map_byte_array() {
    use na_nbt::value::ValueMut;
    use na_nbt::view::OwnVec;

    let mut val = OwnValue::<BE>::ByteArray(OwnVec::from(vec![1i8, 2, 3]));
    let mut_val = val.to_mut();

    mut_val.map(|m| match m {
        MapMut::ByteArray(mut arr) => {
            assert_eq!(arr.len(), 3);
            // Modify the array
            arr[0] = 10;
        }
        _ => panic!("expected ByteArray"),
    });

    // Verify modification
    if let OwnValue::ByteArray(arr) = &val {
        assert_eq!(arr[0], 10);
    }
}

// Test MutValue::map for IntArray type (line 109 coverage)
#[test]
fn test_mut_value_map_int_array() {
    use na_nbt::value::ValueMut;
    use na_nbt::view::OwnVec;
    use zerocopy::byteorder::I32;

    let mut val =
        OwnValue::<BE>::IntArray(OwnVec::from(vec![I32::<BE>::new(100), I32::<BE>::new(200)]));
    let mut_val = val.to_mut();

    mut_val.map(|m| match m {
        MapMut::IntArray(mut arr) => {
            assert_eq!(arr.len(), 2);
            // Modify the array
            arr[0] = I32::<BE>::new(999);
        }
        _ => panic!("expected IntArray"),
    });

    // Verify modification
    if let OwnValue::IntArray(arr) = &val {
        assert_eq!(arr[0].get(), 999);
    }
}

// Test MutValue::map for LongArray type (line 110 coverage)
#[test]
fn test_mut_value_map_long_array() {
    use na_nbt::value::ValueMut;
    use na_nbt::view::OwnVec;
    use zerocopy::byteorder::I64;

    let mut val = OwnValue::<BE>::LongArray(OwnVec::from(vec![
        I64::<BE>::new(1000),
        I64::<BE>::new(2000),
    ]));
    let mut_val = val.to_mut();

    mut_val.map(|m| match m {
        MapMut::LongArray(mut arr) => {
            assert_eq!(arr.len(), 2);
            arr[1] = I64::<BE>::new(9999);
        }
        _ => panic!("expected LongArray"),
    });

    // Verify modification
    if let OwnValue::LongArray(arr) = &val {
        assert_eq!(arr[1].get(), 9999);
    }
}

// Test MutValue::map for String type (line 106 coverage)
#[test]
fn test_mut_value_map_string() {
    use na_nbt::value::ValueMut;
    use na_nbt::view::OwnString;

    let mut val = OwnValue::<BE>::String(OwnString::from(String::from("hello")));
    let mut_val = val.to_mut();

    mut_val.map(|m| match m {
        MapMut::String(mut s) => {
            s.clear();
            s.push_str("modified");
        }
        _ => panic!("expected String"),
    });

    // Verify modification
    if let OwnValue::String(s) = &val {
        assert_eq!(s.decode().into_owned(), "modified");
    }
}

// Test MutValue::map for List type (line 107 coverage)
#[test]
fn test_mut_value_map_list() {
    use na_nbt::value::ValueMut;

    let mut list = OwnList::<BE>::default();
    list.push(1i32);
    list.push(2i32);

    let mut val = OwnValue::<BE>::List(list);
    let mut_val = val.to_mut();

    mut_val.map(|m| match m {
        MapMut::List(mut l) => {
            assert_eq!(l.len(), 2);
            l.push(3i32);
        }
        _ => panic!("expected List"),
    });

    // Verify modification
    if let OwnValue::List(l) = &val {
        assert_eq!(l.len(), 3);
    }
}

// Test MutValue::map for Compound type (line 108 coverage)
#[test]
fn test_mut_value_map_compound() {
    use na_nbt::value::ValueMut;

    let mut comp = OwnCompound::<BE>::default();
    comp.insert("key", 42i32);

    let mut val = OwnValue::<BE>::Compound(comp);
    let mut_val = val.to_mut();

    mut_val.map(|m| match m {
        MapMut::Compound(mut c) => {
            assert!(c.get("key").is_some());
            c.insert("new_key", 100i32);
        }
        _ => panic!("expected Compound"),
    });

    // Verify modification
    if let OwnValue::Compound(c) = &val {
        assert!(c.get("new_key").is_some());
    }
}

// Test compound get_mut with non-existent key (line 768 coverage)
#[test]
fn test_compound_get_mut_missing_key() {
    let mut comp = OwnCompound::<BE>::default();
    comp.insert("exists", 42i32);

    let mut val = OwnValue::<BE>::Compound(comp);

    if let MutValue::Compound(mut c) = val.to_mut() {
        // Test get_mut with non-existent key
        assert!(c.get_mut("nonexistent").is_none());

        // Test get with non-existent key
        assert!(c.get("also_nonexistent").is_none());

        // Verify the existing key still works
        assert!(c.get("exists").is_some());
        assert!(c.get_mut("exists").is_some());
    }
}

// Test MutValue visit_shared for all types
#[test]
fn test_mut_value_visit_shared() {
    use na_nbt::value::{ValueMut, VisitMutShared};
    use na_nbt::view::{OwnString, OwnVec};
    use zerocopy::byteorder::{I32, I64};

    // Test End
    let mut val = OwnValue::<BE>::End(());
    val.to_mut().visit_shared(|v| match v {
        VisitMutShared::End(_) => {}
        _ => panic!("expected End"),
    });

    // Test Byte
    let mut val = OwnValue::<BE>::Byte(42);
    val.to_mut().visit_shared(|v| match v {
        VisitMutShared::Byte(b) => assert_eq!(**b, 42),
        _ => panic!("expected Byte"),
    });

    // Test ByteArray
    let mut val = OwnValue::<BE>::ByteArray(OwnVec::from(vec![1i8, 2, 3]));
    val.to_mut().visit_shared(|v| match v {
        VisitMutShared::ByteArray(arr) => assert_eq!(arr.len(), 3),
        _ => panic!("expected ByteArray"),
    });

    // Test IntArray
    let mut val = OwnValue::<BE>::IntArray(OwnVec::from(vec![I32::<BE>::new(1)]));
    val.to_mut().visit_shared(|v| match v {
        VisitMutShared::IntArray(arr) => assert_eq!(arr.len(), 1),
        _ => panic!("expected IntArray"),
    });

    // Test LongArray
    let mut val = OwnValue::<BE>::LongArray(OwnVec::from(vec![I64::<BE>::new(1)]));
    val.to_mut().visit_shared(|v| match v {
        VisitMutShared::LongArray(arr) => assert_eq!(arr.len(), 1),
        _ => panic!("expected LongArray"),
    });

    // Test String
    let mut val = OwnValue::<BE>::String(OwnString::from(String::from("test")));
    val.to_mut().visit_shared(|v| match v {
        VisitMutShared::String(s) => assert_eq!(s.decode(), "test"),
        _ => panic!("expected String"),
    });

    // Test List
    let mut list = OwnList::<BE>::default();
    list.push(1i32);
    let mut val = OwnValue::<BE>::List(list);
    val.to_mut().visit_shared(|v| match v {
        VisitMutShared::List(l) => assert_eq!(l.len(), 1),
        _ => panic!("expected List"),
    });

    // Test Compound
    let mut comp = OwnCompound::<BE>::default();
    comp.insert("k", 1i32);
    let mut val = OwnValue::<BE>::Compound(comp);
    val.to_mut().visit_shared(|v| match v {
        VisitMutShared::Compound(c) => assert!(c.get("k").is_some()),
        _ => panic!("expected Compound"),
    });
}

// Test MutValue visit_shared for primitive types (lines 61-65 coverage)
#[test]
fn test_mut_value_visit_shared_primitives() {
    use na_nbt::value::{ValueMut, VisitMutShared};
    use zerocopy::byteorder::{F32, F64, I16, I32, I64};

    // Test Short
    let mut val = OwnValue::<BE>::Short(I16::<BE>::new(1000));
    val.to_mut().visit_shared(|v| match v {
        VisitMutShared::Short(s) => assert_eq!(s.get(), 1000),
        _ => panic!("expected Short"),
    });

    // Test Int
    let mut val = OwnValue::<BE>::Int(I32::<BE>::new(100000));
    val.to_mut().visit_shared(|v| match v {
        VisitMutShared::Int(i) => assert_eq!(i.get(), 100000),
        _ => panic!("expected Int"),
    });

    // Test Long
    let mut val = OwnValue::<BE>::Long(I64::<BE>::new(10000000000));
    val.to_mut().visit_shared(|v| match v {
        VisitMutShared::Long(l) => assert_eq!(l.get(), 10000000000),
        _ => panic!("expected Long"),
    });

    // Test Float
    let mut val = OwnValue::<BE>::Float(F32::<BE>::new(std::f32::consts::PI));
    val.to_mut().visit_shared(|v| match v {
        VisitMutShared::Float(f) => assert!((f.get() - std::f32::consts::PI).abs() < 0.01),
        _ => panic!("expected Float"),
    });

    // Test Double
    let mut val = OwnValue::<BE>::Double(F64::<BE>::new(std::f64::consts::E));
    val.to_mut().visit_shared(|v| match v {
        VisitMutShared::Double(d) => assert!((d.get() - std::f64::consts::E).abs() < 0.0001),
        _ => panic!("expected Double"),
    });
}

// Test MutValue to_ref for primitive types (lines 117-123 coverage)
#[test]
fn test_mut_value_to_ref_primitives() {
    use zerocopy::byteorder::{F32, F64, I16, I32, I64};

    // Test End to_ref
    let mut val = OwnValue::<BE>::End(());
    {
        let mut_val = val.to_mut();
        let ref_val = mut_val.to_ref();
        assert_eq!(ref_val.tag_id(), TagID::End);
    }

    // Test Byte to_ref
    let mut val = OwnValue::<BE>::Byte(42);
    {
        let mut_val = val.to_mut();
        let ref_val = mut_val.to_ref();
        assert_eq!(ref_val.tag_id(), TagID::Byte);
        if let RefValue::Byte(b) = ref_val {
            assert_eq!(b, 42);
        }
    }

    // Test Short to_ref
    let mut val = OwnValue::<BE>::Short(I16::<BE>::new(1000));
    {
        let mut_val = val.to_mut();
        let ref_val = mut_val.to_ref();
        assert_eq!(ref_val.tag_id(), TagID::Short);
        if let RefValue::Short(s) = ref_val {
            assert_eq!(s, 1000);
        }
    }

    // Test Int to_ref
    let mut val = OwnValue::<BE>::Int(I32::<BE>::new(100000));
    {
        let mut_val = val.to_mut();
        let ref_val = mut_val.to_ref();
        assert_eq!(ref_val.tag_id(), TagID::Int);
        if let RefValue::Int(i) = ref_val {
            assert_eq!(i, 100000);
        }
    }

    // Test Long to_ref
    let mut val = OwnValue::<BE>::Long(I64::<BE>::new(10000000000));
    {
        let mut_val = val.to_mut();
        let ref_val = mut_val.to_ref();
        assert_eq!(ref_val.tag_id(), TagID::Long);
        if let RefValue::Long(l) = ref_val {
            assert_eq!(l, 10000000000);
        }
    }

    // Test Float to_ref
    let mut val = OwnValue::<BE>::Float(F32::<BE>::new(std::f32::consts::PI));
    {
        let mut_val = val.to_mut();
        let ref_val = mut_val.to_ref();
        assert_eq!(ref_val.tag_id(), TagID::Float);
        if let RefValue::Float(f) = ref_val {
            assert!((f - std::f32::consts::PI).abs() < 0.01);
        }
    }

    // Test Double to_ref
    let mut val = OwnValue::<BE>::Double(F64::<BE>::new(std::f64::consts::E));
    {
        let mut_val = val.to_mut();
        let ref_val = mut_val.to_ref();
        assert_eq!(ref_val.tag_id(), TagID::Double);
        if let RefValue::Double(d) = ref_val {
            assert!((d - std::f64::consts::E).abs() < 0.0001);
        }
    }
}

// Test MutValue to_ref for complex types
#[test]
fn test_mut_value_to_ref_complex() {
    use na_nbt::view::{OwnString, OwnVec};
    use zerocopy::byteorder::{I32, I64};

    // Test ByteArray to_ref
    let mut val = OwnValue::<BE>::ByteArray(OwnVec::from(vec![1i8, 2, 3]));
    {
        let mut_val = val.to_mut();
        let ref_val = mut_val.to_ref();
        assert_eq!(ref_val.tag_id(), TagID::ByteArray);
    }

    // Test String to_ref
    let mut val = OwnValue::<BE>::String(OwnString::from(String::from("hello")));
    {
        let mut_val = val.to_mut();
        let ref_val = mut_val.to_ref();
        assert_eq!(ref_val.tag_id(), TagID::String);
        if let RefValue::String(s) = ref_val {
            assert_eq!(s.decode_lossy().into_owned(), "hello");
        }
    }

    // Test List to_ref
    let mut list = OwnList::<BE>::default();
    list.push(42i32);
    let mut val = OwnValue::<BE>::List(list);
    {
        let mut_val = val.to_mut();
        let ref_val = mut_val.to_ref();
        assert_eq!(ref_val.tag_id(), TagID::List);
    }

    // Test Compound to_ref
    let mut comp = OwnCompound::<BE>::default();
    comp.insert("x", 1i32);
    let mut val = OwnValue::<BE>::Compound(comp);
    {
        let mut_val = val.to_mut();
        let ref_val = mut_val.to_ref();
        assert_eq!(ref_val.tag_id(), TagID::Compound);
    }

    // Test IntArray to_ref
    let mut val =
        OwnValue::<BE>::IntArray(OwnVec::from(vec![I32::<BE>::new(1), I32::<BE>::new(2)]));
    {
        let mut_val = val.to_mut();
        let ref_val = mut_val.to_ref();
        assert_eq!(ref_val.tag_id(), TagID::IntArray);
    }

    // Test LongArray to_ref
    let mut val = OwnValue::<BE>::LongArray(OwnVec::from(vec![I64::<BE>::new(100)]));
    {
        let mut_val = val.to_mut();
        let ref_val = mut_val.to_ref();
        assert_eq!(ref_val.tag_id(), TagID::LongArray);
    }
}

// ============== ValueMut trait method tests ==============

// Test ValueMut::ref_<T> - get shared reference to specific type
#[test]
fn test_value_mut_ref_typed() {
    use na_nbt::tag;
    use na_nbt::value::ValueMut;
    use zerocopy::byteorder::I32;

    // Test ref_ on Int
    let mut val = OwnValue::<BE>::Int(I32::<BE>::new(42));
    let mut_val = val.to_mut();

    // Get shared reference to Int
    let int_ref = mut_val.ref_::<tag::Int>();
    assert!(int_ref.is_some());
    assert_eq!(int_ref.unwrap().get(), 42);

    // Try to get wrong type - should return None
    let byte_ref = mut_val.ref_::<tag::Byte>();
    assert!(byte_ref.is_none());

    // Test ref_ on List
    let mut list = OwnList::<BE>::default();
    list.push(1i32);
    list.push(2i32);
    let mut val = OwnValue::<BE>::List(list);
    let mut_val = val.to_mut();

    let list_ref = mut_val.ref_::<tag::List>();
    assert!(list_ref.is_some());
    assert_eq!(list_ref.unwrap().len(), 2);

    // Test ref_ on Compound
    let mut comp = OwnCompound::<BE>::default();
    comp.insert("key", 100i32);
    let mut val = OwnValue::<BE>::Compound(comp);
    let mut_val = val.to_mut();

    let comp_ref = mut_val.ref_::<tag::Compound>();
    assert!(comp_ref.is_some());
    assert!(comp_ref.unwrap().get("key").is_some());
}

// Test ValueMut::mut_<T> - get mutable reference to specific type
#[test]
fn test_value_mut_mut_typed() {
    use na_nbt::tag;
    use na_nbt::value::ValueMut;
    use zerocopy::byteorder::I32;

    // Test mut_ on Int
    let mut val = OwnValue::<BE>::Int(I32::<BE>::new(42));
    {
        let mut mut_val = val.to_mut();

        // Get mutable reference to Int and modify
        let int_mut = mut_val.mut_::<tag::Int>();
        assert!(int_mut.is_some());
        **int_mut.unwrap() = I32::<BE>::new(100);
    }

    // Verify modification
    if let OwnValue::Int(i) = &val {
        assert_eq!(i.get(), 100);
    }

    // Test mut_ on List
    let mut list = OwnList::<BE>::default();
    list.push(1i32);
    let mut val = OwnValue::<BE>::List(list);
    {
        let mut mut_val = val.to_mut();
        let list_mut = mut_val.mut_::<tag::List>();
        assert!(list_mut.is_some());
        list_mut.unwrap().push(2i32);
    }

    // Verify modification
    if let OwnValue::List(l) = &val {
        assert_eq!(l.len(), 2);
    }

    // Test mut_ on Compound
    let mut comp = OwnCompound::<BE>::default();
    comp.insert("a", 1i32);
    let mut val = OwnValue::<BE>::Compound(comp);
    {
        let mut mut_val = val.to_mut();
        let comp_mut = mut_val.mut_::<tag::Compound>();
        assert!(comp_mut.is_some());
        comp_mut.unwrap().insert("b", 2i32);
    }

    // Verify modification
    if let OwnValue::Compound(c) = &val {
        assert!(c.get("b").is_some());
    }
}

// Test ValueMut::into_<T> - convert into specific type
#[test]
fn test_value_mut_into_typed() {
    use na_nbt::tag;
    use na_nbt::value::ValueMut;
    use zerocopy::byteorder::I32;

    // Test into_ on Int
    let mut val = OwnValue::<BE>::Int(I32::<BE>::new(42));
    let mut_val = val.to_mut();

    let int_owned = mut_val.into_::<tag::Int>();
    assert!(int_owned.is_some());
    assert_eq!(int_owned.unwrap().get(), 42);

    // Test into_ with wrong type returns None
    let mut val = OwnValue::<BE>::Int(I32::<BE>::new(42));
    let mut_val = val.to_mut();
    let byte_owned = mut_val.into_::<tag::Byte>();
    assert!(byte_owned.is_none());

    // Test into_ on List
    let mut list = OwnList::<BE>::default();
    list.push(1i32);
    let mut val = OwnValue::<BE>::List(list);
    let mut_val = val.to_mut();

    let list_owned = mut_val.into_::<tag::List>();
    assert!(list_owned.is_some());
    assert_eq!(list_owned.unwrap().len(), 1);
}

// Test ValueMut::get - index-based access on Value
#[test]
fn test_value_mut_get() {
    use na_nbt::value::ValueMut;

    // Test get with usize index on List value
    let mut list = OwnList::<BE>::default();
    list.push(10i32);
    list.push(20i32);
    let mut val = OwnValue::<BE>::List(list);
    let mut_val = val.to_mut();

    let item = mut_val.get(0usize);
    assert!(item.is_some());
    assert_eq!(item.unwrap().as_int_val(), Some(10));

    let item = mut_val.get(1usize);
    assert!(item.is_some());
    assert_eq!(item.unwrap().as_int_val(), Some(20));

    // OOB access
    let item = mut_val.get(2usize);
    assert!(item.is_none());

    // Test get with string index on Compound value
    let mut comp = OwnCompound::<BE>::default();
    comp.insert("x", 100i32);
    comp.insert("y", 200i32);
    let mut val = OwnValue::<BE>::Compound(comp);
    let mut_val = val.to_mut();

    let item = mut_val.get("x");
    assert!(item.is_some());
    assert_eq!(item.unwrap().as_int_val(), Some(100));

    // Missing key
    let item = mut_val.get("z");
    assert!(item.is_none());
}

// Test ValueMut::get_<T> - typed index-based access
#[test]
fn test_value_mut_get_typed() {
    use na_nbt::tag;
    use na_nbt::value::ValueMut;

    // Test get_ on List
    let mut list = OwnList::<BE>::default();
    list.push(42i32);
    let mut val = OwnValue::<BE>::List(list);
    let mut_val = val.to_mut();

    let item = mut_val.get_::<tag::Int>(0usize);
    assert!(item.is_some());
    assert_eq!(item.unwrap(), 42);

    // Wrong type
    let item = mut_val.get_::<tag::Byte>(0usize);
    assert!(item.is_none());

    // Test get_ on Compound
    let mut comp = OwnCompound::<BE>::default();
    comp.insert("num", 99i32);
    let mut val = OwnValue::<BE>::Compound(comp);
    let mut_val = val.to_mut();

    let item = mut_val.get_::<tag::Int>("num");
    assert!(item.is_some());
    assert_eq!(item.unwrap(), 99);
}

// Test ValueMut::get_mut - mutable index-based access
#[test]
fn test_value_mut_get_mut() {
    use na_nbt::value::ValueMut;
    use zerocopy::byteorder::I32;

    // Test get_mut on List
    let mut list = OwnList::<BE>::default();
    list.push(10i32);
    let mut val = OwnValue::<BE>::List(list);
    {
        let mut mut_val = val.to_mut();
        let item = mut_val.get_mut(0usize);
        assert!(item.is_some());
        if let MutValue::Int(i) = item.unwrap() {
            *i = I32::<BE>::new(999);
        }
    }

    // Verify modification
    if let OwnValue::List(l) = &val {
        assert_eq!(l.get(0).unwrap().as_int_val(), Some(999));
    }

    // Test get_mut on Compound
    let mut comp = OwnCompound::<BE>::default();
    comp.insert("val", 50i32);
    let mut val = OwnValue::<BE>::Compound(comp);
    {
        let mut mut_val = val.to_mut();
        let item = mut_val.get_mut("val");
        assert!(item.is_some());
        if let MutValue::Int(i) = item.unwrap() {
            *i = I32::<BE>::new(500);
        }
    }

    // Verify modification
    if let OwnValue::Compound(c) = &val {
        assert_eq!(c.get("val").unwrap().as_int_val(), Some(500));
    }
}

// Test ValueMut::get_mut_<T> - typed mutable index-based access
#[test]
fn test_value_mut_get_mut_typed() {
    use na_nbt::tag;
    use na_nbt::value::ValueMut;
    use zerocopy::byteorder::I32;

    // Test get_mut_ on List
    let mut list = OwnList::<BE>::default();
    list.push(10i32);
    let mut val = OwnValue::<BE>::List(list);
    {
        let mut mut_val = val.to_mut();
        let item = mut_val.get_mut_::<tag::Int>(0usize);
        assert!(item.is_some());
        *item.unwrap() = I32::<BE>::new(888);
    }

    // Verify modification
    if let OwnValue::List(l) = &val {
        assert_eq!(l.get(0).unwrap().as_int_val(), Some(888));
    }

    // Test get_mut_ on Compound
    let mut comp = OwnCompound::<BE>::default();
    comp.insert("n", 25i32);
    let mut val = OwnValue::<BE>::Compound(comp);
    {
        let mut mut_val = val.to_mut();
        let item = mut_val.get_mut_::<tag::Int>("n");
        assert!(item.is_some());
        *item.unwrap() = I32::<BE>::new(250);
    }

    // Verify modification
    if let OwnValue::Compound(c) = &val {
        assert_eq!(c.get("n").unwrap().as_int_val(), Some(250));
    }

    // Test with wrong type
    let mut list = OwnList::<BE>::default();
    list.push(10i32);
    let mut val = OwnValue::<BE>::List(list);
    {
        let mut mut_val = val.to_mut();
        let item = mut_val.get_mut_::<tag::Byte>(0usize);
        assert!(item.is_none());
    }
}

// ============== ListMut trait method tests ==============

// Test ListMut::get and ListMut::get_<T>
#[test]
fn test_list_mut_get_typed() {
    use na_nbt::tag;

    let mut list = OwnList::<BE>::default();
    list.push(100i32);
    list.push(200i32);

    let mut val = OwnValue::<BE>::List(list);
    if let MutValue::List(list_mut) = val.to_mut() {
        // Test get
        let item = list_mut.get(0);
        assert!(item.is_some());
        assert_eq!(item.unwrap().as_int_val(), Some(100));

        // Test get_<T>
        let item = list_mut.get_::<tag::Int>(1);
        assert!(item.is_some());
        assert_eq!(item.unwrap(), 200);

        // Test get_<T> with wrong type
        let item = list_mut.get_::<tag::Byte>(0);
        assert!(item.is_none());

        // Test OOB
        let item = list_mut.get(99);
        assert!(item.is_none());
    }
}

// Test ListMut::get_mut and ListMut::get_mut_<T>
#[test]
fn test_list_mut_get_mut_typed() {
    use na_nbt::tag;
    use zerocopy::byteorder::I32;

    let mut list = OwnList::<BE>::default();
    list.push(10i32);
    list.push(20i32);

    let mut val = OwnValue::<BE>::List(list);
    if let MutValue::List(mut list_mut) = val.to_mut() {
        // Test get_mut
        let item = list_mut.get_mut(0);
        assert!(item.is_some());
        if let MutValue::Int(i) = item.unwrap() {
            *i = I32::<BE>::new(111);
        }

        // Test get_mut_<T>
        let item = list_mut.get_mut_::<tag::Int>(1);
        assert!(item.is_some());
        *item.unwrap() = I32::<BE>::new(222);

        // Test get_mut_<T> with wrong type
        let item = list_mut.get_mut_::<tag::Byte>(0);
        assert!(item.is_none());

        // Verify modifications
        assert_eq!(list_mut.get(0).unwrap().as_int_val(), Some(111));
        assert_eq!(list_mut.get(1).unwrap().as_int_val(), Some(222));
    }
}

// Test ListMut::pop and ListMut::pop_<T>
#[test]
fn test_list_mut_pop_typed() {
    use na_nbt::tag;

    let mut list = OwnList::<BE>::default();
    list.push(1i32);
    list.push(2i32);
    list.push(3i32);

    let mut val = OwnValue::<BE>::List(list);
    if let MutValue::List(mut list_mut) = val.to_mut() {
        // Test pop
        let popped = list_mut.pop();
        assert!(popped.is_some());
        assert_eq!(popped.unwrap().as_int_val(), Some(3));

        // Test pop_<T>
        let popped = list_mut.pop_::<tag::Int>();
        assert!(popped.is_some());
        assert_eq!(popped.unwrap().get(), 2);

        // Test pop_<T> with wrong type
        let popped = list_mut.pop_::<tag::Byte>();
        assert!(popped.is_none());

        // Remaining list should have 1 element
        assert_eq!(list_mut.len(), 1);
    }
}

// Test ListMut::insert
#[test]
fn test_list_mut_insert() {
    let mut list = OwnList::<BE>::default();
    list.push(1i32);
    list.push(3i32);

    let mut val = OwnValue::<BE>::List(list);
    if let MutValue::List(mut list_mut) = val.to_mut() {
        // Insert in the middle
        list_mut.insert(1, 2i32);

        assert_eq!(list_mut.len(), 3);
        assert_eq!(list_mut.get(0).unwrap().as_int_val(), Some(1));
        assert_eq!(list_mut.get(1).unwrap().as_int_val(), Some(2));
        assert_eq!(list_mut.get(2).unwrap().as_int_val(), Some(3));

        // Insert at beginning
        list_mut.insert(0, 0i32);
        assert_eq!(list_mut.get(0).unwrap().as_int_val(), Some(0));

        // Insert at end
        list_mut.insert(4, 4i32);
        assert_eq!(list_mut.get(4).unwrap().as_int_val(), Some(4));
    }
}

// Test ListMut::remove and ListMut::remove_<T>
#[test]
fn test_list_mut_remove_typed() {
    use na_nbt::tag;

    let mut list = OwnList::<BE>::default();
    list.push(10i32);
    list.push(20i32);
    list.push(30i32);

    let mut val = OwnValue::<BE>::List(list);
    if let MutValue::List(mut list_mut) = val.to_mut() {
        // Test remove
        let removed = list_mut.remove(1);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().as_int_val(), Some(20));
        assert_eq!(list_mut.len(), 2);

        // Test remove_<T>
        let removed = list_mut.remove_::<tag::Int>(0);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().get(), 10);

        // Test remove_<T> with wrong type
        let removed = list_mut.remove_::<tag::Byte>(0);
        assert!(removed.is_none());

        // Test OOB remove
        let removed = list_mut.remove(99);
        assert!(removed.is_none());
    }
}

// ============== TypedListMut trait method tests ==============

// Test TypedListMut get, get_mut, pop, insert, remove
#[test]
fn test_typed_list_mut_methods() {
    use na_nbt::tag;
    use zerocopy::byteorder::I32;

    let mut list = OwnList::<BE>::default();
    list.push(100i32);
    list.push(200i32);
    list.push(300i32);

    let mut typed = list.typed_::<tag::Int>().unwrap();

    // Test get
    let val = typed.get(0);
    assert!(val.is_some());
    assert_eq!(val.unwrap(), 100);

    // Test get_mut
    let val = typed.get_mut(1);
    assert!(val.is_some());
    *val.unwrap() = I32::<BE>::new(222);

    // Verify modification
    assert_eq!(typed.get(1).unwrap(), 222);

    // Test insert
    typed.insert(1, I32::<BE>::new(150));
    assert_eq!(typed.len(), 4);
    assert_eq!(typed.get(1).unwrap(), 150);

    // Test remove
    let removed = typed.remove(1);
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().get(), 150);

    // Test pop
    let popped = typed.pop();
    assert!(popped.is_some());
    assert_eq!(popped.unwrap().get(), 300);

    // OOB access
    assert!(typed.get(99).is_none());
    assert!(typed.get_mut(99).is_none());
}

// Test TypedListMut iteration
#[test]
fn test_typed_list_mut_iteration() {
    use na_nbt::tag;
    use zerocopy::byteorder::I32;

    let mut list = OwnList::<BE>::default();
    list.push(1i32);
    list.push(2i32);
    list.push(3i32);

    // Convert to OwnValue to get MutList, then to MutTypedList
    let mut list_value = OwnValue::<BE>::List(list);

    if let MutValue::List(mut_list) = list_value.to_mut() {
        let mut typed = mut_list.typed_::<tag::Int>().unwrap();

        // Test iter
        let sum: i32 = typed.iter().sum();
        assert_eq!(sum, 6);

        // Test iter_mut
        for val in typed.iter_mut() {
            *val = I32::<BE>::new(val.get() * 10);
        }

        // Verify modifications
        assert_eq!(typed.get(0).unwrap(), 10);
        assert_eq!(typed.get(1).unwrap(), 20);
        assert_eq!(typed.get(2).unwrap(), 30);
    }
}

// ============== CompoundMut trait method tests ==============

// Test CompoundMut::get_<T>
#[test]
fn test_compound_mut_get_typed() {
    use na_nbt::tag;

    let mut comp = OwnCompound::<BE>::default();
    comp.insert("int_val", 42i32);
    comp.insert("str_val", "hello");

    let mut val = OwnValue::<BE>::Compound(comp);
    if let MutValue::Compound(comp_mut) = val.to_mut() {
        // Test get_<T>
        let item = comp_mut.get_::<tag::Int>("int_val");
        assert!(item.is_some());
        assert_eq!(item.unwrap(), 42);

        let item = comp_mut.get_::<tag::String>("str_val");
        assert!(item.is_some());
        assert_eq!(item.unwrap().decode_lossy(), "hello");

        // Test get_<T> with wrong type
        let item = comp_mut.get_::<tag::Byte>("int_val");
        assert!(item.is_none());

        // Test get_<T> with missing key
        let item = comp_mut.get_::<tag::Int>("nonexistent");
        assert!(item.is_none());
    }
}

// Test CompoundMut::get_mut_<T>
#[test]
fn test_compound_mut_get_mut_typed() {
    use na_nbt::tag;
    use zerocopy::byteorder::I32;

    let mut comp = OwnCompound::<BE>::default();
    comp.insert("num", 10i32);

    let mut val = OwnValue::<BE>::Compound(comp);
    if let MutValue::Compound(mut comp_mut) = val.to_mut() {
        // Test get_mut_<T>
        let item = comp_mut.get_mut_::<tag::Int>("num");
        assert!(item.is_some());
        *item.unwrap() = I32::<BE>::new(1000);

        // Verify modification
        assert_eq!(comp_mut.get_::<tag::Int>("num").unwrap(), 1000);

        // Test get_mut_<T> with wrong type
        let item = comp_mut.get_mut_::<tag::Byte>("num");
        assert!(item.is_none());

        // Test get_mut_<T> with missing key
        let item = comp_mut.get_mut_::<tag::Int>("missing");
        assert!(item.is_none());
    }
}

// Test CompoundMut::remove
#[test]
fn test_compound_mut_remove() {
    let mut comp = OwnCompound::<BE>::default();
    comp.insert("a", 1i32);
    comp.insert("b", 2i32);
    comp.insert("c", 3i32);

    let mut val = OwnValue::<BE>::Compound(comp);
    if let MutValue::Compound(mut comp_mut) = val.to_mut() {
        // Test remove existing key
        let removed = comp_mut.remove("b");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().as_int_val(), Some(2));

        // Verify key is gone
        assert!(comp_mut.get("b").is_none());

        // Test remove non-existent key
        let removed = comp_mut.remove("nonexistent");
        assert!(removed.is_none());

        // Remaining keys should still exist
        assert!(comp_mut.get("a").is_some());
        assert!(comp_mut.get("c").is_some());
    }
}

// Test CompoundMut::insert
#[test]
fn test_compound_mut_insert() {
    let mut comp = OwnCompound::<BE>::default();
    comp.insert("existing", 1i32);

    let mut val = OwnValue::<BE>::Compound(comp);
    if let MutValue::Compound(mut comp_mut) = val.to_mut() {
        // Test insert new key
        let old = comp_mut.insert("new_key", 42i32);
        assert!(old.is_none());
        assert_eq!(comp_mut.get_::<tag::Int>("new_key").unwrap(), 42);

        // Test insert replacing existing key
        let old = comp_mut.insert("existing", 999i32);
        assert!(old.is_some());
        assert_eq!(old.unwrap().as_int_val(), Some(1));
        assert_eq!(comp_mut.get_::<tag::Int>("existing").unwrap(), 999);

        // Test insert with different type
        let old = comp_mut.insert("existing", "now a string");
        assert!(old.is_some());
        assert_eq!(old.unwrap().as_int_val(), Some(999));
        assert_eq!(
            comp_mut
                .get_::<tag::String>("existing")
                .unwrap()
                .decode_lossy(),
            "now a string"
        );
    }
}

// ============== ListMut with various element types ==============

// Test ListMut operations on Byte list
#[test]
fn test_list_mut_byte_operations() {
    use na_nbt::tag;

    let mut list = OwnList::<BE>::default();
    list.push(1i8);
    list.push(2i8);
    list.push(3i8);

    let mut val = OwnValue::<BE>::List(list);
    if let MutValue::List(mut list_mut) = val.to_mut() {
        // Test get
        let item = list_mut.get(0);
        assert!(item.is_some());
        assert_eq!(item.unwrap().tag_id(), TagID::Byte);

        // Test get_
        let item = list_mut.get_::<tag::Byte>(1);
        assert!(item.is_some());
        assert_eq!(item.unwrap(), 2);

        // Test get_mut
        let item = list_mut.get_mut(0);
        assert!(item.is_some());
        if let MutValue::Byte(b) = item.unwrap() {
            *b = 10;
        }

        // Test get_mut_
        let item = list_mut.get_mut_::<tag::Byte>(1);
        assert!(item.is_some());
        *item.unwrap() = 20;

        // Test insert
        list_mut.insert(1, 15i8);
        assert_eq!(list_mut.len(), 4);

        // Test pop
        let popped = list_mut.pop();
        assert!(popped.is_some());

        // Test pop_
        let popped = list_mut.pop_::<tag::Byte>();
        assert!(popped.is_some());

        // Test remove
        let removed = list_mut.remove(0);
        assert!(removed.is_some());

        // Test remove_
        let removed = list_mut.remove_::<tag::Byte>(0);
        assert!(removed.is_some());
    }
}

// Test ListMut operations on Short list
#[test]
fn test_list_mut_short_operations() {
    use na_nbt::tag;
    use zerocopy::byteorder::I16;

    let mut list = OwnList::<BE>::default();
    list.push(100i16);
    list.push(200i16);

    let mut val = OwnValue::<BE>::List(list);
    if let MutValue::List(mut list_mut) = val.to_mut() {
        // Test get
        assert_eq!(list_mut.get(0).unwrap().tag_id(), TagID::Short);

        // Test get_
        assert_eq!(list_mut.get_::<tag::Short>(0).unwrap(), 100);

        // Test get_mut
        if let MutValue::Short(s) = list_mut.get_mut(0).unwrap() {
            *s = I16::<BE>::new(1000);
        }

        // Test get_mut_
        *list_mut.get_mut_::<tag::Short>(1).unwrap() = I16::<BE>::new(2000);

        // Test insert
        list_mut.insert(1, 1500i16);

        // Test pop
        let _popped = list_mut.pop();

        // Test pop_
        let _popped = list_mut.pop_::<tag::Short>();

        // Test remove
        let _removed = list_mut.remove(0);
    }
}

// Test ListMut operations on Long list
#[test]
fn test_list_mut_long_operations() {
    use na_nbt::tag;
    use zerocopy::byteorder::I64;

    let mut list = OwnList::<BE>::default();
    list.push(1000000i64);
    list.push(2000000i64);

    let mut val = OwnValue::<BE>::List(list);
    if let MutValue::List(mut list_mut) = val.to_mut() {
        // Test get
        assert_eq!(list_mut.get(0).unwrap().tag_id(), TagID::Long);

        // Test get_
        assert_eq!(list_mut.get_::<tag::Long>(0).unwrap(), 1000000);

        // Test get_mut
        if let MutValue::Long(l) = list_mut.get_mut(0).unwrap() {
            *l = I64::<BE>::new(9999999);
        }

        // Test get_mut_
        *list_mut.get_mut_::<tag::Long>(1).unwrap() = I64::<BE>::new(8888888);

        // Test pop
        let _popped = list_mut.pop();

        // Test pop_
        let _popped = list_mut.pop_::<tag::Long>();
    }
}

// Test ListMut operations on Float list
#[test]
fn test_list_mut_float_operations() {
    use na_nbt::tag;
    use zerocopy::byteorder::F32;

    let mut list = OwnList::<BE>::default();
    list.push(1.5f32);
    list.push(2.5f32);

    let mut val = OwnValue::<BE>::List(list);
    if let MutValue::List(mut list_mut) = val.to_mut() {
        // Test get
        assert_eq!(list_mut.get(0).unwrap().tag_id(), TagID::Float);

        // Test get_
        assert!((list_mut.get_::<tag::Float>(0).unwrap() - 1.5).abs() < 0.01);

        // Test get_mut
        if let MutValue::Float(f) = list_mut.get_mut(0).unwrap() {
            *f = F32::<BE>::new(10.5);
        }

        // Test get_mut_
        *list_mut.get_mut_::<tag::Float>(1).unwrap() = F32::<BE>::new(20.5);

        // Test pop
        let _popped = list_mut.pop();

        // Test pop_
        let _popped = list_mut.pop_::<tag::Float>();
    }
}

// Test ListMut operations on Double list
#[test]
fn test_list_mut_double_operations() {
    use na_nbt::tag;
    use zerocopy::byteorder::F64;

    let mut list = OwnList::<BE>::default();
    list.push(1.111f64);
    list.push(2.222f64);

    let mut val = OwnValue::<BE>::List(list);
    if let MutValue::List(mut list_mut) = val.to_mut() {
        // Test get
        assert_eq!(list_mut.get(0).unwrap().tag_id(), TagID::Double);

        // Test get_
        assert!((list_mut.get_::<tag::Double>(0).unwrap() - 1.111).abs() < 0.001);

        // Test get_mut
        if let MutValue::Double(d) = list_mut.get_mut(0).unwrap() {
            *d = F64::<BE>::new(10.111);
        }

        // Test get_mut_
        *list_mut.get_mut_::<tag::Double>(1).unwrap() = F64::<BE>::new(20.222);

        // Test pop
        let _popped = list_mut.pop();

        // Test pop_
        let _popped = list_mut.pop_::<tag::Double>();
    }
}

// Test ListMut operations on String list
#[test]
fn test_list_mut_string_operations() {
    use na_nbt::tag;

    let mut list = OwnList::<BE>::default();
    list.push("hello");
    list.push("world");

    let mut val = OwnValue::<BE>::List(list);
    if let MutValue::List(mut list_mut) = val.to_mut() {
        // Test get
        assert_eq!(list_mut.get(0).unwrap().tag_id(), TagID::String);

        // Test get_
        let s = list_mut.get_::<tag::String>(0);
        assert!(s.is_some());
        assert_eq!(s.unwrap().decode_lossy(), "hello");

        // Test get_mut
        if let MutValue::String(mut s) = list_mut.get_mut(0).unwrap() {
            s.clear();
            s.push_str("modified");
        }

        // Test get_mut_
        let s = list_mut.get_mut_::<tag::String>(1);
        assert!(s.is_some());

        // Test pop
        let _popped = list_mut.pop();

        // Test pop_
        let _popped = list_mut.pop_::<tag::String>();
    }
}

// Test ListMut operations on ByteArray list
#[test]
fn test_list_mut_byte_array_operations() {
    use na_nbt::tag;

    let mut list = OwnList::<BE>::default();
    list.push(vec![1i8, 2, 3]);
    list.push(vec![4i8, 5, 6]);

    let mut val = OwnValue::<BE>::List(list);
    if let MutValue::List(mut list_mut) = val.to_mut() {
        // Test get
        assert_eq!(list_mut.get(0).unwrap().tag_id(), TagID::ByteArray);

        // Test get_
        let arr = list_mut.get_::<tag::ByteArray>(0);
        assert!(arr.is_some());
        assert_eq!(arr.unwrap().len(), 3);

        // Test get_mut
        let item = list_mut.get_mut(0);
        assert!(item.is_some());

        // Test get_mut_
        let arr = list_mut.get_mut_::<tag::ByteArray>(1);
        assert!(arr.is_some());

        // Test pop
        let _popped = list_mut.pop();

        // Test pop_
        let _popped = list_mut.pop_::<tag::ByteArray>();
    }
}

// Test ListMut operations on IntArray list
#[test]
fn test_list_mut_int_array_operations() {
    use na_nbt::tag;
    use zerocopy::byteorder::I32;

    let mut list = OwnList::<BE>::default();
    list.push(vec![I32::<BE>::new(1), I32::<BE>::new(2)]);
    list.push(vec![I32::<BE>::new(3), I32::<BE>::new(4)]);

    let mut val = OwnValue::<BE>::List(list);
    if let MutValue::List(mut list_mut) = val.to_mut() {
        // Test get
        assert_eq!(list_mut.get(0).unwrap().tag_id(), TagID::IntArray);

        // Test get_
        let arr = list_mut.get_::<tag::IntArray>(0);
        assert!(arr.is_some());

        // Test get_mut
        let item = list_mut.get_mut(0);
        assert!(item.is_some());

        // Test get_mut_
        let arr = list_mut.get_mut_::<tag::IntArray>(1);
        assert!(arr.is_some());

        // Test pop
        let _popped = list_mut.pop();

        // Test pop_
        let _popped = list_mut.pop_::<tag::IntArray>();
    }
}

// Test ListMut operations on LongArray list
#[test]
fn test_list_mut_long_array_operations() {
    use na_nbt::tag;
    use zerocopy::byteorder::I64;

    let mut list = OwnList::<BE>::default();
    list.push(vec![I64::<BE>::new(100), I64::<BE>::new(200)]);
    list.push(vec![I64::<BE>::new(300), I64::<BE>::new(400)]);

    let mut val = OwnValue::<BE>::List(list);
    if let MutValue::List(mut list_mut) = val.to_mut() {
        // Test get
        assert_eq!(list_mut.get(0).unwrap().tag_id(), TagID::LongArray);

        // Test get_
        let arr = list_mut.get_::<tag::LongArray>(0);
        assert!(arr.is_some());

        // Test get_mut
        let item = list_mut.get_mut(0);
        assert!(item.is_some());

        // Test get_mut_
        let arr = list_mut.get_mut_::<tag::LongArray>(1);
        assert!(arr.is_some());

        // Test pop
        let _popped = list_mut.pop();

        // Test pop_
        let _popped = list_mut.pop_::<tag::LongArray>();
    }
}

// Test ListMut operations on nested List
#[test]
fn test_list_mut_nested_list_operations() {
    use na_nbt::tag;

    let mut inner1 = OwnList::<BE>::default();
    inner1.push(1i32);
    let mut inner2 = OwnList::<BE>::default();
    inner2.push(2i32);

    let mut outer = OwnList::<BE>::default();
    outer.push(inner1);
    outer.push(inner2);

    let mut val = OwnValue::<BE>::List(outer);
    if let MutValue::List(mut list_mut) = val.to_mut() {
        // Test get
        assert_eq!(list_mut.get(0).unwrap().tag_id(), TagID::List);

        // Test get_
        let inner = list_mut.get_::<tag::List>(0);
        assert!(inner.is_some());

        // Test get_mut
        let item = list_mut.get_mut(0);
        assert!(item.is_some());

        // Test get_mut_
        let inner = list_mut.get_mut_::<tag::List>(1);
        assert!(inner.is_some());

        // Test pop
        let _popped = list_mut.pop();

        // Test pop_
        let _popped = list_mut.pop_::<tag::List>();
    }
}

// Test ListMut operations on Compound list
#[test]
fn test_list_mut_compound_operations() {
    use na_nbt::tag;

    let mut comp1 = OwnCompound::<BE>::default();
    comp1.insert("a", 1i32);
    let mut comp2 = OwnCompound::<BE>::default();
    comp2.insert("b", 2i32);

    let mut list = OwnList::<BE>::default();
    list.push(comp1);
    list.push(comp2);

    let mut val = OwnValue::<BE>::List(list);
    if let MutValue::List(mut list_mut) = val.to_mut() {
        // Test get
        assert_eq!(list_mut.get(0).unwrap().tag_id(), TagID::Compound);

        // Test get_
        let comp = list_mut.get_::<tag::Compound>(0);
        assert!(comp.is_some());

        // Test get_mut
        let item = list_mut.get_mut(0);
        assert!(item.is_some());

        // Test get_mut_
        let comp = list_mut.get_mut_::<tag::Compound>(1);
        assert!(comp.is_some());

        // Test pop
        let _popped = list_mut.pop();

        // Test pop_
        let _popped = list_mut.pop_::<tag::Compound>();
    }
}

// ============== MutValue::visit tests for all types ==============

// Test MutValue::visit for all primitive types
#[test]
fn test_mut_value_visit_all_primitives() {
    use na_nbt::OwnVec;
    use na_nbt::value::{ValueMut, VisitMut};
    use zerocopy::byteorder::{F32, F64, I16, I32, I64};

    // Test visit Byte
    let mut val = OwnValue::<BE>::Byte(42i8);
    val.to_mut().visit(|v| match v {
        VisitMut::Byte(b) => assert_eq!(**b, 42),
        _ => panic!("Expected Byte"),
    });

    // Test visit Short
    let mut val = OwnValue::<BE>::Short(I16::<BE>::new(1000));
    val.to_mut().visit(|v| match v {
        VisitMut::Short(s) => assert_eq!(s.get(), 1000),
        _ => panic!("Expected Short"),
    });

    // Test visit Long
    let mut val = OwnValue::<BE>::Long(I64::<BE>::new(9999999));
    val.to_mut().visit(|v| match v {
        VisitMut::Long(l) => assert_eq!(l.get(), 9999999),
        _ => panic!("Expected Long"),
    });

    // Test visit Float
    let mut val = OwnValue::<BE>::Float(F32::<BE>::new(std::f32::consts::PI));
    val.to_mut().visit(|v| match v {
        VisitMut::Float(f) => assert!((f.get() - std::f32::consts::PI).abs() < 0.01),
        _ => panic!("Expected Float"),
    });

    // Test visit Double
    let mut val = OwnValue::<BE>::Double(F64::<BE>::new(std::f64::consts::E));
    val.to_mut().visit(|v| match v {
        VisitMut::Double(d) => assert!((d.get() - std::f64::consts::E).abs() < 0.0001),
        _ => panic!("Expected Double"),
    });

    // Test visit ByteArray
    let mut val = OwnValue::<BE>::ByteArray(OwnVec::from(vec![1i8, 2, 3]));
    val.to_mut().visit(|v| match v {
        VisitMut::ByteArray(arr) => assert_eq!(arr.len(), 3),
        _ => panic!("Expected ByteArray"),
    });

    // Test visit IntArray
    let mut val =
        OwnValue::<BE>::IntArray(OwnVec::from(vec![I32::<BE>::new(1), I32::<BE>::new(2)]));
    val.to_mut().visit(|v| match v {
        VisitMut::IntArray(arr) => assert_eq!(arr.len(), 2),
        _ => panic!("Expected IntArray"),
    });

    // Test visit LongArray
    let mut val = OwnValue::<BE>::LongArray(OwnVec::from(vec![I64::<BE>::new(100)]));
    val.to_mut().visit(|v| match v {
        VisitMut::LongArray(arr) => assert_eq!(arr.len(), 1),
        _ => panic!("Expected LongArray"),
    });
}

// Test MutValue::visit with mutation
#[test]
fn test_mut_value_visit_mutation() {
    use na_nbt::value::{ValueMut, VisitMut};
    use zerocopy::byteorder::{F32, F64, I16, I64};

    // Test mutating Byte via visit
    let mut val = OwnValue::<BE>::Byte(10i8);
    val.to_mut().visit(|v| {
        if let VisitMut::Byte(b) = v {
            **b = 99;
        }
    });
    if let OwnValue::Byte(b) = &val {
        assert_eq!(*b, 99);
    }

    // Test mutating Short via visit
    let mut val = OwnValue::<BE>::Short(I16::<BE>::new(100));
    val.to_mut().visit(|v| {
        if let VisitMut::Short(s) = v {
            **s = I16::<BE>::new(999);
        }
    });
    if let OwnValue::Short(s) = &val {
        assert_eq!(s.get(), 999);
    }

    // Test mutating Long via visit
    let mut val = OwnValue::<BE>::Long(I64::<BE>::new(1000));
    val.to_mut().visit(|v| {
        if let VisitMut::Long(l) = v {
            **l = I64::<BE>::new(9999);
        }
    });
    if let OwnValue::Long(l) = &val {
        assert_eq!(l.get(), 9999);
    }

    // Test mutating Float via visit
    let mut val = OwnValue::<BE>::Float(F32::<BE>::new(1.0));
    val.to_mut().visit(|v| {
        if let VisitMut::Float(f) = v {
            **f = F32::<BE>::new(99.9);
        }
    });
    if let OwnValue::Float(f) = &val {
        assert!((f.get() - 99.9).abs() < 0.1);
    }

    // Test mutating Double via visit
    let mut val = OwnValue::<BE>::Double(F64::<BE>::new(1.0));
    val.to_mut().visit(|v| {
        if let VisitMut::Double(d) = v {
            **d = F64::<BE>::new(99.99);
        }
    });
    if let OwnValue::Double(d) = &val {
        assert!((d.get() - 99.99).abs() < 0.01);
    }
}

// Test MutValue::visit for String
#[test]
fn test_mut_value_visit_string() {
    use na_nbt::OwnString;
    use na_nbt::value::{ValueMut, VisitMut};

    // Test visit String
    let mut val = OwnValue::<BE>::String(OwnString::from("hello"));
    val.to_mut().visit(|v| match v {
        VisitMut::String(s) => {
            assert_eq!(s.decode(), "hello");
        }
        _ => panic!("Expected String"),
    });
}

// Test MutValue::visit for End type
#[test]
fn test_mut_value_visit_end() {
    use na_nbt::value::{ValueMut, VisitMut};

    // Create OwnValue::End from unit type
    let mut val = OwnValue::<BE>::End(());
    val.to_mut().visit(|v| match v {
        VisitMut::End(e) => {
            // End contains a unit type
            assert_eq!(**e, ());
        }
        _ => panic!("Expected End"),
    });
}

// Test MutValue::visit for arrays with mutation
#[test]
fn test_mut_value_visit_arrays_mutation() {
    use na_nbt::OwnVec;
    use na_nbt::value::{ValueMut, VisitMut};
    use zerocopy::byteorder::{I32, I64};

    // Test mutating ByteArray
    let mut val = OwnValue::<BE>::ByteArray(OwnVec::from(vec![1i8, 2, 3]));
    val.to_mut().visit(|v| {
        if let VisitMut::ByteArray(arr) = v {
            arr[0] = 99;
        }
    });
    if let OwnValue::ByteArray(arr) = &val {
        assert_eq!(arr[0], 99);
    }

    // Test mutating IntArray
    let mut val = OwnValue::<BE>::IntArray(OwnVec::from(vec![I32::<BE>::new(1)]));
    val.to_mut().visit(|v| {
        if let VisitMut::IntArray(arr) = v {
            arr[0] = I32::<BE>::new(999);
        }
    });
    if let OwnValue::IntArray(arr) = &val {
        assert_eq!(arr[0].get(), 999);
    }

    // Test mutating LongArray
    let mut val = OwnValue::<BE>::LongArray(OwnVec::from(vec![I64::<BE>::new(1)]));
    val.to_mut().visit(|v| {
        if let VisitMut::LongArray(arr) = v {
            arr[0] = I64::<BE>::new(9999);
        }
    });
    if let OwnValue::LongArray(arr) = &val {
        assert_eq!(arr[0].get(), 9999);
    }
}

// ============== TypedListMut trait method tests ==============

// Test TypedListMut::get_mut
#[test]
fn test_typed_list_mut_get_mut() {
    use na_nbt::tag;
    use zerocopy::byteorder::I32;

    let mut list = OwnList::<BE>::default();
    list.push(10i32);
    list.push(20i32);
    list.push(30i32);

    let mut val = OwnValue::<BE>::List(list);
    if let MutValue::List(mut_list) = val.to_mut() {
        let mut typed = mut_list.typed_::<tag::Int>().unwrap();

        // Test get_mut
        let item = typed.get_mut(0);
        assert!(item.is_some());
        *item.unwrap() = I32::<BE>::new(100);

        // Verify modification
        assert_eq!(typed.get(0).unwrap(), 100);

        // Test get_mut on middle element
        let item = typed.get_mut(1);
        assert!(item.is_some());
        *item.unwrap() = I32::<BE>::new(200);

        // Test get_mut OOB
        let item = typed.get_mut(99);
        assert!(item.is_none());
    }
}

// Test TypedListMut::push
#[test]
fn test_typed_list_mut_push() {
    use na_nbt::tag;
    use zerocopy::byteorder::I32;

    let mut list = OwnList::<BE>::default();
    list.push(1i32);

    let mut val = OwnValue::<BE>::List(list);
    if let MutValue::List(mut_list) = val.to_mut() {
        let mut typed = mut_list.typed_::<tag::Int>().unwrap();

        // Test push
        typed.push(I32::<BE>::new(2));
        assert_eq!(typed.len(), 2);
        assert_eq!(typed.get(1).unwrap(), 2);

        // Push more elements
        typed.push(I32::<BE>::new(3));
        typed.push(I32::<BE>::new(4));
        assert_eq!(typed.len(), 4);
        assert_eq!(typed.get(3).unwrap(), 4);
    }
}

// Test TypedListMut::pop
#[test]
fn test_typed_list_mut_pop() {
    use na_nbt::tag;

    let mut list = OwnList::<BE>::default();
    list.push(100i32);
    list.push(200i32);
    list.push(300i32);

    let mut val = OwnValue::<BE>::List(list);
    if let MutValue::List(mut_list) = val.to_mut() {
        let mut typed = mut_list.typed_::<tag::Int>().unwrap();

        // Test pop
        let popped = typed.pop();
        assert!(popped.is_some());
        assert_eq!(popped.unwrap().get(), 300);
        assert_eq!(typed.len(), 2);

        // Pop again
        let popped = typed.pop();
        assert!(popped.is_some());
        assert_eq!(popped.unwrap().get(), 200);

        // Pop last element
        let popped = typed.pop();
        assert!(popped.is_some());
        assert_eq!(popped.unwrap().get(), 100);

        // Pop from empty list
        let popped = typed.pop();
        assert!(popped.is_none());
    }
}

// Test TypedListMut::insert
#[test]
fn test_typed_list_mut_insert() {
    use na_nbt::tag;
    use zerocopy::byteorder::I32;

    let mut list = OwnList::<BE>::default();
    list.push(1i32);
    list.push(3i32);

    let mut val = OwnValue::<BE>::List(list);
    if let MutValue::List(mut_list) = val.to_mut() {
        let mut typed = mut_list.typed_::<tag::Int>().unwrap();

        // Insert in the middle
        typed.insert(1, I32::<BE>::new(2));
        assert_eq!(typed.len(), 3);
        assert_eq!(typed.get(0).unwrap(), 1);
        assert_eq!(typed.get(1).unwrap(), 2);
        assert_eq!(typed.get(2).unwrap(), 3);

        // Insert at beginning
        typed.insert(0, I32::<BE>::new(0));
        assert_eq!(typed.get(0).unwrap(), 0);

        // Insert at end
        typed.insert(4, I32::<BE>::new(4));
        assert_eq!(typed.get(4).unwrap(), 4);
    }
}

// Test TypedListMut::remove
#[test]
fn test_typed_list_mut_remove() {
    use na_nbt::tag;

    let mut list = OwnList::<BE>::default();
    list.push(10i32);
    list.push(20i32);
    list.push(30i32);
    list.push(40i32);

    let mut val = OwnValue::<BE>::List(list);
    if let MutValue::List(mut_list) = val.to_mut() {
        let mut typed = mut_list.typed_::<tag::Int>().unwrap();

        // Remove from middle
        let removed = typed.remove(1);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().get(), 20);
        assert_eq!(typed.len(), 3);

        // Verify remaining elements
        assert_eq!(typed.get(0).unwrap(), 10);
        assert_eq!(typed.get(1).unwrap(), 30);
        assert_eq!(typed.get(2).unwrap(), 40);

        // Remove first element
        let removed = typed.remove(0);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().get(), 10);

        // Remove last element
        let removed = typed.remove(1);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().get(), 40);

        // Remove OOB
        let removed = typed.remove(99);
        assert!(removed.is_none());
    }
}

// ============== ValueRef trait method tests ==============

// Test ValueRef::ref_<T>
#[test]
fn test_value_ref_ref_typed() {
    use na_nbt::tag;
    use na_nbt::value::ValueRef;
    use zerocopy::byteorder::I32;

    // Test ref_ on Int
    let val = OwnValue::<BE>::Int(I32::<BE>::new(42));
    let ref_val = val.to_ref();

    let int_ref = ref_val.ref_::<tag::Int>();
    assert!(int_ref.is_some());
    assert_eq!(*int_ref.unwrap(), 42);

    // Test ref_ with wrong type
    let byte_ref = ref_val.ref_::<tag::Byte>();
    assert!(byte_ref.is_none());

    // Test ref_ on List
    let mut list = OwnList::<BE>::default();
    list.push(1i32);
    list.push(2i32);
    let val = OwnValue::<BE>::List(list);
    let ref_val = val.to_ref();

    let list_ref = ref_val.ref_::<tag::List>();
    assert!(list_ref.is_some());
    assert_eq!(list_ref.unwrap().len(), 2);

    // Test ref_ on Compound
    let mut comp = OwnCompound::<BE>::default();
    comp.insert("key", 100i32);
    let val = OwnValue::<BE>::Compound(comp);
    let ref_val = val.to_ref();

    let comp_ref = ref_val.ref_::<tag::Compound>();
    assert!(comp_ref.is_some());
}

// Test ValueRef::get
#[test]
fn test_value_ref_get() {
    use na_nbt::value::ValueRef;

    // Test get on List with usize index
    let mut list = OwnList::<BE>::default();
    list.push(10i32);
    list.push(20i32);
    list.push(30i32);
    let val = OwnValue::<BE>::List(list);
    let ref_val = val.to_ref();

    let item = ref_val.get(0usize);
    assert!(item.is_some());
    assert_eq!(item.unwrap().as_int_val(), Some(10));

    let item = ref_val.get(2usize);
    assert!(item.is_some());
    assert_eq!(item.unwrap().as_int_val(), Some(30));

    // OOB access
    let item = ref_val.get(99usize);
    assert!(item.is_none());

    // Test get on Compound with string index
    let mut comp = OwnCompound::<BE>::default();
    comp.insert("alpha", 1i32);
    comp.insert("beta", 2i32);
    let val = OwnValue::<BE>::Compound(comp);
    let ref_val = val.to_ref();

    let item = ref_val.get("alpha");
    assert!(item.is_some());
    assert_eq!(item.unwrap().as_int_val(), Some(1));

    let item = ref_val.get("beta");
    assert!(item.is_some());
    assert_eq!(item.unwrap().as_int_val(), Some(2));

    // Missing key
    let item = ref_val.get("gamma");
    assert!(item.is_none());
}

// Test ValueRef::get_<T>
#[test]
fn test_value_ref_get_typed() {
    use na_nbt::tag;
    use na_nbt::value::ValueRef;

    // Test get_ on List
    let mut list = OwnList::<BE>::default();
    list.push(42i32);
    list.push(84i32);
    let val = OwnValue::<BE>::List(list);
    let ref_val = val.to_ref();

    let item = ref_val.get_::<tag::Int>(0usize);
    assert!(item.is_some());
    assert_eq!(item.unwrap(), 42);

    let item = ref_val.get_::<tag::Int>(1usize);
    assert!(item.is_some());
    assert_eq!(item.unwrap(), 84);

    // Test get_ with wrong type
    let item = ref_val.get_::<tag::Byte>(0usize);
    assert!(item.is_none());

    // Test get_ on Compound
    let mut comp = OwnCompound::<BE>::default();
    comp.insert("num", 99i32);
    comp.insert("text", "hello");
    let val = OwnValue::<BE>::Compound(comp);
    let ref_val = val.to_ref();

    let item = ref_val.get_::<tag::Int>("num");
    assert!(item.is_some());
    assert_eq!(item.unwrap(), 99);

    let item = ref_val.get_::<tag::String>("text");
    assert!(item.is_some());
    assert_eq!(item.unwrap().decode_lossy(), "hello");

    // Wrong type
    let item = ref_val.get_::<tag::Byte>("num");
    assert!(item.is_none());
}

// ============== ListRef trait method tests ==============

// Test ListRef::get_<T>
#[test]
fn test_list_ref_get_typed() {
    use na_nbt::tag;

    // Test get_ on Int list
    let mut list = OwnList::<BE>::default();
    list.push(100i32);
    list.push(200i32);
    let val = OwnValue::<BE>::List(list);
    if let RefValue::List(list_ref) = val.to_ref() {
        let item = list_ref.get_::<tag::Int>(0);
        assert!(item.is_some());
        assert_eq!(item.unwrap(), 100);

        let item = list_ref.get_::<tag::Int>(1);
        assert!(item.is_some());
        assert_eq!(item.unwrap(), 200);

        // Wrong type
        let item = list_ref.get_::<tag::Byte>(0);
        assert!(item.is_none());

        // OOB
        let item = list_ref.get_::<tag::Int>(99);
        assert!(item.is_none());
    }

    // Test get_ on Byte list
    let mut list = OwnList::<BE>::default();
    list.push(1i8);
    list.push(2i8);
    let val = OwnValue::<BE>::List(list);
    if let RefValue::List(list_ref) = val.to_ref() {
        let item = list_ref.get_::<tag::Byte>(0);
        assert!(item.is_some());
        assert_eq!(item.unwrap(), 1);
    }

    // Test get_ on String list
    let mut list = OwnList::<BE>::default();
    list.push("hello");
    list.push("world");
    let val = OwnValue::<BE>::List(list);
    if let RefValue::List(list_ref) = val.to_ref() {
        let item = list_ref.get_::<tag::String>(0);
        assert!(item.is_some());
        assert_eq!(item.unwrap().decode_lossy(), "hello");
    }
}

// ============== TypedListRef trait method tests ==============

// Test TypedListRef::get
#[test]
fn test_typed_list_ref_get() {
    use na_nbt::tag;

    // Create list and get TypedList reference
    let mut list = OwnList::<BE>::default();
    list.push(10i32);
    list.push(20i32);
    list.push(30i32);

    let val = OwnValue::<BE>::List(list);
    if let RefValue::List(list_ref) = val.to_ref() {
        let typed_ref = list_ref.typed_::<tag::Int>().unwrap();

        // Test get
        let item = typed_ref.get(0);
        assert!(item.is_some());
        assert_eq!(item.unwrap(), 10);

        let item = typed_ref.get(1);
        assert!(item.is_some());
        assert_eq!(item.unwrap(), 20);

        let item = typed_ref.get(2);
        assert!(item.is_some());
        assert_eq!(item.unwrap(), 30);

        // OOB
        let item = typed_ref.get(99);
        assert!(item.is_none());
    }
}

// Test TypedListRef::get with various types
#[test]
fn test_typed_list_ref_get_all_types() {
    use na_nbt::tag;

    // Test Short
    let mut list = OwnList::<BE>::default();
    list.push(100i16);
    let val = OwnValue::<BE>::List(list);
    if let RefValue::List(list_ref) = val.to_ref() {
        let typed_ref = list_ref.typed_::<tag::Short>().unwrap();
        assert_eq!(typed_ref.get(0).unwrap(), 100);
    }

    // Test Long
    let mut list = OwnList::<BE>::default();
    list.push(1000000i64);
    let val = OwnValue::<BE>::List(list);
    if let RefValue::List(list_ref) = val.to_ref() {
        let typed_ref = list_ref.typed_::<tag::Long>().unwrap();
        assert_eq!(typed_ref.get(0).unwrap(), 1000000);
    }

    // Test Float
    let mut list = OwnList::<BE>::default();
    list.push(std::f32::consts::PI);
    let val = OwnValue::<BE>::List(list);
    if let RefValue::List(list_ref) = val.to_ref() {
        let typed_ref = list_ref.typed_::<tag::Float>().unwrap();
        assert!((typed_ref.get(0).unwrap() - std::f32::consts::PI).abs() < 0.01);
    }

    // Test Double
    let mut list = OwnList::<BE>::default();
    list.push(std::f64::consts::E);
    let val = OwnValue::<BE>::List(list);
    if let RefValue::List(list_ref) = val.to_ref() {
        let typed_ref = list_ref.typed_::<tag::Double>().unwrap();
        assert!((typed_ref.get(0).unwrap() - std::f64::consts::E).abs() < 0.0001);
    }

    // Test Byte
    let mut list = OwnList::<BE>::default();
    list.push(42i8);
    let val = OwnValue::<BE>::List(list);
    if let RefValue::List(list_ref) = val.to_ref() {
        let typed_ref = list_ref.typed_::<tag::Byte>().unwrap();
        assert_eq!(typed_ref.get(0).unwrap(), 42);
    }
}

// Test ListRef::get for all element types (covers all branches in ListRef::get)
#[test]
fn test_list_ref_get_all_element_types() {
    use zerocopy::byteorder::{I32, I64};

    // Test Byte list with ListRef::get (not get_<T>)
    let mut list = OwnList::<BE>::default();
    list.push(1i8);
    list.push(2i8);
    let val = OwnValue::<BE>::List(list);
    if let RefValue::List(list_ref) = val.to_ref() {
        let item = list_ref.get(0);
        assert!(item.is_some());
        assert_eq!(item.unwrap().tag_id(), TagID::Byte);
    }

    // Test Short list with ListRef::get
    let mut list = OwnList::<BE>::default();
    list.push(100i16);
    let val = OwnValue::<BE>::List(list);
    if let RefValue::List(list_ref) = val.to_ref() {
        let item = list_ref.get(0);
        assert!(item.is_some());
        assert_eq!(item.unwrap().tag_id(), TagID::Short);
    }

    // Test Long list with ListRef::get
    let mut list = OwnList::<BE>::default();
    list.push(1000000i64);
    let val = OwnValue::<BE>::List(list);
    if let RefValue::List(list_ref) = val.to_ref() {
        let item = list_ref.get(0);
        assert!(item.is_some());
        assert_eq!(item.unwrap().tag_id(), TagID::Long);
    }

    // Test Float list with ListRef::get
    let mut list = OwnList::<BE>::default();
    list.push(std::f32::consts::PI);
    let val = OwnValue::<BE>::List(list);
    if let RefValue::List(list_ref) = val.to_ref() {
        let item = list_ref.get(0);
        assert!(item.is_some());
        assert_eq!(item.unwrap().tag_id(), TagID::Float);
    }

    // Test Double list with ListRef::get
    let mut list = OwnList::<BE>::default();
    list.push(std::f64::consts::E);
    let val = OwnValue::<BE>::List(list);
    if let RefValue::List(list_ref) = val.to_ref() {
        let item = list_ref.get(0);
        assert!(item.is_some());
        assert_eq!(item.unwrap().tag_id(), TagID::Double);
    }

    // Test String list with ListRef::get
    let mut list = OwnList::<BE>::default();
    list.push("hello");
    let val = OwnValue::<BE>::List(list);
    if let RefValue::List(list_ref) = val.to_ref() {
        let item = list_ref.get(0);
        assert!(item.is_some());
        assert_eq!(item.unwrap().tag_id(), TagID::String);
    }

    // Test ByteArray list with ListRef::get
    let mut list = OwnList::<BE>::default();
    list.push(vec![1i8, 2i8, 3i8]);
    let val = OwnValue::<BE>::List(list);
    if let RefValue::List(list_ref) = val.to_ref() {
        let item = list_ref.get(0);
        assert!(item.is_some());
        assert_eq!(item.unwrap().tag_id(), TagID::ByteArray);
    }

    // Test IntArray list with ListRef::get
    let mut list = OwnList::<BE>::default();
    list.push(vec![I32::<BE>::new(1), I32::<BE>::new(2)]);
    let val = OwnValue::<BE>::List(list);
    if let RefValue::List(list_ref) = val.to_ref() {
        let item = list_ref.get(0);
        assert!(item.is_some());
        assert_eq!(item.unwrap().tag_id(), TagID::IntArray);
    }

    // Test LongArray list with ListRef::get
    let mut list = OwnList::<BE>::default();
    list.push(vec![I64::<BE>::new(100)]);
    let val = OwnValue::<BE>::List(list);
    if let RefValue::List(list_ref) = val.to_ref() {
        let item = list_ref.get(0);
        assert!(item.is_some());
        assert_eq!(item.unwrap().tag_id(), TagID::LongArray);
    }

    // Test nested List with ListRef::get
    let mut inner = OwnList::<BE>::default();
    inner.push(1i32);
    let mut outer = OwnList::<BE>::default();
    outer.push(inner);
    let val = OwnValue::<BE>::List(outer);
    if let RefValue::List(list_ref) = val.to_ref() {
        let item = list_ref.get(0);
        assert!(item.is_some());
        assert_eq!(item.unwrap().tag_id(), TagID::List);
    }

    // Test Compound list with ListRef::get
    let mut comp = OwnCompound::<BE>::default();
    comp.insert("key", 1i32);
    let mut list = OwnList::<BE>::default();
    list.push(comp);
    let val = OwnValue::<BE>::List(list);
    if let RefValue::List(list_ref) = val.to_ref() {
        let item = list_ref.get(0);
        assert!(item.is_some());
        assert_eq!(item.unwrap().tag_id(), TagID::Compound);
    }
}
