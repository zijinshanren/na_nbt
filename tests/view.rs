use na_nbt::{OwnString, OwnVec};
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::{self, Write};
fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

#[test]
fn test_mut_vec_accessors() {
    let mut own = OwnVec::from(vec![1, 2, 3]);
    let mut view = own.to_mut();

    assert_eq!(view.len(), 3);
    assert!(!view.is_empty());
    assert!(view.capacity() >= 3);
    assert_eq!(view.as_ptr(), view.as_slice().as_ptr());
    assert_eq!(view.as_mut_ptr(), view.as_mut_slice().as_mut_ptr());
    assert_eq!(view.as_slice(), &[1, 2, 3]);
    assert_eq!(view.as_mut_slice(), &mut [1, 2, 3]);

    unsafe {
        view.set_len(2);
    }
    assert_eq!(view.len(), 2);
    assert_eq!(view.as_slice(), &[1, 2]);
}

#[test]
fn test_mut_vec_capacity() {
    let mut own = OwnVec::from(vec![1, 2, 3]);
    let mut view = own.to_mut();

    view.reserve(10);
    assert!(view.capacity() >= 13);

    view.reserve_exact(5);
    assert!(view.capacity() >= 13); // Already enough

    assert!(view.try_reserve(100).is_ok());
    assert!(view.try_reserve_exact(100).is_ok());

    view.shrink_to_fit();
    assert!(view.capacity() >= 3);

    view.shrink_to(10);
    view.reserve(5);
    let spare = view.spare_capacity_mut();
    assert!(!spare.is_empty());
}

#[test]
fn test_mut_vec_mutation() {
    let mut own = OwnVec::from(vec![1, 2, 3]);
    let mut view = own.to_mut();

    // push/pop
    view.push(4);
    assert_eq!(view.as_slice(), &[1, 2, 3, 4]);
    assert_eq!(view.pop(), Some(4));
    assert_eq!(view.pop_if(|x| *x == 3), Some(3));
    assert_eq!(view.pop_if(|x| *x == 100), None);
    assert_eq!(view.as_slice(), &[1, 2]);

    // insert/remove
    view.insert(0, 0);
    assert_eq!(view.as_slice(), &[0, 1, 2]);
    assert_eq!(view.remove(0), 0);
    assert_eq!(view.as_slice(), &[1, 2]);

    // swap_remove
    view.push(3);
    assert_eq!(view.swap_remove(0), 1);
    assert_eq!(view.as_slice(), &[3, 2]); // Order not preserved

    // retain
    view.clear();
    view.extend_from_slice(&[1, 2, 3, 4]);
    view.retain(|x| x % 2 == 0);
    assert_eq!(view.as_slice(), &[2, 4]);

    // retain_mut
    view.retain_mut(|x| {
        *x += 1;
        *x > 3
    });
    assert_eq!(view.as_slice(), &[5]);

    // truncate/clear
    view.push(6);
    view.truncate(1);
    assert_eq!(view.as_slice(), &[5]);
    view.clear();
    assert!(view.is_empty());
}

#[test]
fn test_mut_vec_advanced_mutation() {
    let mut own = OwnVec::from(vec![1, 2, 2, 3]);
    let mut view = own.to_mut();

    // dedup
    view.dedup();
    assert_eq!(view.as_slice(), &[1, 2, 3]);

    // dedup_by
    view.push(4);
    view.push(6);
    view.dedup_by(|a, b| *a % 2 == *b % 2);
    assert_eq!(view.as_slice(), &[1, 2, 3, 4]);

    // dedup_by_key
    view.push(4);
    view.dedup_by_key(|a| *a);
    assert_eq!(view.as_slice(), &[1, 2, 3, 4]);

    // extend variants
    view.extend(vec![5]);
    assert_eq!(view.as_slice(), &[1, 2, 3, 4, 5]);
    view.extend_from_slice(&[6]);
    assert_eq!(view.as_slice(), &[1, 2, 3, 4, 5, 6]);
    view.extend_from_within(0..2);
    assert_eq!(view.as_slice(), &[1, 2, 3, 4, 5, 6, 1, 2]);

    // resize
    view.resize(10, 0);
    assert_eq!(view.len(), 10);
    view.resize_with(12, || 1);
    assert_eq!(view.len(), 12);

    // splice_drop
    view.splice_drop(10..12, vec![9]);
    assert_eq!(view.last(), Some(&9));

    // append
    let mut other_vec = vec![10, 11];
    view.append(&mut other_vec);
    assert_eq!(view.len(), 13);

    // append_view
    let mut other_own = OwnVec::from(vec![12, 13]);
    let mut other_view = other_own.to_mut();
    view.append_view(&mut other_view);
    assert_eq!(view.len(), 15);
    assert!(other_view.is_empty());

    // split_off
    let split = view.split_off(13);
    assert_eq!(view.len(), 13);
    assert_eq!(split, vec![12, 13]);

    // drain_drop
    view.drain_drop(11..13);
    assert_eq!(view.len(), 11);
}

