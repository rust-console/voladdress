#![no_std]
#![deny(missing_docs)]

//! A crate for working with volatile addresses / memory mapped IO / MMIO.
//!
//! Types here are **only** intended be used for values that can be read or
//! written in a single machine instruction. Depending on your target, this
//! generally means individual scalar values, or `repr(transparent)` wrappers
//! around said values.
//!
//! If the target data type of a [`VolAddress`] can't be read in a single
//! machine instruction then you can get unwanted data tearing.
//!
//! If your data is a number of identical values in a row consider using
//! [`VolBlock`] or [`VolSeries`]. If your data is irregular you may need to use
//! a grab-bag of [`VolAddress`] entries or something like that.

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

/// Lets you put "Safe" into a generic type parameter.
pub struct Safe;

/// Lets you put "Unsafe" into a generic type parameter.
pub struct Unsafe;
