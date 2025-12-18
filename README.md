# na_nbt

A high-performance NBT (Named Binary Tag) library for Rust with zero-copy parsing and full mutation support.

> ⚠️ **Note:** This crate is under active development. APIs may change between versions.
> Issues and contributions are welcome!

## Features

- **Zero-copy parsing** - Read NBT data without allocating memory for values
- **Full mutation** - Create and modify NBT structures with an owned representation  
- **Endianness support** - Convert between BigEndian and LittleEndian on read or write
- **Generic traits** - Write code that works with any value type
- **Serde integration** - Serialize/deserialize Rust types directly to/from NBT (optional)
- **Shared values** - Thread-safe `Arc`-based values with `bytes` crate (optional)

## Installation

```toml
[dependencies]
na_nbt = "0.1.0"
```

### Optional Features

Both `serde` and `shared` features are enabled by default. To use without optional dependencies:

```toml
[dependencies]
na_nbt = { version = "0.1.0", default-features = false }

# Or enable only what you need:
na_nbt = { version = "0.1.0", default-features = false, features = ["serde"] }
```

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `serde` | Serialize/deserialize Rust types to/from NBT | `serde` |
| `shared` | `SharedValue` with Arc ownership | `bytes` |

## Quick Start

```rust
use na_nbt::read_borrowed;
use zerocopy::byteorder::BigEndian;

let data = [
    0x0a, 0x00, 0x00, // Compound with empty name
    0x01, 0x00, 0x03, b'f', b'o', b'o', 42u8, // Byte "foo" = 42
    0x00, // End
];

let doc = read_borrowed::<BigEndian>(&data).unwrap();
let root = doc.root();

if let Some(value) = root.get("foo") {
    assert_eq!(value.as_byte(), Some(42));
}
```

## Two Parsing Modes

| Mode | Function | Type | Use Case |
|------|----------|------|---------|
| **Zero-copy (borrowed)** | `read_borrowed` | `BorrowedValue` | Fast reads, data lives on stack/slice |
| **Zero-copy (shared)** | `read_shared` | `SharedValue` | Pass values across threads |
| **Owned** | `read_owned` | `OwnedValue` | Need to modify or outlive source data |

### Zero-Copy Mode (Borrowed)

Parses NBT without copying data. Values reference the original byte slice directly.

```rust
use na_nbt::read_borrowed;
use zerocopy::byteorder::BigEndian;

let data: &[u8] = &[0x0a, 0x00, 0x00, 0x00]; // Empty compound
let doc = read_borrowed::<BigEndian>(data).unwrap();
let root = doc.root(); // Zero-copy reference into `data`
```

### Zero-Copy Mode (Shared)

Like borrowed mode, but wraps data in `Arc` for shared ownership. Values are
`Clone`, `Send`, `Sync`, and `'static` - perfect for multi-threaded scenarios.

```rust
use na_nbt::read_shared;
use bytes::Bytes;
use zerocopy::byteorder::BigEndian;

let data = Bytes::from_static(&[0x0a, 0x00, 0x00, 0x00]);
let root = read_shared::<BigEndian>(data).unwrap();

// Can clone and send to other threads
let cloned = root.clone();
std::thread::spawn(move || {
    assert!(cloned.as_compound().is_some());
}).join().unwrap();
```

### Owned Mode

Parses NBT into an owned structure that can be modified.

```rust
use na_nbt::{read_owned, OwnedValue};
use zerocopy::byteorder::{BigEndian, LittleEndian};

let data: &[u8] = &[0x0a, 0x00, 0x00, 0x00];

// Convert from BigEndian source to LittleEndian storage
let mut root: OwnedValue<LittleEndian> = read_owned::<BigEndian, LittleEndian>(data).unwrap();

if let OwnedValue::Compound(ref mut compound) = root {
    compound.insert("score", 100i32);
}
```

## Writing Generic Code

### Trait Hierarchy

```text
ScopedReadableValue (all types)
        ▲
        │
┌───────┴───────┐
│               │
ReadableValue   ScopedWritableValue
(immutable)     (scoped mutation)
                        ▲
                        │
                  WritableValue
                  (full mutation)
```

- `ScopedReadableValue` - Base trait implemented by all value types
- `ReadableValue` - Extends `ScopedReadableValue` with document-lifetime references
- `ScopedWritableValue` - Extends `ScopedReadableValue` with scoped mutation
- `WritableValue` - Extends `ScopedWritableValue` with full mutable references

### Scoped vs Unscoped Methods

The "scoped" suffix indicates **bounded lifetime** and **indirection**:

- **Unscoped** methods (e.g., `get()`, `as_byte_array()`) return references to data already 
  stored in the value type with **document lifetime `'doc`** - can be stored independently.

- **Scoped** methods (e.g., `get_scoped()`, `as_byte_array_scoped()`) construct new view 
  types on demand with **borrow lifetime `'a`** - necessary for types like `OwnedValue` 
  that don't directly store container types.

**When to use which:**
- Use **scoped** methods when writing generic code that works with all value types
- Use **unscoped** methods when you need direct access to stored fields with longer lifetime