#[test]
fn test_mut_vec_traits() {
    let mut own = OwnVec::from(vec![1, 2, 3]);
    let mut view = own.to_mut();

    // Deref/Index
    assert_eq!(view[0], 1);
    view[0] = 10;
    assert_eq!(*view, [10, 2, 3]);

    // Debug
    assert_eq!(format!("{:?}", view), "[10, 2, 3]");

    // Eq/Ord/Hash
    let mut own2 = OwnVec::from(vec![10, 2, 3]);
    let view2 = own2.to_mut();
    assert_eq!(view, view2);
    assert_eq!(view.partial_cmp(&view2), Some(Ordering::Equal));
    assert_eq!(calculate_hash(&view), calculate_hash(&view2));

    // PartialEq with Vec/slice
    assert_eq!(view, vec![10, 2, 3]);
    assert_eq!(view, [10, 2, 3].as_slice());
    assert_eq!(view, [10, 2, 3]);
    assert_eq!(view, &mut [10, 2, 3][..]);

    // Borrow/AsRef
    let b: &[i32] = view.borrow();
    assert_eq!(b, &[10, 2, 3]);
    let r: &[i32] = view.as_ref();
    assert_eq!(r, &[10, 2, 3]);

    // Iter
    assert_eq!(view.iter().count(), 3);
    assert_eq!((&view).into_iter().count(), 3);
    assert_eq!((&mut view).into_iter().count(), 3);

    // Write (u8)
    let mut own_u8 = OwnVec::from(vec![1u8, 2, 3]);
    let mut view_u8 = own_u8.to_mut();
    view_u8.write_all(&[4, 5]).unwrap();
    assert_eq!(view_u8.as_slice(), &[1, 2, 3, 4, 5]);
    view_u8.write_vectored(&[io::IoSlice::new(&[6])]).unwrap();
    assert_eq!(view_u8.as_slice(), &[1, 2, 3, 4, 5, 6]);
    view_u8.flush().unwrap();
}

#[test]
fn test_mut_string_basics() {
    let mut own = OwnString::from("hello");
    let mut view = own.to_mut();

    assert_eq!(view.len(), 5);
    assert!(!view.is_empty());
    assert!(view.capacity() >= 5);
    assert_eq!(view.as_bytes(), b"hello");
    assert_eq!(view.as_ptr(), view.as_bytes().as_ptr());
    assert_eq!(view.as_mut_ptr(), view.as_mut_ptr()); // Just checking API existence
    
    assert_eq!(view.as_mutf8_str().as_bytes(), b"hello");
    assert_eq!(view.decode(), "hello");
}

#[test]
fn test_mut_string_capacity() {
    let mut own = OwnString::from("hello");
    let mut view = own.to_mut();

    view.reserve(10);
    assert!(view.capacity() >= 15);
    view.reserve_exact(5);
    view.try_reserve(10).unwrap();
    view.try_reserve_exact(10).unwrap();
    view.shrink_to_fit();
    view.shrink_to(5);
}

#[test]
fn test_mut_string_mutation() {
    let mut own = OwnString::from("hello");
    let mut view = own.to_mut();

    view.push_str(" world");
    assert_eq!(view.decode(), "hello world");
    
    view.push('!');
    assert_eq!(view.decode(), "hello world!");
    
    view.truncate(5);
    assert_eq!(view.decode(), "hello");
    
    assert_eq!(view.pop(), Some('o'));
    assert_eq!(view.decode(), "hell");
    
    assert_eq!(view.remove(0), 'h');
    assert_eq!(view.decode(), "ell");
    
    view.insert(0, 'H');
    assert_eq!(view.decode(), "Hell");
    
    view.insert_str(1, "i");
    assert_eq!(view.decode(), "Hiell");
    
    view.retain(|c| c != 'i');
    assert_eq!(view.decode(), "Hell");
    
    view.clear();
    assert!(view.is_empty());
}

#[test]
fn test_mut_string_advanced() {
    let mut own = OwnString::from("hello");
    let mut view = own.to_mut();

    let split = view.split_off(2);
    assert_eq!(view.decode(), "he");
    assert_eq!(split, "llo");

    view.extend_from_within(0..1);
    assert_eq!(view.decode(), "heh");
    
    view.replace_range(1..2, "a");
    assert_eq!(view.decode(), "hah");
    
    view.drain_drop(1..2);
    assert_eq!(view.decode(), "hh");
}

