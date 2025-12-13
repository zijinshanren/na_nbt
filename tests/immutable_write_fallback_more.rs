use na_nbt::{read_borrowed, write_value_to_vec, write_value_to_writer, Tag};
use zerocopy::byteorder::{BigEndian, LittleEndian};
use std::io::Cursor;

fn create_list_nbt<F>(element_tag: u8, count: u32, write_elements: F) -> Vec<u8> 
where F: FnOnce(&mut Vec<u8>)
{
    let mut result = vec![
        0x09, // Tag::List
        0x00, 0x00, // empty name
        element_tag, // element type
    ];
    result.extend_from_slice(&count.to_be_bytes());
    write_elements(&mut result);
    result
}

fn create_compound_nbt<F>(write_elements: F) -> Vec<u8>
where F: FnOnce(&mut Vec<u8>)
{
    let mut result = vec![0x0A, 0x00, 0x00]; // Compound, empty root name
    write_elements(&mut result);
    result.push(0x00); // End compound
    result
}

#[test]
fn test_write_list_fallback_primitives() {
    // List of Short
    let data = create_list_nbt(Tag::Short as u8, 2, |buf| {
        buf.extend_from_slice(&0x1234u16.to_be_bytes());
        buf.extend_from_slice(&0x5678u16.to_be_bytes());
    });
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();
    let written = write_value_to_vec::<_, BigEndian, LittleEndian>(&root).unwrap();
    
    // Check header
    assert_eq!(&written[0..8], &[0x09, 0x00, 0x00, 0x02, 0x02, 0x00, 0x00, 0x00]); // LE count
    // Check data LE
    assert_eq!(&written[8..10], &[0x34, 0x12]);
    assert_eq!(&written[10..12], &[0x78, 0x56]);

    // List of Long
    let data = create_list_nbt(Tag::Long as u8, 1, |buf| {
        buf.extend_from_slice(&0x1122334455667788u64.to_be_bytes());
    });
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();
    let written = write_value_to_vec::<_, BigEndian, LittleEndian>(&root).unwrap();
    assert_eq!(&written[8..16], &[0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11]);

    // List of Float
    let val_f = 1.234f32;
    let data = create_list_nbt(Tag::Float as u8, 1, |buf| {
        buf.extend_from_slice(&val_f.to_be_bytes());
    });
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();
    let written = write_value_to_vec::<_, BigEndian, LittleEndian>(&root).unwrap();
    // LE float bytes
    let expected_bytes = val_f.to_le_bytes();
    assert_eq!(&written[8..12], &expected_bytes);

    // List of Double
    let val_d = 123.456f64;
    let data = create_list_nbt(Tag::Double as u8, 1, |buf| {
        buf.extend_from_slice(&val_d.to_be_bytes());
    });
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();
    let written = write_value_to_vec::<_, BigEndian, LittleEndian>(&root).unwrap();
    let expected_bytes = val_d.to_le_bytes();
    assert_eq!(&written[8..16], &expected_bytes);
}

