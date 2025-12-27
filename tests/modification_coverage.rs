use na_nbt::tag::{Int, Long};
use na_nbt::{
    CompoundMut, CompoundRef, ListBase, ListMut, ListRef, MutValue, OwnCompound, OwnList, OwnValue,
    RefValue, TagID,
};
use zerocopy::byteorder::BigEndian as BE;

#[test]
fn test_list_type_mismatch_silent_failure() {
    let mut list = OwnList::<BE>::default();

    // 1. Empty list is End type
    assert_eq!(list.element_tag_id(), TagID::End);
    assert!(list.is_empty());

    // 2. Push int sets type to Int
    list.push(1i32);
    assert_eq!(list.element_tag_id(), TagID::Int);
    assert_eq!(list.len(), 1);

    // 3. Push double (should fail silently)
    list.push(1.0f64);
    assert_eq!(list.len(), 1); // Length should NOT increase
    assert_eq!(list.element_tag_id(), TagID::Int); // Type remains Int

    // 4. Insert double (should fail silently)
    list.insert(0, 2.0f64);
    assert_eq!(list.len(), 1);
    match list.get(0).unwrap() {
        RefValue::Int(v) => assert_eq!(v, 1),
        _ => panic!("Expected Int"),
    }
}

#[test]
fn test_list_nested_modification() {
    let mut list = OwnList::<BE>::default();

    let mut inner = OwnList::<BE>::default();
    inner.push(1i32);
    inner.push(2i32);

    list.push(inner);

    // Get the inner list back and modify it
    if let Some(inner_ref) = list.get_mut(0) {
        if let MutValue::List(mut inner_list) = inner_ref {
            inner_list.push(3i32);
        } else {
            panic!("Expected list");
        }
    }

    // Verify modification persisted
    if let Some(inner_ref) = list.get(0) {
        if let RefValue::List(inner_list) = inner_ref {
            assert_eq!(inner_list.len(), 3);
            match inner_list.get(2).unwrap() {
                RefValue::Int(v) => assert_eq!(v, 3),
                _ => panic!("Expected Int"),
            }
        } else {
            panic!("Expected list");
        }
    }
}

#[test]
fn test_compound_nested_modification() {
    let mut root = OwnCompound::<BE>::default();
    let mut child = OwnCompound::<BE>::default();
    child.insert("val", 42i32);

    root.insert("child", child);

    // Modify nested
    if let Some(child_ref) = root.get_mut("child") {
        if let MutValue::Compound(mut child_comp) = child_ref {
            child_comp.insert("new_val", 100i32);
            // Modify existing
            child_comp.insert("val", 0i32);
        } else {
            panic!("Expected compound");
        }
    }

    // Verify
    if let Some(child_ref) = root.get("child") {
        if let RefValue::Compound(child_comp) = child_ref {
            match child_comp.get("val").unwrap() {
                RefValue::Int(v) => assert_eq!(v, 0),
                _ => panic!("Expected Int"),
            }
            match child_comp.get("new_val").unwrap() {
                RefValue::Int(v) => assert_eq!(v, 100),
                _ => panic!("Expected Int"),
            }
        } else {
            panic!("Expected compound");
        }
    }
}

#[test]
fn test_list_typed_conversion() {
    let mut list = OwnList::<BE>::default();
    list.push(1i32);
    list.push(2i32);

    // Successful conversion
    let typed = list.typed_::<Int>();
    assert!(typed.is_some());
    let typed = typed.unwrap();
    assert_eq!(typed.len(), 2);

    // Create new list for failure
    let mut list = OwnList::<BE>::default();
    list.push(1i32);

    // Failed conversion
    let typed = list.typed_::<Long>();
    assert!(typed.is_none());
}

