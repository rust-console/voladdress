// Note(Lokathor): Required to allow for marker trait bounds on const functions.
#![no_std]
#![feature(const_fn, const_generics)]
#![allow(incomplete_features)]
#![forbid(missing_docs)]
#![forbid(missing_debug_implementations)]
#![allow(clippy::len_without_is_empty)]

//! `voladdress` is a crate that makes it easy to work with volatile memory
//! addresses (eg: memory mapped hardware).
//!
//! When working with volatile memory, it's assumed that you'll generally be
//! working with one or more of:
//!
//! * A single address (`VolAddress`)
//! * A block of contiguous memory addresses (`VolBlock`)
//! * A series of evenly strided memory addresses (`VolSeries`)
//!
//! All the types have `unsafe` _creation_ and then safe _use_, so that the
//! actual usage is as ergonomic as possible. Obviously you tend to use an
//! address far more often than you name an address, so that should be the best
//! part of the experience. Iterators are also provided for the `VolBlock` and
//! `VolSeries` types.
//!
//! For example, on the GBA there's a palette of 256  color values (`u16`) for
//! the background palette starting at `0x500_0000`, so you might write
//! something like
//!
//! ```rust
//! use voladdress::{VolBlock, VolAddress};
//!
//! pub type Color = u16;
//!
//! pub const PALRAM_BG: VolBlock<Color, 256> = unsafe { VolBlock::new(0x500_0000) };
//! ```
//!
//! And then in your actual program you might do something like this
//!
//! ```rust
//! # use voladdress::{VolBlock, VolAddress};
//! # pub type Color = u16;
//! fn main() {
//! #  let palram = vec![0u16; 256];
//! #  let PALRAM_BG: VolBlock<Color, 256> = unsafe { VolBlock::new(palram.as_ptr() as usize) };
//!   let i = 5;
//!   // the palette is all 0 (black) at startup.
//!   assert_eq!(PALRAM_BG.index(i).read(), 0);
//!   // we can make that index blue instead.
//!   const BLUE: u16 = 0b11111;
//!   PALRAM_BG.index(i).write(BLUE);
//!   assert_eq!(PALRAM_BG.index(i).read(), BLUE);
//! }
//! ```
//!
//! You _could_ use an address of any `*mut T` that you have (which is how the
//! tests and doctests work), but the _intent_ is that you use this crate with
//! memory mapped hardware. Exactly what hardware is memory mapped where depends
//! on your target device. Please read your target device's documentation.
//!
//! # Why Use This?
//!
//! It may seem rather silly to have special types for what is basically a `*mut
//! T`. However, when reading and writing with a normal pointer (eg: `*ptr` or
//! `*ptr = x;`) Rust will desugar that to the
//! [read](https://doc.rust-lang.org/core/ptr/fn.read.html) and
//! [write](https://doc.rust-lang.org/core/ptr/fn.write.html) functions. The
//! compiler is allowed to elide these accesses if it "knows" what the value is
//! already going to be, or if it "knows" that the read will never be seen.
//! However, when working with memory mapped hardware the read and write
//! operations have various side effects that the compiler isn't aware of, so
//! the access must not be elided. You have to use
//! [read_volatile](https://doc.rust-lang.org/core/ptr/fn.read_volatile.html)
//! and
//! [write_volatile](https://doc.rust-lang.org/core/ptr/fn.write_volatile.html),
//! which are immune to being elided by the compiler. The rust standard library
//! doesn't have a way to "tag" a pointer as being volatile to force that
//! volatile access always be used, and so we have this crate.
//!
//! There are other crates that address the general issue of volatile memory,
//! but none that I've seen are as easy to use as this one. They generally
//! expect you to cast the target address (a `usize` that you get out of your
//! hardware manual) into a raw pointer to their crate's volatile type (eg: `let
//! p = 1234 as *mut VolatileCell<u16>`), but then you have to dereference that
//! raw pointer _each time_ you call read or write, and it always requires
//! parenthesis too, because of prescience rules (eg: `let a = (*p).read();`).
//! You end up with `unsafe` blocks and parens and asterisks all over the code
//! for no benefit.
//!
//! This crate is much better than any of that. Once you've decided that the
//! initial unsafety is alright, and you've created a `VolAddress` value for
//! your target type at the target address, the `read` and `write` methods are
//! entirely safe to use and don't require the manual de-reference.
//!
//! # Can't you impl `Deref`/`DerefMut` and `Index`/`IndexMut` on these things?
//!
//! No. Absolutely not. They all return `&T` or `&mut T`, which use normal reads
//! and writes, so the accesses can be elided by the compiler. In fact
//! references end up being _more_ aggressive about access elision than happens
//! raw pointers. For standard code this is exactly what we want (it makes the
//! code faster to skip reads and writes we don't need), but with memory mapped
//! hardware this is the opposite of a good time.