#[test]
fn test_write_list_fallback_complex() {
    // List of Strings
    let s1 = "Hello";
    let s2 = "World";
    let data = create_list_nbt(Tag::String as u8, 2, |buf| {
        buf.extend_from_slice(&(s1.len() as u16).to_be_bytes());
        buf.extend_from_slice(s1.as_bytes());
        buf.extend_from_slice(&(s2.len() as u16).to_be_bytes());
        buf.extend_from_slice(s2.as_bytes());
    });
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();
    let written = write_value_to_vec::<_, BigEndian, LittleEndian>(&root).unwrap();
    // Header
    let mut offset = 8;
    // s1 len (LE)
    assert_eq!(&written[offset..offset+2], &(s1.len() as u16).to_le_bytes());
    offset += 2;
    assert_eq!(&written[offset..offset+s1.len()], s1.as_bytes());
    offset += s1.len();
    // s2 len (LE)
    assert_eq!(&written[offset..offset+2], &(s2.len() as u16).to_le_bytes());
    offset += 2;
    assert_eq!(&written[offset..offset+s2.len()], s2.as_bytes());

    // List of Lists (List<List<Int>>)
    let data = create_list_nbt(Tag::List as u8, 1, |buf| {
        // Inner list header: Tag::Int, Count 1
        buf.push(Tag::Int as u8);
        buf.extend_from_slice(&1u32.to_be_bytes());
        // Inner list data
        buf.extend_from_slice(&0x12345678u32.to_be_bytes());
    });
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();
    let written = write_value_to_vec::<_, BigEndian, LittleEndian>(&root).unwrap();
    
    // Outer list header (8 bytes)
    let offset = 8;
    // Inner list: Tag Int (1 byte), Count (4 bytes LE)
    assert_eq!(written[offset], Tag::Int as u8);
    assert_eq!(&written[offset+1..offset+5], &1u32.to_le_bytes());
    // Inner list data: Int (4 bytes LE)
    assert_eq!(&written[offset+5..offset+9], &0x12345678u32.to_le_bytes());

    // List of Compounds
    let data = create_list_nbt(Tag::Compound as u8, 1, |buf| {
        // Compound: Int 'a' = 1
        buf.push(Tag::Int as u8);
        buf.extend_from_slice(&1u16.to_be_bytes()); // name len
        buf.push(b'a'); // name
        buf.extend_from_slice(&1u32.to_be_bytes()); // val
        buf.push(0x00); // End
    });
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();
    let written = write_value_to_vec::<_, BigEndian, LittleEndian>(&root).unwrap();
    
    let offset = 8;
    // Compound data
    assert_eq!(written[offset], Tag::Int as u8);
    assert_eq!(&written[offset+1..offset+3], &1u16.to_le_bytes()); // name len LE
    assert_eq!(written[offset+3], b'a');
    assert_eq!(&written[offset+4..offset+8], &1u32.to_le_bytes()); // val LE
    assert_eq!(written[offset+8], 0x00);
}

#[test]
fn test_write_list_fallback_arrays() {
    // List of ByteArray
    let b1 = vec![1, 2, 3];
    let data = create_list_nbt(Tag::ByteArray as u8, 1, |buf| {
        buf.extend_from_slice(&(b1.len() as u32).to_be_bytes());
        buf.extend_from_slice(&b1);
    });
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();
    let written = write_value_to_vec::<_, BigEndian, LittleEndian>(&root).unwrap();
    
    let offset = 8;
    assert_eq!(&written[offset..offset+4], &(b1.len() as u32).to_le_bytes());
    assert_eq!(&written[offset+4..offset+7], &b1[..]);

    // List of IntArray
    let i1 = vec![0x11223344u32];
    let data = create_list_nbt(Tag::IntArray as u8, 1, |buf| {
        buf.extend_from_slice(&(i1.len() as u32).to_be_bytes());
        for i in &i1 { buf.extend_from_slice(&i.to_be_bytes()); }
    });
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();
    let written = write_value_to_vec::<_, BigEndian, LittleEndian>(&root).unwrap();
    
    let offset = 8;
    assert_eq!(&written[offset..offset+4], &(i1.len() as u32).to_le_bytes());
    assert_eq!(&written[offset+4..offset+8], &i1[0].to_le_bytes());
    
    // List of LongArray
    let l1 = vec![0x1122334455667788u64];
    let data = create_list_nbt(Tag::LongArray as u8, 1, |buf| {
        buf.extend_from_slice(&(l1.len() as u32).to_be_bytes());
        for l in &l1 { buf.extend_from_slice(&l.to_be_bytes()); }
    });
    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();
    let written = write_value_to_vec::<_, BigEndian, LittleEndian>(&root).unwrap();
    
    let offset = 8;
    assert_eq!(&written[offset..offset+4], &(l1.len() as u32).to_le_bytes());
    assert_eq!(&written[offset+4..offset+12], &l1[0].to_le_bytes());
}

