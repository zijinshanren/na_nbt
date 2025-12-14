use na_nbt::{
    ScopedReadableValue, ValueScoped, read_borrowed, write_value_to_vec, write_value_to_writer,
};
use zerocopy::BE;

fn read_write(data: &[u8]) {
    let doc = read_borrowed::<BE>(data).unwrap();
    let root = doc.root();
    let written = write_value_to_vec::<_, BE, BE>(&root).unwrap();
    assert_eq!(written, data);

    let mut vec = Vec::new();
    write_value_to_writer::<_, BE, BE, _>(&mut vec, &root).unwrap();
    assert_eq!(vec, data);
}

#[test]
fn test_read_write() {
    let doc = read_borrowed::<BE>(&[0]).unwrap();
    doc.root().visit_scoped(|v| matches!(v, ValueScoped::End));

    read_write(&[0]);
    read_write(&[11, 0, 0, 0, 0, 0, 2, 1, 2, 3, 4, 5, 6, 7, 8]);
    read_write(&[
        12, 0, 0, 0, 0, 0, 2, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
    ]);
    read_write(&[4, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8]);
    read_write(&[5, 0, 0, 1, 2, 3, 4]);
    read_write(&[6, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8]);
    read_write(&[7, 0, 0, 0, 0, 0, 4, 1, 2, 3, 4]);
    read_write(&[8, 0, 0, 0, 4, 1, 2, 3, 4]);
}
