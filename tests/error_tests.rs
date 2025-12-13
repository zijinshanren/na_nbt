//! Tests for the Error type

use na_nbt::Error;
use std::io;

#[test]
fn test_error_display_end_of_file() {
    let error = Error::EndOfFile;
    assert_eq!(format!("{}", error), "unexpected end of input");
}

#[test]
fn test_error_display_trailing_data() {
    let error = Error::TrailingData(42);
    assert_eq!(
        format!("{}", error),
        "trailing data after end of input: 42 bytes remaining"
    );
}

#[test]
fn test_error_display_invalid_tag_type() {
    let error = Error::InvalidTagType(0xFF);
    assert_eq!(format!("{}", error), "invalid NBT tag type: 0xff");
}

#[test]
fn test_error_display_io() {
    let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let error = Error::IO(io_error);
    assert!(format!("{}", error).contains("file not found"));
}

#[test]
fn test_error_debug() {
    let error = Error::EndOfFile;
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("EndOfFile"));
}

#[test]
fn test_error_is_std_error() {
    fn assert_error<E: std::error::Error>() {}
    assert_error::<Error>();
}

#[test]
fn test_result_type() {
    let ok_result: na_nbt::Result<i32> = Ok(42);
    assert_eq!(ok_result.unwrap(), 42);

    let err_result: na_nbt::Result<i32> = Err(Error::EndOfFile);
    assert!(err_result.is_err());
}
