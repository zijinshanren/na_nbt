use na_nbt::{OwnedList, OwnedCompound, OwnedValue};
use zerocopy::byteorder::BigEndian as BE;

#[test]
fn test_list_modification_primitives() {
    let mut list = OwnedList::<BE>::default();
    
    // Push
    list.push(1i32);
    list.push(2i32);
    list.push(3i32);
    assert_eq!(list.len(), 3);
    
    // Insert middle
    list.insert(1, 4i32); // [1, 4, 2, 3]
    assert_eq!(list.get(1).unwrap().as_int(), Some(4));
    assert_eq!(list.get(2).unwrap().as_int(), Some(2));
    
    // Insert start
    list.insert(0, 5i32); // [5, 1, 4, 2, 3]
    assert_eq!(list.get(0).unwrap().as_int(), Some(5));
    
    // Insert end
    list.insert(5, 6i32); // [5, 1, 4, 2, 3, 6]
    assert_eq!(list.get(5).unwrap().as_int(), Some(6));
    
    // Remove middle
    let removed = list.remove(2); // Remove 4
    assert_eq!(removed.as_int(), Some(4));
    assert_eq!(list.len(), 5); // [5, 1, 2, 3, 6]
    assert_eq!(list.get(2).unwrap().as_int(), Some(2));
    
    // Remove start
    let removed = list.remove(0); // Remove 5
    assert_eq!(removed.as_int(), Some(5));
    assert_eq!(list.len(), 4); // [1, 2, 3, 6]
    assert_eq!(list.get(0).unwrap().as_int(), Some(1));
    
    // Remove end
    let removed = list.remove(3); // Remove 6
    assert_eq!(removed.as_int(), Some(6));
    assert_eq!(list.len(), 3); // [1, 2, 3]
    
    // Pop
    let popped = list.pop(); // Pop 3
    assert_eq!(popped.unwrap().as_int(), Some(3));
    assert_eq!(list.len(), 2); // [1, 2]
}

#[test]
fn test_list_modification_complex() {
    // List of Strings
    let mut list = OwnedList::<BE>::default();
    list.push("a");
    list.push("b");
    list.push("c");
    
    let removed = list.remove(1); // "b"
    if let OwnedValue::String(s) = removed {
        assert_eq!(s.decode(), "b");
    } else { panic!("Expected string"); }
    
    assert_eq!(list.len(), 2);
    assert_eq!(list.get(0).unwrap().as_string().unwrap().decode(), "a");
    assert_eq!(list.get(1).unwrap().as_string().unwrap().decode(), "c");
    
    list.insert(1, "d"); // [a, d, c]
    assert_eq!(list.get(1).unwrap().as_string().unwrap().decode(), "d");
}

#[test]
fn test_compound_modification() {
    let mut comp = OwnedCompound::<BE>::default();
    
    // Insert
    comp.insert("a", 1i32);
    comp.insert("b", 2i32);
    
    // Insert replace
    let old = comp.insert("a", 3i32);
    assert_eq!(old.unwrap().as_int(), Some(1));
    assert_eq!(comp.get("a").unwrap().as_int(), Some(3));
    
    // Remove
    let removed = comp.remove("b");
    assert_eq!(removed.unwrap().as_int(), Some(2));
    assert!(comp.get("b").is_none());
    
    // Remove non-existent
    assert!(comp.remove("z").is_none());
}

#[test]
fn test_compound_insert_complex_replace() {
    let mut comp = OwnedCompound::<BE>::default();
    
    // Insert String
    comp.insert("s", "hello");
    
    // Replace with different length string
    let old = comp.insert("s", "world!"); // longer
    if let OwnedValue::String(s) = old.unwrap() {
        assert_eq!(s.decode(), "hello");
    } else { panic!("Expected string"); }
    
    assert_eq!(comp.get("s").unwrap().as_string().unwrap().decode(), "world!");
    
    // Replace with shorter string
    let old = comp.insert("s", "hi"); 
    assert_eq!(old.unwrap().as_string().unwrap().decode(), "world!");
    assert_eq!(comp.get("s").unwrap().as_string().unwrap().decode(), "hi");
    
    // Ensure structure is intact by inserting another field
    comp.insert("i", 42i32);
    assert_eq!(comp.get("s").unwrap().as_string().unwrap().decode(), "hi");
    assert_eq!(comp.get("i").unwrap().as_int(), Some(42));
}

#[test]
#[should_panic(expected = "index out of bounds")]
fn test_list_insert_oob() {
    let mut list = OwnedList::<BE>::default();
    list.insert(1, 1i32);
}

#[test]
#[should_panic(expected = "index out of bounds")]
fn test_list_remove_oob() {
    let mut list = OwnedList::<BE>::default();
    list.remove(0);
}

