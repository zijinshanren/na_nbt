use na_nbt::{OwnedList, OwnedCompound};
use zerocopy::byteorder::BigEndian as BE;

#[test]
fn test_owned_list_iter_drop_partial() {
    // specific test to cover Drop impl of OwnedListIter
    // We need complex types that actually have Drop logic (nested lists/compounds/arrays)
    
    // Create a list of lists
    let mut list = OwnedList::<BE>::default();
    let mut inner1 = OwnedList::<BE>::default();
    inner1.push(1i32);
    list.push(inner1);
    
    let mut inner2 = OwnedList::<BE>::default();
    inner2.push(2i32);
    list.push(inner2);
    
    let mut inner3 = OwnedList::<BE>::default();
    inner3.push(3i32);
    list.push(inner3);
    
    // Iterate partially
    let mut iter = list.into_iter();
    let first = iter.next();
    assert!(first.is_some());
    
    // Drop iter here. inner2 and inner3 should be dropped by iter.drop()
    drop(iter);
}

#[test]
fn test_owned_compound_iter_drop_partial() {
    // specific test to cover Drop impl of OwnedCompoundIter
    let mut comp = OwnedCompound::<BE>::default();
    
    let mut inner1 = OwnedCompound::<BE>::default();
    inner1.insert("a", 1i32);
    comp.insert("c1", inner1);
    
    let mut inner2 = OwnedCompound::<BE>::default();
    inner2.insert("b", 2i32);
    comp.insert("c2", inner2);
    
    let mut inner3 = OwnedCompound::<BE>::default();
    inner3.insert("c", 3i32);
    comp.insert("c3", inner3);
    
    // Iterate partially
    let mut iter = comp.into_iter();
    let first = iter.next();
    assert!(first.is_some());
    
    // Drop iter here. Remaining elements should be dropped.
    drop(iter);
}

#[test]
fn test_owned_list_iter_drop_all_types() {
    // Create lists of all complex types to trigger all branches in Drop
    
    // ByteArray
    let mut l = OwnedList::<BE>::default();
    l.push(vec![1i8, 2, 3]);
    l.push(vec![4i8, 5, 6]);
    let mut iter = l.into_iter();
    iter.next();
    drop(iter);

    // String
    let mut l = OwnedList::<BE>::default();
    l.push("s1");
    l.push("s2");
    let mut iter = l.into_iter();
    iter.next();
    drop(iter);
    
    // IntArray
    let mut l = OwnedList::<BE>::default();
    l.push(vec![zerocopy::byteorder::I32::<BE>::new(1)]);
    l.push(vec![zerocopy::byteorder::I32::<BE>::new(2)]);
    let mut iter = l.into_iter();
    iter.next();
    drop(iter);
    
    // LongArray
    let mut l = OwnedList::<BE>::default();
    l.push(vec![zerocopy::byteorder::I64::<BE>::new(1)]);
    l.push(vec![zerocopy::byteorder::I64::<BE>::new(2)]);
    let mut iter = l.into_iter();
    iter.next();
    drop(iter);
}

#[test]
fn test_owned_compound_iter_drop_all_types() {
    // All complex types in compound
    let mut c = OwnedCompound::<BE>::default();
    c.insert("ba", vec![1i8]);
    c.insert("st", "str");
    c.insert("li", OwnedList::<BE>::default());
    c.insert("co", OwnedCompound::<BE>::default());
    c.insert("ia", vec![zerocopy::byteorder::I32::<BE>::new(1)]);
    c.insert("la", vec![zerocopy::byteorder::I64::<BE>::new(1)]);
    
    let mut iter = c.into_iter();
    iter.next(); // consume one
    drop(iter); // drop rest
}

