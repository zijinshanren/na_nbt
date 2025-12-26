use std::{
    borrow::{Borrow, BorrowMut},
    cmp::Ordering,
    collections::TryReserveError,
    fmt,
    hash::{Hash, Hasher},
    io::{self, IoSlice, Write},
    marker::PhantomData,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut, Index, IndexMut, RangeBounds},
    ptr,
    slice::{self, SliceIndex},
};

use zerocopy::Unalign;

pub struct MutVec<'a, T> {
    pub(crate) ptr: &'a mut Unalign<usize>,
    pub(crate) len: &'a mut Unalign<usize>,
    pub(crate) cap: &'a mut Unalign<usize>,
    _marker: PhantomData<T>,
}

// SAFETY: VecView is Send/Sync if T is Send/Sync.
// The raw pointer is only used for the Vec's buffer, which is guarded by the mutable references.
unsafe impl<T: Send> Send for MutVec<'_, T> {}
unsafe impl<T: Sync> Sync for MutVec<'_, T> {}

impl<'a, T> MutVec<'a, T> {
    /// Creates a new `VecView` from mutable references to a Vec's raw parts.
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    /// - `*ptr` points to a valid allocation from the global allocator (or dangling if `*cap == 0`)
    /// - `*len <= *cap`
    /// - The first `*len` elements pointed to by `*ptr` are properly initialized
    /// - The allocation has space for `*cap` elements of type `T`
    #[inline]
    pub unsafe fn new(
        ptr: &'a mut Unalign<usize>,
        len: &'a mut Unalign<usize>,
        cap: &'a mut Unalign<usize>,
    ) -> Self {
        debug_assert!(len.get() <= cap.get());
        Self {
            ptr,
            len,
            cap,
            _marker: PhantomData,
        }
    }

    /// Temporarily reconstructs a Vec, calls a closure on it, then writes back the fields.
    #[inline]
    fn with_vec<R>(&mut self, f: impl FnOnce(&mut Vec<T>) -> R) -> R {
        // SAFETY: caller guarantees valid raw parts
        let mut vec = unsafe {
            ManuallyDrop::new(Vec::from_raw_parts(
                self.as_mut_ptr(),
                self.len.get(),
                self.cap.get(),
            ))
        };
        let result = f(&mut vec);
        // Write back potentially changed fields
        self.ptr.set(vec.as_mut_ptr().expose_provenance());
        self.len.set(vec.len());
        self.cap.set(vec.capacity());
        result
    }

    /// Returns the new clone of this [`MutVec<T>`].
    ///
    /// # Safety
    ///
    /// .
    #[inline]
    pub unsafe fn new_clone(&mut self) -> Self {
        MutVec {
            ptr: unsafe { &mut *(self.ptr as *mut _) },
            len: unsafe { &mut *(self.len as *mut _) },
            cap: unsafe { &mut *(self.cap as *mut _) },
            _marker: PhantomData,
        }
    }

    // ============ Basic accessors ============

    /// Returns the number of elements in the vector.
    #[inline]
    pub fn len(&self) -> usize {
        self.len.get()
    }

