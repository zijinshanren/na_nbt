use std::ptr;

use crate::{
    ByteOrder, Result, Tag, cold_path,
    implementation::immutable::{mark::Mark, util::tag_size},
};
