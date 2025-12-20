use honggfuzz::fuzz;

fn main() {
    loop {
        fuzz!(|data: &[u8]| {
            na_nbt_fuzz::test(data);
        });
    }
}