    /// Returns `true` if the vector contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len.get() == 0
    }

    /// Returns the total number of elements the vector can hold without reallocating.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.cap.get()
    }

    /// Returns a raw pointer to the vector's buffer.
    #[inline]
    pub fn as_ptr(&self) -> *const T {
        ptr::with_exposed_provenance(self.ptr.get())
    }

    /// Returns an unsafe mutable pointer to the vector's buffer.
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        ptr::with_exposed_provenance_mut(self.ptr.get())
    }

    /// Extracts a slice containing the entire vector.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        // SAFETY: ptr is valid for len elements
        unsafe { slice::from_raw_parts(self.as_ptr(), self.len.get()) }
    }

    /// Extracts a mutable slice of the entire vector.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: ptr is valid for len elements
        unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len.get()) }
    }

    /// Forces the length of the vector to `new_len`.
    ///
    /// # Safety
    ///
    /// - `new_len` must be less than or equal to `capacity()`
    /// - If `new_len > len()`, the elements at `len()..new_len` must be initialized
    #[inline]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        debug_assert!(new_len <= self.cap.get());
        self.len.set(new_len);
    }

    // ============ Capacity methods ============

    /// Reserves capacity for at least `additional` more elements.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.with_vec(|v| v.reserve(additional));
    }

    /// Reserves the minimum capacity for at least `additional` more elements.
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.with_vec(|v| v.reserve_exact(additional));
    }

    /// Tries to reserve capacity for at least `additional` more elements.
    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.with_vec(|v| v.try_reserve(additional))
    }

    /// Tries to reserve the minimum capacity for at least `additional` more elements.
    #[inline]
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.with_vec(|v| v.try_reserve_exact(additional))
    }

    /// Shrinks the capacity of the vector as much as possible.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.with_vec(|v| v.shrink_to_fit());
    }

    /// Shrinks the capacity of the vector with a lower bound.
    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.with_vec(|v| v.shrink_to(min_capacity));
    }

    /// Returns the remaining spare capacity as a mutable slice of `MaybeUninit<T>`.
    #[inline]
    pub fn spare_capacity_mut(&mut self) -> &mut [core::mem::MaybeUninit<T>] {
        self.with_vec(|v| {
            // SAFETY: spare_capacity_mut returns a slice valid for the vec's lifetime,
            // but we need to extend it. The memory remains valid as we don't drop the vec.
            let spare = v.spare_capacity_mut();
            unsafe { core::slice::from_raw_parts_mut(spare.as_mut_ptr(), spare.len()) }
        })
    }

    // ============ Mutation methods ============

    /// Shortens the vector, keeping the first `len` elements and dropping the rest.
    #[inline]
    pub fn truncate(&mut self, len: usize) {
        self.with_vec(|v| v.truncate(len));
    }

    /// Clears the vector, removing all values.
    #[inline]
    pub fn clear(&mut self) {
        self.with_vec(|v| v.clear());
    }

    /// Appends an element to the back of the vector.
    #[inline]
    pub fn push(&mut self, value: T) {
        self.with_vec(|v| v.push(value));
    }

    /// Removes the last element from the vector and returns it, or `None` if empty.
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        self.with_vec(|v| v.pop())
    }

    #[inline]
    pub fn pop_if(&mut self, predicate: impl FnOnce(&mut T) -> bool) -> Option<T> {
        self.with_vec(|v| v.pop_if(predicate))
    }

    /// Inserts an element at position `index`, shifting all elements after it to the right.
    ///
    /// # Panics
    ///
    /// Panics if `index > len`.
    #[inline]
    pub fn insert(&mut self, index: usize, element: T) {
        self.with_vec(|v| v.insert(index, element));
    }

    /// Removes and returns the element at position `index`, shifting all elements after it to the left.
    ///
    /// # Panics
    ///
    /// Panics if `index >= len`.
    #[inline]
    pub fn remove(&mut self, index: usize) -> T {
        self.with_vec(|v| v.remove(index))
    }

    /// Removes an element from the vector and returns it, replacing it with the last element.
    ///
    /// This does not preserve ordering, but is O(1).
    ///
    /// # Panics
    ///
    /// Panics if `index >= len`.
    #[inline]
    pub fn swap_remove(&mut self, index: usize) -> T {
        self.with_vec(|v| v.swap_remove(index))
    }

    /// Retains only the elements specified by the predicate.
    #[inline]
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.with_vec(|v| v.retain(f));
    }

    /// Retains only the elements specified by the predicate, passing mutable references.
    #[inline]
    pub fn retain_mut<F>(&mut self, f: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        self.with_vec(|v| v.retain_mut(f));
    }

    /// Removes consecutive repeated elements according to the `PartialEq` trait.
    #[inline]
    pub fn dedup(&mut self)
    where
        T: PartialEq,
    {
        self.with_vec(|v| v.dedup());
    }

    /// Removes consecutive elements that satisfy the given predicate.
    #[inline]
    pub fn dedup_by<F>(&mut self, same_bucket: F)
    where
        F: FnMut(&mut T, &mut T) -> bool,
    {
        self.with_vec(|v| v.dedup_by(same_bucket));
    }

    /// Removes consecutive elements that map to the same key.
    #[inline]
    pub fn dedup_by_key<F, K>(&mut self, key: F)
    where
        F: FnMut(&mut T) -> K,
        K: PartialEq,
    {
        self.with_vec(|v| v.dedup_by_key(key));
    }

    /// Appends all elements from a slice to the vector.
    #[inline]
    pub fn extend_from_slice(&mut self, other: &[T])
    where
        T: Clone,
    {
        self.with_vec(|v| v.extend_from_slice(other));
    }

    /// Appends all elements from a slice to the vector by copying.
    #[inline]
    pub fn extend_from_within<R>(&mut self, src: R)
    where
        T: Copy,
        R: RangeBounds<usize>,
    {
        self.with_vec(|v| v.extend_from_within(src));
    }

    /// Resizes the vector to `new_len`, filling new slots with `value`.
    #[inline]
    pub fn resize(&mut self, new_len: usize, value: T)
    where
        T: Clone,
    {
        self.with_vec(|v| v.resize(new_len, value));
    }

    /// Resizes the vector to `new_len`, using a closure to create new values.
    #[inline]
    pub fn resize_with<F>(&mut self, new_len: usize, f: F)
    where
        F: FnMut() -> T,
    {
        self.with_vec(|v| v.resize_with(new_len, f));
    }

    #[inline]
    pub fn splice_drop<R, I>(&mut self, range: R, replace_with: I)
    where
        R: RangeBounds<usize>,
        I: IntoIterator<Item = T>,
    {
        self.with_vec(|v| drop(v.splice(range, replace_with)));
    }

    /// Moves all elements from `other` into `self`, leaving `other` empty.
    #[inline]
    pub fn append(&mut self, other: &mut Vec<T>) {
        self.with_vec(|v| v.append(other));
    }

    /// Moves all elements from another `VecView` into `self`, leaving `other` empty.
    #[inline]
    pub fn append_view(&mut self, other: &mut MutVec<'_, T>) {
        // Reconstruct other as a vec temporarily
        let mut other_vec = unsafe {
            ManuallyDrop::new(Vec::from_raw_parts(
                other.as_mut_ptr(),
                other.len.get(),
                other.cap.get(),
            ))
        };

        self.with_vec(|v| v.append(&mut other_vec));

        // Write back other's fields
        other.ptr.set(other_vec.as_mut_ptr().expose_provenance());
        other.len.set(other_vec.len());
        other.cap.set(other_vec.capacity());
    }

    /// Splits the vector into two at the given index.
    ///
    /// Returns a newly allocated `Vec<T>` containing the elements in `[at, len)`.
    /// After the call, the original vector will be left containing elements `[0, at)`.
    ///
    /// # Panics
    ///
    /// Panics if `at > len`.
    #[inline]
    pub fn split_off(&mut self, at: usize) -> Vec<T> {
        self.with_vec(|v| v.split_off(at))
    }

    /// Drains (removes) the specified range and drops all drained elements.
    /// This is a safe alternative to an unsound `drain` iterator method.
    #[inline]
    pub fn drain_drop<R>(&mut self, range: R)
    where
        R: RangeBounds<usize>,
    {
        self.with_vec(|v| drop(v.drain(range)));
    }

    // ============ Conversion methods ============

    /// Consumes the view and returns the raw parts as mutable references.
    #[inline]
    pub fn into_raw_parts(
        self,
    ) -> (
        &'a mut Unalign<usize>,
        &'a mut Unalign<usize>,
        &'a mut Unalign<usize>,
    ) {
        (self.ptr, self.len, self.cap)
    }
}

// ============ Trait Implementations ============

impl<T> Deref for MutVec<'_, T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> DerefMut for MutVec<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T, I: SliceIndex<[T]>> Index<I> for MutVec<'_, T> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(self.as_slice(), index)
    }
}

impl<T, I: SliceIndex<[T]>> IndexMut<I> for MutVec<'_, T> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(self.as_mut_slice(), index)
    }
}

