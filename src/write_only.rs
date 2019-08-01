//! This is like the top level module, but types here are write only.

use core::{cmp::Ordering, iter::FusedIterator, marker::PhantomData, num::NonZeroUsize};
use typenum::marker_traits::Unsigned;

/// As `VolAddress`, but write only.
#[repr(transparent)]
pub struct WOVolAddress<T> {
  address: NonZeroUsize,
  marker: PhantomData<*mut T>,
}
impl<T> Clone for WOVolAddress<T> {
  #[inline(always)]
  fn clone(&self) -> Self {
    *self
  }
}
impl<T> Copy for WOVolAddress<T> {}
impl<T> PartialEq for WOVolAddress<T> {
  #[inline(always)]
  fn eq(&self, other: &Self) -> bool {
    self.address == other.address
  }
}
impl<T> Eq for WOVolAddress<T> {}
impl<T> PartialOrd for WOVolAddress<T> {
  #[inline(always)]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.address.cmp(&other.address))
  }
}
impl<T> Ord for WOVolAddress<T> {
  #[inline(always)]
  fn cmp(&self, other: &Self) -> Ordering {
    self.address.cmp(&other.address)
  }
}
impl<T> core::fmt::Debug for WOVolAddress<T> {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    write!(f, "WOVolAddress({:p})", *self)
  }
}
impl<T> core::fmt::Pointer for WOVolAddress<T> {
  /// You can request pointer style to get _just_ the inner value with pointer
  /// formatting.
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    write!(f, "{:p}", self.address.get() as *mut T)
  }
}
impl<T> WOVolAddress<T> {
  /// Constructs a new address.
  ///
  /// # Safety
  ///
  /// You must follow the standard safety rules as outlined in the type docs.
  #[inline(always)]
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
  #[inline(always)]
  pub const unsafe fn cast<Z>(self) -> WOVolAddress<Z> {
    // Note(Lokathor): This can't be `Self` because the type parameter changes.
    WOVolAddress {
      address: self.address,
      marker: PhantomData,
    }
  }

  /// Offsets the address by `offset` slots (like `pointer::wrapping_offset`).
  ///
  /// # Safety
  ///
  /// You must follow the standard safety rules as outlined in the type docs.
  #[inline(always)]
  pub const unsafe fn offset(self, offset: isize) -> Self {
    Self {
      address: NonZeroUsize::new_unchecked(self.address.get().wrapping_add(offset as usize * core::mem::size_of::<T>())),
      marker: PhantomData,
    }
  }

  /// Checks that the current target type of this address is aligned at this
  /// address value.
  #[inline(always)]
  pub const fn is_aligned(self) -> bool {
    self.address.get() % core::mem::align_of::<T>() == 0
  }

  /// The `usize` value of this `WOVolAddress`.
  #[inline(always)]
  pub const fn to_usize(self) -> usize {
    self.address.get()
  }

  /// Makes an iterator starting here across the given number of slots.
  ///
  /// # Safety
  ///
  /// The normal safety rules must be correct for each address iterated over.
  #[inline(always)]
  pub const unsafe fn iter_slots(self, slots: usize) -> WOVolIter<T> {
    WOVolIter {
      vol_address: self,
      slots_remaining: slots,
    }
  }

  /// Volatile writes a value to the address.
  ///
  /// Semantically, the value is moved into the function and then forgotten, so
  /// if `T` has a `Drop` impl then that will never get executed. This is "safe"
  /// under Rust's safety rules, but could cause something unintended (eg: a
  /// memory leak).
  #[inline(always)]
  pub fn write(self, val: T) {
    unsafe { (self.address.get() as *mut T).write_volatile(val) }
  }
}

/// A block of addresses all in a row, write only.
///
/// * The `C` parameter is the element count of the block.
pub struct WOVolBlock<T, C: Unsigned> {
  vol_address: WOVolAddress<T>,
  slot_count: PhantomData<C>,
}
impl<T, C: Unsigned> Clone for WOVolBlock<T, C> {
  #[inline(always)]
  fn clone(&self) -> Self {
    *self
  }
}
impl<T, C: Unsigned> Copy for WOVolBlock<T, C> {}
impl<T, C: Unsigned> PartialEq for WOVolBlock<T, C> {
  #[inline(always)]
  fn eq(&self, other: &Self) -> bool {
    self.vol_address == other.vol_address
  }
}
impl<T, C: Unsigned> Eq for WOVolBlock<T, C> {}
impl<T, C: Unsigned> core::fmt::Debug for WOVolBlock<T, C> {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    write!(f, "WOVolBlock({:p}, count={})", self.vol_address.address.get() as *mut T, C::USIZE)
  }
}
impl<T, C: Unsigned> WOVolBlock<T, C> {
  /// Constructs a new `WOVolBlock`.
  ///
  /// # Safety
  ///
  /// The given address must be a valid `WOVolAddress` at each position in the
  /// block for however many slots (`C`).
  #[inline(always)]
  pub const unsafe fn new(address: usize) -> Self {
    Self {
      vol_address: WOVolAddress::new(address),
      slot_count: PhantomData,
    }
  }

