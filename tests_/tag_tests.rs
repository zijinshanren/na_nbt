//! Tests for the Tag enum

use na_nbt::TagID;

#[test]
fn test_tag_values() {
    assert_eq!(TagID::End as u8, 0);
    assert_eq!(TagID::Byte as u8, 1);
    assert_eq!(TagID::Short as u8, 2);
    assert_eq!(TagID::Int as u8, 3);
    assert_eq!(TagID::Long as u8, 4);
    assert_eq!(TagID::Float as u8, 5);
    assert_eq!(TagID::Double as u8, 6);
    assert_eq!(TagID::ByteArray as u8, 7);
    assert_eq!(TagID::String as u8, 8);
    assert_eq!(TagID::List as u8, 9);
    assert_eq!(TagID::Compound as u8, 10);
    assert_eq!(TagID::IntArray as u8, 11);
    assert_eq!(TagID::LongArray as u8, 12);
}

#[test]
fn test_tag_is_primitive() {
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
}

#[test]
fn test_tag_is_array() {
    assert!(TagID::ByteArray.is_array());
    assert!(TagID::IntArray.is_array());
    assert!(TagID::LongArray.is_array());

    assert!(!TagID::End.is_array());
    assert!(!TagID::Byte.is_array());
    assert!(!TagID::Short.is_array());
    assert!(!TagID::Int.is_array());
    assert!(!TagID::Long.is_array());
    assert!(!TagID::Float.is_array());
    assert!(!TagID::Double.is_array());
    assert!(!TagID::String.is_array());
    assert!(!TagID::List.is_array());
    assert!(!TagID::Compound.is_array());
}

#[test]
fn test_tag_is_composite() {
    assert!(TagID::List.is_composite());
    assert!(TagID::Compound.is_composite());

    assert!(!TagID::End.is_composite());
    assert!(!TagID::Byte.is_composite());
    assert!(!TagID::Short.is_composite());
    assert!(!TagID::Int.is_composite());
    assert!(!TagID::Long.is_composite());
    assert!(!TagID::Float.is_composite());
    assert!(!TagID::Double.is_composite());
    assert!(!TagID::ByteArray.is_composite());
    assert!(!TagID::String.is_composite());
    assert!(!TagID::IntArray.is_composite());
    assert!(!TagID::LongArray.is_composite());
}

#[test]
fn test_tag_equality() {
    assert_eq!(TagID::End, TagID::End);
    assert_eq!(TagID::Byte, TagID::Byte);
    assert_ne!(TagID::End, TagID::Byte);
    assert_ne!(TagID::List, TagID::Compound);
}

#[test]
fn test_tag_ordering() {
    assert!(TagID::End < TagID::Byte);
    assert!(TagID::Byte < TagID::Short);
    assert!(TagID::Short < TagID::Int);
    assert!(TagID::Int < TagID::Long);
    assert!(TagID::Long < TagID::Float);
    assert!(TagID::Float < TagID::Double);
    assert!(TagID::Double < TagID::ByteArray);
    assert!(TagID::ByteArray < TagID::String);
    assert!(TagID::String < TagID::List);
    assert!(TagID::List < TagID::Compound);
    assert!(TagID::Compound < TagID::IntArray);
    assert!(TagID::IntArray < TagID::LongArray);
}

#[test]
fn test_tag_debug() {
    assert_eq!(format!("{:?}", TagID::End), "End");
    assert_eq!(format!("{:?}", TagID::Byte), "Byte");
    assert_eq!(format!("{:?}", TagID::Compound), "Compound");
}

#[test]
fn test_tag_copy() {
    let tag = TagID::List;
    let copied: TagID = tag; // Copy
    assert_eq!(tag, copied);
}

#[test]
fn test_tag_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(TagID::Byte);
    set.insert(TagID::Int);
    set.insert(TagID::Byte); // duplicate

    assert_eq!(set.len(), 2);
    assert!(set.contains(&TagID::Byte));
    assert!(set.contains(&TagID::Int));
    assert!(!set.contains(&TagID::Long));
}