impl<T: fmt::Debug> fmt::Debug for MutVec<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_slice(), f)
    }
}

impl<T: PartialEq> PartialEq for MutVec<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T: PartialEq> PartialEq<Vec<T>> for MutVec<'_, T> {
    fn eq(&self, other: &Vec<T>) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T: PartialEq> PartialEq<[T]> for MutVec<'_, T> {
    fn eq(&self, other: &[T]) -> bool {
        self.as_slice() == other
    }
}

impl<T: PartialEq, const N: usize> PartialEq<[T; N]> for MutVec<'_, T> {
    fn eq(&self, other: &[T; N]) -> bool {
        self.as_slice() == other
    }
}

impl<T: PartialEq> PartialEq<&[T]> for MutVec<'_, T> {
    fn eq(&self, other: &&[T]) -> bool {
        self.as_slice() == *other
    }
}

impl<T: PartialEq> PartialEq<&mut [T]> for MutVec<'_, T> {
    fn eq(&self, other: &&mut [T]) -> bool {
        self.as_slice() == *other
    }
}

impl<T: Eq> Eq for MutVec<'_, T> {}

impl<T: PartialOrd> PartialOrd for MutVec<'_, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_slice().partial_cmp(other.as_slice())
    }
}

impl<T: Ord> Ord for MutVec<'_, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_slice().cmp(other.as_slice())
    }
}

impl<T: Hash> Hash for MutVec<'_, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state);
    }
}

impl<T> Borrow<[T]> for MutVec<'_, T> {
    fn borrow(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> BorrowMut<[T]> for MutVec<'_, T> {
    fn borrow_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T> AsRef<[T]> for MutVec<'_, T> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> AsMut<[T]> for MutVec<'_, T> {
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<'a, T> IntoIterator for &'a MutVec<'_, T> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut MutVec<'_, T> {
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_mut_slice().iter_mut()
    }
}

impl<T> Extend<T> for MutVec<'_, T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.with_vec(|v| v.extend(iter));
    }
}

impl<'a, T: Copy + 'a> Extend<&'a T> for MutVec<'_, T> {
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.with_vec(|v| v.extend(iter));
    }
}

impl Write for MutVec<'_, u8> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.with_vec(|v| v.write(buf))
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        self.with_vec(|v| v.write_vectored(bufs))
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.with_vec(|v| v.write_all(buf))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// A view into a String's raw parts, allowing mutable access without owning the String.
///
/// This is similar to `VecView<u8>` but maintains String's UTF-8 invariants.
pub struct MutString<'a> {
    pub(crate) ptr: &'a mut Unalign<usize>,
    pub(crate) len: &'a mut Unalign<usize>,
    pub(crate) cap: &'a mut Unalign<usize>,
}

// SAFETY: StringView is Send/Sync because the underlying data is UTF-8 bytes.
// The raw pointer is only used for the String's buffer, which is guarded by the mutable references.
unsafe impl Send for MutString<'_> {}
unsafe impl Sync for MutString<'_> {}

impl<'a> MutString<'a> {
    /// Creates a new `StringView` from mutable references to a String's raw parts.
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    /// - `*ptr` points to a valid mutf8-encoded allocation from the global allocator (or dangling if `*cap == 0`)
    /// - `*len <= *cap`
    /// - The first `*len` bytes pointed to by `*ptr` are valid mutf8
    /// - The allocation has space for `*cap` bytes
    #[inline]
    pub unsafe fn new(
        ptr: &'a mut Unalign<usize>,
        len: &'a mut Unalign<usize>,
        cap: &'a mut Unalign<usize>,
    ) -> Self {
        debug_assert!(len.get() <= cap.get());
        Self { ptr, len, cap }
    }

    /// Temporarily decodes mutf8 to a String, calls a closure on it, then encodes back to mutf8.
    #[inline]
    fn with_string<R>(&mut self, f: impl FnOnce(&mut String) -> R) -> R {
        use std::borrow::Cow;

        let old_ptr = self.as_mut_ptr();
        let old_len = self.len.get();
        let old_cap = self.cap.get();

        let data = unsafe { slice::from_raw_parts(old_ptr, old_len) };
        let decoded = simd_cesu8::mutf8::decode_lossy(data);

        let mut string = match decoded {
            Cow::Borrowed(_) => {
                // Data is valid UTF-8, we can use it directly as a String
                // SAFETY: We're taking ownership of the buffer that was valid UTF-8
                unsafe { String::from_raw_parts(old_ptr, old_len, old_cap) }
            }
            Cow::Owned(s) => {
                // Data needed decoding, free old buffer
                unsafe {
                    drop(Vec::<u8>::from_raw_parts(old_ptr, old_len, old_cap));
                }
                s
            }
        };

        let result = f(&mut string);

        // Encode back to mutf8
        let encoded = simd_cesu8::mutf8::encode(&string);

        match encoded {
            Cow::Borrowed(_) => {
                // String's UTF-8 bytes are valid mutf8, reuse the String's buffer
                let mut string = ManuallyDrop::new(string);
                let vec = unsafe { string.as_mut_vec() };
                self.ptr.set(vec.as_mut_ptr().expose_provenance());
                self.len.set(vec.len());
                self.cap.set(vec.capacity());
            }
            Cow::Owned(vec) => {
                // Need new allocation for mutf8 encoding
                drop(string);
                let mut vec = ManuallyDrop::new(vec);
                self.ptr.set(vec.as_mut_ptr().expose_provenance());
                self.len.set(vec.len());
                self.cap.set(vec.capacity());
            }
        }

        result
    }

    // ============ Basic accessors ============

    /// Returns the length of this String, in bytes (mutf8 encoded).
    #[inline]
    pub fn len(&self) -> usize {
        self.len.get()
    }