use core::{cmp::Ordering, iter::FusedIterator, marker::PhantomData, num::NonZeroUsize};

pub mod read_only;
pub mod write_only;

// Note(Lokathor): We have to hand implement all the traits for all of our types
// manually because if we use `derive` then they only get derived if the `T` has
// that trait. However, since we're acting like various "pointers" to `T`
// values, the capabilities we offer aren't at all affected by whatever type `T`
// ends up being.

/// Abstracts the use of a volatile memory address.
///
/// If you're trying to do anything other than abstract a memory mapped hardware
/// device then you probably want one of the many other smart pointer types in
/// the standard library.
///
/// It's generally expected that you'll create `VolAddress` values by declaring
/// `const` globals at various points in your code for the various memory
/// locations of the device. This is fine, but please note that volatile access
/// is **not synchronized** and you'll have to arrange for synchronization in
/// some way if you intend to have a multi-threaded program.
///
/// An interrupt running on a core can safely communicate with the main program
/// running **on that same core** if both are using volatile access to the same
/// location. Of course, since you generally can't be sure what core you're
/// going to be running on, this "trick" should only be used for single-core
/// devices.
///
/// # Safety
///
/// In order for values of this type to operate correctly they must follow quite
/// a few safety limits:
///
/// * The declared address must always be
///   "[valid](https://doc.rust-lang.org/core/ptr/index.html#safety)" according
///   to the rules of `core::ptr`.
/// * To be extra clear: the declared address must be non-zero because this type
///   uses the `NonZeroUsize` type internally (it makes the iterators a lot
///   better). It's possible to have a device memory mapped to the zero address,
///   but it is not ever valid to access the null address from within Rust. For
///   that rare situation you'd need to use inline assembly.
/// * The declared address must be aligned for the declared type of `T`.
/// * The declared address must always read as a valid bit pattern for the type
///   `T`, regardless of the state of the memory mapped hardware. If there's any
///   doubt at all, you must instead read or write an unsigned int of the
///   correct bit size (`u16`, `u32`, etc) and then parse the bits by hand.
/// * Any `VolAddress` declared as a compile time `const` must not use a
///   location that would ever be part of any allocator and/or stack frame.
/// * Any `VolAddress` made at runtime from a `*mut` pointer is only valid as
///   long as that `*mut` would be valid, _this is not tracked_ because pointers
///   don't have lifetimes.
///
/// If you're not sure about any of those points, please re-read the hardware
/// specs of your target device and its memory map until you know for sure.
///
/// The _exact_ points of UB are if the address is ever 0 (because it's stored
/// as `NonNullUsize`), or if you ever actually `read` or `write` with an
/// invalidly constructed `VolAddress`.
#[repr(transparent)]
pub struct VolAddress<T> {
  address: NonZeroUsize,
  marker: PhantomData<*mut T>,
}
impl<T> Clone for VolAddress<T> {
  #[inline(always)]
  fn clone(&self) -> Self {
    *self
  }
}
impl<T> Copy for VolAddress<T> {}
impl<T> PartialEq for VolAddress<T> {
  #[inline(always)]
  fn eq(&self, other: &Self) -> bool {
    self.address == other.address
  }
}
impl<T> Eq for VolAddress<T> {}
impl<T> PartialOrd for VolAddress<T> {
  #[inline(always)]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.address.cmp(&other.address))
  }
}
impl<T> Ord for VolAddress<T> {
  #[inline(always)]
  fn cmp(&self, other: &Self) -> Ordering {
    self.address.cmp(&other.address)
  }
}
impl<T> core::fmt::Debug for VolAddress<T> {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    write!(f, "VolAddress({:p})", *self)
  }
}
impl<T> core::fmt::Pointer for VolAddress<T> {
  /// You can request pointer style to get _just_ the inner value with pointer
  /// formatting.
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    write!(f, "{:p}", self.address.get() as *mut T)
  }
}
impl<T> VolAddress<T> {
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
  pub const unsafe fn cast<Z>(self) -> VolAddress<Z> {
    // Note(Lokathor): This can't be `Self` because the type parameter changes.
    VolAddress {
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
  ///
  /// Technically it's a safety violation to even make a `VolAddress` that isn't
  /// aligned. However, I know you're gonna try doing the bad thing, and it's
  /// better to give you a chance to call `is_aligned` and potentially back off
  /// from the operation or throw a `debug_assert!` or something instead of
  /// triggering UB.
  #[inline(always)]
  pub const fn is_aligned(self) -> bool {
    self.address.get() % core::mem::align_of::<T>() == 0
  }

  /// The `usize` value of this `VolAddress`.
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
  pub const unsafe fn iter_slots(self, slots: usize) -> VolIter<T> {
    VolIter {
      vol_address: self,
      slots_remaining: slots,
    }
  }

  // non-const and never can be.

  /// Volatile reads a `Copy` value out of the address.
  ///
  /// The `Copy` bound is actually supposed to be `!Drop`, but rust doesn't
  /// allow negative trait bounds. If your type isn't `Copy` you can use the
  /// `read_non_copy` fallback to do an unsafe read.
  ///
  /// That said, I don't think that you legitimately have hardware that maps to
  /// a Rust type that isn't `Copy`. If you do please tell me, I'm interested to
  /// hear about it.
  #[inline(always)]
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
  #[inline(always)]
  pub unsafe fn read_non_copy(self) -> T {
    (self.address.get() as *mut T).read_volatile()
  }

  /// Volatile writes a value to the address.
  ///
  /// Semantically, the value is moved into the `VolAddress` and then forgotten,
  /// so if `T` has a `Drop` impl then that will never get executed. This is
  /// "safe" under Rust's safety rules, but could cause something unintended
  /// (eg: a memory leak).
  #[inline(always)]
  pub fn write(self, val: T) {
    unsafe { (self.address.get() as *mut T).write_volatile(val) }
  }
}

/// A block of addresses all in a row.
///
/// * The `C` parameter is the element count of the block.
///
/// This is for if you have something like "a block of 256 `u16` values all in a
/// row starting at `0x500_0000`".
pub struct VolBlock<T, const COUNT: usize> {
  vol_address: VolAddress<T>,
}
impl<T, const COUNT: usize> Clone for VolBlock<T, COUNT> {
  #[inline(always)]
  fn clone(&self) -> Self {
    *self
  }
}
impl<T, const COUNT: usize> Copy for VolBlock<T, COUNT> {}
impl<T, const COUNT: usize> PartialEq for VolBlock<T, COUNT> {
  #[inline(always)]
  fn eq(&self, other: &Self) -> bool {
    self.vol_address == other.vol_address
  }
}
impl<T, const COUNT: usize> Eq for VolBlock<T, COUNT> {}
impl<T, const COUNT: usize> core::fmt::Debug for VolBlock<T, COUNT> {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    write!(f, "VolBlock({:p}, count={})", self.vol_address.address.get() as *mut T, COUNT)
  }
}
impl<T, const COUNT: usize> VolBlock<T, COUNT> {
  /// Constructs a new `VolBlock`.
  ///
  /// # Safety
  ///
  /// The given address must be a valid `VolAddress` at each position in the
  /// block for however many slots (`C`).
  #[inline(always)]
  pub const unsafe fn new(address: usize) -> Self {
    Self {
      vol_address: VolAddress::new(address),
    }
  }

