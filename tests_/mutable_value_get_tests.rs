use na_nbt::read_owned;
use zerocopy::byteorder::BigEndian as BE;

#[test]
fn test_mutable_value_get_indexing() {
    let mut data = vec![0x0A, 0x00, 0x00]; // Root
    // List "li" [1, 2]
    data.push(0x09);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"li");
    data.push(0x03);
    data.extend_from_slice(&2u32.to_be_bytes());
    data.extend_from_slice(&1i32.to_be_bytes());
    data.extend_from_slice(&2i32.to_be_bytes());
    // Int "val" = 10
    data.push(0x03);
    data.extend_from_slice(&3u16.to_be_bytes());
    data.extend_from_slice(b"val");
    data.extend_from_slice(&10i32.to_be_bytes());
    data.push(0x00);

    let mut owned = read_owned::<BE, BE>(&data).unwrap();

    // We need a MutableValue to call get() on.
    // owned.as_compound_mut() returns MutableCompound which has get() and get_mut().
    // But MutableValue also has get().

    // Get MutableCompound
    let mut comp = owned.as_compound_mut().unwrap();

    // Test MutableCompound::get (immutable access)
    assert!(comp.get("val").is_some());
    assert_eq!(comp.get("val").unwrap().as_int(), Some(10));

    // Test MutableValue::get (via get_mut from parent)
    // We get a MutableValue ref from comp.get_mut("li")
    if let Some(list_mv) = comp.get_mut("li") {
        // Now list_mv is MutableValue::List. It has get() method.

        // Index with usize
        assert_eq!(list_mv.get(0_usize).unwrap().as_int(), Some(1));
        assert_eq!(list_mv.get(1_usize).unwrap().as_int(), Some(2));
        assert!(list_mv.get(2_usize).is_none());

        // Index with &usize reference? No, index dispatch handles &T.
    } else {
        panic!("missing list");
    }

    // Test MutableValue::get on Compound
    // We can't easily get a MutableValue::Compound that wraps the root, unless we wrap owned in something?
    // owned is OwnedValue. owned.as_compound_mut() gives MutableCompound.
    // To get MutableValue::Compound, we need it to be inside another structure.

    // Let's create a nested compound.
    let mut nested_data = vec![0x0A, 0x00, 0x00];
    nested_data.push(0x0A); // Compound "sub"
    nested_data.extend_from_slice(&3u16.to_be_bytes());
    nested_data.extend_from_slice(b"sub");
    nested_data.push(0x03); // int "i" = 5
    nested_data.extend_from_slice(&1u16.to_be_bytes());
    nested_data.extend_from_slice(b"i");
    nested_data.extend_from_slice(&5i32.to_be_bytes());
    nested_data.push(0x00); // end sub
    nested_data.push(0x00); // end root

    let mut nested_owned = read_owned::<BE, BE>(&nested_data).unwrap();
    let mut root_comp = nested_owned.as_compound_mut().unwrap();

    if let Some(sub_mv) = root_comp.get_mut("sub") {
        // sub_mv is MutableValue::Compound
        // Test get with &str
        assert_eq!(sub_mv.get("i").unwrap().as_int(), Some(5));

        // Test get with String
        assert_eq!(sub_mv.get(String::from("i")).unwrap().as_int(), Some(5));

        // Test get with &String
        let s = String::from("i");
        assert_eq!(sub_mv.get(&s).unwrap().as_int(), Some(5));
    } else {
        panic!("missing sub");
    }
}