    /// Returns `true` if this String has a length of zero.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len.get() == 0
    }

    /// Returns the total number of bytes the String can hold without reallocating.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.cap.get()
    }

    /// Returns a byte slice of this String's contents (mutf8 encoded).
    #[inline]
    pub fn as_mutf8_bytes(&self) -> &[u8] {
        // SAFETY: ptr is valid for len bytes
        unsafe { slice::from_raw_parts(self.as_ptr(), self.len.get()) }
    }

    /// Decodes the mutf8 content and returns the decoded string.
    #[inline]
    pub fn decode(&self) -> std::borrow::Cow<'_, str> {
        simd_cesu8::mutf8::decode_lossy(self.as_mutf8_bytes())
    }

    /// Returns a raw pointer to the String's buffer.
    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        ptr::with_exposed_provenance(self.ptr.get())
    }

    /// Returns an unsafe mutable pointer to the String's buffer.
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        ptr::with_exposed_provenance_mut(self.ptr.get())
    }

    // ============ Capacity methods ============

    /// Reserves capacity for at least `additional` more bytes.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.with_string(|s| s.reserve(additional));
    }

    /// Reserves the minimum capacity for at least `additional` more bytes.
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.with_string(|s| s.reserve_exact(additional));
    }

    /// Tries to reserve capacity for at least `additional` more elements.
    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.with_string(|s| s.try_reserve(additional))
    }

    /// Tries to reserve the minimum capacity for at least `additional` more elements.
    #[inline]
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.with_string(|s| s.try_reserve_exact(additional))
    }

    /// Shrinks the capacity of the String as much as possible.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.with_string(|s| s.shrink_to_fit());
    }

    /// Shrinks the capacity of the String with a lower bound.
    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.with_string(|s| s.shrink_to(min_capacity));
    }

    // ============ Mutation methods ============

    /// Appends a given string slice onto the end of this String.
    #[inline]
    pub fn push_str(&mut self, string: &str) {
        self.with_string(|s| s.push_str(string));
    }

    /// Appends the given char to the end of this String.
    #[inline]
    pub fn push(&mut self, ch: char) {
        self.with_string(|s| s.push(ch));
    }

    /// Shortens this String to the specified length.
    #[inline]
    pub fn truncate(&mut self, new_len: usize) {
        self.with_string(|s| s.truncate(new_len));
    }

    /// Removes the last character from the String and returns it.
    #[inline]
    pub fn pop(&mut self) -> Option<char> {
        self.with_string(|s| s.pop())
    }

    /// Removes a char from this String at a byte position and returns it.
    #[inline]
    pub fn remove(&mut self, idx: usize) -> char {
        self.with_string(|s| s.remove(idx))
    }

    /// Retains only the characters specified by the predicate.
    #[inline]
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(char) -> bool,
    {
        self.with_string(|s| s.retain(f));
    }

    /// Inserts a character into this String at a byte position.
    #[inline]
    pub fn insert(&mut self, idx: usize, ch: char) {
        self.with_string(|s| s.insert(idx, ch));
    }

    /// Inserts a string slice into this String at a byte position.
    #[inline]
    pub fn insert_str(&mut self, idx: usize, string: &str) {
        self.with_string(|s| s.insert_str(idx, string));
    }

    /// Truncates this String, removing all contents.
    #[inline]
    pub fn clear(&mut self) {
        self.with_string(|s| s.clear());
    }

    /// Splits the string into two at the given byte index.
    #[inline]
    pub fn split_off(&mut self, at: usize) -> String {
        self.with_string(|s| s.split_off(at))
    }

    #[inline]
    pub fn extend_from_within<R>(&mut self, src: R)
    where
        R: RangeBounds<usize>,
    {
        self.with_string(|s| s.extend_from_within(src));
    }

    #[inline]
    pub fn replace_range<R>(&mut self, range: R, replace_with: &str)
    where
        R: RangeBounds<usize>,
    {
        self.with_string(|s| s.replace_range(range, replace_with));
    }

    /// Drains (removes) the specified character range and drops all drained characters.
    #[inline]
    pub fn drain_drop<R>(&mut self, range: R)
    where
        R: RangeBounds<usize>,
    {
        self.with_string(|s| drop(s.drain(range)));
    }

    /// Consumes the view and returns the raw parts as mutable references.
    #[inline]
    pub fn into_raw_parts(
        self,
    ) -> (
        &'a mut Unalign<usize>,
        &'a mut Unalign<usize>,
        &'a mut Unalign<usize>,
    ) {
        (self.ptr, self.len, self.cap)
    }
}

// ============ Trait Implementations for StringViewMut ============

impl fmt::Debug for MutString<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&*self.decode(), f)
    }
}

impl fmt::Display for MutString<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&*self.decode(), f)
    }
}

impl PartialEq for MutString<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.as_mutf8_bytes() == other.as_mutf8_bytes()
    }
}

impl PartialEq<String> for MutString<'_> {
    fn eq(&self, other: &String) -> bool {
        &*self.decode() == other.as_str()
    }
}

impl PartialEq<str> for MutString<'_> {
    fn eq(&self, other: &str) -> bool {
        &*self.decode() == other
    }
}

impl PartialEq<&str> for MutString<'_> {
    fn eq(&self, other: &&str) -> bool {
        &*self.decode() == *other
    }
}

impl Eq for MutString<'_> {}

impl PartialOrd for MutString<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MutString<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_mutf8_bytes().cmp(other.as_mutf8_bytes())
    }
}

impl Hash for MutString<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_mutf8_bytes().hash(state);
    }
}

impl AsRef<[u8]> for MutString<'_> {
    fn as_ref(&self) -> &[u8] {
        self.as_mutf8_bytes()
    }
}

impl Write for MutString<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match std::str::from_utf8(buf) {
            Ok(s) => {
                self.push_str(s);
                Ok(buf.len())
            }
            Err(_) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid UTF-8 in write",
            )),
        }
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        match std::str::from_utf8(buf) {
            Ok(s) => {
                self.push_str(s);
                Ok(())
            }
            Err(_) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid UTF-8 in write_all",
            )),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[repr(C)]
pub struct OwnVec<T> {
    pub(crate) ptr: Unalign<usize>,
    pub(crate) len: Unalign<usize>,
    pub(crate) cap: Unalign<usize>,
    _marker: PhantomData<T>,
}

// SAFETY: VecView is Send/Sync if T is Send/Sync.
// The raw pointer is only used for the Vec's buffer, which is guarded by the mutable references.
unsafe impl<T: Send> Send for OwnVec<T> {}
unsafe impl<T: Sync> Sync for OwnVec<T> {}

