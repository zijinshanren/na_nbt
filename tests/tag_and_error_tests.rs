use na_nbt::{Error, Tag};
use std::io;

#[test]
fn test_tag_properties() {
    // Primitives
    assert!(Tag::End.is_primitive());
    assert!(Tag::Byte.is_primitive());
    assert!(Tag::Short.is_primitive());
    assert!(Tag::Int.is_primitive());
    assert!(Tag::Long.is_primitive());
    assert!(Tag::Float.is_primitive());
    assert!(Tag::Double.is_primitive());

    assert!(!Tag::ByteArray.is_primitive());
    assert!(!Tag::String.is_primitive());
    assert!(!Tag::List.is_primitive());
    assert!(!Tag::Compound.is_primitive());
    assert!(!Tag::IntArray.is_primitive());
    assert!(!Tag::LongArray.is_primitive());

    // Arrays
    assert!(Tag::ByteArray.is_array());
    assert!(Tag::IntArray.is_array());
    assert!(Tag::LongArray.is_array());
    assert!(!Tag::List.is_array()); // List is composite, not "Array" tag type
    assert!(!Tag::Byte.is_array());

    // Composite
    assert!(Tag::List.is_composite());
    assert!(Tag::Compound.is_composite());
    assert!(!Tag::ByteArray.is_composite());
    assert!(!Tag::String.is_composite());
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
