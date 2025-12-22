macro_rules! match_tag_id_expand {
    ($macro:ident, $tag_id:expr, $($param:expr),*) => {
        $macro!(
            [(End, TagEnd),
            (Byte, TagByte),
            (Short, TagShort),
            (Int, TagInt),
            (Long, TagLong),
            (Float, TagFloat),
            (Double, TagDouble),
            (ByteArray, TagByteArray),
            (String, TagString),
            (List, TagList),
            (Compound, TagCompound),
            (IntArray, TagIntArray),
            (LongArray, TagLongArray)],
            $tag_id, $($param),*
        )
    };
}
