use na_nbt::{CompoundMut, CompoundRef, ListBase, ListMut, MutValue, OwnCompound, OwnList, OwnValue, RefValue};
use zerocopy::byteorder::BigEndian as BE;

trait ValueHelper {
    fn as_int_val(&self) -> Option<i32>;
    fn as_string_val(&self) -> Option<String>;
    fn as_float_val(&self) -> Option<f32>;
}

impl<'a> ValueHelper for RefValue<'a, BE> {
    fn as_int_val(&self) -> Option<i32> {
        if let RefValue::Int(i) = self {
            Some(*i)
        } else {
            None
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
        if let RefValue::Float(f) = self {
            Some(*f)
        } else {
            None
        }
    }
}

impl ValueHelper for OwnValue<BE> {
    fn as_int_val(&self) -> Option<i32> {
        if let OwnValue::Int(i) = self {
            Some(i.get())
        } else {
            None
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
        if let OwnValue::Float(f) = self {
            Some(f.get())
        } else {
            None
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
                assert!(v >= 1 && v <= 3, "Value {} not in range 1..=3", v);
            }
        }
        assert_eq!(count, 3);
        
        // Also test into_iter
        let mut into_count = 0;
        for item in mut_list {
            into_count += 1;
            if let MutValue::Int(val) = item {
                let v = val.get();
                assert!(v >= 1 && v <= 3);
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
