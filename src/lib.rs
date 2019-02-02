// Note(Lokathor): Required to allow for marker trait bounds on const functions.
#![feature(const_fn)]
#![forbid(missing_docs)]
#![forbid(missing_debug_implementations)]

//! **voladdress** is a crate that makes it easy to work with volatile memory addresses.
//! Specific hardware addresses may have particular read and write rules,
//! and we generally want to use those addresses more often than
//! naming them. This crate provides the utilities for abstracting and accessing them,
//! while preventing the compiler from optimizing our memory accesses away.
//! (eg: raw hardware addresses).
//!
//! For example, on the GBA there's a palette of 256 background color values
//! (`u16`) starting at `0x500_0000`, so you might write something like.
//!
//! ```rust
//! use typenum::consts::U256;
//! use voladdress::{VolBlock, VolAddress};
//!
//! pub type Color = u16;
//!
//! pub const PALRAM_BG: VolBlock<Color,U256> = unsafe { VolBlock::new(0x500_0000) };
//! ```
//!
//! And then in your actual program you might do something like this
//!
//! ```rust
//! # use typenum::consts::U256;
//! # use voladdress::{VolBlock, VolAddress};
//! # pub type Color = u16;
//! fn main() {
//! #  let palram = vec![0u16; 256];
//! #  let PALRAM_BG: VolBlock<Color,U256> = unsafe { VolBlock::new(palram.as_ptr() as usize) };
//!   let i = 5;
//!   // the palette is all 0 at startup.
//!   assert_eq!(PALRAM_BG.index(i).read(), 0);
//!   // we can make that index into white instead.
//!   const WHITE: u16 = 0b0_11111_11111_1111;
//!   PALRAM_BG.index(i).write(WHITE);
//!   assert_eq!(PALRAM_BG.index(i).read(), WHITE);
//! }
//! ```
//!
//! The specific addresses that are safe to use depend on your exact target
//! device. Please read your target device's documentation.
//!
//! (Note: If you look at the source for the tests and doctests you'll see that
//! they make `Vec<_>` values as a stand in for real memory. This is for
//! demonstration and testing _only_. In a real program you should use the
//! actual, correct addresses for your hardware.)
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
//! However, when working with raw hardware addresses the read and write
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
//! hardware manual) into their crate's volatile type (eg: `let p = 1234 as *mut
//! VolatileCell<u16>`), but then you have to dereference that raw pointer _each
//! time_ you call read or write, and it always requires parenthesis too,
//! because of prescience rules (eg: `let a = (*p).read();`). You end up with
//! `unsafe` blocks and parens and asterisks all over the code for no reason.
//!
//! This crate is much better than any of that. Once you've decided that the
//! initial unsafety is alright, and you've created a `VolAddress` value for
//! your target type at the target address, the `read` and `write` methods are
//! entirely safe to use and don't require the manual de-reference.
//!
//! # Can't you just impl `Deref` and `DerefMut` on these things?
//!
//! No. Absolutely not. Both `&T` and `&mut T` use normal reads and writes, so
//! they'll elide the access just like a raw pointer would. In fact they're
//! _more_ aggressive about it than raw pointers are because they assume that
//! either the target value never changes (`&T`) so you don't ever need to read
//! twice, or the target value is exclusively controlled by the local scope
//! (`&mut T`) so you never need to do intermediate writes. For standard memory
//! (in registers or RAM) this is exactly what we want, but with raw hardware
//! addresses this is the opposite of a good time.

use core::{cmp::Ordering, iter::FusedIterator, marker::PhantomData, num::NonZeroUsize};
use typenum::marker_traits::Unsigned;

// Note(Lokathor): We have to hand implement all the traits for all of our types
// manually because if we use `derive` then they only get derived if the `T` has
// that trait. However, since we're acting like various "pointers" to `T`
// values, the capabilities we offer aren't at all affected by whatever type `T`
// ends up being.

