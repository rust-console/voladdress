#![no_std]
#![deny(missing_docs)]
#![allow(clippy::iter_nth_zero)]
#![cfg_attr(test, allow(clippy::redundant_clone))]
#![cfg_attr(test, allow(bad_style))]

//! A crate for working with volatile locations, particularly Memory Mapped IO
//! (MMIO).
//!
//! ## Types
//!
//! The crate's core type is [VolAddress<T, R, W>].
//! * `T` is the element type stored at the address. It is expected that your
//!   element type will be something that the CPU can read and write with a
//!   single instruction. Generally this will be a single integer, float, data
//!   pointer, function pointer, or a `repr(transparent)` wrapper around one of
//!   the other types just listed.
//! * `R` should be [Safe], [Unsafe], or `()`. When `R` is `Safe` then you can
//!   *safely* read from the address. When `R` is `Unsafe` then you can
//!   *unsafely* read from the address. If `R` is any other type then you cannot
//!   read from the address at all. While any possible type can be used here, if
//!   reading isn't intended you should use `()` as the canonical null type.
//! * `W` works like `R` in terms of what types you should use with it, but it
//!   controls writing instead of reading.
//!
//! The `VolAddress` type uses the "unsafe creation, then safe use" style. This
//! allows us to use the fewest `unsafe` blocks overall. Once a `VolAddress` has
//! been unsafely declared, each individual operation using them is generally
//! going to be safe. Some addresses might be unsafe to use even after creation,
//! but this is relatively rare.
//!
//! Here are some example declarations. Note that the address values used are
//! for illustation purposes only, and will vary for each device.
//! ```
//! # use voladdress::*;
//! // read-only
//! pub const VCOUNT: VolAddress<u16, Safe, ()> =
//!   unsafe { VolAddress::new(0x0400_0006) };
//!
//! // write-only
//! pub const BG0_XOFFSET: VolAddress<u16, (), Safe> =
//!   unsafe { VolAddress::new(0x0400_0010) };
//!
//! // read-write
//! pub const BLDALPHA_A: VolAddress<u8, Safe, Safe> =
//!   unsafe { VolAddress::new(0x0400_0052) };
//!
//! // this location has some illegal bit patterns, so it's unsafe
//! // to write to with any random `u16` you might have.
//! pub const RAW_DISPLAY_CONTROL: VolAddress<u16, Safe, Unsafe> =
//!   unsafe { VolAddress::new(0x0400_0000) };
//!
//! // If we use a transparent wrapper and getter/setters, we can
//! // prevent the illegal bit patterns, and now it's safe to write.
//! #[repr(transparent)]
//! pub struct DisplayCtrl(u16);
//! pub const DISPLAY_CONTROL: VolAddress<DisplayCtrl, Safe, Safe> =
//!   unsafe { VolAddress::new(0x0400_0000) };
//! ```
//!
//! ### Multiple Locations
//!
//! Often we have many identically typed values at a regular pattern in memory.
//! These are handled with two very similar types.
//!
//! [VolBlock<T, R, W, const C: usize>] is for when there's many values tightly
//! packed, with no space in between. Use this type when you want to emulate how
//! an array works.
//!
//! [VolSeries<T, R, W, const C: usize, const S: usize>] is for when you have
//! many values strided out at regular intervals, but they have extra space in
//! between each element.
//!
//! In both cases, there's two basic ways to work with the data:
//! * Using `len`, `index`, and `get`, you can produce individual `VolAddress`
//!   values similar to how a slice can produce references into the slice's data
//!   range.
//! * Using `iter` or `iter_range` you can produce an in iterator that will go
//!   over the various `VolAddress` values during the iteration.
//!
//! ```no_run
//! # use voladdress::*;
//! pub const BG_PALETTE: VolBlock<u16, Safe, Safe, 256> =
//!   unsafe { VolBlock::new(0x0500_0000) };
//!
//! pub const COLOR_RED: u16 = 0b11111;
//! BG_PALETTE.index(0).write(COLOR_RED);
//!
//! pub const COLOR_GREEN: u16 = 0b11111_00000;
//! BG_PALETTE.iter_range(1..).for_each(|a| a.write(COLOR_GREEN));
//!
//! pub const MY_ROM_PALETTE_DATA: [u16; 256] = [0xAB; 256];
//! BG_PALETTE
//!   .iter()
//!   .zip(MY_ROM_PALETTE_DATA.iter().copied())
//!   .for_each(|(a, c)| a.write(c));
//! ```
//!
//! ### No Lifetimes
//!
//! Note that `VolAddress`, `VolBlock`, and `VolSeries` are all `Copy` data
//! types, without any lifetime parameter. It is assumed that the MMIO memory
//! map of your device is a fixed part of the device, and that the types from
//! this crate will be used to create `const` declarations that describe that
//! single memory map which is unchanging during the entire program. If the
//! memory mapping of your device *can* change then you must account for this in
//! your declarations.

use core::{
  marker::PhantomData,
  num::NonZeroUsize,
  ptr::{read_volatile, write_volatile},
};

mod voladdress_;
pub use voladdress_::*;

mod volblock;
pub use volblock::*;

mod volseries;
pub use volseries::*;

#[cfg(feature = "experimental_volmatrix")]
mod volmatrix;
#[cfg(feature = "experimental_volmatrix")]
pub use volmatrix::*;

#[cfg(feature = "experimental_volregion")]
mod volregion;
#[cfg(feature = "experimental_volregion")]
pub use volregion::*;

/// Lets you put "Safe" into a generic type parameter.
///
/// This type affects the read and write methods of the volatile address types,
/// but has no effect on its own.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Safe;

/// Lets you put "Unsafe" into a generic type parameter.
///
/// This type affects the read and write methods of the volatile address types,
/// but has no effect on its own.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Unsafe;
