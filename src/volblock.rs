use super::*;

/// A volatile memory block.
///
/// This is intended to model when a portion of memory is an array of identical
/// values in a row, such as a block of 256 `u16` values in a row.
///
/// ## Generic Parameters
/// * `T` / `R` / `W`: These parameters are applied to the [`VolAddress`] type
///   returned when accessing the block in any way (indexing, iteration, etc).
/// * `C`: the count of elements in the block.
///
/// ## Safety
/// * This type stores a base [`VolAddress`] internally, and so you must follow
///   all of those safety rules. Notably, the base address must never be zero.
/// * The address space must legally contain `C` contiguous values of the `T`
///   type, starting from the base address.
/// * The memory block must not wrap around past the end of the address space.
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VolBlock<T, R, W, const C: usize> {
  pub(crate) base: VolAddress<T, R, W>,
}

impl<T, R, W, const C: usize> Clone for VolBlock<T, R, W, C> {
  #[inline]
  #[must_use]
  fn clone(&self) -> Self {
    *self
  }
}
impl<T, R, W, const C: usize> Copy for VolBlock<T, R, W, C> {}

impl<T, R, W, const C: usize> VolBlock<T, R, W, C> {
  /// Constructs the value.
  ///
  /// ## Safety
  /// * As per the type docs.
  #[inline]
  #[must_use]
  pub const unsafe fn new(base: usize) -> Self {
    Self { base: VolAddress::new(base) }
  }

  /// The length of this block (in elements).
  #[inline]
  #[must_use]
  #[allow(clippy::len_without_is_empty)]
  pub const fn len(self) -> usize {
    C
  }

  /// Converts the `VolBlock` the `usize` for the start of the block.
  #[inline]
  #[must_use]
  pub const fn as_usize(self) -> usize {
    self.base.address.get()
  }

  /// Converts the `VolBlock` into an individual const pointer.
  ///
  /// This should usually only be used when you need to call a foreign function
  /// that expects a pointer.
  #[inline]
  #[must_use]
  pub const fn as_ptr(self) -> *const T {
    self.base.address.get() as *const T
  }

  /// Converts the `VolBlock` into an individual mut pointer.
  ///
  /// This should usually only be used when you need to call a foreign function
  /// that expects a pointer.
  #[inline]
  #[must_use]
  pub const fn as_mut_ptr(self) -> *mut T {
    self.base.address.get() as *mut T
  }

  /// Converts the `VolBlock` into a const slice pointer.
  #[inline]
  #[must_use]
  // TODO(2022-10-15): const fn this at some point in the future (1.64 minimum)
  pub fn as_slice_ptr(self) -> *const [T] {
    core::ptr::slice_from_raw_parts(self.base.address.get() as *const T, C)
  }

  /// Converts the `VolBlock` into a mut slice pointer.
  #[inline]
  #[must_use]
  // TODO(2022-10-15): const fn this at some point in the future (unstable)
  pub fn as_slice_mut_ptr(self) -> *mut [T] {
    core::ptr::slice_from_raw_parts_mut(self.base.address.get() as *mut T, C)
  }

  /// Indexes to the `i`th position of the memory block.
  ///
  /// ## Panics
  /// * If the index is out of bounds this will panic.
  #[inline]
  #[must_use]
  #[track_caller]
  pub const fn index(self, i: usize) -> VolAddress<T, R, W> {
    assert!(i < C);
    unsafe { self.base.add(i) }
  }

  /// Gets the address of the `i`th position, if it's in bounds.
  #[inline]
  #[must_use]
  pub const fn get(self, i: usize) -> Option<VolAddress<T, R, W>> {
    if i < C {
      Some(unsafe { self.base.add(i) })
    } else {
      None
    }
  }

  /// Creates an iterator over the addresses of the memory block.
  #[inline]
  #[must_use]
  pub const fn iter(self) -> VolBlockIter<T, R, W> {
    VolBlockIter { base: self.base, count: C }
  }

  /// Makes an iterator over the range bounds given.
  ///
  /// If the range given is empty then your iterator will be empty.
  ///
  /// ## Panics
  /// * If the start or end of the range are out of bounds for the block.
  #[inline]
  #[must_use]
  #[track_caller]
  pub fn iter_range<RB: core::ops::RangeBounds<usize>>(
    self, r: RB,
  ) -> VolBlockIter<T, R, W> {
    // TODO: some day make this a const fn, once start_bound and end_bound are
    // made into const fn, but that requires const trait impls.
    use core::ops::Bound;
    let start_inclusive: usize = match r.start_bound() {
      Bound::Included(i) => *i,
      Bound::Excluded(x) => x + 1,
      Bound::Unbounded => 0,
    };
    assert!(start_inclusive < C);
    let end_exclusive: usize = match r.end_bound() {
      Bound::Included(i) => i + 1,
      Bound::Excluded(x) => *x,
      Bound::Unbounded => C,
    };
    assert!(end_exclusive <= C);
    let count = end_exclusive.saturating_sub(start_inclusive);
    VolBlockIter { base: self.index(start_inclusive), count }
  }