/// Abstracts the use of a volatile hardware address.
///
/// If you're trying to do anything other than abstract a volatile hardware
/// device then you _do not want to use this type_. Use one of the many other
/// smart pointer types.
///
/// A volatile address doesn't store a value in the normal way: It maps to some
/// real hardware _other than_ RAM, and that hardware might have any sort of
/// strange rules. The specifics of reading and writing depend on the hardware
/// being mapped. For example, a particular address might be read only (ignoring
/// writes), write only (returning some arbitrary value if you read it),
/// "normal" read write (where you read back what you wrote), or some complex
/// read-write situation where writes have an effect but you _don't_ read back
/// what you wrote.
///
/// The design of this type is set up so that _creation_ is unsafe, and _use_ is
/// safe. This gives an optimal experience, since you'll use memory locations a
/// lot more often than you try to name them, on average.
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
/// * The declared address must be non-null (it uses the `NonNull` optimization
///   for better iteration results). This shouldn't be a big problem, since
///   hardware can't live at the null address.
/// * The declared address must be aligned for the declared type of `T`.
/// * The declared address must _always_ be
///   "[valid](https://doc.rust-lang.org/core/ptr/index.html#safety)" according
///   to the rules of `core::ptr`. Don't pick a type if the hardware might show
///   invalid bit patterns. If there's _any_ doubt at all, you must instead read
///   or write an unsigned int of the correct bit size and then parse the bits
///   by hand.
/// * The declared address must be a part of the address space that Rust's
///   allocator and/or stack frames will never use.
///
/// If you're not sure about any of those points, please re-read the hardware
/// specs of your target device and its memory map until you know.
///
/// The _exact_ points of UB are if the address is ever 0, or if you ever
/// actually `read` or `write` with an invalidly constructed `VolAddress`.
#[repr(transparent)]
pub struct VolAddress<T> {
  address: NonZeroUsize,
  marker: PhantomData<*mut T>,
}
impl<T> Clone for VolAddress<T> {
  fn clone(&self) -> Self {
    *self
  }
}
impl<T> Copy for VolAddress<T> {}
impl<T> PartialEq for VolAddress<T> {
  fn eq(&self, other: &Self) -> bool {
    self.address == other.address
  }
}
impl<T> Eq for VolAddress<T> {}
impl<T> PartialOrd for VolAddress<T> {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.address.cmp(&other.address))
  }
}
impl<T> Ord for VolAddress<T> {
  fn cmp(&self, other: &Self) -> Ordering {
    self.address.cmp(&other.address)
  }
}
impl<T> core::fmt::Debug for VolAddress<T> {
  /// The basic formatting uses the pointer style
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    write!(f, "{:p}", self)
  }
}
impl<T> core::fmt::Pointer for VolAddress<T> {
  /// You can request pointer style, but it's already on by default
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    write!(f, "VolAddress({:p})", self.address.get() as *mut T)
  }
}
impl<T> VolAddress<T> {
  /// Constructs a new address.
  ///
  /// # Safety
  ///
  /// You must follow the standard safety rules as outlined in the type docs.
  pub const unsafe fn new(address: usize) -> Self {
    VolAddress {
      address: NonZeroUsize::new_unchecked(address),
      marker: PhantomData,
    }
  }

