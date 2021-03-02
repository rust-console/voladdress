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
  base: VolAddress<T, R, W>,
}

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
  pub const unsafe fn len(self) -> usize {
    C
  }

  /// Indexes to the `i`th position of the memory block.
  ///
  /// ## Panics
  /// * If the index is out of bounds this will panic.
  #[inline]
  #[must_use]
  #[track_caller]
  pub const fn index(self, i: usize) -> VolAddress<T, R, W> {
    if i < C {
      unsafe { self.base.add(i) }
    } else {
      // Note(Lokathor): We force a const panic by indexing out of bounds.
      #[allow(unconditional_panic)]
      unsafe {
        VolAddress::new([usize::MAX][1])
      }
    }
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
}

impl<T, R, W, const C: usize> Clone for VolBlock<T, R, W, C> {
  #[inline]
  #[must_use]
  fn clone(&self) -> Self {
    *self
  }
}
impl<T, R, W, const C: usize> Copy for VolBlock<T, R, W, C> {}

impl<T, R, W, const C: usize> core::fmt::Debug for VolBlock<T, R, W, C> {
  #[cold]
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    write!(f, "VolBlock<{elem_ty}, r{readability}, w{writeability}, c{count}>({address:#X})",
      elem_ty = core::any::type_name::<T>(),
      readability=core::any::type_name::<R>(),
      writeability=core::any::type_name::<W>(),
      count=C,
      address=self.base.address.get())
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
  fn next(&mut self) -> Option<Self::Item> {
    if self.count > 0 {
      let out = Some(self.base);
      self.count -= 1;
      self.base = unsafe { self.base.add(1) };
      out
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

  #[inline]
  #[must_use]
  fn last(self) -> Option<Self::Item> {
    if self.count > 0 {
      Some(unsafe { self.base.add(self.count - 1) })
    } else {
      None
    }
  }

  #[inline]
  fn nth(&mut self, n: usize) -> Option<Self::Item> {
    if n < self.count {
      self.count -= n;
      self.base = unsafe { self.base.add(1 + n) };
      Some(self.base)
    } else {
      self.count = 0;
      None
    }
  }
}

impl<T, R, W> core::iter::DoubleEndedIterator for VolBlockIter<T, R, W> {
  #[inline]
  fn next_back(&mut self) -> Option<Self::Item> {
    if self.count > 0 {
      let out = Some(unsafe { self.base.add(self.count - 1) });
      self.count -= 1;
      out
    } else {
      None
    }
  }

  #[inline]
  fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
    if n < self.count {
      self.count -= n;
      Some(unsafe { self.base.add(1 + n) })
    } else {
      self.count = 0;
      None
    }
  }
}

#[test]
#[allow(bad_style)]
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
#[allow(bad_style)]
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
