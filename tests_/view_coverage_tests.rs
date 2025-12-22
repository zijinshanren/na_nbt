use na_nbt::OwnedValue;
use zerocopy::byteorder::BigEndian as BE;
use std::borrow::{Borrow, BorrowMut};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::cmp::Ordering;

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

#[test]
fn test_vec_view_own_coverage() {
    let vec = vec![1i8, 2, 3];
    let val = OwnedValue::<BE>::from(vec.clone());
    
    if let OwnedValue::ByteArray(mut view) = val {
        // PartialEq
        assert_eq!(view, view);
        assert_eq!(view, vec);
        assert_eq!(view, vec.as_slice());
        assert_eq!(view, [1i8, 2, 3]); // array

        // PartialOrd, Ord
        assert_eq!(view.partial_cmp(&view), Some(Ordering::Equal));
        assert_eq!(view.cmp(&view), Ordering::Equal);

        // Hash
        assert_eq!(calculate_hash(&view), calculate_hash(&view));

        // Borrow
        let b: &[i8] = view.borrow();
        assert_eq!(b, &[1, 2, 3]);
        let bm: &mut [i8] = view.borrow_mut();
        bm[0] = 10;
        assert_eq!(view[0], 10);

        // AsRef, AsMut
        let r: &[i8] = view.as_ref();
        assert_eq!(r, &[10, 2, 3]);
        let m: &mut [i8] = view.as_mut();
        m[0] = 1;
        assert_eq!(view[0], 1);

        // IntoIterator (ref)
        let mut count = 0;
        for _ in &view {
            count += 1;
        }
        assert_eq!(count, 3);

        // IntoIterator (mut ref)
        for x in &mut view {
            *x += 1;
        }
        assert_eq!(view.as_slice(), &[2, 3, 4]);

        // Extend
        view.extend(vec![5, 6]);
        assert_eq!(view.as_slice(), &[2, 3, 4, 5, 6]);
        
        view.extend(&[7i8]);
        assert_eq!(view.as_slice(), &[2, 3, 4, 5, 6, 7]);

        // Write (u8 impl, but T is i8, so Write might not be implemented for VecViewOwn<i8>)
        // Write is implemented for VecViewOwn<u8>.
        // OwnedValue::ByteArray is VecViewOwn<i8>. 
        // So we can't test Write on ByteArray.
    } else {
        panic!("Expected ByteArray");
    }
}

#[test]
fn test_vec_view_mut_coverage() {
    let vec = vec![1i8, 2, 3];
    let mut val = OwnedValue::<BE>::from(vec.clone());
    
    let mut view = val.as_byte_array_mut().unwrap();

    // PartialEq
    assert_eq!(view, view);
    assert_eq!(view, vec);
    assert_eq!(view, vec.as_slice());
    assert_eq!(view, [1i8, 2, 3]);

    // PartialOrd, Ord
    assert_eq!(view.partial_cmp(&view), Some(Ordering::Equal));
    assert_eq!(view.cmp(&view), Ordering::Equal);

    // Hash
    assert_eq!(calculate_hash(&view), calculate_hash(&view));

    // Borrow
    let b: &[i8] = view.borrow();
    assert_eq!(b, &[1, 2, 3]);
    let bm: &mut [i8] = view.borrow_mut();
    bm[0] = 10;
    assert_eq!(view[0], 10);

    // AsRef, AsMut
    let r: &[i8] = view.as_ref();
    assert_eq!(r, &[10, 2, 3]);
    let m: &mut [i8] = view.as_mut();
    m[0] = 1;
    assert_eq!(view[0], 1);

    // IntoIterator (ref)
    let mut count = 0;
    for _ in &view {
        count += 1;
    }
    assert_eq!(count, 3);

    // IntoIterator (mut ref)
    for x in &mut view {
        *x += 1;
    }
    assert_eq!(view.as_slice(), &[2, 3, 4]);

    // Extend
    view.extend(vec![5, 6]);
    assert_eq!(view.as_slice(), &[2, 3, 4, 5, 6]);
    
    view.extend(&[7i8]);
    assert_eq!(view.as_slice(), &[2, 3, 4, 5, 6, 7]);
}