  /// The length of this block (in elements)
  #[inline(always)]
  pub const fn len(self) -> usize {
    C::USIZE
  }

  /// Gives an iterator over the slots of this block.
  #[inline(always)]
  pub const fn iter(self) -> WOVolIter<T> {
    WOVolIter {
      vol_address: self.vol_address,
      slots_remaining: C::USIZE,
    }
  }

  /// Unchecked indexing into the block.
  ///
  /// # Safety
  ///
  /// The slot given must be in bounds.
  #[inline(always)]
  pub const unsafe fn index_unchecked(self, slot: usize) -> WOVolAddress<T> {
    self.vol_address.offset(slot as isize)
  }

  /// Checked "indexing" style access of the block, giving either a
  /// `WOVolAddress` or a panic.
  #[inline(always)]
  pub fn index(self, slot: usize) -> WOVolAddress<T> {
    if slot < C::USIZE {
      unsafe { self.index_unchecked(slot) }
    } else {
      panic!("Index Requested: {} >= Slot Count: {}", slot, C::USIZE)
    }
  }

  /// Checked "getting" style access of the block, giving an Option value.
  #[inline(always)]
  pub fn get(self, slot: usize) -> Option<WOVolAddress<T>> {
    if slot < C::USIZE {
      unsafe { Some(self.index_unchecked(slot)) }
    } else {
      None
    }
  }
}

/// A series of evenly strided addresses, write only.
///
/// * The `C` parameter is the element count of the series.
/// * The `S` parameter is the stride (in bytes) from one element to the next.
pub struct WOVolSeries<T, C: Unsigned, S: Unsigned> {
  vol_address: WOVolAddress<T>,
  slot_count: PhantomData<C>,
  stride: PhantomData<S>,
}
impl<T, C: Unsigned, S: Unsigned> Clone for WOVolSeries<T, C, S> {
  #[inline(always)]
  fn clone(&self) -> Self {
    *self
  }
}
impl<T, C: Unsigned, S: Unsigned> Copy for WOVolSeries<T, C, S> {}
impl<T, C: Unsigned, S: Unsigned> PartialEq for WOVolSeries<T, C, S> {
  #[inline(always)]
  fn eq(&self, other: &Self) -> bool {
    self.vol_address == other.vol_address
  }
}
impl<T, C: Unsigned, S: Unsigned> Eq for WOVolSeries<T, C, S> {}
impl<T, C: Unsigned, S: Unsigned> core::fmt::Debug for WOVolSeries<T, C, S> {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    write!(
      f,
      "WOVolSeries({:p}, count={}, series={})",
      self.vol_address.address.get() as *mut T,
      C::USIZE,
      S::USIZE
    )
  }
}
impl<T, C: Unsigned, S: Unsigned> WOVolSeries<T, C, S> {
  /// Constructs a new `WOVolSeries`.
  ///
  /// # Safety
  ///
  /// The given address must be a valid `WOVolAddress` at each position in the
  /// series for however many slots (`C`), strided by the selected amount (`S`).
  #[inline(always)]
  pub const unsafe fn new(address: usize) -> Self {
    Self {
      vol_address: WOVolAddress::new(address),
      slot_count: PhantomData,
      stride: PhantomData,
    }
  }

  /// The length of this series (in elements)
  #[inline(always)]
  pub const fn len(self) -> usize {
    C::USIZE
  }

