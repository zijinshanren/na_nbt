use na_nbt::{OwnedValue, Tag, ScopedReadableValue, ScopedWritableValue};
use zerocopy::byteorder::BigEndian as BE;

#[test]
fn test_owned_value_trait_wrappers() {
    // 1. Byte
    let mut val = OwnedValue::<BE>::from(10i8);
    assert_eq!(ScopedReadableValue::tag_id(&val), Tag::Byte);
    assert!(ScopedReadableValue::is_byte(&val));
    assert_eq!(ScopedReadableValue::as_byte(&val), Some(10));
    assert!(!ScopedReadableValue::is_short(&val));
    assert!(ScopedReadableValue::as_short(&val).is_none());

    assert!(ScopedWritableValue::set_byte(&mut val, 20));
    assert_eq!(ScopedReadableValue::as_byte(&val), Some(20));
    assert!(ScopedWritableValue::update_byte(&mut val, |b| b + 1));
    assert_eq!(ScopedReadableValue::as_byte(&val), Some(21));
    
    // Negative set
    assert!(!ScopedWritableValue::set_short(&mut val, 1));
    assert!(!ScopedWritableValue::update_short(&mut val, |s| s));

    // 2. Short
    let mut val = OwnedValue::<BE>::from(10i16);
    assert!(ScopedReadableValue::is_short(&val));
    assert_eq!(ScopedReadableValue::as_short(&val), Some(10));
    assert!(ScopedWritableValue::set_short(&mut val, 20));
    assert_eq!(ScopedReadableValue::as_short(&val), Some(20));
    assert!(ScopedWritableValue::update_short(&mut val, |v| v + 1));
    assert_eq!(ScopedReadableValue::as_short(&val), Some(21));

    // 3. Int
    let mut val = OwnedValue::<BE>::from(10i32);
    assert!(ScopedReadableValue::is_int(&val));
    assert_eq!(ScopedReadableValue::as_int(&val), Some(10));
    assert!(ScopedWritableValue::set_int(&mut val, 20));
    assert_eq!(ScopedReadableValue::as_int(&val), Some(20));
    assert!(ScopedWritableValue::update_int(&mut val, |v| v + 1));
    assert_eq!(ScopedReadableValue::as_int(&val), Some(21));

    // 4. Long
    let mut val = OwnedValue::<BE>::from(10i64);
    assert!(ScopedReadableValue::is_long(&val));
    assert_eq!(ScopedReadableValue::as_long(&val), Some(10));
    assert!(ScopedWritableValue::set_long(&mut val, 20));
    assert_eq!(ScopedReadableValue::as_long(&val), Some(20));
    assert!(ScopedWritableValue::update_long(&mut val, |v| v + 1));
    assert_eq!(ScopedReadableValue::as_long(&val), Some(21));

    // 5. Float
    let mut val = OwnedValue::<BE>::from(10.5f32);
    assert!(ScopedReadableValue::is_float(&val));
    assert_eq!(ScopedReadableValue::as_float(&val), Some(10.5));
    assert!(ScopedWritableValue::set_float(&mut val, 20.5));
    assert_eq!(ScopedReadableValue::as_float(&val), Some(20.5));
    assert!(ScopedWritableValue::update_float(&mut val, |v| v + 1.0));
    assert_eq!(ScopedReadableValue::as_float(&val), Some(21.5));

    // 6. Double
    let mut val = OwnedValue::<BE>::from(10.5f64);
    assert!(ScopedReadableValue::is_double(&val));
    assert_eq!(ScopedReadableValue::as_double(&val), Some(10.5));
    assert!(ScopedWritableValue::set_double(&mut val, 20.5));
    assert_eq!(ScopedReadableValue::as_double(&val), Some(20.5));
    assert!(ScopedWritableValue::update_double(&mut val, |v| v + 1.0));
    assert_eq!(ScopedReadableValue::as_double(&val), Some(21.5));

    // 7. ByteArray
    let val = OwnedValue::<BE>::from(vec![1i8, 2, 3]);
    assert!(ScopedReadableValue::is_byte_array(&val));
    assert_eq!(ScopedReadableValue::as_byte_array(&val), Some(&[1i8, 2, 3][..]));
    
    // 8. String
    let val = OwnedValue::<BE>::from("hello");
    assert!(ScopedReadableValue::is_string(&val));
    // as_string_scoped returns specialized string type, we just check existence
    assert!(ScopedReadableValue::as_string_scoped(&val).is_some());

    // 9. List
    let val = OwnedValue::<BE>::from(na_nbt::OwnedList::<BE>::default());
    assert!(ScopedReadableValue::is_list(&val));
    assert!(ScopedReadableValue::as_list_scoped(&val).is_some());

    // 10. Compound
    let val = OwnedValue::<BE>::from(na_nbt::OwnedCompound::<BE>::default());
    assert!(ScopedReadableValue::is_compound(&val));
    assert!(ScopedReadableValue::as_compound_scoped(&val).is_some());

    // 11. IntArray
    let val = OwnedValue::<BE>::from(vec![zerocopy::byteorder::I32::<BE>::new(1)]);
    assert!(ScopedReadableValue::is_int_array(&val));
    assert!(ScopedReadableValue::as_int_array(&val).is_some());

    // 12. LongArray
    let val = OwnedValue::<BE>::from(vec![zerocopy::byteorder::I64::<BE>::new(1)]);
    assert!(ScopedReadableValue::is_long_array(&val));
    assert!(ScopedReadableValue::as_long_array(&val).is_some());

    // 13. End
    let val = OwnedValue::<BE>::End;
    assert!(ScopedReadableValue::is_end(&val));
    assert!(ScopedReadableValue::as_end(&val).is_some());
}

