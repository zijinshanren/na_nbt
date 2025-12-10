use zerocopy::byteorder;

use crate::{
    index::Index,
    value_trait::{ReadableConfig, ValueScoped},
};

pub trait ScopedReadableValue<'doc>: Send + Sync + Sized {
    type Config: ReadableConfig;

    fn as_end(&self) -> Option<()>;
    fn is_end(&self) -> bool;

    fn as_byte(&self) -> Option<i8>;
    fn is_byte(&self) -> bool;

    fn as_short(&self) -> Option<i16>;
    fn is_short(&self) -> bool;

    fn as_int(&self) -> Option<i32>;
    fn is_int(&self) -> bool;

    fn as_long(&self) -> Option<i64>;
    fn is_long(&self) -> bool;

    fn as_float(&self) -> Option<f32>;
    fn is_float(&self) -> bool;

    fn as_double(&self) -> Option<f64>;
    fn is_double(&self) -> bool;

    fn as_byte_array<'a>(&'a self) -> Option<&'a [i8]>
    where
        'doc: 'a;
    fn is_byte_array(&self) -> bool;

    fn as_string_scoped<'a>(&'a self) -> Option<<Self::Config as ReadableConfig>::String<'a>>
    where
        'doc: 'a;
    fn is_string(&self) -> bool;

    fn as_list_scoped<'a>(&'a self) -> Option<<Self::Config as ReadableConfig>::List<'a>>
    where
        'doc: 'a;
    fn is_list(&self) -> bool;

    fn as_compound_scoped<'a>(&'a self) -> Option<<Self::Config as ReadableConfig>::Compound<'a>>
    where
        'doc: 'a;
    fn is_compound(&self) -> bool;

    fn as_int_array<'a>(
        &'a self,
    ) -> Option<&'a [byteorder::I32<<Self::Config as ReadableConfig>::ByteOrder>]>
    where
        'doc: 'a;
    fn is_int_array(&self) -> bool;

    fn as_long_array<'a>(
        &'a self,
    ) -> Option<&'a [byteorder::I64<<Self::Config as ReadableConfig>::ByteOrder>]>
    where
        'doc: 'a;
    fn is_long_array(&self) -> bool;

    fn get_scoped<'a, I: Index>(
        &'a self,
        index: I,
    ) -> Option<<Self::Config as ReadableConfig>::Value<'a>>
    where
        'doc: 'a;

    fn visit_scoped<'a, R>(
        &'a self,
        match_fn: impl FnOnce(ValueScoped<'a, Self::Config>) -> R,
    ) -> R
    where
        'doc: 'a;
}

pub trait ScopedReadableList<'doc>: IntoIterator + Send + Sync + Sized {
    type Config: ReadableConfig;

    fn get_scoped<'a>(
        &'a self,
        index: usize,
    ) -> Option<<Self::Config as ReadableConfig>::Value<'a>>
    where
        'doc: 'a;

    fn iter_scoped<'a>(&'a self) -> <Self::Config as ReadableConfig>::ListIter<'a>
    where
        'doc: 'a;
}

pub trait ScopedReadableCompound<'doc>: IntoIterator + Send + Sync + Sized {
    type Config: ReadableConfig;

    fn get_scoped<'a>(&'a self, key: &str) -> Option<<Self::Config as ReadableConfig>::Value<'a>>
    where
        'doc: 'a;

    fn iter_scoped<'a>(&'a self) -> <Self::Config as ReadableConfig>::CompoundIter<'a>
    where
        'doc: 'a;
}