#[test]
fn test_mut_string_traits() {
    let mut own = OwnString::from("hello");
    let mut view = own.to_mut();

    // Debug/Display
    assert_eq!(format!("{:?}", view), "\"hello\"");
    assert_eq!(format!("{}", view), "hello");

    // Eq/Ord/Hash
    let mut own2 = OwnString::from("hello");
    let view2 = own2.to_mut();
    assert_eq!(view, view2);
    assert_eq!(view.partial_cmp(&view2), Some(Ordering::Equal));
    assert_eq!(calculate_hash(&view), calculate_hash(&view2));

    // PartialEq variants
    assert_eq!(view, String::from("hello"));
    assert_eq!(view, "hello");
    assert_eq!(view, "hello"); // &str

    // Write
    view.write_all(b" world").unwrap();
    assert_eq!(view.decode(), "hello world");
    
    assert!(view.write(b"\xFF").is_err()); // Invalid UTF-8
    assert!(view.write_all(b"\xFF").is_err());
    view.flush().unwrap();
}

#[test]
fn test_own_vec() {
    let mut own = OwnVec::from(vec![1, 2, 3]);
    let own2 = OwnVec::from(&[1, 2, 3][..]);
    assert_eq!(own, own2);
    
    assert_eq!(own.len(), 3);
    assert!(!own.is_empty());
    assert_eq!(own.as_slice(), &[1, 2, 3]);
    
    own.push(4);
    assert_eq!(own.len(), 4);
    
    // Test conversion
    let mut view = own.to_mut();
    view.push(5);
    
    // Check traits on OwnVec
    let mut own3 = OwnVec::default();
    assert!(own3.is_empty());
    own3.extend(vec![1]);
    own3.extend(&[2]);
    assert_eq!(own3.as_slice(), &[1, 2]);
    
    // Write
    let mut own_u8 = OwnVec::from(vec![1u8]);
    own_u8.write_all(&[2]).unwrap();
    assert_eq!(own_u8.as_slice(), &[1, 2]);

    // Traits
    let own_cmp = OwnVec::from(vec![1, 2, 3, 4, 5]);
    assert_eq!(own.partial_cmp(&own_cmp), Some(Ordering::Equal));
    assert_eq!(own.cmp(&own_cmp), Ordering::Equal);
    assert_eq!(calculate_hash(&own), calculate_hash(&own_cmp));
    
    let mut count = 0;
    for _ in &own { count += 1; }
    assert_eq!(count, 5);
    for _ in &mut own { count += 1; }
    assert_eq!(count, 10);
    
    // Extend ref
    let val = 100;
    own.extend(&[val]); // Extend<&T>
    assert_eq!(own.last(), Some(&100));
}

#[test]
fn test_own_string() {
    let mut own = OwnString::from("hello");
    let own2 = OwnString::from(String::from("hello"));
    assert_eq!(own, own2);
    assert_eq!(OwnString::default().len(), 0);
    
    assert_eq!(own.len(), 5);
    assert_eq!(own.decode(), "hello");
    
    own.push('!');
    assert_eq!(own.decode(), "hello!");
    
    let mut view = own.to_mut();
    view.push('?');
    
    let mut own3 = OwnString::from(vec![b'a', b'b']);
    assert_eq!(own3.decode(), "ab");
    
    own3.write_all(b"c").unwrap();
    assert_eq!(own3.decode(), "abc");

    // Traits
    let own_cmp = OwnString::from(own.decode().to_string());
    assert_eq!(own.partial_cmp(&own_cmp), Some(Ordering::Equal));
    assert_eq!(own.cmp(&own_cmp), Ordering::Equal);
    assert_eq!(calculate_hash(&own), calculate_hash(&own_cmp));
}

#[test]
fn test_mut_vec_new_clone() {
    let mut own = OwnVec::from(vec![1, 2, 3]);
    let mut view = own.to_mut();
    
    // Safety: we are cloning the view, which creates aliasing mutable pointers if we are not careful.
    // However, new_clone returns a new struct with same pointers.
    // The method is unsafe.
    
    unsafe {
        let view_clone = view.new_clone();
        assert_eq!(view_clone.len(), 3);
        // We shouldn't use both simultaneously for mutation in safe code, but for testing `new_clone` existence:
        assert_eq!(view_clone.as_slice(), &[1, 2, 3]);
    }
}

#[test]
fn test_mut_vec_into_raw_parts() {
    let mut own = OwnVec::from(vec![1, 2, 3]);
    let view = own.to_mut();
    let (ptr, len, cap) = view.into_raw_parts();
    assert_eq!(len.get(), 3);
    assert!(cap.get() >= 3);
    assert!(ptr.get() != 0);
}

#[test]
fn test_mut_string_into_raw_parts() {
    let mut own = OwnString::from("hello");
    let view = own.to_mut();
    let (ptr, len, cap) = view.into_raw_parts();
    assert_eq!(len.get(), 5);
    assert!(cap.get() >= 5);
    assert!(ptr.get() != 0);
}