  /// The length of this block (in elements)
  #[inline(always)]
  pub const fn len(self) -> usize {
    COUNT
  }

  /// Gives an iterator over the slots of this block.
  #[inline(always)]
  pub const fn iter(self) -> VolIter<T> {
    VolIter {
      vol_address: self.vol_address,
      slots_remaining: COUNT,
    }
  }

  /// Unchecked indexing into the block.
  ///
  /// # Safety
  ///
  /// The slot given must be in bounds.
  #[inline(always)]
  pub const unsafe fn index_unchecked(self, slot: usize) -> VolAddress<T> {
    self.vol_address.offset(slot as isize)
  }

  /// Checked "indexing" style access of the block, giving either a `VolAddress`
  /// or a panic.
  #[inline(always)]
  pub fn index(self, slot: usize) -> VolAddress<T> {
    if slot < COUNT {
      unsafe { self.index_unchecked(slot) }
    } else {
      panic!("Index Requested: {} >= Slot Count: {}", slot, COUNT)
    }
  }

  /// Checked "getting" style access of the block, giving an Option value.
  #[inline(always)]
  pub fn get(self, slot: usize) -> Option<VolAddress<T>> {
    if slot < COUNT {
      unsafe { Some(self.index_unchecked(slot)) }
    } else {
      None
    }
  }
}

/// A series of evenly strided addresses.
///
/// * The `C` parameter is the element count of the series.
/// * The `S` parameter is the stride (in bytes) from one element to the next.
///
/// This is for when you have something like "a series of 128 `u16` values every
/// 16 bytes starting at `0x700_0000`".
pub struct VolSeries<T, const COUNT: usize, const STRIDE: usize> {
  vol_address: VolAddress<T>,
}
impl<T, const COUNT: usize, const STRIDE: usize> Clone for VolSeries<T, COUNT, STRIDE> {
  #[inline(always)]
  fn clone(&self) -> Self {
    *self
  }
}
impl<T, const COUNT: usize, const STRIDE: usize> Copy for VolSeries<T, COUNT, STRIDE> {}
impl<T, const COUNT: usize, const STRIDE: usize> PartialEq for VolSeries<T, COUNT, STRIDE> {
  #[inline(always)]
  fn eq(&self, other: &Self) -> bool {
    self.vol_address == other.vol_address
  }
}
impl<T, const COUNT: usize, const STRIDE: usize> Eq for VolSeries<T, COUNT, STRIDE> {}
impl<T, const COUNT: usize, const STRIDE: usize> core::fmt::Debug for VolSeries<T, COUNT, STRIDE> {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    write!(
      f,
      "VolSeries({:p}, count={}, series={})",
      self.vol_address.address.get() as *mut T,
      COUNT,
      STRIDE
    )
  }
}
impl<T, const COUNT: usize, const STRIDE: usize> VolSeries<T, COUNT, STRIDE> {
  /// Constructs a new `VolSeries`.
  ///
  /// # Safety
  ///
  /// The given address must be a valid `VolAddress` at each position in the
  /// series for COUNT slots with stride STRIDE.
  #[inline(always)]
  pub const unsafe fn new(address: usize) -> Self {
    Self {
      vol_address: VolAddress::new(address),
    }
  }

