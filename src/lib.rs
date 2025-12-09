pub mod error;
mod implementation;
pub mod index;
pub mod util;
// pub mod value_trait;
pub mod view;

pub use implementation::immutable::{BorrowedValue, SharedValue};
pub use implementation::immutable::{read_borrowed, read_shared};

pub use implementation::mutable::write;
pub use implementation::mutable::{
    ImmutableCompound, ImmutableList, ImmutableString, ImmutableValue, MutableCompound,
    MutableList, MutableValue, Name, OwnedCompound, OwnedList, OwnedValue,
};

pub use error::NbtError;