  /// Casts the type of `T` into type `Z`.
  ///
  /// # Safety
  ///
  /// You must follow the standard safety rules as outlined in the type docs.
  pub const unsafe fn cast<Z>(self) -> VolAddress<Z> {
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
  pub const unsafe fn offset(self, offset: isize) -> Self {
    VolAddress {
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
  pub const fn is_aligned(self) -> bool {
    self.address.get() % core::mem::align_of::<T>() == 0
  }

  /// Makes an iterator starting here across the given number of slots.
  ///
  /// # Safety
  ///
  /// The normal safety rules must be correct for each address iterated over.
  pub const unsafe fn iter_slots(self, slots: usize) -> VolIter<T> {
    VolIter { vol_address: self, slots_remaining: slots }
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
  /// This is _not_ a move, it forms a bit duplicate of the current address
  /// value. If `T` has a `Drop` trait that does anything it is up to you to
  /// ensure that repeated drops do not cause UB (such as a double free).
  pub unsafe fn read_non_copy(self) -> T {
    (self.address.get() as *mut T).read_volatile()
  }

  /// Volatile writes a value to the address.
  ///
  /// Semantically, the value is moved into the `VolAddress` and then forgotten,
  /// so if `T` has a `Drop` impl then that will never get executed. This is
  /// "safe" under Rust's safety rules, but could cause something unintended
  /// (eg: a memory leak).
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
pub struct VolBlock<T, C: Unsigned> {
  vol_address: VolAddress<T>,
  slot_count: PhantomData<C>,
}
impl<T, C:Unsigned> Clone for VolBlock<T, C> {
  fn clone(&self) -> Self {
    *self
  }
}
impl<T, C:Unsigned> Copy for VolBlock<T, C> { }
impl<T, C:Unsigned> PartialEq for VolBlock<T, C> {
  fn eq(&self, other: &Self) -> bool {
    self.vol_address == other.vol_address
  }
}
impl<T, C:Unsigned> Eq for VolBlock<T, C> {}
impl<T, C:Unsigned> core::fmt::Debug for VolBlock<T, C> {
  // The basic formatting uses the pointer style
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    write!(f, "VolBlock({:p}, c={})", self.vol_address.address.get() as *mut T, C::USIZE)
  }
}
impl<T, C:Unsigned> VolBlock<T, C> {
  /// Constructs a new `VolBlock`.
  ///
  /// # Safety
  ///
  /// The given address must be a valid `VolAddress` at each position in the
  /// block for however many slots (`C`).
  pub const unsafe fn new(address: usize) -> Self {
    Self { vol_address: VolAddress::new(address), slot_count: PhantomData }
  }

  /// Gives an iterator over the slots of this block.
  pub const fn iter(self) -> VolIter<T> {
    VolIter {
      vol_address: self.vol_address,
      slots_remaining: C::USIZE,
    }
  }

  /// Unchecked indexing into the block.
  ///
  /// # Safety
  ///
  /// The slot given must be in bounds.
  pub const unsafe fn index_unchecked(self, slot: usize) -> VolAddress<T> {
    self.vol_address.offset(slot as isize)
  }

  /// Checked "indexing" style access of the block, giving either a `VolAddress` or a panic.
  pub fn index(self, slot: usize) -> VolAddress<T> {
    if slot < C::USIZE {
      unsafe { self.vol_address.offset(slot as isize) }
    } else {
      panic!("Index Requested: {} >= Slot Count: {}", slot, C::USIZE)
    }
  }

  /// Checked "getting" style access of the block, giving an Option value.
  pub fn get(self, slot: usize) -> Option<VolAddress<T>> {
    if slot < C::USIZE {
      unsafe { Some(self.vol_address.offset(slot as isize)) }
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
pub struct VolSeries<T, C: Unsigned, S: Unsigned> {
  vol_address: VolAddress<T>,
  slot_count: PhantomData<C>,
  stride: PhantomData<S>,
}
impl<T, C: Unsigned, S: Unsigned> Clone for VolSeries<T, C, S> {
  fn clone(&self) -> Self {
    *self
  }
}
impl<T, C: Unsigned, S: Unsigned> Copy for VolSeries<T, C, S> { }
impl<T, C: Unsigned, S: Unsigned> PartialEq for VolSeries<T, C, S> {
  fn eq(&self, other: &Self) -> bool {
    self.vol_address == other.vol_address
  }
}
impl<T, C: Unsigned, S: Unsigned> Eq for VolSeries<T, C, S> {}
impl<T, C: Unsigned, S: Unsigned> core::fmt::Debug for VolSeries<T, C, S> {
  // The basic formatting uses the pointer style
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    write!(f, "VolSeries({:p}, c={}, s={})", self.vol_address.address.get() as *mut T, C::USIZE, S::USIZE)
  }
}
impl<T, C: Unsigned, S: Unsigned> VolSeries<T, C, S> {
  /// Constructs a new `VolSeries`.
  ///
  /// # Safety
  ///
  /// The given address must be a valid `VolAddress` at each position in the
  /// series for however many slots (`C`), strided by the selected amount (`S`).
  pub const unsafe fn new(address: usize) -> Self {
    Self { vol_address: VolAddress::new(address), slot_count: PhantomData, stride: PhantomData }
  }

  /// Gives an iterator over the slots of this series.
  pub const fn iter(self) -> VolStridingIter<T, S> {
    VolStridingIter {
      vol_address: self.vol_address,
      slots_remaining: C::USIZE,
      stride: PhantomData
    }
  }

  /// Unchecked indexing into the block.
  ///
  /// # Safety
  ///
  /// The slot given must be in bounds.
  pub const unsafe fn index_unchecked(self, slot: usize) -> VolAddress<T> {
    self.vol_address.offset(slot as isize)
  }

  /// Checked "indexing" style access of the block, giving either a `VolAddress` or a panic.
  pub fn index(self, slot: usize) -> VolAddress<T> {
    if slot < C::USIZE {
      unsafe { self.vol_address.offset(slot as isize) }
    } else {
      panic!("Index Requested: {} >= Slot Count: {}", slot, C::USIZE)
    }
  }

  /// Checked "getting" style access of the block, giving an Option value.
  pub fn get(self, slot: usize) -> Option<VolAddress<T>> {
    if slot < C::USIZE {
      unsafe { Some(self.vol_address.offset(slot as isize)) }
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
  fn clone(&self) -> Self {
    Self {
      vol_address: self.vol_address,
      slots_remaining: self.slots_remaining,
    }
  }
}
impl<T> PartialEq for VolIter<T> {
  fn eq(&self, other: &Self) -> bool {
    self.vol_address == other.vol_address && self.slots_remaining == other.slots_remaining
  }
}
impl<T> Eq for VolIter<T> {}
impl<T> Iterator for VolIter<T> {
  type Item = VolAddress<T>;

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
}
impl<T> FusedIterator for VolIter<T> {}
impl<T> core::fmt::Debug for VolIter<T> {
  // The basic formatting uses the pointer style
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    write!(f, "VolIter({:p}, remaining={})", self.vol_address.address.get() as *mut T, self.slots_remaining)
  }
}

/// An iterator that produces strided `VolAddress` values.
pub struct VolStridingIter<T, S: Unsigned> {
  vol_address: VolAddress<T>,
  slots_remaining: usize,
  stride: PhantomData<S>,
}
impl<T, S: Unsigned> Clone for VolStridingIter<T, S> {
  fn clone(&self) -> Self {
    Self {
      vol_address: self.vol_address,
      slots_remaining: self.slots_remaining,
      stride: PhantomData
    }
  }
}
impl<T, S: Unsigned> PartialEq for VolStridingIter<T, S> {
  fn eq(&self, other: &Self) -> bool {
    self.vol_address == other.vol_address && self.slots_remaining == other.slots_remaining
  }
}
impl<T, S: Unsigned> Eq for VolStridingIter<T, S> {}
impl<T, S: Unsigned> Iterator for VolStridingIter<T, S> {
  type Item = VolAddress<T>;

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
}
impl<T, S: Unsigned> FusedIterator for VolStridingIter<T, S> {}
impl<T, S: Unsigned> core::fmt::Debug for VolStridingIter<T, S> {
  // The basic formatting uses the pointer style
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    write!(f, "VolStridingIter({:p}, remaining={}, s={})", self.vol_address.address.get() as *mut T, self.slots_remaining, S::USIZE)
  }
}