| Trait | Capability | Implemented By |
|-------|------------|----------------|
| `ScopedReadableValue` | Read primitives, iterate (scoped) | All value types |
| `ReadableValue` | + Document-lifetime references | `BorrowedValue`, `SharedValue`, `ImmutableValue` |
| `ScopedWritableValue` | + Mutation (scoped) | `OwnedValue`, `MutableValue` |
| `WritableValue` | + Mutable references to containers | `MutableValue` |

```rust
use na_nbt::{ScopedReadableValue, ScopedReadableCompound, ReadableString};

fn print_compound_keys<'doc>(value: &impl ScopedReadableValue<'doc>) {
    if let Some(compound) = value.as_compound_scoped() {
        for (key, _) in compound.iter() {
            println!("Key: {}", key.decode());
        }
    }
}
```

## Type Overview

### Zero-Copy Types

| Type | Description |
|------|-------------|
| `ReadonlyValue` | Underlying zero-copy value |
| `BorrowedValue` | Type alias for borrowed data |
| `SharedValue` | Type alias for `Arc`-wrapped data |

### Owned Types

| Type | Description |
|------|-------------|
| `OwnedValue` | Fully owned, mutable NBT value |
| `MutableValue` | Mutable view into an `OwnedValue` |
| `ImmutableValue` | Immutable view into an `OwnedValue` |

## Feature Comparison

| Feature | `BorrowedValue` | `OwnedValue` |
|---------|-----------------|--------------|
| Zero-copy parsing | ✅ | ❌ |
| Modify values | ❌ | ✅ |
| Outlives source | ❌ | ✅ |
| Endianness conversion | On write | On read or write |
| Memory usage | Minimal | Proportional to data |

## Serde Integration

Serialize and deserialize Rust types directly to/from NBT binary format using [serde](https://serde.rs).

### Basic Usage

```rust
use serde::{Serialize, Deserialize};
use na_nbt::{to_vec_be, from_slice_be};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Player {
    name: String,
    health: f32,
    score: i32,
}

// Serialize to NBT
let player = Player {
    name: "Steve".to_string(),
    health: 20.0,
    score: 100,
};
let bytes = to_vec_be(&player).unwrap();

// Deserialize from NBT
let loaded: Player = from_slice_be(&bytes).unwrap();
assert_eq!(player, loaded);
```

### Convenience Functions

| Function | Description |
|----------|-------------|
| `to_vec_be` / `to_vec_le` | Serialize to `Vec<u8>` |
| `to_writer_be` / `to_writer_le` | Serialize to any `io::Write` |
| `from_slice_be` / `from_slice_le` | Deserialize from `&[u8]` |
| `from_reader_be` / `from_reader_le` | Deserialize from any `io::Read` |

The `_be` suffix means big-endian (Java Edition), `_le` means little-endian (Bedrock Edition).

### File I/O

```rust
use std::fs::File;
use na_nbt::{to_writer_be, from_reader_be};

// Write to file
let mut file = File::create("player.nbt")?;
to_writer_be(&mut file, &player)?;

// Read from file
let file = File::open("player.nbt")?;
let player: Player = from_reader_be(file)?;
```

### Type Mapping

| Rust Type | NBT Tag |
|-----------|---------|
| `bool`, `i8`, `u8` | Byte |
| `i16`, `u16` | Short |
| `i32`, `u32`, `char` | Int |
| `i64`, `u64` | Long |
| `f32` | Float |
| `f64` | Double |
| `String`, `&str` | String |
| `Vec<T>` | List |
| struct | Compound |
| `HashMap<String, T>` | Compound |
| `Option<T>` | Compound |
| enum variants | Various (see docs) |

### Native Array Types

NBT has efficient native array types (`ByteArray`, `IntArray`, `LongArray`).

**Deserialization is automatic!** Native arrays are detected and read correctly to `Vec<T>`:

```rust
#[derive(Deserialize)]
struct ChunkData {
    block_states: Vec<i64>,  // Auto-detects LongArray or List<Long>
    biomes: Vec<i32>,        // Auto-detects IntArray or List<Int>
    heightmap: Vec<i8>,      // Auto-detects ByteArray or List<Byte>
}
```

**For serialization**, use `#[serde(with = "...")]` for zero-copy performance:

```rust
#[derive(Serialize, Deserialize)]
struct ChunkData {
    #[serde(with = "na_nbt::long_array")]
    block_states: Vec<i64>,  // Zero-copy → LongArray
    
    #[serde(with = "na_nbt::int_array")]
    biomes: Vec<i32>,        // Zero-copy → IntArray
    
    #[serde(with = "na_nbt::byte_array")]
    heightmap: Vec<i8>,      // Zero-copy → ByteArray
}
```

For full serde documentation, see the [`de`](https://docs.rs/na_nbt/latest/na_nbt/de/) and [`ser`](https://docs.rs/na_nbt/latest/na_nbt/ser/) module docs.

## Contributing

This crate is under active development. Contributions are welcome!

- **Bug reports**: Please open an issue with a minimal reproducible example
- **Feature requests**: Open an issue describing the use case
- **Pull requests**: Fork the repo, make your changes, and submit a PR

## License

MIT OR Apache-2.0
