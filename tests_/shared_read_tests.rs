use na_nbt::{read_shared, SharedValue};
use bytes::Bytes;
use zerocopy::byteorder::BigEndian;
use std::thread;

fn create_test_data() -> Vec<u8> {
    let mut data = vec![0x0A, 0x00, 0x00]; // Compound
    // Int "val" = 42
    data.push(0x03);
    data.extend_from_slice(&3u16.to_be_bytes());
    data.extend_from_slice(b"val");
    data.extend_from_slice(&42i32.to_be_bytes());
    data.push(0x00);
    data
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_read_shared_basic() {
    let data = create_test_data();
    let bytes = Bytes::from(data);
    let root = read_shared::<BigEndian>(bytes).unwrap();
    
    if let SharedValue::Compound(c) = root {
        let val = c.get("val").unwrap();
        assert_eq!(val.as_int(), Some(42));
    } else {
        panic!("Expected compound");
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_shared_value_send_sync_static() {
    let data = create_test_data();
    let bytes = Bytes::from(data);
    
    // We can move the root to another thread because it is 'static and Send/Sync
    let root = read_shared::<BigEndian>(bytes).unwrap();
    
    let handle = thread::spawn(move || {
        if let SharedValue::Compound(c) = root {
            let val = c.get("val").unwrap();
            assert_eq!(val.as_int(), Some(42));
            val.as_int().unwrap()
        } else {
            0
        }
    });
    
    let res = handle.join().unwrap();
    assert_eq!(res, 42);
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_shared_value_clone() {
    let data = create_test_data();
    let bytes = Bytes::from(data);
    let root = read_shared::<BigEndian>(bytes).unwrap();
    
    let root2 = root.clone();
    
    if let SharedValue::Compound(c) = root {
        assert_eq!(c.get("val").unwrap().as_int(), Some(42));
    }
    
    // root2 should still be valid
    if let SharedValue::Compound(c) = root2 {
        assert_eq!(c.get("val").unwrap().as_int(), Some(42));
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_shared_value_access() {
    // Create a more complex structure
    let mut data = vec![0x0A, 0x00, 0x00];
    // List "li" [1, 2]
    data.push(0x09);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.extend_from_slice(b"li");
    data.push(0x03); // Int
    data.extend_from_slice(&2u32.to_be_bytes()); // Count
    data.extend_from_slice(&1i32.to_be_bytes());
    data.extend_from_slice(&2i32.to_be_bytes());
    data.push(0x00);
    
    let bytes = Bytes::from(data);
    let root = read_shared::<BigEndian>(bytes).unwrap();
    
    let comp = root.as_compound().unwrap();
    let list_val = comp.get("li").unwrap();
    let list = list_val.as_list().unwrap();
    
    let mut sum = 0;
    for v in list.iter() {
        sum += v.as_int().unwrap();
    }
    assert_eq!(sum, 3);
}

