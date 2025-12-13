use na_nbt::{read_borrowed, read_owned, Error};
use zerocopy::byteorder::BigEndian as BE;

#[test]
fn test_borrowed_trailing_data_primitive() {
    let mut data = vec![0x01, 0x00, 0x00, 10]; // Byte 10
    data.push(0xFF); // Trailing byte
    
    let res = read_borrowed::<BE>(&data);
    match res {
        Err(Error::TrailingData(n)) => assert_eq!(n, 1),
        _ => panic!("Expected TrailingData, got Ok"),
    }
}

#[test]
fn test_borrowed_trailing_data_compound() {
    let mut data = vec![0x0A, 0x00, 0x00]; // Root compound
    data.push(0x00); // End tag
    data.push(0xFF); // Trailing
    
    let res = read_borrowed::<BE>(&data);
    match res {
        Err(Error::TrailingData(n)) => assert_eq!(n, 1),
        _ => panic!("Expected TrailingData, got Ok"),
    }
}

#[test]
fn test_borrowed_trailing_data_list() {
    let mut data = vec![0x09, 0x00, 0x00]; // Root list
    data.push(0x01); // Byte
    data.extend_from_slice(&1u32.to_be_bytes()); // Count 1
    data.push(10); // Value
    data.push(0xFF); // Trailing
    
    let res = read_borrowed::<BE>(&data);
    match res {
        Err(Error::TrailingData(n)) => assert_eq!(n, 1),
        _ => panic!("Expected TrailingData, got Ok"),
    }
}

#[test]
fn test_owned_trailing_data() {
    let mut data = vec![0x01, 0x00, 0x00, 10];
    data.push(0xFF);
    
    let res = read_owned::<BE, BE>(&data);
    match res {
        Err(Error::TrailingData(n)) => assert_eq!(n, 1),
        _ => panic!("Expected TrailingData, got Ok"),
    }
}

#[test]
fn test_deeply_nested_structure() {
    // 100 nested lists
    let depth = 100;
    let mut data = vec![0x09, 0x00, 0x00]; // Root list
    
    // Header for each level
    for _ in 0..depth {
        data.push(0x09); // Type List
        data.extend_from_slice(&1u32.to_be_bytes()); // Count 1
    }
    
    // Inner most element: List of Bytes
    data.push(0x01); // Type Byte
    data.extend_from_slice(&1u32.to_be_bytes()); // Count 1
    data.push(42); // Value
    
    let doc = read_borrowed::<BE>(&data).unwrap();
    let mut val = doc.root();
    
    for _ in 0..depth {
        let list = val.as_list().unwrap();
        assert_eq!(list.len(), 1);
        val = list.get(0).unwrap();
    }
    
    let inner_list = val.as_list().unwrap();
    assert_eq!(inner_list.len(), 1);
    assert_eq!(inner_list.get(0).unwrap().as_byte(), Some(42));
}

