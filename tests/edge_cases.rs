//! Edge case tests for errors and boundaries

use na_nbt::{read_borrowed, read_owned, write_value_to_writer, Error};
use zerocopy::byteorder::BigEndian as BE;
use std::io::{self, Write};

fn create_byte_nbt(value: i8) -> Vec<u8> {
    vec![0x01, 0x00, 0x00, value as u8]
}

#[test]
fn test_trailing_data_borrowed() {
    let mut data = create_byte_nbt(42);
    data.push(0xFF); // trailing extra byte
    let res = read_borrowed::<BE>(&data);
    match res {
        Err(Error::TrailingData(1)) => {}
        other => panic!("unexpected result"),
    }
}

#[test]
fn test_trailing_data_owned() {
    let mut data = create_byte_nbt(42);
    data.extend_from_slice(&[0xAA, 0xBB]);
    let res = read_owned::<BE, BE>(&data);
    match res {
        Err(Error::TrailingData(2)) => {}
        other => panic!("unexpected result"),
    }
}

#[test]
fn test_invalid_tag_type_borrowed() {
    // Tag 0xFF as root tag with empty name should be invalid
    let data = vec![0xFF, 0x00, 0x00];
    let res = read_borrowed::<BE>(&data);
    match res {
        Err(Error::InvalidTagType(0xFF)) => {}
        other => panic!("unexpected result"),
    }
}

#[test]
fn test_invalid_tag_type_owned() {
    let data = vec![0xFF, 0x00, 0x00];
    let res = read_owned::<BE, BE>(&data);
    match res {
        Err(Error::InvalidTagType(0xFF)) => {}
        other => panic!("unexpected result"),
    }
}

#[test]
fn test_invalid_element_type_in_list() {
    // Build an empty list with invalid element type (0xFF) and length 0
    let data = vec![0x09, 0x00, 0x00, 0xFF, 0x00, 0x00, 0x00, 0x01];
    let res = read_borrowed::<BE>(&data);
    match res {
        Err(e) => match e {
            Error::InvalidTagType(15) => {}
            _ => panic!("unexpected result: {:?}", e),
        },
        Ok(_) => panic!("expected error"),
    }
}

#[test]
fn test_eof_in_name_borrowed() {
    // Tag: Byte, declared name length = 4 but only provide 1 byte of name
    let data = vec![0x01, 0x00, 0x04, b'a'];
    let res = read_borrowed::<BE>(&data);
    match res {
        Err(Error::EndOfFile) => {}
        other => panic!("unexpected result"),
    }
}

struct BadWriter;
impl Write for BadWriter {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        Err(io::Error::new(io::ErrorKind::Other, "write fail"))
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn test_write_writer_io_error() {
    let data = create_byte_nbt(11);
    let doc = read_borrowed::<BE>(&data).unwrap();
    let root = doc.root();
    let mut w = BadWriter;
    let res = write_value_to_writer::<_, BE, BE, _>(&mut w, &root);
    assert!(matches!(res.unwrap_err(), Error::IO(_)));
}

#[test]
fn test_owned_list_get_out_of_range() {
    let data = vec![0x09, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x01, 0x02];
    // List of bytes [1,2]
    let owned = read_owned::<BE, BE>(&data).unwrap();
    if let na_nbt::OwnedValue::List(list) = owned {
        assert!(list.get(10).is_none()); // out of bounds
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_mutable_get_mut_out_of_range() {
    let data = vec![0x09, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x01, 0x02];
    let mut owned = read_owned::<BE, BE>(&data).unwrap();
    if let na_nbt::OwnedValue::List(mut list) = owned {
        assert!(list.get_mut(10).is_none());
    } else {
        panic!("expected list");
    }
}