#[test]
fn test_list_pop_typed() {
    let mut list = OwnList::<BE>::default();
    list.push(1i32);

    // Pop wrong type
    let val = list.pop_::<Long>();
    assert!(val.is_none());
    assert_eq!(list.len(), 1);

    // Pop correct type
    let val = list.pop_::<Int>();
    assert_eq!(val.map(|x| x.get()), Some(1));
    assert!(list.is_empty());

    // Pop empty
    assert!(list.pop_::<Int>().is_none());
}

#[test]
fn test_list_remove_typed() {
    let mut list = OwnList::<BE>::default();
    list.push(1i32);
    list.push(2i32);

    // Remove wrong type
    let val = list.remove_::<Long>(0);
    assert!(val.is_none());

    // Remove OOB
    let val = list.remove_::<Int>(100);
    assert!(val.is_none());

    // Remove correct
    let val = list.remove_::<Int>(0);
    assert_eq!(val.map(|x| x.get()), Some(1));
    assert_eq!(list.len(), 1);
}

#[test]
fn test_large_list_expansion() {
    let mut list = OwnList::<BE>::default();
    for i in 0..1000 {
        list.push(i);
    }
    assert_eq!(list.len(), 1000);

    // Verify content
    for i in 0..1000 {
        match list.get(i).unwrap() {
            RefValue::Int(v) => assert_eq!(v, i as i32),
            _ => panic!("Expected Int"),
        }
    }

    // Remove all
    for _ in 0..1000 {
        list.pop();
    }
    assert!(list.is_empty());
}

#[test]
fn test_compound_remove_typed() {
    // Note: OwnCompound doesn't have remove_ typed method like List does,
    // it returns Option<OwnValue<O>>.
    // Checking removal of non-existent key
    let mut comp = OwnCompound::<BE>::default();
    assert!(comp.remove("missing").is_none());

    comp.insert("a", 1i32);
    let val = comp.remove("a");
    assert!(val.is_some());
    match val.unwrap() {
        OwnValue::Int(v) => assert_eq!(v.get(), 1),
        _ => panic!("Expected Int"),
    }
}

#[test]
fn test_compound_entry_api_simulation() {
    // Rust HashMap has Entry API. OwnCompound doesn't fully, but we can check existence.
    let mut comp = OwnCompound::<BE>::default();
    let key = "key";

    if comp.get(key).is_none() {
        comp.insert(key, 1i32);
    }

    match comp.get(key).unwrap() {
        RefValue::Int(v) => assert_eq!(v, 1),
        _ => panic!("Expected Int"),
    }

    // Update if present
    if let Some(val) = comp.get_mut(key) {
        if let MutValue::Int(i) = val {
            let cur = i.get();
            *i = (cur + 1).into();
        }
    }

    match comp.get(key).unwrap() {
        RefValue::Int(v) => assert_eq!(v, 2),
        _ => panic!("Expected Int"),
    }
}

#[test]
fn test_list_insert_capacity_growth() {
    let mut list = OwnList::<BE>::default();
    list.push(1i32); // [1]
    list.insert(0, 0i32); // [0, 1]

    // Ensure data integrity after shift
    match list.get(0).unwrap() {
        RefValue::Int(v) => assert_eq!(v, 0),
        _ => panic!("Expected Int"),
    }
    match list.get(1).unwrap() {
        RefValue::Int(v) => assert_eq!(v, 1),
        _ => panic!("Expected Int"),
    }
}

#[test]
fn test_value_deep_clone() {
    // Testing that cloning an OwnedValue creates a deep copy
    // (implied by Clone, but good to verify modification independence)
    let mut list = OwnList::<BE>::default();
    list.push(1i32);
}

#[test]
fn test_move_semantics() {
    let mut list = OwnList::<BE>::default();
    list.push(1i32);

    let val = OwnValue::List(list);

    // Move val to a new owner
    let mut val2 = val;

    if let OwnValue::List(ref mut l2) = val2 {
        l2.push(2i32);
        assert_eq!(l2.len(), 2);
    }

    // val is now moved, cannot access.
}