impl<T> Default for OwnVec<T> {
    fn default() -> Self {
        vec![].into()
    }
}

impl<T> OwnVec<T> {
    /// Creates a new `VecView` from mutable references to a Vec's raw parts.
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    /// - `*ptr` points to a valid allocation from the global allocator (or dangling if `*cap == 0`)
    /// - `*len <= *cap`
    /// - The first `*len` elements pointed to by `*ptr` are properly initialized
    /// - The allocation has space for `*cap` elements of type `T`
    #[inline]
    pub unsafe fn new(ptr: Unalign<usize>, len: Unalign<usize>, cap: Unalign<usize>) -> Self {
        debug_assert!(len.get() <= cap.get());
        Self {
            ptr,
            len,
            cap,
            _marker: PhantomData,
        }
    }

    /// Temporarily reconstructs a Vec, calls a closure on it, then writes back the fields.
    #[inline]
    fn with_vec<R>(&mut self, f: impl FnOnce(&mut Vec<T>) -> R) -> R {
        // SAFETY: caller guarantees valid raw parts
        let mut vec = unsafe {
            ManuallyDrop::new(Vec::from_raw_parts(
                self.as_mut_ptr(),
                self.len.get(),
                self.cap.get(),
            ))
        };
        let result = f(&mut vec);
        // Write back potentially changed fields
        self.ptr.set(vec.as_mut_ptr().expose_provenance());
        self.len.set(vec.len());
        self.cap.set(vec.capacity());
        result
    }

    // ============ Basic accessors ============

    /// Returns the number of elements in the vector.
    #[inline]
    pub fn len(&self) -> usize {
        self.len.get()
    }

    /// Returns `true` if the vector contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len.get() == 0
    }

    /// Returns the total number of elements the vector can hold without reallocating.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.cap.get()
    }

    /// Returns a raw pointer to the vector's buffer.
    #[inline]
    pub fn as_ptr(&self) -> *const T {
        ptr::with_exposed_provenance(self.ptr.get())
    }

    /// Returns an unsafe mutable pointer to the vector's buffer.
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        ptr::with_exposed_provenance_mut(self.ptr.get())
    }

    /// Extracts a slice containing the entire vector.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        // SAFETY: ptr is valid for len elements
        unsafe { slice::from_raw_parts(self.as_ptr(), self.len.get()) }
    }

    /// Extracts a mutable slice of the entire vector.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: ptr is valid for len elements
        unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len.get()) }
    }

    /// Forces the length of the vector to `new_len`.
    ///
    /// # Safety
    ///
    /// - `new_len` must be less than or equal to `capacity()`
    /// - If `new_len > len()`, the elements at `len()..new_len` must be initialized
    #[inline]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        debug_assert!(new_len <= self.cap.get());
        self.len.set(new_len);
    }

    // ============ Capacity methods ============

    /// Reserves capacity for at least `additional` more elements.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.with_vec(|v| v.reserve(additional));
    }

    /// Reserves the minimum capacity for at least `additional` more elements.
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.with_vec(|v| v.reserve_exact(additional));
    }

    /// Tries to reserve capacity for at least `additional` more elements.
    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.with_vec(|v| v.try_reserve(additional))
    }

    /// Tries to reserve the minimum capacity for at least `additional` more elements.
    #[inline]
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.with_vec(|v| v.try_reserve_exact(additional))
    }

    /// Shrinks the capacity of the vector as much as possible.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.with_vec(|v| v.shrink_to_fit());
    }

    /// Shrinks the capacity of the vector with a lower bound.
    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.with_vec(|v| v.shrink_to(min_capacity));
    }

    /// Returns the remaining spare capacity as a mutable slice of `MaybeUninit<T>`.
    #[inline]
    pub fn spare_capacity_mut(&mut self) -> &mut [core::mem::MaybeUninit<T>] {
        self.with_vec(|v| {
            // SAFETY: spare_capacity_mut returns a slice valid for the vec's lifetime,
            // but we need to extend it. The memory remains valid as we don't drop the vec.
            let spare = v.spare_capacity_mut();
            unsafe { core::slice::from_raw_parts_mut(spare.as_mut_ptr(), spare.len()) }
        })
    }

    // ============ Mutation methods ============

    /// Shortens the vector, keeping the first `len` elements and dropping the rest.
    #[inline]
    pub fn truncate(&mut self, len: usize) {
        self.with_vec(|v| v.truncate(len));
    }

    /// Clears the vector, removing all values.
    #[inline]
    pub fn clear(&mut self) {
        self.with_vec(|v| v.clear());
    }

    /// Appends an element to the back of the vector.
    #[inline]
    pub fn push(&mut self, value: T) {
        self.with_vec(|v| v.push(value));
    }

    /// Removes the last element from the vector and returns it, or `None` if empty.
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        self.with_vec(|v| v.pop())
    }

    #[inline]
    pub fn pop_if(&mut self, predicate: impl FnOnce(&mut T) -> bool) -> Option<T> {
        self.with_vec(|v| v.pop_if(predicate))
    }

    /// Inserts an element at position `index`, shifting all elements after it to the right.
    ///
    /// # Panics
    ///
    /// Panics if `index > len`.
    #[inline]
    pub fn insert(&mut self, index: usize, element: T) {
        self.with_vec(|v| v.insert(index, element));
    }

    /// Removes and returns the element at position `index`, shifting all elements after it to the left.
    ///
    /// # Panics
    ///
    /// Panics if `index >= len`.
    #[inline]
    pub fn remove(&mut self, index: usize) -> T {
        self.with_vec(|v| v.remove(index))
    }

    /// Removes an element from the vector and returns it, replacing it with the last element.
    ///
    /// This does not preserve ordering, but is O(1).
    ///
    /// # Panics
    ///
    /// Panics if `index >= len`.
    #[inline]
    pub fn swap_remove(&mut self, index: usize) -> T {
        self.with_vec(|v| v.swap_remove(index))
    }

    /// Retains only the elements specified by the predicate.
    #[inline]
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.with_vec(|v| v.retain(f));
    }

    /// Retains only the elements specified by the predicate, passing mutable references.
    #[inline]
    pub fn retain_mut<F>(&mut self, f: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        self.with_vec(|v| v.retain_mut(f));
    }

    /// Removes consecutive repeated elements according to the `PartialEq` trait.
    #[inline]
    pub fn dedup(&mut self)
    where
        T: PartialEq,
    {
        self.with_vec(|v| v.dedup());
    }

    /// Removes consecutive elements that satisfy the given predicate.
    #[inline]
    pub fn dedup_by<F>(&mut self, same_bucket: F)
    where
        F: FnMut(&mut T, &mut T) -> bool,
    {
        self.with_vec(|v| v.dedup_by(same_bucket));
    }

    /// Removes consecutive elements that map to the same key.
    #[inline]
    pub fn dedup_by_key<F, K>(&mut self, key: F)
    where
        F: FnMut(&mut T) -> K,
        K: PartialEq,
    {
        self.with_vec(|v| v.dedup_by_key(key));
    }

    /// Appends all elements from a slice to the vector.
    #[inline]
    pub fn extend_from_slice(&mut self, other: &[T])
    where
        T: Clone,
    {
        self.with_vec(|v| v.extend_from_slice(other));
    }

    /// Appends all elements from a slice to the vector by copying.
    #[inline]
    pub fn extend_from_within<R>(&mut self, src: R)
    where
        T: Copy,
        R: RangeBounds<usize>,
    {
        self.with_vec(|v| v.extend_from_within(src));
    }

    /// Resizes the vector to `new_len`, filling new slots with `value`.
    #[inline]
    pub fn resize(&mut self, new_len: usize, value: T)
    where
        T: Clone,
    {
        self.with_vec(|v| v.resize(new_len, value));
    }

    /// Resizes the vector to `new_len`, using a closure to create new values.
    #[inline]
    pub fn resize_with<F>(&mut self, new_len: usize, f: F)
    where
        F: FnMut() -> T,
    {
        self.with_vec(|v| v.resize_with(new_len, f));
    }

    #[inline]
    pub fn splice_drop<R, I>(&mut self, range: R, replace_with: I)
    where
        R: RangeBounds<usize>,
        I: IntoIterator<Item = T>,
    {
        self.with_vec(|v| drop(v.splice(range, replace_with)));
    }

    /// Moves all elements from `other` into `self`, leaving `other` empty.
    #[inline]
    pub fn append(&mut self, other: &mut Vec<T>) {
        self.with_vec(|v| v.append(other));
    }

    /// Moves all elements from another `VecView` into `self`, leaving `other` empty.
    #[inline]
    pub fn append_view(&mut self, other: &mut OwnVec<T>) {
        // Reconstruct other as a vec temporarily
        let mut other_vec = unsafe {
            ManuallyDrop::new(Vec::from_raw_parts(
                other.as_mut_ptr(),
                other.len.get(),
                other.cap.get(),
            ))
        };

        self.with_vec(|v| v.append(&mut other_vec));

        // Write back other's fields
        other.ptr.set(other_vec.as_mut_ptr().expose_provenance());
        other.len.set(other_vec.len());
        other.cap.set(other_vec.capacity());
    }

    /// Splits the vector into two at the given index.
    ///
    /// Returns a newly allocated `Vec<T>` containing the elements in `[at, len)`.
    /// After the call, the original vector will be left containing elements `[0, at)`.
    ///
    /// # Panics
    ///
    /// Panics if `at > len`.
    #[inline]
    pub fn split_off(&mut self, at: usize) -> Vec<T> {
        self.with_vec(|v| v.split_off(at))
    }

    /// Drains (removes) the specified range and drops all drained elements.
    /// This is a safe alternative to the unsound `drain` iterator method.
    #[inline]
    pub fn drain_drop<R>(&mut self, range: R)
    where
        R: RangeBounds<usize>,
    {
        self.with_vec(|v| drop(v.drain(range)));
    }

    // ============ Conversion methods ============

    /// Consumes the view and returns the raw parts as mutable references.
    #[inline]
    pub fn into_raw_parts(self) -> (Unalign<usize>, Unalign<usize>, Unalign<usize>) {
        let me = ManuallyDrop::new(self);
        (me.ptr, me.len, me.cap)
    }
}

