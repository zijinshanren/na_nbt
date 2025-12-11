use std::marker::PhantomData;

use zerocopy::byteorder;

use crate::{
    ByteOrder, ImmutableCompound, ImmutableList, ImmutableString, IntoOwnedValue, MutableCompound,
    MutableList, MutableValue, OwnedValue, ReadableConfig, ScopedReadableCompound,
    ScopedReadableList, ScopedReadableValue, ScopedWritableCompound, ScopedWritableList,
    ScopedWritableValue, ValueMut, ValueMutScoped, ValueScoped, WritableCompound, WritableConfig,
    WritableList, WritableValue,
    implementation::mutable::{
        iter::{MutableCompoundIter, MutableListIter},
        trait_impl::Config,
    },
    index::Index,
    view::{StringViewMut, VecViewMut},
};

impl<O: ByteOrder> WritableConfig for Config<O> {
    type ValueMut<'s> = MutableValue<'s, O>;
    type ListMut<'s> = MutableList<'s, O>;
    type ListIterMut<'s> = MutableListIter<'s, O>;
    type CompoundMut<'s> = MutableCompound<'s, O>;
    type CompoundIterMut<'s> = MutableCompoundIter<'s, O>;
}

impl<'doc, O: ByteOrder> ScopedReadableValue<'doc> for MutableValue<'doc, O> {
    type Config = Config<O>;

    #[inline]
    fn as_end(&self) -> Option<()> {
        self.as_end()
    }

    #[inline]
    fn is_end(&self) -> bool {
        self.is_end()
    }

    #[inline]
    fn as_byte(&self) -> Option<i8> {
        self.as_byte()
    }

    #[inline]
    fn is_byte(&self) -> bool {
        self.is_byte()
    }

    #[inline]
    fn as_short(&self) -> Option<i16> {
        self.as_short()
    }

    #[inline]
    fn is_short(&self) -> bool {
        self.is_short()
    }

    #[inline]
    fn as_int(&self) -> Option<i32> {
        self.as_int()
    }

    #[inline]
    fn is_int(&self) -> bool {
        self.is_int()
    }

    #[inline]
    fn as_long(&self) -> Option<i64> {
        self.as_long()
    }

    #[inline]
    fn is_long(&self) -> bool {
        self.is_long()
    }

    #[inline]
    fn as_float(&self) -> Option<f32> {
        self.as_float()
    }

    #[inline]
    fn is_float(&self) -> bool {
        self.is_float()
    }

    #[inline]
    fn as_double(&self) -> Option<f64> {
        self.as_double()
    }

    #[inline]
    fn is_double(&self) -> bool {
        self.is_double()
    }

    #[inline]
    fn as_byte_array<'a>(&'a self) -> Option<&'a [i8]>
    where
        'doc: 'a,
    {
        self.as_byte_array()
    }

    #[inline]
    fn is_byte_array(&self) -> bool {
        self.is_byte_array()
    }

    #[inline]
    fn as_string_scoped<'a>(&'a self) -> Option<<Self::Config as ReadableConfig>::String<'a>>
    where
        'doc: 'a,
    {
        self.as_string()
    }

    #[inline]
    fn is_string(&self) -> bool {
        self.is_string()
    }

    #[inline]
    fn as_list_scoped<'a>(&'a self) -> Option<<Self::Config as ReadableConfig>::List<'a>>
    where
        'doc: 'a,
    {
        self.as_list()
    }

    #[inline]
    fn is_list(&self) -> bool {
        self.is_list()
    }

    #[inline]
    fn as_compound_scoped<'a>(&'a self) -> Option<<Self::Config as ReadableConfig>::Compound<'a>>
    where
        'doc: 'a,
    {
        self.as_compound()
    }

    #[inline]
    fn is_compound(&self) -> bool {
        self.is_compound()
    }

    #[inline]
    fn as_int_array<'a>(
        &'a self,
    ) -> Option<&'a [byteorder::I32<<Self::Config as ReadableConfig>::ByteOrder>]>
    where
        'doc: 'a,
    {
        self.as_int_array()
    }

    #[inline]
    fn is_int_array(&self) -> bool {
        self.is_int_array()
    }

    #[inline]
    fn as_long_array<'a>(
        &'a self,
    ) -> Option<&'a [byteorder::I64<<Self::Config as ReadableConfig>::ByteOrder>]>
    where
        'doc: 'a,
    {
        self.as_long_array()
    }

    #[inline]
    fn is_long_array(&self) -> bool {
        self.is_long_array()
    }

    #[inline]
    fn get_scoped<'a, I: Index>(
        &'a self,
        index: I,
    ) -> Option<<Self::Config as ReadableConfig>::Value<'a>>
    where
        'doc: 'a,
    {
        self.get(index)
    }

    fn visit_scoped<'a, R>(&'a self, match_fn: impl FnOnce(ValueScoped<'a, Self::Config>) -> R) -> R
    where
        'doc: 'a,
    {
        match self {
            MutableValue::End => match_fn(ValueScoped::End),
            MutableValue::Byte(value) => match_fn(ValueScoped::Byte(**value)),
            MutableValue::Short(value) => match_fn(ValueScoped::Short(value.get())),
            MutableValue::Int(value) => match_fn(ValueScoped::Int(value.get())),
            MutableValue::Long(value) => match_fn(ValueScoped::Long(value.get())),
            MutableValue::Float(value) => match_fn(ValueScoped::Float(value.get())),
            MutableValue::Double(value) => match_fn(ValueScoped::Double(value.get())),
            MutableValue::ByteArray(value) => match_fn(ValueScoped::ByteArray(value)),
            MutableValue::String(value) => match_fn(ValueScoped::String(ImmutableString {
                data: value.as_mutf8_bytes(),
            })),
            MutableValue::List(value) => match_fn(ValueScoped::List(ImmutableList {
                data: value.data.as_ptr(),
                _marker: PhantomData,
            })),
            MutableValue::Compound(value) => match_fn(ValueScoped::Compound(ImmutableCompound {
                data: value.data.as_ptr(),
                _marker: PhantomData,
            })),
            MutableValue::IntArray(value) => match_fn(ValueScoped::IntArray(value)),
            MutableValue::LongArray(value) => match_fn(ValueScoped::LongArray(value)),
        }
    }
}