  /// Gives an iterator over the slots of this series.
  #[inline(always)]
  pub const fn iter(self) -> WOVolStridingIter<T, S> {
    WOVolStridingIter {
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
  #[inline(always)]
  pub const unsafe fn index_unchecked(self, slot: usize) -> WOVolAddress<T> {
    self.vol_address.cast::<u8>().offset((S::USIZE * slot) as isize).cast::<T>()
  }

  /// Checked "indexing" style access into the series, giving either a
  /// `WOVolAddress` or a panic.
  #[inline(always)]
  pub fn index(self, slot: usize) -> WOVolAddress<T> {
    if slot < C::USIZE {
      unsafe { self.index_unchecked(slot) }
    } else {
      panic!("Index Requested: {} >= Slot Count: {}", slot, C::USIZE)
    }
  }

  /// Checked "getting" style access into the series, giving an Option value.
  #[inline(always)]
  pub fn get(self, slot: usize) -> Option<WOVolAddress<T>> {
    if slot < C::USIZE {
      unsafe { Some(self.index_unchecked(slot)) }
    } else {
      None
    }
  }
}

/// An iterator that produces consecutive `WOVolAddress` values.
pub struct WOVolIter<T> {
  vol_address: WOVolAddress<T>,
  slots_remaining: usize,
}
impl<T> Clone for WOVolIter<T> {
  #[inline(always)]
  fn clone(&self) -> Self {
    Self {
      vol_address: self.vol_address,
      slots_remaining: self.slots_remaining,
    }
  }
}
impl<T> PartialEq for WOVolIter<T> {
  #[inline(always)]
  fn eq(&self, other: &Self) -> bool {
    self.vol_address == other.vol_address && self.slots_remaining == other.slots_remaining
  }
}
impl<T> Eq for WOVolIter<T> {}
impl<T> Iterator for WOVolIter<T> {
  type Item = WOVolAddress<T>;

  #[inline]
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

  #[inline(always)]
  fn size_hint(&self) -> (usize, Option<usize>) {
    (self.slots_remaining, Some(self.slots_remaining))
  }

  #[inline(always)]
  fn count(self) -> usize {
    self.slots_remaining
  }

  #[inline(always)]
  fn last(self) -> Option<Self::Item> {
    if self.slots_remaining > 0 {
      Some(unsafe { self.vol_address.offset(self.slots_remaining as isize) })
    } else {
      None
    }
  }

  #[inline]
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

  #[inline(always)]
  fn max(self) -> Option<Self::Item> {
    self.last()
  }

  #[inline(always)]
  fn min(mut self) -> Option<Self::Item> {
    self.nth(0)
  }
}
impl<T> FusedIterator for WOVolIter<T> {}
impl<T> core::fmt::Debug for WOVolIter<T> {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    write!(
      f,
      "WOVolIter({:p}, remaining={})",
      self.vol_address.address.get() as *mut T,
      self.slots_remaining
    )
  }
}

/// An iterator that produces strided `WOVolAddress` values.
pub struct WOVolStridingIter<T, S: Unsigned> {
  vol_address: WOVolAddress<T>,
  slots_remaining: usize,
  stride: PhantomData<S>,
}
impl<T, S: Unsigned> Clone for WOVolStridingIter<T, S> {
  #[inline(always)]
  fn clone(&self) -> Self {
    Self {
      vol_address: self.vol_address,
      slots_remaining: self.slots_remaining,
      stride: PhantomData,
    }
  }
}
impl<T, S: Unsigned> PartialEq for WOVolStridingIter<T, S> {
  #[inline(always)]
  fn eq(&self, other: &Self) -> bool {
    self.vol_address == other.vol_address && self.slots_remaining == other.slots_remaining
  }
}
impl<T, S: Unsigned> Eq for WOVolStridingIter<T, S> {}
impl<T, S: Unsigned> Iterator for WOVolStridingIter<T, S> {
  type Item = WOVolAddress<T>;

  #[inline]
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

  #[inline(always)]
  fn size_hint(&self) -> (usize, Option<usize>) {
    (self.slots_remaining, Some(self.slots_remaining))
  }

  #[inline(always)]
  fn count(self) -> usize {
    self.slots_remaining
  }

  #[inline(always)]
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

  #[inline]
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

  #[inline(always)]
  fn max(self) -> Option<Self::Item> {
    self.last()
  }

  #[inline(always)]
  fn min(mut self) -> Option<Self::Item> {
    self.nth(0)
  }
}
impl<T, S: Unsigned> FusedIterator for WOVolStridingIter<T, S> {}
impl<T, S: Unsigned> core::fmt::Debug for WOVolStridingIter<T, S> {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    write!(
      f,
      "WOVolStridingIter({:p}, remaining={}, stride={})",
      self.vol_address.address.get() as *mut T,
      self.slots_remaining,
      S::USIZE
    )
  }
}