  /// The length of this series (in elements)
  #[inline(always)]
  pub const fn len(self) -> usize {
    COUNT
  }

  /// Gives an iterator over the slots of this series.
  #[inline(always)]
  pub const fn iter(self) -> VolStridingIter<T, STRIDE> {
    VolStridingIter {
      vol_address: self.vol_address,
      slots_remaining: COUNT,
    }
  }

  /// Unchecked indexing into the series.
  ///
  /// # Safety
  ///
  /// The slot given must be in bounds.
  #[inline(always)]
  pub const unsafe fn index_unchecked(self, slot: usize) -> VolAddress<T> {
    self.vol_address.cast::<u8>().offset((STRIDE * slot) as isize).cast::<T>()
  }

  /// Checked "indexing" style access into the series, giving either a `VolAddress` or a panic.
  #[inline(always)]
  pub fn index(self, slot: usize) -> VolAddress<T> {
    if slot < COUNT {
      unsafe { self.index_unchecked(slot) }
    } else {
      panic!("Index Requested: {} >= Slot Count: {}", slot, COUNT)
    }
  }

  /// Checked "getting" style access into the series, giving an Option value.
  #[inline(always)]
  pub fn get(self, slot: usize) -> Option<VolAddress<T>> {
    if slot < COUNT {
      unsafe { Some(self.index_unchecked(slot)) }
    } else {
      None
    }
  }
}