#[test]
fn test_mutable_value_trait_wrappers() {
    // We reuse the OwnedValue structure to get MutableValues via get_mut
    use na_nbt::read_owned;
    let mut data = vec![0x0A, 0x00, 0x00];
    
    // Add all types to compound
    // Byte 'b'
    data.push(0x01); data.extend_from_slice(&0x0001u16.to_be_bytes()); data.push(b'b'); data.push(10i8 as u8);
    // Short 's'
    data.push(0x02); data.extend_from_slice(&0x0001u16.to_be_bytes()); data.push(b's'); data.extend_from_slice(&10i16.to_be_bytes());
    // Int 'i'
    data.push(0x03); data.extend_from_slice(&0x0001u16.to_be_bytes()); data.push(b'i'); data.extend_from_slice(&10i32.to_be_bytes());
    // Long 'l'
    data.push(0x04); data.extend_from_slice(&0x0001u16.to_be_bytes()); data.push(b'l'); data.extend_from_slice(&10i64.to_be_bytes());
    // Float 'f'
    data.push(0x05); data.extend_from_slice(&0x0001u16.to_be_bytes()); data.push(b'f'); data.extend_from_slice(&10.5f32.to_be_bytes());
    // Double 'd'
    data.push(0x06); data.extend_from_slice(&0x0001u16.to_be_bytes()); data.push(b'd'); data.extend_from_slice(&10.5f64.to_be_bytes());
    
    data.push(0x00); // end

    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    let mut comp = owned.as_compound_mut().unwrap();

    // 1. Byte
    if let Some(mut mv) = comp.get_mut("b") {
        assert!(ScopedReadableValue::is_byte(&mv));
        assert_eq!(ScopedReadableValue::as_byte(&mv), Some(10));
        assert!(ScopedWritableValue::set_byte(&mut mv, 20));
        assert_eq!(ScopedReadableValue::as_byte(&mv), Some(20));
    } else { panic!("missing b") }

    // 2. Short
    if let Some(mut mv) = comp.get_mut("s") {
        assert!(ScopedReadableValue::is_short(&mv));
        assert_eq!(ScopedReadableValue::as_short(&mv), Some(10));
        assert!(ScopedWritableValue::set_short(&mut mv, 20));
        assert_eq!(ScopedReadableValue::as_short(&mv), Some(20));
    } else { panic!("missing s") }

    // 3. Int
    if let Some(mut mv) = comp.get_mut("i") {
        assert!(ScopedReadableValue::is_int(&mv));
        assert!(ScopedWritableValue::set_int(&mut mv, 20));
        assert_eq!(ScopedReadableValue::as_int(&mv), Some(20));
    } else { panic!("missing i") }

    // 4. Long
    if let Some(mut mv) = comp.get_mut("l") {
        assert!(ScopedReadableValue::is_long(&mv));
        assert!(ScopedWritableValue::set_long(&mut mv, 20));
        assert_eq!(ScopedReadableValue::as_long(&mv), Some(20));
    } else { panic!("missing l") }

    // 5. Float
    if let Some(mut mv) = comp.get_mut("f") {
        assert!(ScopedReadableValue::is_float(&mv));
        assert!(ScopedWritableValue::set_float(&mut mv, 20.5));
        assert_eq!(ScopedReadableValue::as_float(&mv), Some(20.5));
    } else { panic!("missing f") }

    // 6. Double
    if let Some(mut mv) = comp.get_mut("d") {
        assert!(ScopedReadableValue::is_double(&mv));
        assert!(ScopedWritableValue::set_double(&mut mv, 20.5));
        assert_eq!(ScopedReadableValue::as_double(&mv), Some(20.5));
    } else { panic!("missing d") }
}

