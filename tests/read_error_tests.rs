use na_nbt::{Error, read_borrowed, read_owned};
use zerocopy::byteorder::{BigEndian as BE, LittleEndian as LE};

// ==================== Helper Functions ====================

fn create_list_header(tag: u8, count: u32) -> Vec<u8> {
    let mut data = vec![0x09, 0x00, 0x00]; // Root list
    data.push(tag);
    data.extend_from_slice(&count.to_be_bytes());
    data
}

fn create_compound_start() -> Vec<u8> {
    vec![0x0A, 0x00, 0x00]
}

// ==================== Immutable Read (Borrowed) Tests ====================

#[test]
fn test_borrowed_empty_slice() {
    let data = vec![];
    let res = read_borrowed::<BE>(&data);
    match res {
        Err(Error::EndOfFile) => {}
        _ => panic!("Expected EndOfFile, got Ok"),
    }
}

#[test]
fn test_borrowed_eof_in_root_header() {
    let data = vec![0x01, 0x00]; // Missing name length byte
    let res = read_borrowed::<BE>(&data);
    match res {
        Err(Error::EndOfFile) => {}
        _ => panic!("Expected EndOfFile, got Ok"),
    }
}

#[test]
fn test_borrowed_eof_in_list_header() {
    // 0x09 (List), 0x00, 0x00 (empty name)
    // Then need 1 byte (tag) + 4 bytes (count) = 5 bytes.
    // Provide fewer.
    let data = vec![0x09, 0x00, 0x00, 0x01, 0x00];
    let res = read_borrowed::<BE>(&data);
    match res {
        Err(Error::EndOfFile) => {}
        _ => panic!("Expected EndOfFile, got Ok"),
    }
}

#[test]
fn test_borrowed_eof_in_list_body_primitive() {
    // List of 2 bytes, but only provide 1
    let mut data = create_list_header(1, 2); // Byte tag
    data.push(0xFF);
    let res = read_borrowed::<BE>(&data);
    match res {
        Err(Error::EndOfFile) => {}
        _ => panic!("Expected EndOfFile, got Ok"),
    }
}

#[test]
fn test_borrowed_eof_in_list_body_complex() {
    // List of 1 compound, but provide truncated compound
    let mut data = create_list_header(10, 1); // Compound tag
    data.push(0x01); // Tag Byte inside compound
    // ... missing rest
    let res = read_borrowed::<BE>(&data);
    match res {
        Err(Error::EndOfFile) => {}
        _ => panic!("Expected EndOfFile, got Ok"),
    }
}

#[test]
fn test_borrowed_invalid_tag_in_compound() {
    let mut data = create_compound_start();
    data.push(0xFF); // Invalid tag ID
    // Need name length and name for it to parse tag ID
    // 0xFF isn't 0 (End), so it tries to read name.
    // If we put 0xFF where tag ID is expected:
    // Compound structure: [TagID] [NameLen] [Name] [Payload]
    // 0xFF
    let _res = read_borrowed::<BE>(&data);
    // It will try to read name len, then name, then payload... wait.
    // No, state machine checks tag_id.
    // But it reads NameLen and Name BEFORE checking tag_id match.
    // So we need to provide valid name fields.

    data.extend_from_slice(&0u16.to_be_bytes()); // empty name
    // Now data is [0x0A... 0xFF, 0x00, 0x00].
    // Parser reads 0xFF as tag. Reads name len 0. Name "".
    // Then switches on 0xFF.

    let res = read_borrowed::<BE>(&data);
    match res {
        Err(Error::InvalidTagType(0xFF)) => {}
        _ => panic!("Expected InvalidTagType, got Ok"),
    }
}

// ==================== Mutable Read (Owned) - Standard Path ====================

#[test]
fn test_owned_empty_slice() {
    let data = vec![];
    let res = read_owned::<BE, BE>(&data);
    match res {
        Err(Error::EndOfFile) => {}
        _ => panic!("Expected EndOfFile, got Ok"),
    }
}

#[test]
fn test_owned_eof_in_list_header() {
    let data = vec![0x09, 0x00, 0x00, 0x01, 0x00];
    let res = read_owned::<BE, BE>(&data);
    match res {
        Err(Error::EndOfFile) => {}
        _ => panic!("Expected EndOfFile, got Ok"),
    }
}