// ============ Trait Implementations ============

impl<T> Deref for OwnVec<T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> DerefMut for OwnVec<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T, I: SliceIndex<[T]>> Index<I> for OwnVec<T> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(self.as_slice(), index)
    }
}

impl<T, I: SliceIndex<[T]>> IndexMut<I> for OwnVec<T> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(self.as_mut_slice(), index)
    }
}

impl<T: fmt::Debug> fmt::Debug for OwnVec<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_slice(), f)
    }
}

impl<T: PartialEq> PartialEq for OwnVec<T> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T: PartialEq> PartialEq<Vec<T>> for OwnVec<T> {
    fn eq(&self, other: &Vec<T>) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T: PartialEq> PartialEq<[T]> for OwnVec<T> {
    fn eq(&self, other: &[T]) -> bool {
        self.as_slice() == other
    }
}

impl<T: PartialEq, const N: usize> PartialEq<[T; N]> for OwnVec<T> {
    fn eq(&self, other: &[T; N]) -> bool {
        self.as_slice() == other
    }
}

impl<T: PartialEq> PartialEq<&[T]> for OwnVec<T> {
    fn eq(&self, other: &&[T]) -> bool {
        self.as_slice() == *other
    }
}

impl<T: PartialEq> PartialEq<&mut [T]> for OwnVec<T> {
    fn eq(&self, other: &&mut [T]) -> bool {
        self.as_slice() == *other
    }
}

impl<T: Eq> Eq for OwnVec<T> {}

impl<T: PartialOrd> PartialOrd for OwnVec<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_slice().partial_cmp(other.as_slice())
    }
}

impl<T: Ord> Ord for OwnVec<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_slice().cmp(other.as_slice())
    }
}

impl<T: Hash> Hash for OwnVec<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state);
    }
}

