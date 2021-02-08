#![no_std]
#![deny(missing_docs)]

//! A crate for working with volatile addresses / memory mapped IO / MMIO.
//!
//! Types here are **only** intended be used for values that can be read or
//! written in a single machine instruction. Depending on your target, this
//! generally means individual scalar values, or `repr(transparent)` wrappers
//! around said values.
//!
//! It is possible to use them with larger values but be aware that the `read`
//! and `write` operations perform the full read or write every time. If you
//! have a 16 byte struct and change a single byte field, the *entire* 16 bytes
//! are then written, not just the byte you changed.
//!
//! If your data is a number of identical values in a row consider using
//! [`VolBlock`] or [`VolSeries`]. If your data is irregular you may need to use
//! a grab-bag of [`VolAddress`] entries or something like that.

// TODO: crate docs that explain how to model weird stuff.

use core::{
  marker::PhantomData,
  num::NonZeroUsize,
  ptr::{read_volatile, write_volatile},
};

mod plain_voladdress;
pub use plain_voladdress::*;

mod volblock;
pub use volblock::*;

mod volseries;
pub use volseries::*;

/// Lets you put "No" into a generic type parameter.
pub struct No;

/// Lets you put "Yes" into a generic type parameter.
pub struct Yes;

/// Lets you put "Unsafe" into a generic type parameter.
pub struct Unsafe;