#[test]
fn test_trait_wrappers_list_compound() {
    use na_nbt::{
        ScopedReadableList, ScopedWritableList, ScopedReadableCompound, ScopedWritableCompound,
        ScopedReadableValue, ScopedWritableValue,
    };
    use na_nbt::read_owned;
    
    // Create nested structure: { "li": [1, 2], "co": { "a": 1 } }
    let mut data = vec![0x0A, 0x00, 0x00]; // root
    // list "li" of ints
    data.push(0x09); data.extend_from_slice(&0x0002u16.to_be_bytes()); data.extend_from_slice(b"li");
    data.push(0x03); // int
    data.extend_from_slice(&2u32.to_be_bytes());
    data.extend_from_slice(&1i32.to_be_bytes());
    data.extend_from_slice(&2i32.to_be_bytes());
    // compound "co"
    data.push(0x0A); data.extend_from_slice(&0x0002u16.to_be_bytes()); data.extend_from_slice(b"co");
    data.push(0x03); data.extend_from_slice(&0x0001u16.to_be_bytes()); data.push(b'a'); data.extend_from_slice(&1i32.to_be_bytes());
    data.push(0x00); // end of "co"
    data.push(0x00); // end of root

    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    let mut root_comp = owned.as_compound_mut().unwrap();

    // === MutableList via traits ===
    if let Some(mut mv) = root_comp.get_mut("li") {
        // as_list_mut_scoped
        let mut list_scoped = ScopedWritableValue::as_list_mut_scoped(&mut mv).unwrap();
        
        // ScopedReadableList
        assert_eq!(ScopedReadableList::len(&list_scoped), 2);
        assert!(!ScopedReadableList::is_empty(&list_scoped));
        assert_eq!(ScopedReadableList::tag_id(&list_scoped), Tag::Int);
        
        // get_scoped
        assert_eq!(ScopedReadableList::get_scoped(&list_scoped, 0).unwrap().as_int(), Some(1));
        
        // iter_scoped
        assert_eq!(ScopedReadableList::iter_scoped(&list_scoped).count(), 2);

        // ScopedWritableList
        // push
        ScopedWritableList::push(&mut list_scoped, 3i32);
        assert_eq!(ScopedReadableList::len(&list_scoped), 3);
        
        // insert
        ScopedWritableList::insert(&mut list_scoped, 0, 0i32);
        assert_eq!(ScopedReadableList::len(&list_scoped), 4);
        assert_eq!(ScopedReadableList::get_scoped(&list_scoped, 0).unwrap().as_int(), Some(0));
        
        // push_unchecked
        unsafe { ScopedWritableList::push_unchecked(&mut list_scoped, 4i32); }
        assert_eq!(ScopedReadableList::len(&list_scoped), 5);

        // insert_unchecked
        unsafe { ScopedWritableList::insert_unchecked(&mut list_scoped, 0, -1i32); }
        assert_eq!(ScopedReadableList::len(&list_scoped), 6);
        assert_eq!(ScopedReadableList::get_scoped(&list_scoped, 0).unwrap().as_int(), Some(-1));

        // pop
        let popped = ScopedWritableList::pop(&mut list_scoped);
        assert!(popped.is_some());
        
        // remove
        let removed = ScopedWritableList::remove(&mut list_scoped, 0);
        assert!(removed.as_int().is_some());
        
        // get_mut (scoped) - requires walking down to value
        if let Some(mut val_mut) = ScopedWritableList::get_mut(&mut list_scoped, 0) {
             ScopedWritableValue::set_int(&mut val_mut, 100);
        }
        assert_eq!(ScopedReadableList::get_scoped(&list_scoped, 0).unwrap().as_int(), Some(100));
        
        // iter_mut
        for mut v in ScopedWritableList::iter_mut(&mut list_scoped) {
            ScopedWritableValue::update_int(&mut v, |x| x + 1);
        }
    } else { panic!("missing li") }

    // === MutableCompound via traits ===
    if let Some(mut mv) = root_comp.get_mut("co") {
        let mut comp_scoped = ScopedWritableValue::as_compound_mut_scoped(&mut mv).unwrap();
        
        // ScopedReadableCompound
        assert!(ScopedReadableCompound::get_scoped(&comp_scoped, "a").is_some());
        assert!(ScopedReadableCompound::iter_scoped(&comp_scoped).count() > 0);
        
        // ScopedWritableCompound
        // insert
        ScopedWritableCompound::insert(&mut comp_scoped, "b", 2i32);
        assert!(ScopedReadableCompound::get_scoped(&comp_scoped, "b").is_some());
        
        // get_mut
        if let Some(mut val_mut) = ScopedWritableCompound::get_mut(&mut comp_scoped, "b") {
            ScopedWritableValue::set_int(&mut val_mut, 20);
        }
        
        // iter_mut
        for (_, mut v) in ScopedWritableCompound::iter_mut(&mut comp_scoped) {
             if let Some(i) = ScopedReadableValue::as_int(&v) {
                 ScopedWritableValue::set_int(&mut v, i + 1);
             }
        }
        
        // remove
        assert!(ScopedWritableCompound::remove(&mut comp_scoped, "a").is_some());
    } else { panic!("missing co") }
    
    // === OwnedList/Compound via traits ===
    // We can use the root 'owned' value which is a Compound
    
    // OwnedCompound
    assert!(ScopedReadableCompound::get_scoped(&owned.as_compound().unwrap(), "li").is_some());
    // owned is OwnedValue::Compound, but traits are implemented for OwnedCompound, not OwnedValue::Compound directly? 
    // No, OwnedValue implements ScopedReadableValue.
    // OwnedCompound implements ScopedReadableCompound.
    
    if let OwnedValue::Compound(ref c) = owned {
        assert!(ScopedReadableCompound::get_scoped(c, "li").is_some());
        assert!(ScopedReadableCompound::iter_scoped(c).count() > 0);
    }
    
    if let OwnedValue::Compound(ref mut c) = owned {
        ScopedWritableCompound::insert(c, "new", 1i8);
        assert!(ScopedWritableCompound::get_mut(c, "new").is_some());
        ScopedWritableCompound::remove(c, "new");
        ScopedWritableCompound::iter_mut(c);
    }
    
    // OwnedList
    // Construct one manually
    let mut list = na_nbt::OwnedList::<BE>::default();
    ScopedWritableList::push(&mut list, 1i32);
    assert_eq!(ScopedReadableList::len(&list), 1);
    assert_eq!(ScopedReadableList::tag_id(&list), Tag::Int);
    assert!(ScopedReadableList::get_scoped(&list, 0).is_some());
    
    // push_unchecked
    unsafe { ScopedWritableList::push_unchecked(&mut list, 2i32); }
    assert_eq!(ScopedReadableList::len(&list), 2);

    ScopedWritableList::pop(&mut list);
    ScopedWritableList::pop(&mut list);
    assert!(ScopedReadableList::is_empty(&list));
    
    ScopedWritableList::push(&mut list, 1i32);
    ScopedWritableList::insert(&mut list, 0, 2i32);
    
    // insert_unchecked
    unsafe { ScopedWritableList::insert_unchecked(&mut list, 0, 3i32); }
    assert_eq!(ScopedReadableList::len(&list), 3);

    ScopedWritableList::remove(&mut list, 0);
    ScopedWritableList::get_mut(&mut list, 0);
    ScopedWritableList::iter_mut(&mut list);
}

