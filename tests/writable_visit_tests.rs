use na_nbt::{read_owned, WritableValue, ValueMut};
use zerocopy::byteorder::BigEndian;

#[test]
fn test_writable_visit_mut_mutable() {
    let mut data = vec![0x0A, 0x00, 0x00];
    // Int "i" = 10
    data.push(0x03);
    data.extend_from_slice(&1u16.to_be_bytes());
    data.push(b'i');
    data.extend_from_slice(&10i32.to_be_bytes());
    data.push(0x00);
    
    let mut owned = read_owned::<BigEndian, BigEndian>(&data).unwrap();
    let mut comp = owned.as_compound_mut().unwrap();
    let mut mv = comp.get_mut("i").unwrap();
    
    WritableValue::visit_mut(&mut mv, |v| {
        match v {
            ValueMut::Int(val) => *val += 5,
            _ => panic!("Expected Int"),
        }
    });
    
    assert_eq!(owned.get("i").unwrap().as_int(), Some(15));
}