  /// View the volatile block as an equivalent spanned region.
  ///
  /// This method exists because unfortunately the typing of the `Deref` trait
  /// doesn't allow for a Block to deref into a Region, so we have to provide
  /// the conversion through this manual method.
  #[inline]
  #[must_use]
  pub const fn as_region(self) -> VolRegion<T, R, W> {
    VolRegion { addr: self.base, len: C }
  }

  /// Casts a block to an address to an equivalent sized array.
  ///
  /// ## Safety
  /// * As per the general `VolAddress` construction rules.
  /// * It is *highly likely* that on any device this is safe, but because of
  ///   possible strangeness with volatile side effects this is marked as an
  ///   `unsafe` method.
  #[inline]
  #[must_use]
  #[cfg(feature = "experimental_volregion")]
  pub const unsafe fn as_voladdress(self) -> VolAddress<[T; C], R, W> {
    self.base.cast::<[T; C]>()
  }
}

#[test]
fn test_volblock_iter_range() {
  let block: VolBlock<u8, Unsafe, Unsafe, 10> = unsafe { VolBlock::new(1) };
  //
  let i = block.iter_range(..);
  assert_eq!(i.base.as_usize(), 1);
  assert_eq!(i.count, 10);
  //
  let i = block.iter_range(2..);
  assert_eq!(i.base.as_usize(), 1 + 2);
  assert_eq!(i.count, 10 - 2);
  //
  let i = block.iter_range(2..=5);
  assert_eq!(i.base.as_usize(), 1 + 2);
  assert_eq!(i.count, 4);
  //
  let i = block.iter_range(..4);
  assert_eq!(i.base.as_usize(), 1);
  assert_eq!(i.count, 4);
  //
  let i = block.iter_range(..=4);
  assert_eq!(i.base.as_usize(), 1);
  assert_eq!(i.count, 5);
}

#[test]
#[should_panic]
fn test_volblock_iter_range_low_bound_panic() {
  let block: VolBlock<u8, Unsafe, Unsafe, 10> = unsafe { VolBlock::new(1) };
  //
  let _i = block.iter_range(10..);
}

#[test]
#[should_panic]
fn test_volblock_iter_range_high_bound_panic() {
  let block: VolBlock<u8, Unsafe, Unsafe, 10> = unsafe { VolBlock::new(1) };
  //
  let _i = block.iter_range(..=10);
}

impl<T, R, W, const C: usize> core::fmt::Debug for VolBlock<T, R, W, C> {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    write!(f, "VolBlock<{elem_ty}, r{readability}, w{writeability}, c{count}>(0x{address:#X})",
      elem_ty = core::any::type_name::<T>(),
      readability=core::any::type_name::<R>(),
      writeability=core::any::type_name::<W>(),
      count=C,
      address=self.base.address.get())
  }
}

impl<T, R, W, const C: usize> core::fmt::Pointer for VolBlock<T, R, W, C> {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    write!(f, "0x{address:#X}", address = self.base.address.get())
  }
}

/// An iterator over a volatile block.
///
/// You will generally not construct types of this value yourself. Instead, you
/// obtain them via the [`VolBlock::iter`](VolBlock::iter) method.
#[repr(C)]
pub struct VolBlockIter<T, R, W> {
  pub(crate) base: VolAddress<T, R, W>,
  pub(crate) count: usize,
}

impl<T, R, W> Clone for VolBlockIter<T, R, W> {
  #[inline]
  #[must_use]
  fn clone(&self) -> Self {
    Self { base: self.base, count: self.count }
  }
}

impl<T, R, W> core::iter::Iterator for VolBlockIter<T, R, W> {
  type Item = VolAddress<T, R, W>;

  #[inline]
  fn nth(&mut self, n: usize) -> Option<Self::Item> {
    if n < self.count {
      let out = Some(unsafe { self.base.add(n) });
      self.count -= n + 1;
      self.base = unsafe { self.base.add(n + 1) };
      out
    } else {
      self.count = 0;
      None
    }
  }

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    self.nth(0)
  }

  #[inline]
  #[must_use]
  fn last(mut self) -> Option<Self::Item> {
    if self.count > 0 {
      self.nth(self.count - 1)
    } else {
      None
    }
  }

  #[inline]
  #[must_use]
  fn size_hint(&self) -> (usize, Option<usize>) {
    (self.count, Some(self.count))
  }

  #[inline]
  #[must_use]
  fn count(self) -> usize {
    self.count
  }
}