#[test]
fn test_owned_value_array_mut_access() {
    // ByteArray
    let mut val = OwnedValue::<BE>::from(vec![1i8, 2, 3]);
    if let Some(mut view) = ScopedWritableValue::as_byte_array_mut_scoped(&mut val) {
        view[0] = 10;
    }
    assert_eq!(ScopedReadableValue::as_byte_array(&val).unwrap()[0], 10);

    // String
    let mut val = OwnedValue::<BE>::from("foo");
    if let Some(mut view) = ScopedWritableValue::as_string_mut_scoped(&mut val) {
        view.push_str("bar");
    }
    assert_eq!(ScopedReadableValue::as_string_scoped(&val).unwrap().decode(), "foobar");

    // IntArray
    let mut val = OwnedValue::<BE>::from(vec![zerocopy::byteorder::I32::<BE>::new(1)]);
    if let Some(mut view) = ScopedWritableValue::as_int_array_mut_scoped(&mut val) {
        view[0].set(2);
    }
    assert_eq!(ScopedReadableValue::as_int_array(&val).unwrap()[0].get(), 2);

    // LongArray
    let mut val = OwnedValue::<BE>::from(vec![zerocopy::byteorder::I64::<BE>::new(1)]);
    if let Some(mut view) = ScopedWritableValue::as_long_array_mut_scoped(&mut val) {
        view[0].set(2);
    }
    assert_eq!(ScopedReadableValue::as_long_array(&val).unwrap()[0].get(), 2);
}