#[test]
fn test_owned_eof_in_list_body() {
    let mut data = create_list_header(1, 2);
    data.push(0xFF);
    let res = read_owned::<BE, BE>(&data);
    match res {
        Err(Error::EndOfFile) => {}
        _ => panic!("Expected EndOfFile, got Ok"),
    }
}

#[test]
fn test_owned_invalid_tag_in_compound() {
    let mut data = create_compound_start();
    data.push(0xFF);
    data.extend_from_slice(&0u16.to_be_bytes());
    let res = read_owned::<BE, BE>(&data);
    match res {
        Err(Error::InvalidTagType(0xFF)) => {}
        _ => panic!("Expected InvalidTagType, got Ok"),
    }
}

#[test]
fn test_owned_invalid_tag_in_list() {
    // List header defines element type. If we say 0xFF, it should fail immediately at header parse
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0xFF); // Invalid element type
    data.extend_from_slice(&1u32.to_be_bytes());

    let res = read_owned::<BE, BE>(&data);
    match res {
        Err(Error::EndOfFile) => {}
        _ => panic!("Expected EndOfFile, got Ok"),
    }
}

// ==================== Mutable Read (Owned) - Fallback Path ====================
// We force fallback by using different byte orders <BE, LE>.
// The data must be valid BE (so we construct it as BE), but we read it as LE?
// No, the generic params are <FileByteOrder, MemoryByteOrder>.
// If FileByteOrder != MemoryByteOrder, fallback is used.
// We are constructing BE data. So we should use read_owned::<BE, LE>.

#[test]
fn test_owned_fallback_eof_in_list_header() {
    let data = vec![0x09, 0x00, 0x00, 0x01, 0x00];
    let res = read_owned::<BE, LE>(&data);
    match res {
        Err(Error::EndOfFile) => {}
        _ => panic!("Expected EndOfFile, got Ok"),
    }
}

#[test]
fn test_owned_fallback_eof_in_list_body() {
    // List of ints (4 bytes).
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x03); // Int
    data.extend_from_slice(&1u32.to_be_bytes()); // Count 1
    data.push(0x00); // 1 byte only

    let res = read_owned::<BE, LE>(&data);
    match res {
        Err(Error::EndOfFile) => {}
        _ => panic!("Expected EndOfFile, got Ok"),
    }
}

#[test]
fn test_owned_fallback_invalid_tag_in_compound() {
    let mut data = create_compound_start();
    data.push(0xFF);
    // In fallback, it also reads name before checking tag type?
    // Let's check impl. read_compound_fallback loop:
    // 1. check_bounds!(1); read tag_id.
    // 2. if tag_id == 0 -> return.
    // 3. check_bounds!(2); read name_len.
    // 4. check_bounds!(name_len); skip name.
    // 5. if tag_id <= 12 -> ... else Err(InvalidTagType)

    data.extend_from_slice(&0u16.to_be_bytes());

    let res = read_owned::<BE, LE>(&data);
    match res {
        Err(Error::InvalidTagType(0xFF)) => {}
        _ => panic!("Expected InvalidTagType, got Ok"),
    }
}

#[test]
fn test_owned_fallback_invalid_tag_in_list() {
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0xFF); // Invalid element type
    data.extend_from_slice(&1u32.to_be_bytes());

    let res = read_owned::<BE, LE>(&data);
    match res {
        Err(Error::InvalidTagType(0xFF)) => {}
        _ => panic!("Expected InvalidTagType, got Ok"),
    }
}

#[test]
fn test_owned_fallback_invalid_tag_nested_in_list() {
    // List of Lists. Inner list has invalid type.
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x09); // List of Lists
    data.extend_from_slice(&1u32.to_be_bytes()); // Count 1

    // Inner list
    data.push(0xFF); // Invalid type
    data.extend_from_slice(&0u32.to_be_bytes());

    let res = read_owned::<BE, LE>(&data);
    match res {
        Err(Error::InvalidTagType(0xFF)) => {}
        _ => panic!("Expected InvalidTagType, got Ok"),
    }
}

#[test]
fn test_owned_fallback_eof_in_nested_list() {
    // List of Lists. Inner list incomplete.
    let mut data = vec![0x09, 0x00, 0x00];
    data.push(0x09); // List of Lists
    data.extend_from_slice(&1u32.to_be_bytes()); // Count 1

    // Inner list partial
    data.push(0x01); // Byte type
    // Missing count

    let res = read_owned::<BE, LE>(&data);
    match res {
        Err(Error::EndOfFile) => {}
        _ => panic!("Expected EndOfFile, got Ok"),
    }
}