impl<T, R, W> core::iter::DoubleEndedIterator for VolBlockIter<T, R, W> {
  #[inline]
  fn next_back(&mut self) -> Option<Self::Item> {
    self.nth_back(0)
  }

  #[inline]
  fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
    if n < self.count {
      let out = Some(unsafe { self.base.add(self.count - (n + 1)) });
      self.count -= n + 1;
      out
    } else {
      self.count = 0;
      None
    }
  }
}

#[test]
fn test_impl_Iterator_for_VolBlockIter() {
  let i: VolBlockIter<u16, (), ()> = VolBlockIter {
    base: unsafe { VolAddress::new(core::mem::align_of::<u16>()) },
    count: 4,
  };

  let mut i_c = i.clone().map(|a| a.as_usize());
  assert_eq!(i_c.next(), Some(2));
  assert_eq!(i_c.next(), Some(4));
  assert_eq!(i_c.next(), Some(6));
  assert_eq!(i_c.next(), Some(8));
  assert_eq!(i_c.next(), None);
  assert_eq!(i_c.next(), None);

  let i_c = i.clone();
  assert_eq!(i_c.size_hint(), (4, Some(4)));

  let i_c = i.clone();
  assert_eq!(i_c.count(), 4);

  let i_c = i.clone().map(|a| a.as_usize());
  assert_eq!(i_c.last(), Some(8));

  let mut i_c = i.clone().map(|a| a.as_usize());
  assert_eq!(i_c.nth(0), Some(2));
  assert_eq!(i_c.nth(0), Some(4));
  assert_eq!(i_c.nth(0), Some(6));
  assert_eq!(i_c.nth(0), Some(8));
  assert_eq!(i_c.nth(0), None);
  assert_eq!(i_c.nth(0), None);

  let mut i_c = i.clone().map(|a| a.as_usize());
  assert_eq!(i_c.nth(1), Some(4));
  assert_eq!(i_c.nth(1), Some(8));
  assert_eq!(i_c.nth(1), None);
  assert_eq!(i_c.nth(1), None);

  let mut i_c = i.clone().map(|a| a.as_usize());
  assert_eq!(i_c.nth(2), Some(6));
  assert_eq!(i_c.nth(2), None);
  assert_eq!(i_c.nth(2), None);

  let mut i_c = i.clone().map(|a| a.as_usize());
  assert_eq!(i_c.nth(3), Some(8));
  assert_eq!(i_c.nth(3), None);
  assert_eq!(i_c.nth(3), None);

  let mut i_c = i.clone().map(|a| a.as_usize());
  assert_eq!(i_c.nth(4), None);
  assert_eq!(i_c.nth(4), None);
}

#[test]
fn test_impl_DoubleEndedIterator_for_VolBlockIter() {
  let i: VolBlockIter<u16, (), ()> = VolBlockIter {
    base: unsafe { VolAddress::new(core::mem::align_of::<u16>()) },
    count: 4,
  };

  let mut i_c = i.clone().map(|a| a.as_usize());
  assert_eq!(i_c.next_back(), Some(8));
  assert_eq!(i_c.next_back(), Some(6));
  assert_eq!(i_c.next_back(), Some(4));
  assert_eq!(i_c.next_back(), Some(2));
  assert_eq!(i_c.next_back(), None);
  assert_eq!(i_c.next_back(), None);

  let mut i_c = i.clone().map(|a| a.as_usize());
  assert_eq!(i_c.nth_back(0), Some(8));
  assert_eq!(i_c.nth_back(0), Some(6));
  assert_eq!(i_c.nth_back(0), Some(4));
  assert_eq!(i_c.nth_back(0), Some(2));
  assert_eq!(i_c.nth_back(0), None);
  assert_eq!(i_c.nth_back(0), None);

  let mut i_c = i.clone().map(|a| a.as_usize());
  assert_eq!(i_c.nth_back(1), Some(6));
  assert_eq!(i_c.nth_back(1), Some(2));
  assert_eq!(i_c.nth_back(1), None);
  assert_eq!(i_c.nth_back(1), None);

  let mut i_c = i.clone().map(|a| a.as_usize());
  assert_eq!(i_c.nth_back(2), Some(4));
  assert_eq!(i_c.nth_back(2), None);
  assert_eq!(i_c.nth_back(2), None);

  let mut i_c = i.clone().map(|a| a.as_usize());
  assert_eq!(i_c.nth_back(3), Some(2));
  assert_eq!(i_c.nth_back(3), None);
  assert_eq!(i_c.nth_back(3), None);

  let mut i_c = i.clone().map(|a| a.as_usize());
  assert_eq!(i_c.nth_back(4), None);
  assert_eq!(i_c.nth_back(4), None);
}
