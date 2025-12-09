use zerocopy::byteorder;

use crate::{
    OwnedCompound, OwnedList, OwnedValue,
    implementation::mutable::util::{
        compound_insert_byte, compound_insert_byte_array, compound_insert_compound,
        compound_insert_double, compound_insert_end, compound_insert_float, compound_insert_int,
        compound_insert_int_array, compound_insert_list, compound_insert_long,
        compound_insert_long_array, compound_insert_short, compound_insert_string,
        compound_insert_value, list_insert_byte, list_insert_byte_array,
        list_insert_byte_array_unchecked, list_insert_byte_unchecked, list_insert_compound,
        list_insert_compound_unchecked, list_insert_double, list_insert_double_unchecked,
        list_insert_end, list_insert_end_unchecked, list_insert_float, list_insert_float_unchecked,
        list_insert_int, list_insert_int_array, list_insert_int_array_unchecked,
        list_insert_int_unchecked, list_insert_list, list_insert_list_unchecked, list_insert_long,
        list_insert_long_array, list_insert_long_array_unchecked, list_insert_long_unchecked,
        list_insert_short, list_insert_short_unchecked, list_insert_string,
        list_insert_string_unchecked, list_insert_value, list_insert_value_unchecked,
        list_push_byte, list_push_byte_array, list_push_byte_array_unchecked,
        list_push_byte_unchecked, list_push_compound, list_push_compound_unchecked,
        list_push_double, list_push_double_unchecked, list_push_end, list_push_end_unchecked,
        list_push_float, list_push_float_unchecked, list_push_int, list_push_int_array,
        list_push_int_array_unchecked, list_push_int_unchecked, list_push_list,
        list_push_list_unchecked, list_push_long, list_push_long_array,
        list_push_long_array_unchecked, list_push_long_unchecked, list_push_short,
        list_push_short_unchecked, list_push_string, list_push_string_unchecked, list_push_value,
        list_push_value_unchecked,
    },
    util::ByteOrder,
    view::VecViewMut,
};

mod private {
    use zerocopy::byteorder;

    use crate::{OwnedCompound, OwnedList, OwnedValue, util::ByteOrder};

    pub trait Sealed<O: ByteOrder> {}
    impl<O: ByteOrder> Sealed<O> for () {}
    impl<O: ByteOrder> Sealed<O> for i8 {}
    impl<O: ByteOrder> Sealed<O> for byteorder::I16<O> {}
    impl<O: ByteOrder> Sealed<O> for u8 {}
    impl<O: ByteOrder> Sealed<O> for i16 {}
    impl<O: ByteOrder> Sealed<O> for byteorder::I32<O> {}
    impl<O: ByteOrder> Sealed<O> for u16 {}
    impl<O: ByteOrder> Sealed<O> for i32 {}
    impl<O: ByteOrder> Sealed<O> for byteorder::I64<O> {}
    impl<O: ByteOrder> Sealed<O> for u32 {}
    impl<O: ByteOrder> Sealed<O> for i64 {}
    impl<O: ByteOrder> Sealed<O> for byteorder::F32<O> {}
    impl<O: ByteOrder> Sealed<O> for f32 {}
    impl<O: ByteOrder> Sealed<O> for byteorder::F64<O> {}
    impl<O: ByteOrder> Sealed<O> for f64 {}
    impl<O: ByteOrder> Sealed<O> for &[i8] {}
    impl<O: ByteOrder, const N: usize> Sealed<O> for [i8; N] {}
    impl<O: ByteOrder> Sealed<O> for Vec<i8> {}
    impl<O: ByteOrder> Sealed<O> for &str {}
    impl<O: ByteOrder> Sealed<O> for String {}
    impl<O: ByteOrder> Sealed<O> for OwnedList<O> {}
    impl<O: ByteOrder> Sealed<O> for OwnedCompound<O> {}
    impl<O: ByteOrder> Sealed<O> for &[byteorder::I32<O>] {}
    impl<O: ByteOrder, const N: usize> Sealed<O> for [byteorder::I32<O>; N] {}
    impl<O: ByteOrder> Sealed<O> for Vec<byteorder::I32<O>> {}
    impl<O: ByteOrder> Sealed<O> for &[byteorder::I64<O>] {}
    impl<O: ByteOrder, const N: usize> Sealed<O> for [byteorder::I64<O>; N] {}
    impl<O: ByteOrder> Sealed<O> for Vec<byteorder::I64<O>> {}
    impl<O: ByteOrder> Sealed<O> for OwnedValue<O> {}
}

