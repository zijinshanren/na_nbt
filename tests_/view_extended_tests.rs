use na_nbt::OwnedValue;
use zerocopy::byteorder::BigEndian as BE;

#[test]
fn test_vec_view_own_extended() {
    // Construct via OwnedValue
    let val = OwnedValue::<BE>::from(vec![1i8, 2, 3]);
    let mut v = if let OwnedValue::ByteArray(v) = val { v } else { panic!() };
    
    // reserve_exact
    v.reserve_exact(10);
    assert!(v.capacity() >= 13);
    
    // shrink_to
    v.shrink_to(4);
    assert!(v.capacity() >= 4); // Exact behavior depends on allocator, but >= 4
    
    // pop_if
    assert_eq!(v.pop_if(|&mut x| x == 3), Some(3));
    assert_eq!(v.pop_if(|&mut x| x == 100), None);
    assert_eq!(v.len(), 2); // [1, 2]
    
    // dedup
    v.push(2);
    v.push(3);
    v.push(3); // [1, 2, 2, 3, 3]
    v.dedup();
    assert_eq!(v.as_slice(), &[1, 2, 3]);
    
    // dedup_by
    v.push(4); 
    v.push(6); // [1, 2, 3, 4, 6]
    v.dedup_by(|a, b| *a % 2 == *b % 2); // 4 and 6 are both even
    assert_eq!(v.as_slice(), &[1, 2, 3, 4]); // 6 removed
    
    // dedup_by_key
    v.push(5); // [1, 2, 3, 4, 5]
    v.dedup_by_key(|a| *a / 2); // 2/2=1, 3/2=1 (dedup 3); 4/2=2, 5/2=2 (dedup 5)
    // 1(0), 2(1), 3(1) -> 2 kept; 4(2), 5(2) -> 4 kept
    // [1, 2, 4]
    assert_eq!(v.as_slice(), &[1, 2, 4]);
    
    // extend_from_within
    v.extend_from_within(0..2); // append 1, 2
    assert_eq!(v.as_slice(), &[1, 2, 4, 1, 2]);
    
    // resize
    v.resize(7, 0);
    assert_eq!(v.as_slice(), &[1, 2, 4, 1, 2, 0, 0]);
    
    // resize_with
    v.resize_with(9, || 9);
    assert_eq!(v.as_slice(), &[1, 2, 4, 1, 2, 0, 0, 9, 9]);
    
    // split_off
    let tail = v.split_off(7);
    assert_eq!(v.as_slice(), &[1, 2, 4, 1, 2, 0, 0]);
    assert_eq!(tail, vec![9, 9]);
    
    // splice_drop
    v.splice_drop(5..7, vec![8, 8]);
    assert_eq!(v.as_slice(), &[1, 2, 4, 1, 2, 8, 8]);
    
    // append
    let mut other = vec![10i8];
    v.append(&mut other);
    assert_eq!(v.as_slice(), &[1, 2, 4, 1, 2, 8, 8, 10]);
    
    // append_view
    let other_val = OwnedValue::<BE>::from(vec![11i8]);
    let mut other_view = if let OwnedValue::ByteArray(v) = other_val { v } else { panic!() };
    v.append_view(&mut other_view);
    assert_eq!(v.as_slice(), &[1, 2, 4, 1, 2, 8, 8, 10, 11]);
    assert!(other_view.is_empty());
    
    // drain_drop
    v.drain_drop(7..9);
    assert_eq!(v.as_slice(), &[1, 2, 4, 1, 2, 8, 8]);
    
    // retain_mut
    v.retain_mut(|x| { *x += 1; *x % 2 == 0 }); // increment, keep even
    // 1->2(keep), 2->3(drop), 4->5(drop), 1->2(keep), 2->3(drop), 8->9(drop), 8->9(drop)
    // [2, 2]
    assert_eq!(v.as_slice(), &[2, 2]);
}

#[test]
fn test_string_view_own_extended() {
    let val = OwnedValue::<BE>::from("hello");
    let mut s = if let OwnedValue::String(s) = val { s } else { panic!() };
    
    // reserve_exact
    s.reserve_exact(10);
    assert!(s.capacity() >= 15);
    
    // shrink_to
    s.shrink_to(5);
    assert!(s.capacity() >= 5);
    
    // extend_from_within
    s.extend_from_within(0..2); // "he"
    assert_eq!(s.decode(), "hellohe");
    
    // replace_range
    s.replace_range(5..7, " world");
    assert_eq!(s.decode(), "hello world");
    
    // retain
    s.retain(|c| c != 'l');
    assert_eq!(s.decode(), "heo word");
}

