use std::slice;

use zerocopy::byteorder;

use crate::{ByteOrder, NBT, TagID, cold_path, mutable_tag_size};

pub unsafe fn compound_get<O: ByteOrder, F, R>(data: *const u8, key: &str, map: F) -> Option<R>
where
    F: FnOnce(TagID, *const u8) -> Option<R>,
{
    let name = simd_cesu8::mutf8::encode(key);

    unsafe {
        let mut ptr = data;
        loop {
            let tag_id = *ptr.cast();
            ptr = ptr.add(1);

            if tag_id == TagID::End {
                cold_path();
                return None;
            }

            let name_len = byteorder::U16::<O>::from_bytes(*ptr.cast()).get();
            ptr = ptr.add(2);

            let name_bytes = slice::from_raw_parts(ptr, name_len as usize);
            ptr = ptr.add(name_len as usize);

            if name == name_bytes {
                return map(tag_id, ptr);
            }

            ptr = ptr.add(mutable_tag_size(tag_id));
        }
    }
}
