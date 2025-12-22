//! Example: Reading NBT files from disk
//!
//! This example demonstrates how to read NBT files using na_nbt.
//! NBT files can be either uncompressed or gzip/zlib compressed.
//! Compressed files are automatically decompressed using flate2.
//!
//! Run with: cargo run --example read_file -- <path_to_nbt_file>

use std::env;
use std::fs::File;
use std::io::{BufReader, Read};

use flate2::read::GzDecoder;
use na_nbt::{
    ReadableString, ScopedReadableList, ScopedReadableValue, ValueScoped, from_slice_be,
    read_borrowed,
};
use serde::Deserialize;
use zerocopy::BigEndian;

/// A simple struct to demonstrate serde deserialization.
/// Adjust fields based on your NBT file structure.
#[allow(dead_code)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct LevelData {
    #[serde(default)]
    level_name: Option<String>,
    #[serde(default)]
    spawn_x: Option<i32>,
    #[serde(default)]
    spawn_y: Option<i32>,
    #[serde(default)]
    spawn_z: Option<i32>,
}

/// Pretty-print any NBT value recursively
fn dump<'doc>(value: &impl ScopedReadableValue<'doc>) -> String {
    dump_inner(value, 0)
}

fn dump_inner<'doc>(value: &impl ScopedReadableValue<'doc>, indent: usize) -> String {
    let pad = "  ".repeat(indent);
    value.visit_scoped(|v| match v {
        ValueScoped::End => format!("{pad}End"),
        ValueScoped::Byte(v) => format!("{pad}Byte({v})"),
        ValueScoped::Short(v) => format!("{pad}Short({v})"),
        ValueScoped::Int(v) => format!("{pad}Int({v})"),
        ValueScoped::Long(v) => format!("{pad}Long({v})"),
        ValueScoped::Float(v) => format!("{pad}Float({v})"),
        ValueScoped::Double(v) => format!("{pad}Double({v})"),
        ValueScoped::ByteArray(v) => format!("{pad}ByteArray({} bytes)", v.len()),
        ValueScoped::String(v) => format!("{pad}String({:?})", v.decode()),
        ValueScoped::IntArray(v) => format!("{pad}IntArray({} ints)", v.len()),
        ValueScoped::LongArray(v) => format!("{pad}LongArray({} longs)", v.len()),
        ValueScoped::List(list) => {
            let mut out = format!("{pad}List[{}] {{\n", list.len());
            for item in list {
                out.push_str(&dump_inner(&item, indent + 1));
                out.push('\n');
            }
            out.push_str(&format!("{pad}}}"));
            out
        }
        ValueScoped::Compound(compound) => {
            let mut out = format!("{pad}Compound {{\n");
            for (key, val) in compound {
                let nested = dump_inner(&val, indent + 1);
                out.push_str(&format!(
                    "{}  {:?}: {}\n",
                    pad,
                    key.decode(),
                    nested.trim_start()
                ));
            }
            out.push_str(&format!("{pad}}}"));
            out
        }
    })
}

/// Compression type detected from file header
#[derive(Debug, Clone, Copy)]
enum Compression {
    None,
    Gzip,
    Zlib,
}

/// Detect compression type from the first bytes of data
fn detect_compression(data: &[u8]) -> Compression {
    if data.len() >= 2 {
        // Gzip magic: 0x1f 0x8b
        if data[0] == 0x1f && data[1] == 0x8b {
            return Compression::Gzip;
        }
        // Zlib magic: 0x78 followed by 0x01, 0x5e, 0x9c, or 0xda
        if data[0] == 0x78 && matches!(data[1], 0x01 | 0x5e | 0x9c | 0xda) {
            return Compression::Zlib;
        }
    }
    Compression::None
}

/// Read and decompress file data if needed
fn read_nbt_file(path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut raw_data = Vec::new();
    reader.read_to_end(&mut raw_data)?;

    let compression = detect_compression(&raw_data);
    println!("Compression: {:?}", compression);

    let data = match compression {
        Compression::None => raw_data,
        Compression::Gzip => {
            let mut decoder = GzDecoder::new(&raw_data[..]);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed)?;
            decompressed
        }
        Compression::Zlib => {
            let mut decoder = flate2::read::ZlibDecoder::new(&raw_data[..]);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed)?;
            decompressed
        }
    };

    Ok(data)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: cargo run --example read_file -- <path_to_nbt_file>");
        println!();
        println!("Examples:");
        println!("  cargo run --example read_file -- level.dat");
        println!("  cargo run --example read_file -- player.dat");
        println!();
        println!("Supported formats:");
        println!("  - Uncompressed NBT");
        println!("  - Gzip compressed NBT (.dat files)");
        println!("  - Zlib compressed NBT");
        return Ok(());
    }

    let path = &args[1];
    println!("Reading NBT file: {}", path);
    println!();

    // Read and decompress file
    let data = read_nbt_file(path)?;
    println!("Decompressed size: {} bytes", data.len());
    println!();

    // Method 1: Zero-copy parsing (for inspection/traversal)
    println!("=== Zero-copy parsing ===");
    match read_borrowed::<BigEndian>(&data) {
        Ok(doc) => {
            let root = doc.root();
            println!("{}", dump(&root));
        }
        Err(e) => {
            println!("Failed to parse as BigEndian NBT: {}", e);
            println!("Try LittleEndian if this is a Bedrock Edition file.");
        }
    }
    println!();

    // Method 2: Serde deserialization (for typed access)
    println!("=== Serde deserialization ===");
    match from_slice_be::<LevelData>(&data) {
        Ok(level) => {
            println!("{:#?}", level);
        }
        Err(e) => {
            println!("Serde deserialization failed: {}", e);
            println!("(This is expected if the file structure doesn't match LevelData)");
        }
    }

    Ok(())
}
