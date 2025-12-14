use na_nbt::read_owned;
use zerocopy::byteorder::BigEndian as BE;

#[allow(clippy::needless_borrows_for_generic_args)]
#[test]
fn test_index_mut_access() {
    let mut data = vec![0x0A, 0x00, 0x00]; // root compound
    // list "li" [1, 2]
    data.push(0x09);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"li");
    data.push(0x03);
    data.extend_from_slice(&2u32.to_be_bytes());
    data.extend_from_slice(&1i32.to_be_bytes());
    data.extend_from_slice(&2i32.to_be_bytes());
    // int "val" = 10
    data.push(0x03);
    data.extend_from_slice(&3u16.to_be_bytes());
    data.extend_from_slice(b"val");
    data.extend_from_slice(&10i32.to_be_bytes());
    data.push(0x00);

    let mut owned = read_owned::<BE, BE>(&data).unwrap();

    // Index mut with &str
    if let Some(mut mv) = owned.get_mut("val") {
        mv.set_int(20);
    }
    assert_eq!(owned.get("val").unwrap().as_int(), Some(20));

    // Index mut with String
    let key = String::from("val");
    if let Some(mut mv) = owned.get_mut(key.clone()) {
        // Pass String by value
        mv.set_int(30);
    }
    assert_eq!(owned.get("val").unwrap().as_int(), Some(30));

    // Index mut with &String
    if let Some(mut mv) = owned.get_mut(&key) {
        mv.set_int(40);
    }
    assert_eq!(owned.get("val").unwrap().as_int(), Some(40));

    // Index mut with usize on list
    if let Some(mut list) = owned.get_mut("li") {
        if let Some(mut el) = list.get_mut(0_usize) {
            // Pass usize by value
            el.set_int(100);
        }
        if let Some(mut el) = list.get_mut(&1_usize) {
            // Pass &usize
            el.set_int(200);
        }
    }

    // Check list values
    let list = owned.get("li").unwrap();
    assert_eq!(list.get(0).unwrap().as_int(), Some(100));
    assert_eq!(list.get(1).unwrap().as_int(), Some(200));

    // Index mut with nested references
    let idx = 0_usize;
    if let Some(mut list) = owned.get_mut("li")
        && let Some(mut el) = list.get_mut(&&idx)
    {
        // Pass &&usize
        el.set_int(101);
    }
    assert_eq!(owned.get("li").unwrap().get(0).unwrap().as_int(), Some(101));
}
