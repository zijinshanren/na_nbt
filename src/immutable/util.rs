macro_rules! match_tag_id_expand {
    ($macro:ident, $tag_id:expr, $($param:expr),*) => {
        $macro!(
            [End, Byte, Short, Int, Long, Float, Double, ByteArray, String, List, Compound, IntArray, LongArray],
            $tag_id, $($param),*
        )
    };
}