#[test]
fn test_vec_view_mut_extended() {
    let vec = vec![1i8, 2, 3];
    let mut val = OwnedValue::<BE>::from(vec);
    let mut v = val.as_byte_array_mut().unwrap();
    
    // reserve_exact
    v.reserve_exact(10);
    assert!(v.capacity() >= 13);
    
    // shrink_to
    v.shrink_to(4);
    assert!(v.capacity() >= 4);
    
    // pop_if
    assert_eq!(v.pop_if(|x| *x == 3), Some(3));
    assert_eq!(v.len(), 2);
    
    // dedup
    v.push(2); // [1, 2, 2]
    v.dedup();
    assert_eq!(v.as_slice(), &[1, 2]);
    
    // dedup_by
    v.push(4); // [1, 2, 4] even
    v.push(6); // [1, 2, 4, 6] even
    v.dedup_by(|a, b| *a % 2 == *b % 2);
    // 2, 4, 6 are all even, so they are considered equal. Only the first (2) is kept.
    assert_eq!(v.as_slice(), &[1, 2]); 
    
    // dedup_by_key
    v.push(5); // [1, 2, 4, 5]
    v.dedup_by_key(|a| *a % 2);
    // 1(1), 2(0), 4(0) -> drop 4, 5(1)
    // [1, 2, 5]
    assert_eq!(v.as_slice(), &[1, 2, 5]);
    
    // extend_from_within
    v.extend_from_within(0..2); // 1, 2
    assert_eq!(v.as_slice(), &[1, 2, 5, 1, 2]);
    
    // resize
    v.resize(7, 9);
    assert_eq!(v.as_slice(), &[1, 2, 5, 1, 2, 9, 9]);
    
    // resize_with
    v.resize_with(8, || 0);
    assert_eq!(v.as_slice(), &[1, 2, 5, 1, 2, 9, 9, 0]);
    
    // split_off
    let tail = v.split_off(6);
    assert_eq!(v.as_slice(), &[1, 2, 5, 1, 2, 9]);
    assert_eq!(tail, vec![9, 0]);
    
    // splice_drop
    v.splice_drop(4..6, vec![3, 3]);
    assert_eq!(v.as_slice(), &[1, 2, 5, 1, 3, 3]);
    
    // append
    let mut other = vec![10i8];
    v.append(&mut other);
    assert_eq!(v.as_slice(), &[1, 2, 5, 1, 3, 3, 10]);
    
    // append_view
    // Need unsafe to construct a separate VecViewMut from a separate vec to pass to append_view
    // Or simpler: assume append_view works if append works, as it just wraps logic.
    // But we can test it using another OwnedValue? No, OwnedValue consumes the vec.
    // We can use Unsafe creation.
    
    let other_vec = vec![11i8];
    let mut other_val = OwnedValue::<BE>::from(other_vec);
    // as_byte_array_mut borrows val mutably. We can't have both mutable at same time?
    // v borrows val. We need another val.
    
    // v is borrowing 'val'. We create 'other_val'. They are independent.
    // But 'v' is a reference. 
    // We need to pass `&mut VecViewMut`. 
    // We can't easily do this because `as_byte_array_mut` returns `Option<&mut VecViewMut>`.
    // We need to hold two mutable borrows to two different OwnedValues? Yes that works.
    
    {
        let mut other_view_ref = other_val.as_byte_array_mut().unwrap();
        v.append_view(&mut other_view_ref);
    }
    assert_eq!(v.as_slice(), &[1, 2, 5, 1, 3, 3, 10, 11]);
    
    // drain_drop
    v.drain_drop(6..8);
    assert_eq!(v.as_slice(), &[1, 2, 5, 1, 3, 3]);
    
    // retain_mut
    v.retain_mut(|x| { *x *= 2; *x < 10 });
    // 1->2, 2->4, 5->10(drop), 1->2, 3->6, 3->6
    // [2, 4, 2, 6, 6]
    assert_eq!(v.as_slice(), &[2, 4, 2, 6, 6]);
}

#[test]
fn test_string_view_mut_extended() {
    let s = String::from("hello");
    let mut val = OwnedValue::<BE>::from(s);
    let mut v = val.as_string_mut().unwrap();
    
    // reserve_exact
    v.reserve_exact(10);
    assert!(v.capacity() >= 15);
    
    // shrink_to
    v.shrink_to(5);
    assert!(v.capacity() >= 5);
    
    // extend_from_within
    v.extend_from_within(0..2);
    assert_eq!(v.to_string(), "hellohe");
    
    // replace_range
    v.replace_range(5..7, " world");
    assert_eq!(v.to_string(), "hello world");
    
    // retain
    v.retain(|c| c != 'l');
    assert_eq!(v.to_string(), "heo word");
}

