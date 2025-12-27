use na_nbt::{
    BigEndian, BorrowedValue, LittleEndian, OwnValue, RefCompound, RefList, RefValue, SharedValue,
    ValueBase, ValueRef,
    tag::{Compound, End, Int, IntArray, List, TypedList},
};

#[test]
fn default() {
    assert!(SharedValue::<LittleEndian>::default().is_::<End>());
    assert!(SharedValue::<BigEndian>::default().is_::<End>());
    assert!(BorrowedValue::<LittleEndian>::default().is_::<End>());
    assert!(BorrowedValue::<BigEndian>::default().is_::<End>());
    assert!(RefValue::<LittleEndian>::default().is_::<End>());
    assert!(RefValue::<BigEndian>::default().is_::<End>());
    assert!(OwnValue::<LittleEndian>::default().is_::<End>());
    assert!(OwnValue::<BigEndian>::default().is_::<End>());

    for _ in SharedValue::<LittleEndian>::default()
        .into_::<List>()
        .unwrap_or_default()
        .into_iter()
    {
        panic!("List is not empty");
    }

    for _ in SharedValue::<BigEndian>::default()
        .into_::<List>()
        .unwrap_or_default()
        .into_iter()
    {
        panic!("List is not empty");
    }

    for _ in SharedValue::<LittleEndian>::default()
        .into_::<TypedList<Int>>()
        .unwrap_or_default()
        .into_iter()
    {
        panic!("List is not empty");
    }

    for _ in SharedValue::<BigEndian>::default()
        .into_::<TypedList<Int>>()
        .unwrap_or_default()
        .into_iter()
    {
        panic!("List is not empty");
    }

    for _ in SharedValue::<LittleEndian>::default()
        .into_::<IntArray>()
        .unwrap_or_default()
        .into_iter()
    {
        panic!("IntArray is not empty");
    }

    for _ in SharedValue::<BigEndian>::default()
        .into_::<IntArray>()
        .unwrap_or_default()
        .into_iter()
    {
        panic!("IntArray is not empty");
    }

    for _ in SharedValue::<LittleEndian>::default()
        .into_::<Compound>()
        .unwrap_or_default()
        .into_iter()
    {
        panic!("Compound is not empty");
    }

    for _ in SharedValue::<BigEndian>::default()
        .into_::<Compound>()
        .unwrap_or_default()
        .into_iter()
    {
        panic!("Compound is not empty");
    }

    for _ in RefList::<LittleEndian>::default().into_iter() {
        panic!("List is not empty");
    }

    for _ in RefList::<BigEndian>::default().into_iter() {
        panic!("List is not empty");
    }

    for _ in RefCompound::<LittleEndian>::default().into_iter() {
        panic!("Compound is not empty");
    }

    for _ in RefCompound::<BigEndian>::default().into_iter() {
        panic!("Compound is not empty");
    }
}