pub trait IntoOwnedValue<O: ByteOrder>: private::Sealed<O> {
    #[doc(hidden)]
    fn compound_insert(self, data: &mut VecViewMut<'_, u8>, key: &str) -> Option<OwnedValue<O>>;

    #[doc(hidden)]
    fn list_push(self, data: &mut VecViewMut<'_, u8>);

    #[doc(hidden)]
    unsafe fn list_push_unchecked(self, data: &mut VecViewMut<'_, u8>);

    #[doc(hidden)]
    fn list_insert(self, data: &mut VecViewMut<'_, u8>, index: usize);

    #[doc(hidden)]
    unsafe fn list_insert_unchecked(self, data: &mut VecViewMut<'_, u8>, index: usize);
}

impl<O: ByteOrder> IntoOwnedValue<O> for () {
    #[inline]
    fn compound_insert(self, data: &mut VecViewMut<'_, u8>, key: &str) -> Option<OwnedValue<O>> {
        compound_insert_end(data, key, self)
    }
    #[inline]
    fn list_push(self, data: &mut VecViewMut<'_, u8>) {
        list_push_end::<O>(data, self);
    }
    #[inline]
    unsafe fn list_push_unchecked(self, data: &mut VecViewMut<'_, u8>) {
        unsafe { list_push_end_unchecked::<O>(data, self) };
    }
    #[inline]
    fn list_insert(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        list_insert_end::<O>(data, index, self);
    }
    #[inline]
    unsafe fn list_insert_unchecked(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        unsafe { list_insert_end_unchecked::<O>(data, index, self) };
    }
}

impl<O: ByteOrder> IntoOwnedValue<O> for i8 {
    #[inline]
    fn compound_insert(self, data: &mut VecViewMut<'_, u8>, key: &str) -> Option<OwnedValue<O>> {
        compound_insert_byte(data, key, self)
    }
    #[inline]
    fn list_push(self, data: &mut VecViewMut<'_, u8>) {
        list_push_byte::<O>(data, self);
    }
    #[inline]
    unsafe fn list_push_unchecked(self, data: &mut VecViewMut<'_, u8>) {
        unsafe { list_push_byte_unchecked::<O>(data, self) };
    }
    #[inline]
    fn list_insert(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        list_insert_byte::<O>(data, index, self);
    }
    #[inline]
    unsafe fn list_insert_unchecked(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        unsafe { list_insert_byte_unchecked::<O>(data, index, self) };
    }
}

impl<O: ByteOrder> IntoOwnedValue<O> for byteorder::I16<O> {
    #[inline]
    fn compound_insert(self, data: &mut VecViewMut<'_, u8>, key: &str) -> Option<OwnedValue<O>> {
        compound_insert_short(data, key, self)
    }
    #[inline]
    fn list_push(self, data: &mut VecViewMut<'_, u8>) {
        list_push_short::<O>(data, self);
    }
    #[inline]
    unsafe fn list_push_unchecked(self, data: &mut VecViewMut<'_, u8>) {
        unsafe { list_push_short_unchecked::<O>(data, self) };
    }
    #[inline]
    fn list_insert(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        list_insert_short::<O>(data, index, self);
    }
    #[inline]
    unsafe fn list_insert_unchecked(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        unsafe { list_insert_short_unchecked::<O>(data, index, self) };
    }
}

impl<O: ByteOrder> IntoOwnedValue<O> for u8 {
    #[inline]
    fn compound_insert(self, data: &mut VecViewMut<'_, u8>, key: &str) -> Option<OwnedValue<O>> {
        (self as i16).compound_insert(data, key)
    }
    #[inline]
    fn list_push(self, data: &mut VecViewMut<'_, u8>) {
        IntoOwnedValue::<O>::list_push(self as i16, data)
    }
    #[inline]
    unsafe fn list_push_unchecked(self, data: &mut VecViewMut<'_, u8>) {
        unsafe { IntoOwnedValue::<O>::list_push_unchecked(self as i16, data) }
    }
    #[inline]
    fn list_insert(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        IntoOwnedValue::<O>::list_insert(self as i16, data, index)
    }
    #[inline]
    unsafe fn list_insert_unchecked(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        unsafe { IntoOwnedValue::<O>::list_insert_unchecked(self as i16, data, index) }
    }
}

impl<O: ByteOrder> IntoOwnedValue<O> for i16 {
    #[inline]
    fn compound_insert(self, data: &mut VecViewMut<'_, u8>, key: &str) -> Option<OwnedValue<O>> {
        byteorder::I16::<O>::from(self).compound_insert(data, key)
    }
    #[inline]
    fn list_push(self, data: &mut VecViewMut<'_, u8>) {
        byteorder::I16::<O>::from(self).list_push(data)
    }
    #[inline]
    unsafe fn list_push_unchecked(self, data: &mut VecViewMut<'_, u8>) {
        unsafe { byteorder::I16::<O>::from(self).list_push_unchecked(data) }
    }
    #[inline]
    fn list_insert(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        byteorder::I16::<O>::from(self).list_insert(data, index)
    }
    #[inline]
    unsafe fn list_insert_unchecked(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        unsafe { byteorder::I16::<O>::from(self).list_insert_unchecked(data, index) }
    }
}

impl<O: ByteOrder> IntoOwnedValue<O> for byteorder::I32<O> {
    #[inline]
    fn compound_insert(self, data: &mut VecViewMut<'_, u8>, key: &str) -> Option<OwnedValue<O>> {
        compound_insert_int(data, key, self)
    }
    #[inline]
    fn list_push(self, data: &mut VecViewMut<'_, u8>) {
        list_push_int::<O>(data, self);
    }
    #[inline]
    unsafe fn list_push_unchecked(self, data: &mut VecViewMut<'_, u8>) {
        unsafe { list_push_int_unchecked::<O>(data, self) };
    }
    #[inline]
    fn list_insert(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        list_insert_int::<O>(data, index, self);
    }
    #[inline]
    unsafe fn list_insert_unchecked(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        unsafe { list_insert_int_unchecked::<O>(data, index, self) };
    }
}

impl<O: ByteOrder> IntoOwnedValue<O> for u16 {
    #[inline]
    fn compound_insert(self, data: &mut VecViewMut<'_, u8>, key: &str) -> Option<OwnedValue<O>> {
        (self as i32).compound_insert(data, key)
    }
    #[inline]
    fn list_push(self, data: &mut VecViewMut<'_, u8>) {
        IntoOwnedValue::<O>::list_push(self as i32, data)
    }
    #[inline]
    unsafe fn list_push_unchecked(self, data: &mut VecViewMut<'_, u8>) {
        unsafe { IntoOwnedValue::<O>::list_push_unchecked(self as i32, data) }
    }
    #[inline]
    fn list_insert(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        IntoOwnedValue::<O>::list_insert(self as i32, data, index)
    }
    #[inline]
    unsafe fn list_insert_unchecked(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        unsafe { IntoOwnedValue::<O>::list_insert_unchecked(self as i32, data, index) }
    }
}

impl<O: ByteOrder> IntoOwnedValue<O> for i32 {
    #[inline]
    fn compound_insert(self, data: &mut VecViewMut<'_, u8>, key: &str) -> Option<OwnedValue<O>> {
        byteorder::I32::<O>::from(self).compound_insert(data, key)
    }
    #[inline]
    fn list_push(self, data: &mut VecViewMut<'_, u8>) {
        byteorder::I32::<O>::from(self).list_push(data)
    }
    #[inline]
    unsafe fn list_push_unchecked(self, data: &mut VecViewMut<'_, u8>) {
        unsafe { byteorder::I32::<O>::from(self).list_push_unchecked(data) }
    }
    #[inline]
    fn list_insert(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        byteorder::I32::<O>::from(self).list_insert(data, index)
    }
    #[inline]
    unsafe fn list_insert_unchecked(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        unsafe { byteorder::I32::<O>::from(self).list_insert_unchecked(data, index) }
    }
}

impl<O: ByteOrder> IntoOwnedValue<O> for byteorder::I64<O> {
    #[inline]
    fn compound_insert(self, data: &mut VecViewMut<'_, u8>, key: &str) -> Option<OwnedValue<O>> {
        compound_insert_long(data, key, self)
    }
    #[inline]
    fn list_push(self, data: &mut VecViewMut<'_, u8>) {
        list_push_long::<O>(data, self);
    }
    #[inline]
    unsafe fn list_push_unchecked(self, data: &mut VecViewMut<'_, u8>) {
        unsafe { list_push_long_unchecked::<O>(data, self) };
    }
    #[inline]
    fn list_insert(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        list_insert_long::<O>(data, index, self);
    }
    #[inline]
    unsafe fn list_insert_unchecked(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        unsafe { list_insert_long_unchecked::<O>(data, index, self) };
    }
}

impl<O: ByteOrder> IntoOwnedValue<O> for u32 {
    #[inline]
    fn compound_insert(self, data: &mut VecViewMut<'_, u8>, key: &str) -> Option<OwnedValue<O>> {
        (self as i64).compound_insert(data, key)
    }
    #[inline]
    fn list_push(self, data: &mut VecViewMut<'_, u8>) {
        IntoOwnedValue::<O>::list_push(self as i64, data)
    }
    #[inline]
    unsafe fn list_push_unchecked(self, data: &mut VecViewMut<'_, u8>) {
        unsafe { IntoOwnedValue::<O>::list_push_unchecked(self as i64, data) }
    }
    #[inline]
    fn list_insert(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        IntoOwnedValue::<O>::list_insert(self as i64, data, index)
    }
    #[inline]
    unsafe fn list_insert_unchecked(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        unsafe { IntoOwnedValue::<O>::list_insert_unchecked(self as i64, data, index) }
    }
}

impl<O: ByteOrder> IntoOwnedValue<O> for i64 {
    #[inline]
    fn compound_insert(self, data: &mut VecViewMut<'_, u8>, key: &str) -> Option<OwnedValue<O>> {
        byteorder::I64::<O>::from(self).compound_insert(data, key)
    }
    #[inline]
    fn list_push(self, data: &mut VecViewMut<'_, u8>) {
        byteorder::I64::<O>::from(self).list_push(data)
    }
    #[inline]
    unsafe fn list_push_unchecked(self, data: &mut VecViewMut<'_, u8>) {
        unsafe { byteorder::I64::<O>::from(self).list_push_unchecked(data) }
    }
    #[inline]
    fn list_insert(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        byteorder::I64::<O>::from(self).list_insert(data, index)
    }
    #[inline]
    unsafe fn list_insert_unchecked(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        unsafe { byteorder::I64::<O>::from(self).list_insert_unchecked(data, index) }
    }
}

impl<O: ByteOrder> IntoOwnedValue<O> for byteorder::F32<O> {
    #[inline]
    fn compound_insert(self, data: &mut VecViewMut<'_, u8>, key: &str) -> Option<OwnedValue<O>> {
        compound_insert_float(data, key, self)
    }
    #[inline]
    fn list_push(self, data: &mut VecViewMut<'_, u8>) {
        list_push_float::<O>(data, self);
    }
    #[inline]
    unsafe fn list_push_unchecked(self, data: &mut VecViewMut<'_, u8>) {
        unsafe { list_push_float_unchecked::<O>(data, self) };
    }
    #[inline]
    fn list_insert(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        list_insert_float::<O>(data, index, self);
    }
    #[inline]
    unsafe fn list_insert_unchecked(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        unsafe { list_insert_float_unchecked::<O>(data, index, self) };
    }
}

impl<O: ByteOrder> IntoOwnedValue<O> for f32 {
    #[inline]
    fn compound_insert(self, data: &mut VecViewMut<'_, u8>, key: &str) -> Option<OwnedValue<O>> {
        byteorder::F32::<O>::from(self).compound_insert(data, key)
    }
    #[inline]
    fn list_push(self, data: &mut VecViewMut<'_, u8>) {
        byteorder::F32::<O>::from(self).list_push(data)
    }
    #[inline]
    unsafe fn list_push_unchecked(self, data: &mut VecViewMut<'_, u8>) {
        unsafe { byteorder::F32::<O>::from(self).list_push_unchecked(data) }
    }
    #[inline]
    fn list_insert(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        byteorder::F32::<O>::from(self).list_insert(data, index)
    }
    #[inline]
    unsafe fn list_insert_unchecked(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        unsafe { byteorder::F32::<O>::from(self).list_insert_unchecked(data, index) }
    }
}

impl<O: ByteOrder> IntoOwnedValue<O> for byteorder::F64<O> {
    #[inline]
    fn compound_insert(self, data: &mut VecViewMut<'_, u8>, key: &str) -> Option<OwnedValue<O>> {
        compound_insert_double(data, key, self)
    }
    #[inline]
    fn list_push(self, data: &mut VecViewMut<'_, u8>) {
        list_push_double::<O>(data, self);
    }
    #[inline]
    unsafe fn list_push_unchecked(self, data: &mut VecViewMut<'_, u8>) {
        unsafe { list_push_double_unchecked::<O>(data, self) };
    }
    #[inline]
    fn list_insert(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        list_insert_double::<O>(data, index, self);
    }
    #[inline]
    unsafe fn list_insert_unchecked(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        unsafe { list_insert_double_unchecked::<O>(data, index, self) };
    }
}

impl<O: ByteOrder> IntoOwnedValue<O> for f64 {
    #[inline]
    fn compound_insert(self, data: &mut VecViewMut<'_, u8>, key: &str) -> Option<OwnedValue<O>> {
        byteorder::F64::<O>::from(self).compound_insert(data, key)
    }
    #[inline]
    fn list_push(self, data: &mut VecViewMut<'_, u8>) {
        byteorder::F64::<O>::from(self).list_push(data)
    }
    #[inline]
    unsafe fn list_push_unchecked(self, data: &mut VecViewMut<'_, u8>) {
        unsafe { byteorder::F64::<O>::from(self).list_push_unchecked(data) }
    }
    #[inline]
    fn list_insert(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        byteorder::F64::<O>::from(self).list_insert(data, index)
    }
    #[inline]
    unsafe fn list_insert_unchecked(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        unsafe { byteorder::F64::<O>::from(self).list_insert_unchecked(data, index) }
    }
}

impl<O: ByteOrder> IntoOwnedValue<O> for &[i8] {
    #[inline]
    fn compound_insert(self, data: &mut VecViewMut<'_, u8>, key: &str) -> Option<OwnedValue<O>> {
        compound_insert_byte_array(data, key, self.into())
    }
    #[inline]
    fn list_push(self, data: &mut VecViewMut<'_, u8>) {
        list_push_byte_array::<O>(data, self.into());
    }
    #[inline]
    unsafe fn list_push_unchecked(self, data: &mut VecViewMut<'_, u8>) {
        unsafe { list_push_byte_array_unchecked::<O>(data, self.into()) };
    }
    #[inline]
    fn list_insert(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        list_insert_byte_array::<O>(data, index, self.into());
    }
    #[inline]
    unsafe fn list_insert_unchecked(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        unsafe { list_insert_byte_array_unchecked::<O>(data, index, self.into()) };
    }
}

impl<O: ByteOrder, const N: usize> IntoOwnedValue<O> for [i8; N] {
    #[inline]
    fn compound_insert(self, data: &mut VecViewMut<'_, u8>, key: &str) -> Option<OwnedValue<O>> {
        self.to_vec().compound_insert(data, key)
    }
    #[inline]
    fn list_push(self, data: &mut VecViewMut<'_, u8>) {
        IntoOwnedValue::<O>::list_push(self.to_vec(), data)
    }
    #[inline]
    unsafe fn list_push_unchecked(self, data: &mut VecViewMut<'_, u8>) {
        unsafe { IntoOwnedValue::<O>::list_push_unchecked(self.to_vec(), data) }
    }
    #[inline]
    fn list_insert(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        IntoOwnedValue::<O>::list_insert(self.to_vec(), data, index)
    }
    #[inline]
    unsafe fn list_insert_unchecked(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        unsafe { IntoOwnedValue::<O>::list_insert_unchecked(self.to_vec(), data, index) }
    }
}

impl<O: ByteOrder> IntoOwnedValue<O> for Vec<i8> {
    #[inline]
    fn compound_insert(self, data: &mut VecViewMut<'_, u8>, key: &str) -> Option<OwnedValue<O>> {
        compound_insert_byte_array(data, key, self.into())
    }
    #[inline]
    fn list_push(self, data: &mut VecViewMut<'_, u8>) {
        list_push_byte_array::<O>(data, self.into());
    }
    #[inline]
    unsafe fn list_push_unchecked(self, data: &mut VecViewMut<'_, u8>) {
        unsafe { list_push_byte_array_unchecked::<O>(data, self.into()) };
    }
    #[inline]
    fn list_insert(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        list_insert_byte_array::<O>(data, index, self.into());
    }
    #[inline]
    unsafe fn list_insert_unchecked(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        unsafe { list_insert_byte_array_unchecked::<O>(data, index, self.into()) };
    }
}

impl<O: ByteOrder> IntoOwnedValue<O> for &str {
    #[inline]
    fn compound_insert(self, data: &mut VecViewMut<'_, u8>, key: &str) -> Option<OwnedValue<O>> {
        compound_insert_string(data, key, self.into())
    }
    #[inline]
    fn list_push(self, data: &mut VecViewMut<'_, u8>) {
        list_push_string::<O>(data, self.into());
    }
    #[inline]
    unsafe fn list_push_unchecked(self, data: &mut VecViewMut<'_, u8>) {
        unsafe { list_push_string_unchecked::<O>(data, self.into()) };
    }
    #[inline]
    fn list_insert(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        list_insert_string::<O>(data, index, self.into());
    }
    #[inline]
    unsafe fn list_insert_unchecked(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        unsafe { list_insert_string_unchecked::<O>(data, index, self.into()) };
    }
}

impl<O: ByteOrder> IntoOwnedValue<O> for String {
    #[inline]
    fn compound_insert(self, data: &mut VecViewMut<'_, u8>, key: &str) -> Option<OwnedValue<O>> {
        compound_insert_string(data, key, self.into())
    }
    #[inline]
    fn list_push(self, data: &mut VecViewMut<'_, u8>) {
        list_push_string::<O>(data, self.into());
    }
    #[inline]
    unsafe fn list_push_unchecked(self, data: &mut VecViewMut<'_, u8>) {
        unsafe { list_push_string_unchecked::<O>(data, self.into()) };
    }
    #[inline]
    fn list_insert(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        list_insert_string::<O>(data, index, self.into());
    }
    #[inline]
    unsafe fn list_insert_unchecked(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        unsafe { list_insert_string_unchecked::<O>(data, index, self.into()) };
    }
}

impl<O: ByteOrder> IntoOwnedValue<O> for OwnedList<O> {
    #[inline]
    fn compound_insert(self, data: &mut VecViewMut<'_, u8>, key: &str) -> Option<OwnedValue<O>> {
        compound_insert_list(data, key, self)
    }
    #[inline]
    fn list_push(self, data: &mut VecViewMut<'_, u8>) {
        list_push_list::<O>(data, self);
    }
    #[inline]
    unsafe fn list_push_unchecked(self, data: &mut VecViewMut<'_, u8>) {
        unsafe { list_push_list_unchecked::<O>(data, self) };
    }
    #[inline]
    fn list_insert(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        list_insert_list::<O>(data, index, self);
    }
    #[inline]
    unsafe fn list_insert_unchecked(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        unsafe { list_insert_list_unchecked::<O>(data, index, self) };
    }
}

impl<O: ByteOrder> IntoOwnedValue<O> for OwnedCompound<O> {
    #[inline]
    fn compound_insert(self, data: &mut VecViewMut<'_, u8>, key: &str) -> Option<OwnedValue<O>> {
        compound_insert_compound(data, key, self)
    }
    #[inline]
    fn list_push(self, data: &mut VecViewMut<'_, u8>) {
        list_push_compound::<O>(data, self);
    }
    #[inline]
    unsafe fn list_push_unchecked(self, data: &mut VecViewMut<'_, u8>) {
        unsafe { list_push_compound_unchecked::<O>(data, self) };
    }
    #[inline]
    fn list_insert(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        list_insert_compound::<O>(data, index, self);
    }
    #[inline]
    unsafe fn list_insert_unchecked(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        unsafe { list_insert_compound_unchecked::<O>(data, index, self) };
    }
}

impl<O: ByteOrder> IntoOwnedValue<O> for &[byteorder::I32<O>] {
    #[inline]
    fn compound_insert(self, data: &mut VecViewMut<'_, u8>, key: &str) -> Option<OwnedValue<O>> {
        compound_insert_int_array(data, key, self.into())
    }
    #[inline]
    fn list_push(self, data: &mut VecViewMut<'_, u8>) {
        list_push_int_array::<O>(data, self.into());
    }
    #[inline]
    unsafe fn list_push_unchecked(self, data: &mut VecViewMut<'_, u8>) {
        unsafe { list_push_int_array_unchecked::<O>(data, self.into()) };
    }
    #[inline]
    fn list_insert(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        list_insert_int_array::<O>(data, index, self.into());
    }
    #[inline]
    unsafe fn list_insert_unchecked(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        unsafe { list_insert_int_array_unchecked::<O>(data, index, self.into()) };
    }
}

impl<O: ByteOrder, const N: usize> IntoOwnedValue<O> for [byteorder::I32<O>; N] {
    #[inline]
    fn compound_insert(self, data: &mut VecViewMut<'_, u8>, key: &str) -> Option<OwnedValue<O>> {
        self.to_vec().compound_insert(data, key)
    }
    #[inline]
    fn list_push(self, data: &mut VecViewMut<'_, u8>) {
        self.to_vec().list_push(data)
    }
    #[inline]
    unsafe fn list_push_unchecked(self, data: &mut VecViewMut<'_, u8>) {
        unsafe { self.to_vec().list_push_unchecked(data) }
    }
    #[inline]
    fn list_insert(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        self.to_vec().list_insert(data, index)
    }
    #[inline]
    unsafe fn list_insert_unchecked(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        unsafe { self.to_vec().list_insert_unchecked(data, index) }
    }
}

impl<O: ByteOrder> IntoOwnedValue<O> for Vec<byteorder::I32<O>> {
    #[inline]
    fn compound_insert(self, data: &mut VecViewMut<'_, u8>, key: &str) -> Option<OwnedValue<O>> {
        compound_insert_int_array(data, key, self.into())
    }
    #[inline]
    fn list_push(self, data: &mut VecViewMut<'_, u8>) {
        list_push_int_array::<O>(data, self.into());
    }
    #[inline]
    unsafe fn list_push_unchecked(self, data: &mut VecViewMut<'_, u8>) {
        unsafe { list_push_int_array_unchecked::<O>(data, self.into()) };
    }
    #[inline]
    fn list_insert(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        list_insert_int_array::<O>(data, index, self.into());
    }
    #[inline]
    unsafe fn list_insert_unchecked(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        unsafe { list_insert_int_array_unchecked::<O>(data, index, self.into()) };
    }
}

impl<O: ByteOrder> IntoOwnedValue<O> for &[byteorder::I64<O>] {
    #[inline]
    fn compound_insert(self, data: &mut VecViewMut<'_, u8>, key: &str) -> Option<OwnedValue<O>> {
        compound_insert_long_array(data, key, self.into())
    }
    #[inline]
    fn list_push(self, data: &mut VecViewMut<'_, u8>) {
        list_push_long_array::<O>(data, self.into());
    }
    #[inline]
    unsafe fn list_push_unchecked(self, data: &mut VecViewMut<'_, u8>) {
        unsafe { list_push_long_array_unchecked::<O>(data, self.into()) };
    }
    #[inline]
    fn list_insert(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        list_insert_long_array::<O>(data, index, self.into());
    }
    #[inline]
    unsafe fn list_insert_unchecked(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        unsafe { list_insert_long_array_unchecked::<O>(data, index, self.into()) };
    }
}

impl<O: ByteOrder, const N: usize> IntoOwnedValue<O> for [byteorder::I64<O>; N] {
    #[inline]
    fn compound_insert(self, data: &mut VecViewMut<'_, u8>, key: &str) -> Option<OwnedValue<O>> {
        self.to_vec().compound_insert(data, key)
    }
    #[inline]
    fn list_push(self, data: &mut VecViewMut<'_, u8>) {
        self.to_vec().list_push(data)
    }
    #[inline]
    unsafe fn list_push_unchecked(self, data: &mut VecViewMut<'_, u8>) {
        unsafe { self.to_vec().list_push_unchecked(data) }
    }
    #[inline]
    fn list_insert(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        self.to_vec().list_insert(data, index)
    }
    #[inline]
    unsafe fn list_insert_unchecked(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        unsafe { self.to_vec().list_insert_unchecked(data, index) }
    }
}

impl<O: ByteOrder> IntoOwnedValue<O> for Vec<byteorder::I64<O>> {
    #[inline]
    fn compound_insert(self, data: &mut VecViewMut<'_, u8>, key: &str) -> Option<OwnedValue<O>> {
        compound_insert_long_array(data, key, self.into())
    }
    #[inline]
    fn list_push(self, data: &mut VecViewMut<'_, u8>) {
        list_push_long_array::<O>(data, self.into());
    }
    #[inline]
    unsafe fn list_push_unchecked(self, data: &mut VecViewMut<'_, u8>) {
        unsafe { list_push_long_array_unchecked::<O>(data, self.into()) };
    }
    #[inline]
    fn list_insert(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        list_insert_long_array::<O>(data, index, self.into());
    }
    #[inline]
    unsafe fn list_insert_unchecked(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        unsafe { list_insert_long_array_unchecked::<O>(data, index, self.into()) };
    }
}

impl<O: ByteOrder> IntoOwnedValue<O> for OwnedValue<O> {
    #[inline]
    fn compound_insert(self, data: &mut VecViewMut<'_, u8>, key: &str) -> Option<OwnedValue<O>> {
        compound_insert_value(data, key, self)
    }
    #[inline]
    fn list_push(self, data: &mut VecViewMut<'_, u8>) {
        list_push_value::<O>(data, self);
    }
    #[inline]
    unsafe fn list_push_unchecked(self, data: &mut VecViewMut<'_, u8>) {
        unsafe { list_push_value_unchecked::<O>(data, self) };
    }
    #[inline]
    fn list_insert(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        list_insert_value::<O>(data, index, self);
    }
    #[inline]
    unsafe fn list_insert_unchecked(self, data: &mut VecViewMut<'_, u8>, index: usize) {
        unsafe { list_insert_value_unchecked::<O>(data, index, self) };
    }
}