impl<'s, O: ByteOrder> ScopedWritableValue<'s> for MutableValue<'s, O> {
    type ConfigMut = Config<O>;

    #[inline]
    fn as_byte_mut<'a>(&'a mut self) -> Option<&'a mut i8>
    where
        's: 'a,
    {
        self.as_byte_mut()
    }

    #[inline]
    fn set_byte(&mut self, data: i8) -> bool {
        self.set_byte(data)
    }

    #[inline]
    fn update_byte(&mut self, f: impl FnOnce(i8) -> i8) -> bool {
        self.update_byte(f)
    }

    #[inline]
    fn as_short_mut<'a>(
        &'a mut self,
    ) -> Option<&'a mut byteorder::I16<<Self::ConfigMut as ReadableConfig>::ByteOrder>>
    where
        's: 'a,
    {
        self.as_short_mut()
    }

    #[inline]
    fn set_short(&mut self, data: i16) -> bool {
        self.set_short(data)
    }

    #[inline]
    fn update_short(&mut self, f: impl FnOnce(i16) -> i16) -> bool {
        self.update_short(f)
    }

    #[inline]
    fn as_int_mut<'a>(
        &'a mut self,
    ) -> Option<&'a mut byteorder::I32<<Self::ConfigMut as ReadableConfig>::ByteOrder>>
    where
        's: 'a,
    {
        self.as_int_mut()
    }

    #[inline]
    fn set_int(&mut self, data: i32) -> bool {
        self.set_int(data)
    }

    #[inline]
    fn update_int(&mut self, f: impl FnOnce(i32) -> i32) -> bool {
        self.update_int(f)
    }

    #[inline]
    fn as_long_mut<'a>(
        &'a mut self,
    ) -> Option<&'a mut byteorder::I64<<Self::ConfigMut as ReadableConfig>::ByteOrder>>
    where
        's: 'a,
    {
        self.as_long_mut()
    }

    #[inline]
    fn set_long(&mut self, data: i64) -> bool {
        self.set_long(data)
    }

    #[inline]
    fn update_long(&mut self, f: impl FnOnce(i64) -> i64) -> bool {
        self.update_long(f)
    }

    #[inline]
    fn as_float_mut<'a>(
        &'a mut self,
    ) -> Option<&'a mut byteorder::F32<<Self::ConfigMut as ReadableConfig>::ByteOrder>>
    where
        's: 'a,
    {
        self.as_float_mut()
    }

    #[inline]
    fn set_float(&mut self, data: f32) -> bool {
        self.set_float(data)
    }

    #[inline]
    fn update_float(&mut self, f: impl FnOnce(f32) -> f32) -> bool {
        self.update_float(f)
    }

    #[inline]
    fn as_double_mut<'a>(
        &'a mut self,
    ) -> Option<&'a mut byteorder::F64<<Self::ConfigMut as ReadableConfig>::ByteOrder>>
    where
        's: 'a,
    {
        self.as_double_mut()
    }

    #[inline]
    fn set_double(&mut self, data: f64) -> bool {
        self.set_double(data)
    }

    #[inline]
    fn update_double(&mut self, f: impl FnOnce(f64) -> f64) -> bool {
        self.update_double(f)
    }

    #[inline]
    fn as_byte_array_mut_scoped<'a>(&'a mut self) -> Option<VecViewMut<'a, i8>>
    where
        's: 'a,
    {
        self.as_byte_array_mut()
            .map(|value| unsafe { VecViewMut::new(value.ptr, value.len, value.cap) })
    }

    #[inline]
    fn as_string_mut_scoped<'a>(&'a mut self) -> Option<StringViewMut<'a>>
    where
        's: 'a,
    {
        self.as_string_mut()
            .map(|value| unsafe { StringViewMut::new(value.ptr, value.len, value.cap) })
    }

    #[inline]
    fn as_list_mut_scoped<'a>(
        &'a mut self,
    ) -> Option<<Self::ConfigMut as WritableConfig>::ListMut<'a>>
    where
        's: 'a,
    {
        self.as_list_mut().map(|value| unsafe {
            MutableList {
                data: VecViewMut::new(value.data.ptr, value.data.len, value.data.cap),
                _marker: PhantomData,
            }
        })
    }

    #[inline]
    fn as_compound_mut_scoped<'a>(
        &'a mut self,
    ) -> Option<<Self::ConfigMut as WritableConfig>::CompoundMut<'a>>
    where
        's: 'a,
    {
        self.as_compound_mut().map(|value| unsafe {
            MutableCompound {
                data: VecViewMut::new(value.data.ptr, value.data.len, value.data.cap),
                _marker: PhantomData,
            }
        })
    }

    #[inline]
    fn as_int_array_mut_scoped<'a>(
        &'a mut self,
    ) -> Option<VecViewMut<'a, byteorder::I32<<Self::ConfigMut as ReadableConfig>::ByteOrder>>>
    where
        's: 'a,
    {
        self.as_int_array_mut()
            .map(|value| unsafe { VecViewMut::new(value.ptr, value.len, value.cap) })
    }

    #[inline]
    fn as_long_array_mut_scoped<'a>(
        &'a mut self,
    ) -> Option<VecViewMut<'a, byteorder::I64<<Self::ConfigMut as ReadableConfig>::ByteOrder>>>
    where
        's: 'a,
    {
        self.as_long_array_mut()
            .map(|value| unsafe { VecViewMut::new(value.ptr, value.len, value.cap) })
    }

    #[inline]
    fn get_mut<'a, I: Index>(
        &'a mut self,
        index: I,
    ) -> Option<<Self::ConfigMut as WritableConfig>::ValueMut<'a>>
    where
        's: 'a,
    {
        self.get_mut(index)
    }

    fn visit_mut_scoped<'a, R>(
        &'a mut self,
        match_fn: impl FnOnce(ValueMutScoped<'a, Self::ConfigMut>) -> R,
    ) -> R
    where
        's: 'a,
    {
        match self {
            MutableValue::End => match_fn(ValueMutScoped::End),
            MutableValue::Byte(value) => match_fn(ValueMutScoped::Byte(value)),
            MutableValue::Short(value) => match_fn(ValueMutScoped::Short(value)),
            MutableValue::Int(value) => match_fn(ValueMutScoped::Int(value)),
            MutableValue::Long(value) => match_fn(ValueMutScoped::Long(value)),
            MutableValue::Float(value) => match_fn(ValueMutScoped::Float(value)),
            MutableValue::Double(value) => match_fn(ValueMutScoped::Double(value)),
            MutableValue::ByteArray(value) => match_fn(ValueMutScoped::ByteArray(unsafe {
                VecViewMut::new(value.ptr, value.len, value.cap)
            })),
            MutableValue::String(value) => match_fn(ValueMutScoped::String(unsafe {
                StringViewMut::new(value.ptr, value.len, value.cap)
            })),
            MutableValue::List(value) => match_fn(ValueMutScoped::List(unsafe {
                MutableList {
                    data: VecViewMut::new(value.data.ptr, value.data.len, value.data.cap),
                    _marker: PhantomData,
                }
            })),
            MutableValue::Compound(value) => match_fn(ValueMutScoped::Compound(unsafe {
                MutableCompound {
                    data: VecViewMut::new(value.data.ptr, value.data.len, value.data.cap),
                    _marker: PhantomData,
                }
            })),
            MutableValue::IntArray(value) => match_fn(ValueMutScoped::IntArray(unsafe {
                VecViewMut::new(value.ptr, value.len, value.cap)
            })),
            MutableValue::LongArray(value) => match_fn(ValueMutScoped::LongArray(unsafe {
                VecViewMut::new(value.ptr, value.len, value.cap)
            })),
        }
    }
}

