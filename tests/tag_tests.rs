//! Tests for the Tag enum

use na_nbt::Tag;

#[test]
fn test_tag_values() {
    assert_eq!(Tag::End as u8, 0);
    assert_eq!(Tag::Byte as u8, 1);
    assert_eq!(Tag::Short as u8, 2);
    assert_eq!(Tag::Int as u8, 3);
    assert_eq!(Tag::Long as u8, 4);
    assert_eq!(Tag::Float as u8, 5);
    assert_eq!(Tag::Double as u8, 6);
    assert_eq!(Tag::ByteArray as u8, 7);
    assert_eq!(Tag::String as u8, 8);
    assert_eq!(Tag::List as u8, 9);
    assert_eq!(Tag::Compound as u8, 10);
    assert_eq!(Tag::IntArray as u8, 11);
    assert_eq!(Tag::LongArray as u8, 12);
}

#[test]
fn test_tag_is_primitive() {
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
}

#[test]
fn test_tag_is_array() {
    assert!(Tag::ByteArray.is_array());
    assert!(Tag::IntArray.is_array());
    assert!(Tag::LongArray.is_array());

    assert!(!Tag::End.is_array());
    assert!(!Tag::Byte.is_array());
    assert!(!Tag::Short.is_array());
    assert!(!Tag::Int.is_array());
    assert!(!Tag::Long.is_array());
    assert!(!Tag::Float.is_array());
    assert!(!Tag::Double.is_array());
    assert!(!Tag::String.is_array());
    assert!(!Tag::List.is_array());
    assert!(!Tag::Compound.is_array());
}

#[test]
fn test_tag_is_composite() {
    assert!(Tag::List.is_composite());
    assert!(Tag::Compound.is_composite());

    assert!(!Tag::End.is_composite());
    assert!(!Tag::Byte.is_composite());
    assert!(!Tag::Short.is_composite());
    assert!(!Tag::Int.is_composite());
    assert!(!Tag::Long.is_composite());
    assert!(!Tag::Float.is_composite());
    assert!(!Tag::Double.is_composite());
    assert!(!Tag::ByteArray.is_composite());
    assert!(!Tag::String.is_composite());
    assert!(!Tag::IntArray.is_composite());
    assert!(!Tag::LongArray.is_composite());
}

#[test]
fn test_tag_equality() {
    assert_eq!(Tag::End, Tag::End);
    assert_eq!(Tag::Byte, Tag::Byte);
    assert_ne!(Tag::End, Tag::Byte);
    assert_ne!(Tag::List, Tag::Compound);
}

#[test]
fn test_tag_ordering() {
    assert!(Tag::End < Tag::Byte);
    assert!(Tag::Byte < Tag::Short);
    assert!(Tag::Short < Tag::Int);
    assert!(Tag::Int < Tag::Long);
    assert!(Tag::Long < Tag::Float);
    assert!(Tag::Float < Tag::Double);
    assert!(Tag::Double < Tag::ByteArray);
    assert!(Tag::ByteArray < Tag::String);
    assert!(Tag::String < Tag::List);
    assert!(Tag::List < Tag::Compound);
    assert!(Tag::Compound < Tag::IntArray);
    assert!(Tag::IntArray < Tag::LongArray);
}

#[test]
fn test_tag_debug() {
    assert_eq!(format!("{:?}", Tag::End), "End");
    assert_eq!(format!("{:?}", Tag::Byte), "Byte");
    assert_eq!(format!("{:?}", Tag::Compound), "Compound");
}

#[test]
fn test_tag_clone() {
    let tag = Tag::Compound;
    let cloned = tag.clone();
    assert_eq!(tag, cloned);
}

#[test]
fn test_tag_copy() {
    let tag = Tag::List;
    let copied: Tag = tag; // Copy
    assert_eq!(tag, copied);
}

#[test]
fn test_tag_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(Tag::Byte);
    set.insert(Tag::Int);
    set.insert(Tag::Byte); // duplicate

    assert_eq!(set.len(), 2);
    assert!(set.contains(&Tag::Byte));
    assert!(set.contains(&Tag::Int));
    assert!(!set.contains(&Tag::Long));
}