impl<T> Borrow<[T]> for OwnVec<T> {
    fn borrow(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> BorrowMut<[T]> for OwnVec<T> {
    fn borrow_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T> AsRef<[T]> for OwnVec<T> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> AsMut<[T]> for OwnVec<T> {
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<'a, T> IntoIterator for &'a OwnVec<T> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut OwnVec<T> {
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_mut_slice().iter_mut()
    }
}

impl<T> Extend<T> for OwnVec<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.with_vec(|v| v.extend(iter));
    }
}

impl<'a, T: Copy + 'a> Extend<&'a T> for OwnVec<T> {
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.with_vec(|v| v.extend(iter));
    }
}

impl Write for OwnVec<u8> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.with_vec(|v| v.write(buf))
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        self.with_vec(|v| v.write_vectored(bufs))
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.with_vec(|v| v.write_all(buf))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<T> From<Vec<T>> for OwnVec<T> {
    fn from(value: Vec<T>) -> Self {
        let mut me = ManuallyDrop::new(value);
        OwnVec {
            ptr: Unalign::new(me.as_mut_ptr().expose_provenance()),
            len: Unalign::new(me.len()),
            cap: Unalign::new(me.capacity()),
            _marker: PhantomData,
        }
    }
}

impl<T: Clone> From<&[T]> for OwnVec<T> {
    fn from(value: &[T]) -> Self {
        value.to_vec().into()
    }
}

impl<T> Drop for OwnVec<T> {
    fn drop(&mut self) {
        unsafe { Vec::from_raw_parts(self.as_mut_ptr(), self.len.get(), self.cap.get()) };
    }
}

/// A view into a String's raw parts, allowing mutable access without owning the String.
///
/// This is similar to `VecView<u8>` but maintains String's MUTF-8 encoded invariants.
#[repr(C)]
pub struct OwnString {
    pub(crate) ptr: Unalign<usize>,
    pub(crate) len: Unalign<usize>,
    pub(crate) cap: Unalign<usize>,
}

// SAFETY: StringView is Send/Sync because the underlying data is UTF-8 bytes.
// The raw pointer is only used for the String's buffer, which is guarded by the mutable references.
unsafe impl Send for OwnString {}
unsafe impl Sync for OwnString {}

impl Default for OwnString {
    fn default() -> Self {
        vec![].into()
    }
}

impl OwnString {
    /// Creates a new `StringView` from mutable references to a String's raw parts.
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    /// - `*ptr` points to a valid mutf8-encoded allocation from the global allocator (or dangling if `*cap == 0`)
    /// - `*len <= *cap`
    /// - The first `*len` bytes pointed to by `*ptr` are valid mutf8
    /// - The allocation has space for `*cap` bytes
    #[inline]
    pub unsafe fn new(ptr: Unalign<usize>, len: Unalign<usize>, cap: Unalign<usize>) -> Self {
        debug_assert!(len.get() <= cap.get());
        Self { ptr, len, cap }
    }

    /// Temporarily decodes mutf8 to a String, calls a closure on it, then encodes back to mutf8.
    #[inline]
    fn with_string<R>(&mut self, f: impl FnOnce(&mut String) -> R) -> R {
        use std::borrow::Cow;

        let old_ptr = self.as_mut_ptr();
        let old_len = self.len.get();
        let old_cap = self.cap.get();

        let data = unsafe { slice::from_raw_parts(old_ptr, old_len) };
        let decoded = simd_cesu8::mutf8::decode_lossy(data);

        let mut string = match decoded {
            Cow::Borrowed(_) => {
                // Data is valid UTF-8, we can use it directly as a String
                // SAFETY: We're taking ownership of the buffer that was valid UTF-8
                unsafe { String::from_raw_parts(old_ptr, old_len, old_cap) }
            }
            Cow::Owned(s) => {
                // Data needed decoding, free old buffer
                unsafe {
                    drop(Vec::<u8>::from_raw_parts(old_ptr, old_len, old_cap));
                }
                s
            }
        };

        let result = f(&mut string);

        // Encode back to mutf8
        let encoded = simd_cesu8::mutf8::encode(&string);

        match encoded {
            Cow::Borrowed(_) => {
                // String's UTF-8 bytes are valid mutf8, reuse the String's buffer
                let mut string = ManuallyDrop::new(string);
                let vec = unsafe { string.as_mut_vec() };
                self.ptr.set(vec.as_mut_ptr().expose_provenance());
                self.len.set(vec.len());
                self.cap.set(vec.capacity());
            }
            Cow::Owned(vec) => {
                // Need new allocation for mutf8 encoding
                drop(string);
                let mut vec = ManuallyDrop::new(vec);
                self.ptr.set(vec.as_mut_ptr().expose_provenance());
                self.len.set(vec.len());
                self.cap.set(vec.capacity());
            }
        }

        result
    }

    // ============ Basic accessors ============

    /// Returns the length of this String, in bytes (mutf8 encoded).
    #[inline]
    pub fn len(&self) -> usize {
        self.len.get()
    }