impl<'s, O: ByteOrder> WritableValue<'s> for MutableValue<'s, O> {
    #[inline]
    fn as_byte_array_mut<'a>(&'a mut self) -> Option<&'a mut VecViewMut<'s, i8>>
    where
        's: 'a,
    {
        self.as_byte_array_mut()
    }

    #[inline]
    fn as_string_mut<'a>(&'a mut self) -> Option<&'a mut StringViewMut<'s>>
    where
        's: 'a,
    {
        self.as_string_mut()
    }

    #[inline]
    fn as_list_mut<'a>(
        &'a mut self,
    ) -> Option<&'a mut <Self::ConfigMut as WritableConfig>::ListMut<'s>>
    where
        's: 'a,
    {
        self.as_list_mut()
    }

    #[inline]
    fn as_compound_mut<'a>(
        &'a mut self,
    ) -> Option<&'a mut <Self::ConfigMut as WritableConfig>::CompoundMut<'s>>
    where
        's: 'a,
    {
        self.as_compound_mut()
    }

    #[inline]
    fn as_int_array_mut<'a>(
        &'a mut self,
    ) -> Option<
        &'a mut VecViewMut<'s, byteorder::I32<<Self::ConfigMut as ReadableConfig>::ByteOrder>>,
    >
    where
        's: 'a,
    {
        self.as_int_array_mut()
    }

    #[inline]
    fn as_long_array_mut<'a>(
        &'a mut self,
    ) -> Option<
        &'a mut VecViewMut<'s, byteorder::I64<<Self::ConfigMut as ReadableConfig>::ByteOrder>>,
    >
    where
        's: 'a,
    {
        self.as_long_array_mut()
    }

    fn visit_mut<'a, R>(
        &'a mut self,
        match_fn: impl FnOnce(crate::ValueMut<'a, 's, Self::ConfigMut>) -> R,
    ) -> R
    where
        's: 'a,
    {
        match self {
            MutableValue::End => match_fn(ValueMut::End),
            MutableValue::Byte(value) => match_fn(ValueMut::Byte(value)),
            MutableValue::Short(value) => match_fn(ValueMut::Short(value)),
            MutableValue::Int(value) => match_fn(ValueMut::Int(value)),
            MutableValue::Long(value) => match_fn(ValueMut::Long(value)),
            MutableValue::Float(value) => match_fn(ValueMut::Float(value)),
            MutableValue::Double(value) => match_fn(ValueMut::Double(value)),
            MutableValue::ByteArray(value) => match_fn(ValueMut::ByteArray(value)),
            MutableValue::String(value) => match_fn(ValueMut::String(value)),
            MutableValue::List(value) => match_fn(ValueMut::List(value)),
            MutableValue::Compound(value) => match_fn(ValueMut::Compound(value)),
            MutableValue::IntArray(value) => match_fn(ValueMut::IntArray(value)),
            MutableValue::LongArray(value) => match_fn(ValueMut::LongArray(value)),
        }
    }
}

