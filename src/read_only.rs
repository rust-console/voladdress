//! This is like the top level module, but types here are read only.

use core::{cmp::Ordering, iter::FusedIterator, marker::PhantomData, num::NonZeroUsize};
use typenum::marker_traits::Unsigned;

/// As `VolAddress`, but read only.
#[repr(transparent)]
pub struct ROVolAddress<T> {
  address: NonZeroUsize,
  marker: PhantomData<*mut T>,
}
impl<T> Clone for ROVolAddress<T> {
  fn clone(&self) -> Self {
    *self
  }
}
impl<T> Copy for ROVolAddress<T> {}
impl<T> PartialEq for ROVolAddress<T> {
  fn eq(&self, other: &Self) -> bool {
    self.address == other.address
  }
}
impl<T> Eq for ROVolAddress<T> {}
impl<T> PartialOrd for ROVolAddress<T> {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.address.cmp(&other.address))
  }
}
impl<T> Ord for ROVolAddress<T> {
  fn cmp(&self, other: &Self) -> Ordering {
    self.address.cmp(&other.address)
  }
}
impl<T> core::fmt::Debug for ROVolAddress<T> {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    write!(f, "ROVolAddress({:p})", *self)
  }
}
impl<T> core::fmt::Pointer for ROVolAddress<T> {
  /// You can request pointer style to get _just_ the inner value with pointer
  /// formatting.
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    write!(f, "{:p}", self.address.get() as *mut T)
  }
}
impl<T> ROVolAddress<T> {
  /// Constructs a new address.
  ///
  /// # Safety
  ///
  /// You must follow the standard safety rules as outlined in the type docs.
  pub const unsafe fn new(address: usize) -> Self {
    Self {
      address: NonZeroUsize::new_unchecked(address),
      marker: PhantomData,
    }
  }

  /// Casts the type of `T` into type `Z`.
  ///
  /// # Safety
  ///
  /// You must follow the standard safety rules as outlined in the type docs.
  pub const unsafe fn cast<Z>(self) -> ROVolAddress<Z> {
    // Note(Lokathor): This can't be `Self` because the type parameter changes.
    ROVolAddress {
      address: self.address,
      marker: PhantomData,
    }
  }

  /// Offsets the address by `offset` slots (like `pointer::wrapping_offset`).
  ///
  /// # Safety
  ///
  /// You must follow the standard safety rules as outlined in the type docs.
  pub const unsafe fn offset(self, offset: isize) -> Self {
    Self {
      address: NonZeroUsize::new_unchecked(self.address.get().wrapping_add(offset as usize * core::mem::size_of::<T>())),
      marker: PhantomData,
    }
  }

  /// Checks that the current target type of this address is aligned at this
  /// address value.
  pub const fn is_aligned(self) -> bool {
    self.address.get() % core::mem::align_of::<T>() == 0
  }

  /// The `usize` value of this `ROVolAddress`.
  pub const fn to_usize(self) -> usize {
    self.address.get()
  }

  /// Makes an iterator starting here across the given number of slots.
  ///
  /// # Safety
  ///
  /// The normal safety rules must be correct for each address iterated over.
  pub const unsafe fn iter_slots(self, slots: usize) -> ROVolIter<T> {
    ROVolIter {
      vol_address: self,
      slots_remaining: slots,
    }
  }

  // non-const and never can be.

  /// Volatile reads a `Copy` value out of the address.
  pub fn read(self) -> T
  where
    T: Copy,
  {
    unsafe { (self.address.get() as *mut T).read_volatile() }
  }

  /// Volatile reads a value out of the address with no trait bound.
  ///
  /// # Safety
  ///
  /// This is _not_ a move, it forms a bit duplicate of the current value at the
  /// address. If `T` has a `Drop` trait that does anything it is up to you to
  /// ensure that repeated drops do not cause UB (such as a double free).
  pub unsafe fn read_non_copy(self) -> T {
    (self.address.get() as *mut T).read_volatile()
  }
}

/// A block of addresses all in a row, read only.
///
/// * The `C` parameter is the element count of the block.
pub struct ROVolBlock<T, C: Unsigned> {
  vol_address: ROVolAddress<T>,
  slot_count: PhantomData<C>,
}
impl<T, C: Unsigned> Clone for ROVolBlock<T, C> {
  fn clone(&self) -> Self {
    *self
  }
}
impl<T, C: Unsigned> Copy for ROVolBlock<T, C> {}
impl<T, C: Unsigned> PartialEq for ROVolBlock<T, C> {
  fn eq(&self, other: &Self) -> bool {
    self.vol_address == other.vol_address
  }
}
impl<T, C: Unsigned> Eq for ROVolBlock<T, C> {}
impl<T, C: Unsigned> core::fmt::Debug for ROVolBlock<T, C> {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    write!(f, "ROVolBlock({:p}, count={})", self.vol_address.address.get() as *mut T, C::USIZE)
  }
}
impl<T, C: Unsigned> ROVolBlock<T, C> {
  /// Constructs a new `ROVolBlock`.
  ///
  /// # Safety
  ///
  /// The given address must be a valid `ROVolAddress` at each position in the
  /// block for however many slots (`C`).
  pub const unsafe fn new(address: usize) -> Self {
    Self {
      vol_address: ROVolAddress::new(address),
      slot_count: PhantomData,
    }
  }