    /// Returns `true` if this String has a length of zero.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len.get() == 0
    }

    /// Returns the total number of bytes the String can hold without reallocating.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.cap.get()
    }

    /// Returns a byte slice of this String's contents (mutf8 encoded).
    #[inline]
    pub fn as_mutf8_bytes(&self) -> &[u8] {
        // SAFETY: ptr is valid for len bytes
        unsafe { slice::from_raw_parts(self.as_ptr(), self.len.get()) }
    }

    /// Decodes the mutf8 content and returns the decoded string.
    #[inline]
    pub fn decode(&self) -> std::borrow::Cow<'_, str> {
        simd_cesu8::mutf8::decode_lossy(self.as_mutf8_bytes())
    }

    /// Returns a raw pointer to the String's buffer.
    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        ptr::with_exposed_provenance(self.ptr.get())
    }

    /// Returns an unsafe mutable pointer to the String's buffer.
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        ptr::with_exposed_provenance_mut(self.ptr.get())
    }

    // ============ Capacity methods ============

    /// Reserves capacity for at least `additional` more bytes.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.with_string(|s| s.reserve(additional));
    }

    /// Reserves the minimum capacity for at least `additional` more bytes.
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.with_string(|s| s.reserve_exact(additional));
    }

    /// Tries to reserve capacity for at least `additional` more elements.
    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.with_string(|s| s.try_reserve(additional))
    }

    /// Tries to reserve the minimum capacity for at least `additional` more elements.
    #[inline]
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.with_string(|s| s.try_reserve_exact(additional))
    }

    /// Shrinks the capacity of the String as much as possible.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.with_string(|s| s.shrink_to_fit());
    }

    /// Shrinks the capacity of the String with a lower bound.
    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.with_string(|s| s.shrink_to(min_capacity));
    }

    // ============ Mutation methods ============

    /// Appends a given string slice onto the end of this String.
    #[inline]
    pub fn push_str(&mut self, string: &str) {
        self.with_string(|s| s.push_str(string));
    }

    /// Appends the given char to the end of this String.
    #[inline]
    pub fn push(&mut self, ch: char) {
        self.with_string(|s| s.push(ch));
    }

    /// Shortens this String to the specified length.
    #[inline]
    pub fn truncate(&mut self, new_len: usize) {
        self.with_string(|s| s.truncate(new_len));
    }

    /// Removes the last character from the String and returns it.
    #[inline]
    pub fn pop(&mut self) -> Option<char> {
        self.with_string(|s| s.pop())
    }

    /// Removes a char from this String at a byte position and returns it.
    #[inline]
    pub fn remove(&mut self, idx: usize) -> char {
        self.with_string(|s| s.remove(idx))
    }

    /// Retains only the characters specified by the predicate.
    #[inline]
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(char) -> bool,
    {
        self.with_string(|s| s.retain(f));
    }

    /// Inserts a character into this String at a byte position.
    #[inline]
    pub fn insert(&mut self, idx: usize, ch: char) {
        self.with_string(|s| s.insert(idx, ch));
    }

    /// Inserts a string slice into this String at a byte position.
    #[inline]
    pub fn insert_str(&mut self, idx: usize, string: &str) {
        self.with_string(|s| s.insert_str(idx, string));
    }

    /// Truncates this String, removing all contents.
    #[inline]
    pub fn clear(&mut self) {
        self.with_string(|s| s.clear());
    }

    /// Splits the string into two at the given byte index.
    #[inline]
    pub fn split_off(&mut self, at: usize) -> String {
        self.with_string(|s| s.split_off(at))
    }

    #[inline]
    pub fn extend_from_within<R>(&mut self, src: R)
    where
        R: RangeBounds<usize>,
    {
        self.with_string(|s| s.extend_from_within(src));
    }

    #[inline]
    pub fn replace_range<R>(&mut self, range: R, replace_with: &str)
    where
        R: RangeBounds<usize>,
    {
        self.with_string(|s| s.replace_range(range, replace_with));
    }

    /// Drains (removes) the specified character range and drops all drained characters.
    #[inline]
    pub fn drain_drop<R>(&mut self, range: R)
    where
        R: RangeBounds<usize>,
    {
        self.with_string(|s| drop(s.drain(range)));
    }

    /// Consumes the view and returns the raw parts as mutable references.
    #[inline]
    pub fn into_raw_parts(self) -> (Unalign<usize>, Unalign<usize>, Unalign<usize>) {
        let me = ManuallyDrop::new(self);
        (me.ptr, me.len, me.cap)
    }
}

// ============ Trait Implementations for StringViewOwn ============

impl fmt::Debug for OwnString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&*self.decode(), f)
    }
}

impl fmt::Display for OwnString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&*self.decode(), f)
    }
}

impl PartialEq for OwnString {
    fn eq(&self, other: &Self) -> bool {
        self.as_mutf8_bytes() == other.as_mutf8_bytes()
    }
}

impl PartialEq<String> for OwnString {
    fn eq(&self, other: &String) -> bool {
        &*self.decode() == other.as_str()
    }
}

impl PartialEq<str> for OwnString {
    fn eq(&self, other: &str) -> bool {
        &*self.decode() == other
    }
}

impl PartialEq<&str> for OwnString {
    fn eq(&self, other: &&str) -> bool {
        &*self.decode() == *other
    }
}

impl Eq for OwnString {}

impl PartialOrd for OwnString {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OwnString {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_mutf8_bytes().cmp(other.as_mutf8_bytes())
    }
}

impl Hash for OwnString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_mutf8_bytes().hash(state);
    }
}

impl AsRef<[u8]> for OwnString {
    fn as_ref(&self) -> &[u8] {
        self.as_mutf8_bytes()
    }
}

impl Write for OwnString {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match std::str::from_utf8(buf) {
            Ok(s) => {
                self.push_str(s);
                Ok(buf.len())
            }
            Err(_) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid UTF-8 in write",
            )),
        }
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        match std::str::from_utf8(buf) {
            Ok(s) => {
                self.push_str(s);
                Ok(())
            }
            Err(_) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid UTF-8 in write_all",
            )),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl From<Vec<u8>> for OwnString {
    fn from(value: Vec<u8>) -> Self {
        let mut encoded = ManuallyDrop::new(value);
        OwnString {
            ptr: Unalign::new(encoded.as_mut_ptr().expose_provenance()),
            len: Unalign::new(encoded.len()),
            cap: Unalign::new(encoded.capacity()),
        }
    }
}

impl From<&[u8]> for OwnString {
    fn from(value: &[u8]) -> Self {
        value.to_vec().into()
    }
}

impl From<String> for OwnString {
    fn from(value: String) -> Self {
        simd_cesu8::mutf8::encode(&value).into_owned().into()
    }
}

impl From<&str> for OwnString {
    fn from(value: &str) -> Self {
        simd_cesu8::mutf8::encode(value).into_owned().into()
    }
}

impl Drop for OwnString {
    fn drop(&mut self) {
        // Drop as Vec<u8> since internal format is mutf8, not UTF-8
        unsafe {
            drop(Vec::<u8>::from_raw_parts(
                self.as_mut_ptr(),
                self.len.get(),
                self.cap.get(),
            ));
        }
    }
}
