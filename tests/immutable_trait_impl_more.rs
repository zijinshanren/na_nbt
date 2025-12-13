use na_nbt::{read_borrowed, ReadableList, ReadableCompound, ReadableValue, ScopedReadableList, ReadableString, ScopedReadableValue};
use zerocopy::byteorder::BigEndian as BE;

fn create_complex_compound() -> Vec<u8> {
    let mut result = vec![0x0A, 0x00, 0x00];
    // name length 0 for root
    // Int 'i' = 1
    result.push(0x03);
    result.extend_from_slice(&0x0001u16.to_be_bytes());
    result.push(b'i');
    result.extend_from_slice(&1i32.to_be_bytes());

    // String 's' = "s"
    result.push(0x08);
    result.extend_from_slice(&0x0001u16.to_be_bytes());
    result.push(b's');
    result.extend_from_slice(&0x0001u16.to_be_bytes());
    result.push(b's');

    // List 'lst' strings ["x","y"]
    result.push(0x09);
    result.extend_from_slice(&0x0003u16.to_be_bytes());
    result.extend_from_slice(b"lst");
    result.push(0x08);
    result.extend_from_slice(&0x00000002u32.to_be_bytes());
    result.extend_from_slice(&0x0001u16.to_be_bytes());
    result.push(b'x');
    result.extend_from_slice(&0x0001u16.to_be_bytes());
    result.push(b'y');

    // compound 'sub' { 'j': 2 }
    result.push(0x0A);
    result.extend_from_slice(&0x0003u16.to_be_bytes());
    result.extend_from_slice(b"sub");
    result.push(0x03);
    result.extend_from_slice(&0x0001u16.to_be_bytes());
    result.push(b'j');
    result.extend_from_slice(&2i32.to_be_bytes());
    result.push(0x00);

    result.push(0x00);
    result
}

#[test]
fn immutable_trait_wrapper_methods() {
    let data = create_complex_compound();
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();

    // as_compound
    let comp = ReadableValue::as_compound(&root).unwrap();
    assert_eq!(ScopedReadableValue::as_int(&ReadableCompound::get(comp, "i").unwrap()), Some(1));
    let s_val = ReadableCompound::get(comp, "s").unwrap();
    let s_str = ReadableValue::as_string(&s_val).unwrap();
    assert_eq!(ReadableString::decode(s_str).to_string(), "s");

    let binding_lst = ReadableCompound::get(comp, "lst").unwrap();
    let lst = ReadableValue::as_list(&binding_lst).unwrap();
    assert_eq!(ScopedReadableList::len(lst), 2);
    
    let mut iter = ReadableList::iter(lst);
    assert_eq!(ReadableString::decode(ReadableValue::as_string(&iter.next().unwrap()).unwrap()).to_string(), "x");
    assert_eq!(ReadableString::decode(ReadableValue::as_string(&iter.next().unwrap()).unwrap()).to_string(), "y");

    // sub compound
    let binding_sub = ReadableCompound::get(comp, "sub").unwrap();
    let sub = ReadableValue::as_compound(&binding_sub).unwrap();
    assert_eq!(ScopedReadableValue::as_int(&ReadableCompound::get(sub, "j").unwrap()), Some(2));
}
