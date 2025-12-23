macro_rules! match_tag_id_expand {
    ($macro:ident, $tag_id:expr, $($param:expr),*) => {
        $macro!(
            [(End, End),
            (Byte, Byte),
            (Short, Short),
            (Int, Int),
            (Long, Long),
            (Float, Float),
            (Double, Double),
            (ByteArray, ByteArray),
            (String, String),
            (List, List),
            (Compound, Compound),
            (IntArray, IntArray),
            (LongArray, LongArray)],
            $tag_id, $($param),*
        )
    };
}