#[test]
fn test_write_compound_fallback_all_types() {
    let data = create_compound_nbt(|buf| {
        // Byte
        buf.push(Tag::Byte as u8);
        buf.extend_from_slice(&1u16.to_be_bytes()); buf.push(b'b');
        buf.push(1);

        // Short
        buf.push(Tag::Short as u8);
        buf.extend_from_slice(&1u16.to_be_bytes()); buf.push(b's');
        buf.extend_from_slice(&0x1234u16.to_be_bytes());

        // Int
        buf.push(Tag::Int as u8);
        buf.extend_from_slice(&1u16.to_be_bytes()); buf.push(b'i');
        buf.extend_from_slice(&0x12345678u32.to_be_bytes());
        
        // Long
        buf.push(Tag::Long as u8);
        buf.extend_from_slice(&1u16.to_be_bytes()); buf.push(b'l');
        buf.extend_from_slice(&0x1122334455667788u64.to_be_bytes());
        
        // Float
        buf.push(Tag::Float as u8);
        buf.extend_from_slice(&1u16.to_be_bytes()); buf.push(b'f');
        buf.extend_from_slice(&1.5f32.to_be_bytes());
        
        // Double
        buf.push(Tag::Double as u8);
        buf.extend_from_slice(&1u16.to_be_bytes()); buf.push(b'd');
        buf.extend_from_slice(&1.5f64.to_be_bytes());
        
        // ByteArray
        buf.push(Tag::ByteArray as u8);
        buf.extend_from_slice(&2u16.to_be_bytes()); buf.extend_from_slice(b"ba");
        buf.extend_from_slice(&2u32.to_be_bytes()); buf.extend_from_slice(&[1, 2]);
        
        // String
        buf.push(Tag::String as u8);
        buf.extend_from_slice(&2u16.to_be_bytes()); buf.extend_from_slice(b"st");
        buf.extend_from_slice(&3u16.to_be_bytes()); buf.extend_from_slice(b"val");
        
        // List (of bytes)
        buf.push(Tag::List as u8);
        buf.extend_from_slice(&2u16.to_be_bytes()); buf.extend_from_slice(b"li");
        buf.push(Tag::Byte as u8); buf.extend_from_slice(&1u32.to_be_bytes());
        buf.push(10);
        
        // Compound
        buf.push(Tag::Compound as u8);
        buf.extend_from_slice(&2u16.to_be_bytes()); buf.extend_from_slice(b"co");
        buf.push(Tag::End as u8); // Empty compound
        
        // IntArray
        buf.push(Tag::IntArray as u8);
        buf.extend_from_slice(&2u16.to_be_bytes()); buf.extend_from_slice(b"ia");
        buf.extend_from_slice(&1u32.to_be_bytes()); buf.extend_from_slice(&0x12345678u32.to_be_bytes());
        
        // LongArray
        buf.push(Tag::LongArray as u8);
        buf.extend_from_slice(&2u16.to_be_bytes()); buf.extend_from_slice(b"la");
        buf.extend_from_slice(&1u32.to_be_bytes()); buf.extend_from_slice(&0x1122334455667788u64.to_be_bytes());
    });

    let doc = read_borrowed::<BigEndian>(&data).unwrap();
    let root = doc.root();
    
    // Test write_value_to_vec with fallback
    let written = write_value_to_vec::<_, BigEndian, LittleEndian>(&root).unwrap();
    
    // Test write_value_to_writer with fallback
    let mut cursor = Cursor::new(Vec::new());
    write_value_to_writer::<_, BigEndian, LittleEndian, _>(&mut cursor, &root).unwrap();
    let written_writer = cursor.into_inner();
    
    assert_eq!(written, written_writer);
    
    // We can verify some key parts to ensure swap happened
    // Just find the IntArray tag (0x0B) and check its content
    // We need to be careful about offsets, but we know the structure.
    // Or we can just read it back as LittleEndian!
    
    let doc_le = read_borrowed::<LittleEndian>(&written).unwrap();
    let root_le = doc_le.root();
    let comp = root_le.as_compound().unwrap();
    
    assert_eq!(comp.get("b").unwrap().as_byte(), Some(1));
    assert_eq!(comp.get("s").unwrap().as_short(), Some(0x1234));
    assert_eq!(comp.get("i").unwrap().as_int(), Some(0x12345678));
    assert_eq!(comp.get("l").unwrap().as_long(), Some(0x1122334455667788));
    assert_eq!(comp.get("f").unwrap().as_float(), Some(1.5));
    assert_eq!(comp.get("d").unwrap().as_double(), Some(1.5));
    assert_eq!(comp.get("ba").unwrap().as_byte_array().unwrap(), &[1, 2]);
    assert_eq!(comp.get("st").unwrap().as_string().unwrap().decode(), "val");
    assert_eq!(comp.get("li").unwrap().as_list().unwrap().len(), 1);
    assert_eq!(comp.get("co").unwrap().as_compound().unwrap().iter().count(), 0);
    assert_eq!(comp.get("ia").unwrap().as_int_array().unwrap()[0].get(), 0x12345678);
    assert_eq!(comp.get("la").unwrap().as_long_array().unwrap()[0].get(), 0x1122334455667788);
}

