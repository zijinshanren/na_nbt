use na_nbt::{Error, TagID};
use std::io;

#[test]
fn test_tag_properties() {
    // Primitives
    assert!(TagID::End.is_primitive());
    assert!(TagID::Byte.is_primitive());
    assert!(TagID::Short.is_primitive());
    assert!(TagID::Int.is_primitive());
    assert!(TagID::Long.is_primitive());
    assert!(TagID::Float.is_primitive());
    assert!(TagID::Double.is_primitive());

    assert!(!TagID::ByteArray.is_primitive());
    assert!(!TagID::String.is_primitive());
    assert!(!TagID::List.is_primitive());
    assert!(!TagID::Compound.is_primitive());
    assert!(!TagID::IntArray.is_primitive());
    assert!(!TagID::LongArray.is_primitive());

    // Arrays
    assert!(TagID::ByteArray.is_array());
    assert!(TagID::IntArray.is_array());
    assert!(TagID::LongArray.is_array());
    assert!(!TagID::List.is_array()); // List is composite, not "Array" tag type
    assert!(!TagID::Byte.is_array());

    // Composite
    assert!(TagID::List.is_composite());
    assert!(TagID::Compound.is_composite());
    assert!(!TagID::ByteArray.is_composite());
    assert!(!TagID::String.is_composite());
}

#[test]
fn test_error_display() {
    let e = Error::EndOfFile;
    assert_eq!(e.to_string(), "unexpected end of input");

    let e = Error::InvalidTagType(0xFF);
    assert_eq!(e.to_string(), "invalid NBT tag type: 0xff");

    let e = Error::TrailingData(5);
    assert_eq!(
        e.to_string(),
        "trailing data after end of input: 5 bytes remaining"
    );

    let io_err = io::Error::other("io error");
    let e = Error::IO(io_err);
    assert_eq!(e.to_string(), "io error");
}