impl<'doc, O: ByteOrder> ScopedReadableList<'doc> for MutableList<'doc, O> {
    type Config = Config<O>;

    #[inline]
    fn tag_id(&self) -> u8 {
        self.tag_id()
    }

    #[inline]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    #[inline]
    fn get_scoped<'a>(&'a self, index: usize) -> Option<<Self::Config as ReadableConfig>::Value<'a>>
    where
        'doc: 'a,
    {
        self.get(index)
    }

    #[inline]
    fn iter_scoped<'a>(&'a self) -> <Self::Config as ReadableConfig>::ListIter<'a>
    where
        'doc: 'a,
    {
        self.iter()
    }
}

impl<'s, O: ByteOrder> ScopedWritableList<'s> for MutableList<'s, O> {
    type ConfigMut = Config<O>;

    #[inline]
    fn get_mut<'a>(
        &'a mut self,
        index: usize,
    ) -> Option<<Self::ConfigMut as WritableConfig>::ValueMut<'a>>
    where
        's: 'a,
    {
        self.get_mut(index)
    }

    #[inline]
    fn iter_mut<'a>(&'a mut self) -> <Self::ConfigMut as WritableConfig>::ListIterMut<'a>
    where
        's: 'a,
    {
        self.iter_mut()
    }

    #[inline]
    fn push<V: IntoOwnedValue<<Self::ConfigMut as ReadableConfig>::ByteOrder>>(
        &mut self,
        value: V,
    ) {
        self.push(value);
    }

    #[inline]
    unsafe fn push_unchecked<V: IntoOwnedValue<<Self::ConfigMut as ReadableConfig>::ByteOrder>>(
        &mut self,
        value: V,
    ) {
        unsafe { self.push_unchecked(value) };
    }

    #[inline]
    fn insert<V: IntoOwnedValue<<Self::ConfigMut as ReadableConfig>::ByteOrder>>(
        &mut self,
        index: usize,
        value: V,
    ) {
        self.insert(index, value);
    }

    #[inline]
    unsafe fn insert_unchecked<
        V: IntoOwnedValue<<Self::ConfigMut as ReadableConfig>::ByteOrder>,
    >(
        &mut self,
        index: usize,
        value: V,
    ) {
        unsafe { self.insert_unchecked(index, value) };
    }

    #[inline]
    fn pop(&mut self) -> Option<OwnedValue<<Self::ConfigMut as ReadableConfig>::ByteOrder>> {
        self.pop()
    }

    #[inline]
    fn remove(
        &mut self,
        index: usize,
    ) -> OwnedValue<<Self::ConfigMut as ReadableConfig>::ByteOrder> {
        self.remove(index)
    }
}