  /// The length of this block (in elements)
  pub const fn len(self) -> usize {
    C::USIZE
  }

  /// Gives an iterator over the slots of this block.
  pub const fn iter(self) -> ROVolIter<T> {
    ROVolIter {
      vol_address: self.vol_address,
      slots_remaining: C::USIZE,
    }
  }

  /// Unchecked indexing into the block.
  ///
  /// # Safety
  ///
  /// The slot given must be in bounds.
  pub const unsafe fn index_unchecked(self, slot: usize) -> ROVolAddress<T> {
    self.vol_address.offset(slot as isize)
  }

  /// Checked "indexing" style access of the block, giving either a `ROVolAddress` or a panic.
  pub fn index(self, slot: usize) -> ROVolAddress<T> {
    if slot < C::USIZE {
      unsafe { self.index_unchecked(slot) }
    } else {
      panic!("Index Requested: {} >= Slot Count: {}", slot, C::USIZE)
    }
  }

  /// Checked "getting" style access of the block, giving an Option value.
  pub fn get(self, slot: usize) -> Option<ROVolAddress<T>> {
    if slot < C::USIZE {
      unsafe { Some(self.index_unchecked(slot)) }
    } else {
      None
    }
  }
}

/// A series of evenly strided addresses, read only.
///
/// * The `C` parameter is the element count of the series.
/// * The `S` parameter is the stride (in bytes) from one element to the next.
pub struct ROVolSeries<T, C: Unsigned, S: Unsigned> {
  vol_address: ROVolAddress<T>,
  slot_count: PhantomData<C>,
  stride: PhantomData<S>,
}
impl<T, C: Unsigned, S: Unsigned> Clone for ROVolSeries<T, C, S> {
  fn clone(&self) -> Self {
    *self
  }
}
impl<T, C: Unsigned, S: Unsigned> Copy for ROVolSeries<T, C, S> {}
impl<T, C: Unsigned, S: Unsigned> PartialEq for ROVolSeries<T, C, S> {
  fn eq(&self, other: &Self) -> bool {
    self.vol_address == other.vol_address
  }
}
impl<T, C: Unsigned, S: Unsigned> Eq for ROVolSeries<T, C, S> {}
impl<T, C: Unsigned, S: Unsigned> core::fmt::Debug for ROVolSeries<T, C, S> {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    write!(
      f,
      "ROVolSeries({:p}, count={}, series={})",
      self.vol_address.address.get() as *mut T,
      C::USIZE,
      S::USIZE
    )
  }
}
impl<T, C: Unsigned, S: Unsigned> ROVolSeries<T, C, S> {
  /// Constructs a new `ROVolSeries`.
  ///
  /// # Safety
  ///
  /// The given address must be a valid `ROVolAddress` at each position in the
  /// series for however many slots (`C`), strided by the selected amount (`S`).
  pub const unsafe fn new(address: usize) -> Self {
    Self {
      vol_address: ROVolAddress::new(address),
      slot_count: PhantomData,
      stride: PhantomData,
    }
  }

  /// The length of this series (in elements)
  pub const fn len(self) -> usize {
    C::USIZE
  }

  /// Gives an iterator over the slots of this series.
  pub const fn iter(self) -> ROVolStridingIter<T, S> {
    ROVolStridingIter {
      vol_address: self.vol_address,
      slots_remaining: C::USIZE,
      stride: PhantomData,
    }
  }

  /// Unchecked indexing into the series.
  ///
  /// # Safety
  ///
  /// The slot given must be in bounds.
  pub const unsafe fn index_unchecked(self, slot: usize) -> ROVolAddress<T> {
    self.vol_address.cast::<u8>().offset((S::USIZE * slot) as isize).cast::<T>()
  }

  /// Checked "indexing" style access into the series, giving either a `ROVolAddress` or a panic.
  pub fn index(self, slot: usize) -> ROVolAddress<T> {
    if slot < C::USIZE {
      unsafe { self.index_unchecked(slot) }
    } else {
      panic!("Index Requested: {} >= Slot Count: {}", slot, C::USIZE)
    }
  }

  /// Checked "getting" style access into the series, giving an Option value.
  pub fn get(self, slot: usize) -> Option<ROVolAddress<T>> {
    if slot < C::USIZE {
      unsafe { Some(self.index_unchecked(slot)) }
    } else {
      None
    }
  }
}

