use na_nbt::{OwnedValue, read_owned};
use zerocopy::byteorder::BigEndian as BE;

fn create_compound_with_lists() -> Vec<u8> {
    // Root compound header
    let mut result = vec![0x0A, 0x00, 0x00];

    // Int List 'ints' [1,2]
    result.push(0x09);
    result.extend_from_slice(&0x0004u16.to_be_bytes());
    result.extend_from_slice(b"ints");
    result.push(0x03); // element type = Int
    result.extend_from_slice(&0x00000002u32.to_be_bytes());
    result.extend_from_slice(&1i32.to_be_bytes());
    result.extend_from_slice(&2i32.to_be_bytes());

    // String list 'strs' ["a"]
    result.push(0x09);
    result.extend_from_slice(&0x0004u16.to_be_bytes());
    result.extend_from_slice(b"strs");
    result.push(0x08);
    result.extend_from_slice(&0x00000001u32.to_be_bytes());
    result.extend_from_slice(&0x0001u16.to_be_bytes());
    result.push(b'a');

    // ByteArray list 'bas' [[1]]
    result.push(0x09);
    result.extend_from_slice(&0x0003u16.to_be_bytes());
    result.extend_from_slice(b"bas");
    result.push(0x07);
    result.extend_from_slice(&0x00000001u32.to_be_bytes());
    result.extend_from_slice(&0x00000001u32.to_be_bytes());
    result.push(1u8);

    // Compound list 'comps' [ { a: 1 } ]
    result.push(0x09);
    result.extend_from_slice(&0x0005u16.to_be_bytes());
    result.extend_from_slice(b"comps");
    result.push(0x0A); // element type Compound
    result.extend_from_slice(&0x00000001u32.to_be_bytes());
    // first compound element
    result.push(0x03); // Int tag inside compound
    result.extend_from_slice(&0x0001u16.to_be_bytes());
    result.push(b'a');
    result.extend_from_slice(&1i32.to_be_bytes());
    result.push(0x00); // end inner compound

    // End root compound
    result.push(0x00);
    result
}

#[test]
fn owned_list_push_insert_unchecked_various() {
    let data = create_compound_with_lists();
    let owned = read_owned::<BE, BE>(&data).unwrap();

    if let OwnedValue::Compound(mut c) = owned {
        // Int list unchecked insert/push
        if let Some(mut mv) = c.get_mut("ints")
            && let na_nbt::MutableValue::List(ref mut l) = mv
        {
            unsafe { l.push_unchecked(3i32) };
            assert_eq!(l.get(2).unwrap().as_int(), Some(3));
        }

        // String list unchecked push
        if let Some(mut mv) = c.get_mut("strs")
            && let na_nbt::MutableValue::List(ref mut l) = mv
        {
            unsafe { l.push_unchecked("bb") };
            assert_eq!(
                l.get(1).unwrap().as_string().unwrap().decode().to_string(),
                "bb"
            );
        }

        // ByteArray list unchecked push/insert
        if let Some(mut mv) = c.get_mut("bas")
            && let na_nbt::MutableValue::List(ref mut l) = mv
        {
            unsafe { l.push_unchecked(vec![7i8, 8i8]) };
            assert_eq!(l.get(1).unwrap().as_byte_array().unwrap()[0], 7i8);
        }

        // Compound list: push an OwnedValue::Compound via OwnedList::push_unchecked
        if let Some(mut mv) = c.get_mut("comps")
            && let na_nbt::MutableValue::List(ref mut l) = mv
        {
            // Construct OwnedCompound with a single int 'b' = 2
            let comp_data = vec![
                0x0A, 0x00, 0x00, 0x03, 0x00, 0x01, b'b', 0x00, 0x00, 0x00, 0x02, 0x00,
            ];
            let owned_comp = read_owned::<BE, BE>(&comp_data).unwrap();
            unsafe { l.push_unchecked(owned_comp) };
            // Reacquire view after potential reallocation
        }
        if let Some(mut mv) = c.get_mut("comps")
            && let na_nbt::MutableValue::List(ref mut l) = mv
        {
            // verify last element's inner int
            let got = l
                .get(1)
                .unwrap()
                .as_compound()
                .unwrap()
                .get("b")
                .unwrap()
                .as_int();
            assert_eq!(got, Some(2));
        }
    } else {
        panic!("expected compound");
    }
}