#[test]
fn test_string_view_own_coverage() {
    let s = String::from("hello");
    let val = OwnedValue::<BE>::from(s.clone());

    if let OwnedValue::String(mut view) = val {
        // Display
        assert_eq!(format!("{}", view), "hello");
        assert_eq!(format!("{:?}", view), "\"hello\"");

        // PartialEq
        assert_eq!(view, view);
        assert_eq!(view, s);
        assert_eq!(view, "hello");
        assert_eq!(view, "hello"); // &str

        // PartialOrd, Ord
        assert_eq!(view.partial_cmp(&view), Some(Ordering::Equal));
        assert_eq!(view.cmp(&view), Ordering::Equal);

        // Hash
        assert_eq!(calculate_hash(&view), calculate_hash(&view));

        // AsRef<[u8]>
        let bytes: &[u8] = view.as_ref();
        assert_eq!(bytes, b"hello");

        // Write
        write!(view, " world").unwrap();
        assert_eq!(view.to_string(), "hello world");

        // Write error (invalid utf8)
        let res = view.write(b"\xFF");
        assert!(res.is_err());
        let res = view.write_all(b"\xFF");
        assert!(res.is_err());
        
        view.flush().unwrap();
        
    } else {
        panic!("Expected String");
    }
}

#[test]
fn test_string_view_mut_coverage() {
    let s = String::from("hello");
    let mut val = OwnedValue::<BE>::from(s.clone());

    let mut view = val.as_string_mut().unwrap();

    // Display
    assert_eq!(format!("{}", view), "hello");
    assert_eq!(format!("{:?}", view), "\"hello\"");

    // PartialEq
    assert_eq!(view, view);
    assert_eq!(view, s);
    assert_eq!(view, "hello");
    assert_eq!(view, "hello");

    // PartialOrd, Ord
    assert_eq!(view.partial_cmp(&view), Some(Ordering::Equal));
    assert_eq!(view.cmp(&view), Ordering::Equal);

    // Hash
    assert_eq!(calculate_hash(&view), calculate_hash(&view));

    // AsRef<[u8]>
    let bytes: &[u8] = view.as_ref();
    assert_eq!(bytes, b"hello");

    // Write
    write!(view, " world").unwrap();
    assert_eq!(view.to_string(), "hello world");

    // Write error
    let res = view.write(b"\xFF");
    assert!(res.is_err());
    let res = view.write_all(b"\xFF");
    assert!(res.is_err());
    
    view.flush().unwrap();
}

#[test]
fn test_vec_view_mut_mutation() {
    let vec = vec![1i8, 2, 3];
    let mut val = OwnedValue::<BE>::from(vec);
    let mut view = val.as_byte_array_mut().unwrap();

    // Capacity
    view.reserve(10);
    assert!(view.capacity() >= 13);
    
    // Mutation
    view.push(4);
    assert_eq!(view.as_slice(), &[1, 2, 3, 4]);
    assert_eq!(view.pop(), Some(4));
    
    view.insert(0, 0);
    assert_eq!(view.as_slice(), &[0, 1, 2, 3]);
    
    assert_eq!(view.remove(0), 0);
    assert_eq!(view.as_slice(), &[1, 2, 3]);
    
    view.truncate(1);
    assert_eq!(view.as_slice(), &[1]);
    
    view.clear();
    assert!(view.is_empty());
    
    view.extend_from_slice(&[10, 11]);
    assert_eq!(view.as_slice(), &[10, 11]);

    // swap_remove
    let ret = view.swap_remove(0);
    assert_eq!(ret, 10);
    assert_eq!(view.as_slice(), &[11]);

    // retain
    view.push(12);
    view.retain(|x| *x % 2 == 0);
    assert_eq!(view.as_slice(), &[12]);
}

#[test]
fn test_string_view_mut_mutation() {
    let s = String::from("hello");
    let mut val = OwnedValue::<BE>::from(s);
    let mut view = val.as_string_mut().unwrap();

    view.push('!');
    assert_eq!(view.to_string(), "hello!");
    
    assert_eq!(view.pop(), Some('!'));
    
    view.insert(0, 'H');
    assert_eq!(view.to_string(), "Hhello");
    
    assert_eq!(view.remove(0), 'H');
    
    view.truncate(2);
    assert_eq!(view.to_string(), "he");
    
    view.clear();
    assert!(view.is_empty());
    
    view.push_str("world");
    assert_eq!(view.to_string(), "world");
    
    let tail = view.split_off(3);
    assert_eq!(view.to_string(), "wor");
    assert_eq!(tail, "ld");
    
    view.replace_range(1..2, "a");
    assert_eq!(view.to_string(), "war");
}
