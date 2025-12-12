use std::ptr;

use crate::{
    ByteOrder, Result, Tag, cold_path,
    implementation::immutable::{mark::Mark, util::tag_size},
};

pub unsafe fn write_unsafe<O: ByteOrder>(
    tag_id: Tag,
    payload: *const u8,
    mark: *const Mark,
) -> Result<Vec<u8>> {
    unsafe {
        if tag_id == Tag::End {
            cold_path();
            return Ok(vec![0]);
        }

        let size = tag_size::<O>(tag_id, payload, mark).0;
        let mut buf = Vec::<u8>::with_capacity(1 + 2 + size);
        let buf_ptr = buf.as_mut_ptr();
        ptr::write(buf_ptr.cast(), tag_id);
        ptr::write(buf_ptr.add(1).cast(), [0u8, 0u8]);
        ptr::copy_nonoverlapping(payload, buf_ptr.add(3).cast(), size);
        buf.set_len(1 + 2 + size);
        Ok(buf)
    }
}

pub unsafe fn write_unsafe_fallback<O: ByteOrder, R: ByteOrder>(
    tag_id: Tag,
    payload: *const u8,
    mark: *const Mark,
) -> Result<R> {
    todo!()
}