impl<'s, O: ByteOrder> WritableList<'s> for MutableList<'s, O> {}

impl<'doc, O: ByteOrder> ScopedReadableCompound<'doc> for MutableCompound<'doc, O> {
    type Config = Config<O>;

    #[inline]
    fn get_scoped<'a>(&'a self, key: &str) -> Option<<Self::Config as ReadableConfig>::Value<'a>>
    where
        'doc: 'a,
    {
        self.get(key)
    }

    #[inline]
    fn iter_scoped<'a>(&'a self) -> <Self::Config as ReadableConfig>::CompoundIter<'a>
    where
        'doc: 'a,
    {
        self.iter()
    }
}

impl<'s, O: ByteOrder> ScopedWritableCompound<'s> for MutableCompound<'s, O> {
    type ConfigMut = Config<O>;

    #[inline]
    fn get_mut<'a>(
        &'a mut self,
        key: &str,
    ) -> Option<<Self::ConfigMut as WritableConfig>::ValueMut<'a>>
    where
        's: 'a,
    {
        self.get_mut(key)
    }

    #[inline]
    fn iter_mut<'a>(&'a mut self) -> <Self::ConfigMut as WritableConfig>::CompoundIterMut<'a>
    where
        's: 'a,
    {
        self.iter_mut()
    }

    #[inline]
    fn insert<V: IntoOwnedValue<<Self::ConfigMut as ReadableConfig>::ByteOrder>>(
        &mut self,
        key: &str,
        value: V,
    ) -> Option<OwnedValue<<Self::ConfigMut as ReadableConfig>::ByteOrder>> {
        self.insert(key, value)
    }

    #[inline]
    fn remove(
        &mut self,
        key: &str,
    ) -> Option<OwnedValue<<Self::ConfigMut as ReadableConfig>::ByteOrder>> {
        self.remove(key)
    }
}

impl<'s, O: ByteOrder> WritableCompound<'s> for MutableCompound<'s, O> {}
