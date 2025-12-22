use na_nbt::{OwnedList, OwnedCompound, OwnedValue};
use zerocopy::byteorder::BigEndian as BE;

#[test]
#[should_panic(expected = "index out of bounds")]
fn test_list_insert_out_of_bounds() {
    let mut list = OwnedList::<BE>::default();
    list.push(1i8);
    // len is 1, insert at 2 should panic
    list.insert(2, 2i8);
}

#[test]
#[should_panic(expected = "tag mismatch")]
fn test_list_push_tag_mismatch() {
    let mut list = OwnedList::<BE>::default();
    list.push(1i8); // Sets tag to Byte
    list.push(1i16); // Try to push Short
}

#[test]
#[should_panic(expected = "tag mismatch")]
fn test_list_insert_tag_mismatch() {
    let mut list = OwnedList::<BE>::default();
    list.push(1i8);
    list.insert(0, 1i16);
}

// #[test]
// #[should_panic(expected = "tag mismatch")]
// fn test_list_push_value_tag_mismatch() {
//    let mut list = OwnedList::<BE>::default();
//    list.push(1i8);
//    let val = OwnedValue::<BE>::from(1i16);
//    unsafe {
//        use na_nbt::IntoOwnedValue;
//        val.list_push(&mut unsafe { na_nbt::view::VecViewMut::new(&mut list.data.ptr, &mut list.data.len, &mut list.data.cap) });
//    }
// }

// Accessing private fields (list.data) is tricky from integration tests.
// The `OwnedList` methods wrap `util` functions.
// I can trigger "tag mismatch" via `OwnedList::push` easily.
// But `list_insert_value` is internal? No, `OwnedList::insert` uses `IntoOwnedValue::list_insert`.
// `IntoOwnedValue` for `OwnedValue` calls `list_insert_value`.

#[test]
#[should_panic(expected = "tag mismatch")]
fn test_owned_value_list_push_mismatch() {
    let mut list = OwnedList::<BE>::default();
    list.push(1i8);
    let val = OwnedValue::<BE>::from(1i16);
    // This calls OwnedValue::list_push -> util::list_push_value
    list.push(val); 
}

#[test]
#[should_panic(expected = "cannot insert TAG_END")]
fn test_compound_insert_end() {
    let mut comp = OwnedCompound::<BE>::default();
    comp.insert("key", OwnedValue::End);
}

#[test]
fn test_compound_replace_behavior() {
    let mut comp = OwnedCompound::<BE>::default();
    comp.insert("key", 1i8);
    let old = comp.insert("key", 2i8);
    
    assert_eq!(old.unwrap().as_byte(), Some(1));
    assert_eq!(comp.get("key").unwrap().as_byte(), Some(2));
    
    // Replace with different type (should work for Compound)
    let old = comp.insert("key", "string");
    assert_eq!(old.unwrap().as_byte(), Some(2));
    assert_eq!(comp.get("key").unwrap().as_string().unwrap().decode(), "string");
}

#[test]
fn test_list_remove_bounds() {
    let mut list = OwnedList::<BE>::default();
    list.push(1i8);
    let val = list.remove(0);
    assert_eq!(val.as_byte(), Some(1));
    assert!(list.is_empty());
}

#[test]
#[should_panic(expected = "index out of bounds")]
fn test_list_remove_out_of_bounds() {
    let mut list = OwnedList::<BE>::default();
    list.remove(0);
}