/// An iterator that produces consecutive `VolAddress` values.
pub struct VolIter<T> {
  vol_address: VolAddress<T>,
  slots_remaining: usize,
}
impl<T> Clone for VolIter<T> {
  #[inline(always)]
  fn clone(&self) -> Self {
    Self {
      vol_address: self.vol_address,
      slots_remaining: self.slots_remaining,
    }
  }
}
impl<T> PartialEq for VolIter<T> {
  #[inline(always)]
  fn eq(&self, other: &Self) -> bool {
    self.vol_address == other.vol_address && self.slots_remaining == other.slots_remaining
  }
}
impl<T> Eq for VolIter<T> {}
impl<T> Iterator for VolIter<T> {
  type Item = VolAddress<T>;

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
impl<T> FusedIterator for VolIter<T> {}
impl<T> core::fmt::Debug for VolIter<T> {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    write!(
      f,
      "VolIter({:p}, remaining={})",
      self.vol_address.address.get() as *mut T,
      self.slots_remaining
    )
  }
}

/// An iterator that produces strided `VolAddress` values.
pub struct VolStridingIter<T, const STRIDE: usize> {
  vol_address: VolAddress<T>,
  slots_remaining: usize,
}
impl<T, const STRIDE: usize> Clone for VolStridingIter<T, STRIDE> {
  #[inline(always)]
  fn clone(&self) -> Self {
    Self {
      vol_address: self.vol_address,
      slots_remaining: self.slots_remaining,
    }
  }
}
impl<T, const STRIDE: usize> PartialEq for VolStridingIter<T, STRIDE> {
  #[inline(always)]
  fn eq(&self, other: &Self) -> bool {
    self.vol_address == other.vol_address && self.slots_remaining == other.slots_remaining
  }
}
impl<T, const STRIDE: usize> Eq for VolStridingIter<T, STRIDE> {}
impl<T, const STRIDE: usize> Iterator for VolStridingIter<T, STRIDE> {
  type Item = VolAddress<T>;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    if self.slots_remaining > 0 {
      let out = self.vol_address;
      unsafe {
        self.slots_remaining -= 1;
        self.vol_address = self.vol_address.cast::<u8>().offset(STRIDE as isize).cast::<T>();
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
          .offset((STRIDE * self.slots_remaining) as isize)
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
        let out = self.vol_address.cast::<u8>().offset((STRIDE * n) as isize).cast::<T>();
        let jump = n + 1;
        self.slots_remaining -= jump;
        self.vol_address = self.vol_address.cast::<u8>().offset((STRIDE * jump) as isize).cast::<T>();
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
impl<T, const STRIDE: usize> FusedIterator for VolStridingIter<T, STRIDE> {}
impl<T, const STRIDE: usize> core::fmt::Debug for VolStridingIter<T, STRIDE> {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    write!(
      f,
      "VolStridingIter({:p}, remaining={}, stride={})",
      self.vol_address.address.get() as *mut T,
      self.slots_remaining,
      STRIDE
    )
  }
}
