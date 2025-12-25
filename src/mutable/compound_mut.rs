use std::{marker::PhantomData, ptr, slice};

use zerocopy::byteorder;

use crate::{
    ByteOrder, CompoundBase, ConfigMut, ConfigRef, MutValue, MutVec, MutableConfig, RefString,
    TagID, cold_path, mutable_tag_size,
};

pub struct MutCompound<'s, O: ByteOrder> {
    pub(crate) data: MutVec<'s, u8>,
    pub(crate) _marker: PhantomData<O>,
}

// impl<'s, O: ByteOrder> IntoIterator for MutCompound<'s, O> {
//     type Item = (RefString<'s>, MutValue<'s, O>);
//     type IntoIter = MutCompoundIter<'s, O>;

//     fn into_iter(mut self) -> Self::IntoIter {
//         MutCompoundIter {
//             data: self.data.as_mut_ptr(),
//             _marker: PhantomData,
//         }
//     }
// }

impl<'s, O: ByteOrder> MutCompound<'s, O> {}

// impl<'s, O: ByteOrder> CompoundBase<'s> for MutCompound<'s, O> {
//     type ConfigRef = MutableConfig<O>;

//     fn compound_get_impl<'a>(
//         &'a self,
//         key: &str,
//     ) -> Option<(TagID, <Self::ConfigRef as ConfigRef>::ReadParams<'a>)> {
//         let name = simd_cesu8::mutf8::encode(key);

//         unsafe {
//             let mut ptr = self.data.as_ptr();
//             loop {
//                 let tag_id = *ptr.cast();
//                 ptr = ptr.add(1);

//                 if tag_id == TagID::End {
//                     cold_path();
//                     return None;
//                 }

//                 let name_len = byteorder::U16::<O>::from_bytes(*ptr.cast()).get();
//                 ptr = ptr.add(2);

//                 let name_bytes = slice::from_raw_parts(ptr, name_len as usize);
//                 ptr = ptr.add(name_len as usize);

//                 if name == name_bytes {
//                     return Some((tag_id, ptr));
//                 }

//                 ptr = ptr.add(mutable_tag_size(tag_id));
//             }
//         }
//     }
// }

pub struct MutCompoundIter<'s, O: ByteOrder> {
    data: *mut u8,
    _marker: PhantomData<(&'s (), O)>,
}

impl<'s, O: ByteOrder> Default for MutCompoundIter<'s, O> {
    #[inline]
    fn default() -> Self {
        Self {
            data: ptr::null_mut(),
            _marker: PhantomData,
        }
    }
}

unsafe impl<'s, O: ByteOrder> Send for MutCompoundIter<'s, O> {}
unsafe impl<'s, O: ByteOrder> Sync for MutCompoundIter<'s, O> {}

// impl<'s, O: ByteOrder> Iterator for MutCompoundIter<'s, O> {
//     type Item = (RefString<'s>, MutValue<'s, O>);

//     fn next(&mut self) -> Option<Self::Item> {
//         unsafe {
//             let tag_id = *self.data.cast();

//             if tag_id == TagID::End {
//                 cold_path();
//                 return None;
//             }

//             let name_len = byteorder::U16::<O>::from_bytes(*self.data.add(1).cast()).get();
//             let name = RefString {
//                 data: slice::from_raw_parts(self.data.add(3), name_len as usize),
//             };

//             let value = <MutableConfig<O> as ConfigMut>::read_value_mut(
//                 tag_id,
//                 self.data.add(3 + name_len as usize),
//             );

//             self.data = self
//                 .data
//                 .add(3 + name_len as usize + mutable_tag_size(tag_id));

//             Some((name, value))
//         }
//     }
// }
