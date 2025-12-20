use afl::fuzz;

fn main() {
    fuzz!(|data: &[u8]| {
        na_nbt_fuzz::test(data);
    });
}