/// An iterator that produces consecutive `ROVolAddress` values.
pub struct ROVolIter<T> {
  vol_address: ROVolAddress<T>,
  slots_remaining: usize,
}
impl<T> Clone for ROVolIter<T> {
  fn clone(&self) -> Self {
    Self {
      vol_address: self.vol_address,
      slots_remaining: self.slots_remaining,
    }
  }
}
impl<T> PartialEq for ROVolIter<T> {
  fn eq(&self, other: &Self) -> bool {
    self.vol_address == other.vol_address && self.slots_remaining == other.slots_remaining
  }
}
impl<T> Eq for ROVolIter<T> {}
impl<T> Iterator for ROVolIter<T> {
  type Item = ROVolAddress<T>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.slots_remaining > 0 {
      let out = self.vol_address;
      unsafe {
        self.slots_remaining -= 1;
        self.vol_address = self.vol_address.offset(1);
      }
      Some(out)
    } else {
      None
    }
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    (self.slots_remaining, Some(self.slots_remaining))
  }

  fn count(self) -> usize {
    self.slots_remaining
  }

  fn last(self) -> Option<Self::Item> {
    if self.slots_remaining > 0 {
      Some(unsafe { self.vol_address.offset(self.slots_remaining as isize) })
    } else {
      None
    }
  }

  fn nth(&mut self, n: usize) -> Option<Self::Item> {
    if self.slots_remaining > n {
      // somewhere in bounds
      unsafe {
        let out = self.vol_address.offset(n as isize);
        let jump = n + 1;
        self.slots_remaining -= jump;
        self.vol_address = self.vol_address.offset(jump as isize);
        Some(out)
      }
    } else {
      // out of bounds!
      self.slots_remaining = 0;
      None
    }
  }

  fn max(self) -> Option<Self::Item> {
    self.last()
  }

  fn min(mut self) -> Option<Self::Item> {
    self.nth(0)
  }
}
impl<T> FusedIterator for ROVolIter<T> {}
impl<T> core::fmt::Debug for ROVolIter<T> {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    write!(
      f,
      "ROVolIter({:p}, remaining={})",
      self.vol_address.address.get() as *mut T,
      self.slots_remaining
    )
  }
}

/// An iterator that produces strided `ROVolAddress` values.
pub struct ROVolStridingIter<T, S: Unsigned> {
  vol_address: ROVolAddress<T>,
  slots_remaining: usize,
  stride: PhantomData<S>,
}
impl<T, S: Unsigned> Clone for ROVolStridingIter<T, S> {
  fn clone(&self) -> Self {
    Self {
      vol_address: self.vol_address,
      slots_remaining: self.slots_remaining,
      stride: PhantomData,
    }
  }
}
impl<T, S: Unsigned> PartialEq for ROVolStridingIter<T, S> {
  fn eq(&self, other: &Self) -> bool {
    self.vol_address == other.vol_address && self.slots_remaining == other.slots_remaining
  }
}
impl<T, S: Unsigned> Eq for ROVolStridingIter<T, S> {}
impl<T, S: Unsigned> Iterator for ROVolStridingIter<T, S> {
  type Item = ROVolAddress<T>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.slots_remaining > 0 {
      let out = self.vol_address;
      unsafe {
        self.slots_remaining -= 1;
        self.vol_address = self.vol_address.cast::<u8>().offset(S::ISIZE).cast::<T>();
      }
      Some(out)
    } else {
      None
    }
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    (self.slots_remaining, Some(self.slots_remaining))
  }

  fn count(self) -> usize {
    self.slots_remaining
  }

  fn last(self) -> Option<Self::Item> {
    if self.slots_remaining > 0 {
      Some(unsafe {
        self
          .vol_address
          .cast::<u8>()
          .offset(S::ISIZE * (self.slots_remaining as isize))
          .cast::<T>()
      })
    } else {
      None
    }
  }

  fn nth(&mut self, n: usize) -> Option<Self::Item> {
    if self.slots_remaining > n {
      // somewhere in bounds
      unsafe {
        let out = self.vol_address.cast::<u8>().offset(S::ISIZE * (n as isize)).cast::<T>();
        let jump = n + 1;
        self.slots_remaining -= jump;
        self.vol_address = self.vol_address.cast::<u8>().offset(S::ISIZE * (jump as isize)).cast::<T>();
        Some(out)
      }
    } else {
      // out of bounds!
      self.slots_remaining = 0;
      None
    }
  }

  fn max(self) -> Option<Self::Item> {
    self.last()
  }

  fn min(mut self) -> Option<Self::Item> {
    self.nth(0)
  }
}
impl<T, S: Unsigned> FusedIterator for ROVolStridingIter<T, S> {}
impl<T, S: Unsigned> core::fmt::Debug for ROVolStridingIter<T, S> {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    write!(
      f,
      "ROVolStridingIter({:p}, remaining={}, stride={})",
      self.vol_address.address.get() as *mut T,
      self.slots_remaining,
      S::USIZE
    )
  }
}
